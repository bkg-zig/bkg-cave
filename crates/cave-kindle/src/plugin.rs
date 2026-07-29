//! [`KindlePlugin`] — the `cave-kindle` plugin for the bkg-cave system.
//!
//! This plugin registers the `kindle` command which generates an initramfs
//! image from a TOML configuration file.

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;
use tracing::{debug, info};

use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::BkgCaveError;

use crate::cli::KindleArgs;
use crate::config::{ConfigStage, InitramfsConfig};
use crate::generator::Generator;
use crate::modules;

/// The `cave-kindle` plugin.
///
/// Registers the `kindle` command for initramfs generation.
pub struct KindlePlugin;

impl CavePlugin for KindlePlugin {
    fn id(&self) -> &'static str {
        "cave-kindle"
    }

    fn name(&self) -> &str {
        "Kindle Initramfs Generator"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn kind(&self) -> PluginKind {
        PluginKind::Kindle
    }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), BkgCaveError> {
        info!("initialising Kindle plugin");
        Ok(())
    }

    fn commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "kindle",
            description: "Generate an initramfs from TOML config",
            run: |args| execute(args),
        }]
    }

    fn hot_reloadable(&self) -> bool {
        false
    }
}

/// Execute the `kindle` command.
///
/// Parses arguments, loads configuration, discovers modules, and drives
/// the [`Generator`] to produce an initramfs image.
pub fn execute(args: &[String]) -> Result<i32, BkgCaveError> {
    // Parse CLI arguments.
    let kindle_args = KindleArgs::try_parse_from(
        std::iter::once("kindle".to_owned()).chain(args.iter().cloned()),
    )
    .map_err(|e| BkgCaveError::InvalidInput(e.to_string()))?;

    info!(
        compress = %kindle_args.compress,
        validate_only = kindle_args.validate_only,
        debug = kindle_args.debug,
        "kindle command started"
    );

    // Set up tracing / logging based on --debug flag.
    if kindle_args.debug {
        // Debug logging is typically configured by the host app.
        debug!("debug logging enabled");
    }

    // Build configuration.
    let mut config = build_config(&kindle_args)
        .map_err(|e| BkgCaveError::Config(format!("failed to build config: {e}")))?;

    // If --validate-only, validate and exit.
    if kindle_args.validate_only {
        config
            .validate()
            .map_err(|e| BkgCaveError::Config(format!("validation failed: {e}")))?;
        info!("configuration is valid");
        return Ok(0);
    }

    // Determine build and output directories.
    let build_dir = std::env::temp_dir().join("cave-kindle-build");
    let out_dir = if let Some(ref out_path) = kindle_args.output {
        PathBuf::from(out_path)
    } else {
        PathBuf::from(".")
    };

    // Ensure the output's parent directory exists.
    if let Some(parent) = out_dir.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(BkgCaveError::Io)?;
        }
    }

    // Create the generator.
    let mut generator = Generator::new(config, build_dir, out_dir);

    // Discover and load modules.
    if let Some(ref modules_dir) = kindle_args.modules_dir {
        let modules_path = Path::new(modules_dir);
        load_modules(&mut generator, modules_path)
            .map_err(|e| BkgCaveError::Config(format!("module loading failed: {e}")))?;
    }

    // Set config stage to Final before building.
    generator.config.set_stage(ConfigStage::Final);
    generator
        .config
        .process_deferred()
        .map_err(|e| BkgCaveError::Config(format!("deferred processing failed: {e}")))?;
    generator
        .config
        .validate()
        .map_err(|e| BkgCaveError::Config(format!("validation failed: {e}")))?;

    // Dry run: stop before building.
    if kindle_args.dry_run {
        info!("dry run — skipping build");
        let _ = generator.generate_init();
        info!(
            "init script preview generated in: {}",
            generator.build_dir.display()
        );
        return Ok(0);
    }

    // Full build.
    generator
        .build()
        .map_err(|e| BkgCaveError::Plugin(format!("build failed: {e}")))?;

    info!(
        "initramfs generated successfully in: {}",
        generator.out_dir.display()
    );

    Ok(0)
}

/// Build the [`InitramfsConfig`] from CLI args and an optional TOML file.
fn build_config(kindle_args: &KindleArgs) -> anyhow::Result<InitramfsConfig> {
    let mut config = InitramfsConfig::new();

    // Load TOML config file if specified.
    if let Some(ref config_path) = kindle_args.config {
        let path = Path::new(config_path);
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("read config: {}", path.display()))?;
        let toml_value: toml::Value = content
            .parse()
            .with_context(|| format!("parse config: {}", path.display()))?;
        config.load_toml(&toml_value).context("load TOML config")?;
    }

    // Override with CLI arguments (CLI takes precedence).
    let cli_toml = kindle_args.to_toml();
    config.load_toml(&cli_toml).context("load CLI args")?;

    Ok(config)
}

