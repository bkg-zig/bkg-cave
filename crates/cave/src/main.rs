//! cave — bkg-cave main binary.
//!
//! Orchestrates the three plugin domains:
//! - `cave surge` — VM runner (QEMU/KVM with host rootfs)
//! - `cave kindle` — Initramfs generator (TOML → CPIO)
//! - `cave forge` — OS image builder (pipeline manifest → image)
//!
//! Supports hot-reload of dynamic plugins from `/usr/lib/cave/plugins/`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use cave_core::plugin::{PluginContext, PluginKind, PluginManager};

/// bkg-cave — modular developer toolchain for Linux systems.
///
/// Three domains, one tool:
///   surge  — Start a VM with the host distribution as rootfs
///   kindle — Generate initramfs images from TOML configuration
///   forge  — Build OS images from declarative pipeline manifests
#[derive(Parser)]
#[command(name = "cave", version, about, long_about = None)]
struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    command: Command,
}

/// Remaining raw arguments passed to subcommands.
#[derive(clap::Args)]
struct RawArgs {
    /// Arguments for the subcommand
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    args: Vec<String>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Start a VM with the host rootfs via QEMU/KVM
    Surge(RawArgs),
    /// Generate an initramfs from TOML configuration
    Kindle(RawArgs),
    /// Build OS images from pipeline manifests
    Forge(RawArgs),
    /// List all loaded plugins
    Plugins,
}

fn main() -> ExitCode {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match run(cli) {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            tracing::error!("{:#}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<i32, anyhow::Error> {
    // Create plugin manager
    let mut pm = PluginManager::new();

    // Plugin context shared by all plugins
    let ctx = PluginContext {
        config_dir: dirs().config_dir,
        data_dir: dirs().data_dir,
    };

    // Load static plugins
    pm.load_static(Box::new(cave_surge::SurgePlugin), &ctx)
        .map_err(|e| anyhow::anyhow!("Failed to load surge plugin: {}", e))?;

    pm.load_static(Box::new(cave_kindle::KindlePlugin), &ctx)
        .map_err(|e| anyhow::anyhow!("Failed to load kindle plugin: {}", e))?;

    pm.load_static(Box::new(cave_forge::ForgePlugin), &ctx)
        .map_err(|e| anyhow::anyhow!("Failed to load forge plugin: {}", e))?;

    // Route CLI commands
    match cli.command {
        Command::Surge(raw) => pm.execute("surge", &raw.args),
        Command::Kindle(raw) => pm.execute("kindle", &raw.args),
        Command::Forge(raw) => pm.execute("forge", &raw.args),
        Command::Plugins => {
            println!("Loaded plugins:");
            for (id, name, kind) in pm.list_plugins() {
                let kind_str = match kind {
                    PluginKind::Surge => "surge",
                    PluginKind::Kindle => "kindle",
                    PluginKind::Forge => "forge",
                    PluginKind::Utility => "utility",
                };
                println!("  {:20} {} ({})", id, name, kind_str);
            }
            Ok(0)
        }
    }
    .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Standard directory layout for cave runtime files.
fn dirs() -> CaveDirs {
    let base = PathBuf::from("/usr/lib/cave");

    CaveDirs {
        config_dir: base.join("config"),
        data_dir: base.join("data"),
        plugin_dir: base.join("plugins"),
    }
}

struct CaveDirs {
    #[allow(dead_code)]
    config_dir: PathBuf,
    #[allow(dead_code)]
    data_dir: PathBuf,
    #[allow(dead_code)]
    plugin_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_surge_subcommand() {
        let cli = Cli::try_parse_from(["cave", "surge", "--dry-run"]);
        assert!(cli.is_ok());
        let cmd = cli.unwrap().command;
        match cmd {
            Command::Surge(raw) => assert!(raw.args.contains(&"--dry-run".to_string())),
            _ => panic!("expected Surge"),
        }
    }

    #[test]
    fn test_cli_kindle_subcommand() {
        let cli = Cli::try_parse_from(["cave", "kindle", "--dry-run", "--config", "test.toml"]);
        assert!(cli.is_ok());
        let cmd = cli.unwrap().command;
        match cmd {
            Command::Kindle(raw) => {
                assert!(raw.args.contains(&"--dry-run".to_string()));
                assert!(raw.args.contains(&"--config".to_string()));
                assert!(raw.args.contains(&"test.toml".to_string()));
            }
            _ => panic!("expected Kindle"),
        }
    }

    #[test]
    fn test_cli_forge_subcommand() {
        let cli = Cli::try_parse_from(["cave", "forge", "--manifest", "test.json"]);
        assert!(cli.is_ok());
        let cmd = cli.unwrap().command;
        match cmd {
            Command::Forge(raw) => {
                assert!(raw.args.contains(&"--manifest".to_string()));
                assert!(raw.args.contains(&"test.json".to_string()));
            }
            _ => panic!("expected Forge"),
        }
    }

    #[test]
    fn test_cli_plugins_subcommand() {
        let cli = Cli::try_parse_from(["cave", "plugins"]);
        assert!(cli.is_ok());
        assert!(matches!(cli.unwrap().command, Command::Plugins));
    }

    #[test]
    fn test_plugins_runs() {
        let cli = Cli::try_parse_from(["cave", "plugins"]).unwrap();
        let result = run(cli);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code, 0);
    }
}