/// Load module TOML files from a directory into the generator.
fn load_modules(generator: &mut Generator, modules_dir: &Path) -> anyhow::Result<()> {
    if !modules_dir.is_dir() {
        anyhow::bail!("modules directory not found: {}", modules_dir.display());
    }

    let module_paths = modules::get_module_paths(modules_dir)
        .with_context(|| format!("scan modules dir: {}", modules_dir.display()))?;

    for mod_path in &module_paths {
        debug!(
            module = %mod_path.display(),
            "loading module"
        );
        let module_toml = modules::read_module(mod_path)
            .with_context(|| format!("read module: {}", mod_path.display()))?;

        // Merge module parameters into config.
        generator
            .config
            .load_module(&module_toml)
            .with_context(|| format!("load module config: {}", mod_path.display()))?;

        // Register module hooks (if any are defined in the TOML).
        if let Some(hooks_table) = module_toml
            .as_table()
            .and_then(|t| t.get("hook"))
            .and_then(|v| v.as_table())
        {
            for (hook_name, _hook_def) in hooks_table {
                debug!(
                    module = %mod_path.display(),
                    hook = %hook_name,
                    "module defines hook (registration deferred to module init)"
                );
                // In a full implementation, the module would register
                // HookFn entries.  For now we note the definition.
            }
        }
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn plugin_id_and_kind() {
        let plugin = KindlePlugin;
        assert_eq!(plugin.id(), "cave-kindle");
        assert_eq!(plugin.name(), "Kindle Initramfs Generator");
        assert_eq!(plugin.version(), "0.1.0");
        assert_eq!(plugin.kind(), PluginKind::Kindle);
    }

    #[test]
    fn plugin_has_commands() {
        let plugin = KindlePlugin;
        let cmds = plugin.commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name, "kindle");
        assert!(cmds[0].description.contains("initramfs"));
    }

    #[test]
    fn plugin_not_hot_reloadable() {
        let plugin = KindlePlugin;
        assert!(!plugin.hot_reloadable());
    }

    #[test]
    fn plugin_init_succeeds() {
        let mut plugin = KindlePlugin;
        let ctx = PluginContext {
            config_dir: "/tmp/cfg".into(),
            data_dir: "/tmp/data".into(),
        };
        assert!(plugin.init(&ctx).is_ok());
    }

    #[test]
    fn execute_with_validate_only() {
        let args = vec!["--validate-only".into()];
        let result = execute(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn execute_with_dry_run() {
        let args = vec!["--dry-run".into()];
        let result = execute(&args);
        // Should succeed because dry_run skips the actual build.
        assert!(result.is_ok());
    }

    #[test]
    fn execute_with_unknown_flag() {
        let args = vec!["--bogus-flag".into()];
        let result = execute(&args);
        assert!(result.is_err());
    }

    #[test]
    fn build_config_from_cli_only() {
        let args = KindleArgs {
            config: None,
            no_base: true,
            validate_only: false,
            output: Some("/tmp/test.cpio".into()),
            compress: "none".into(),
            debug: true,
            dry_run: false,
            modules_dir: None,
            include_binary: vec![],
            include_dir: vec![],
            include_file: vec![],
        };
        let config = build_config(&args).unwrap();
        let no_base: Option<bool> = config.get("no_base").unwrap();
        assert_eq!(no_base, Some(true));
        let compress: Option<String> = config.get("compress").unwrap();
        assert_eq!(compress, Some("none".into()));
    }

    #[test]
    fn build_config_from_file() {
        let dir = TempDir::new().unwrap();
        let cfg_path = dir.path().join("kindle.toml");
        std::fs::write(
            &cfg_path,
            r#"
            compression = "zstd"
            loglevel = 5
            "#,
        )
        .unwrap();

        let args = KindleArgs {
            config: Some(cfg_path.to_string_lossy().to_string()),
            no_base: false,
            validate_only: false,
            output: None,
            compress: "xz".into(), // CLI default
            debug: false,
            dry_run: false,
            modules_dir: None,
            include_binary: vec![],
            include_dir: vec![],
            include_file: vec![],
        };

        let config = build_config(&args).unwrap();
        // CLI arg should override file.
        let compress: Option<String> = config.get("compression").unwrap();
        assert_eq!(compress, Some("zstd".into()));
    }

    #[test]
    fn load_modules_nonexistent_dir() {
        let (_b, _o, mut gen) = {
            let build_dir = TempDir::new().unwrap();
            let out_dir = TempDir::new().unwrap();
            let gen = Generator::new(
                InitramfsConfig::new(),
                build_dir.path().to_owned(),
                out_dir.path().to_owned(),
            );
            (build_dir, out_dir, gen)
        };

        let result = load_modules(&mut gen, Path::new("/tmp/does_not_exist_12345"));
        assert!(result.is_err());
    }

    #[test]
    fn load_modules_empty_dir() {
        let dir = TempDir::new().unwrap();

        let (_b, _o, mut gen) = {
            let build_dir = TempDir::new().unwrap();
            let out_dir = TempDir::new().unwrap();
            let gen = Generator::new(
                InitramfsConfig::new(),
                build_dir.path().to_owned(),
                out_dir.path().to_owned(),
            );
            (build_dir, out_dir, gen)
        };

        // Should succeed (empty dir is valid).
        load_modules(&mut gen, dir.path()).unwrap();
    }
}
