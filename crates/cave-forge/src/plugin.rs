//! [`ForgePlugin`] — implementation of [`CavePlugin`] for the pipeline
//! OS image builder.
//!
//! This plugin registers a single `forge` command that reads a pipeline
//! manifest, resolves sources, validates modules, and executes all stages
//! in dependency order inside bubblewrap sandboxes, producing a content-
//! addressed output tree.

use std::path::Path;

use cave_core::{BkgCaveError, CavePlugin, PluginContext, PluginKind, RegisteredCommand};

use clap::Parser;

use crate::buildroot::BuildRoot;
use crate::cli::ForgeArgs;
use crate::meta::Index;
use crate::objectstore::ObjectStore;
use crate::pipeline::Manifest;
use crate::sources::download_sources;

/// The cave-forge plugin instance.
///
/// This plugin is stateless — it can be safely shared across threads and
/// reused after hot-reload.
pub struct ForgePlugin;

impl ForgePlugin {
    /// Create a new `ForgePlugin`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ForgePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CavePlugin for ForgePlugin {
    /// Unique, stable identifier.
    fn id(&self) -> &'static str {
        "cave-forge"
    }

    /// Human-readable display name.
    fn name(&self) -> &str {
        "Forge OS Image Builder"
    }

    /// Semantic version.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Plugin kind.
    fn kind(&self) -> PluginKind {
        PluginKind::Forge
    }

    /// Initialise the plugin.
    ///
    /// ForgePlugin is stateless — init simply logs success.
    fn init(&mut self, ctx: &PluginContext) -> Result<(), BkgCaveError> {
        tracing::info!(
            config_dir = %ctx.config_dir.display(),
            data_dir = %ctx.data_dir.display(),
            "ForgePlugin initialised"
        );
        Ok(())
    }

    /// Return the single `forge` command.
    fn commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "forge",
            description: "Build OS images from pipeline manifests",
            run: execute,
        }]
    }

    /// ForgePlugin does not support hot-reload because it holds no mutable
    /// state.
    fn hot_reloadable(&self) -> bool {
        false
    }
}

/// Execute the `forge` command.
///
/// This function is the entry point for the plugin's command dispatch.
/// It parses CLI arguments, loads the manifest, sets up the build
/// environment, runs all pipelines, and exports the result.
///
/// # Arguments
///
/// * `args` — command-line arguments (everything after `forge`).
///
/// # Returns
///
/// * `Ok(0)` on success.
/// * `Ok(exit_code)` on controlled failure.
/// * `Err(BkgCaveError)` on unexpected errors.
pub fn execute(args: &[String]) -> Result<i32, BkgCaveError> {
    // ── 1. Parse CLI arguments ──────────────────────────────────────────────
    let forge_args = ForgeArgs::try_parse_from(
        std::iter::once("cave-forge").chain(args.iter().map(String::as_str)),
    )
    .map_err(|e| BkgCaveError::InvalidInput(format!("argument parsing error: {e}")))?;

    tracing::info!(?forge_args, "forge command started");

    // ── 2. Set up object store ──────────────────────────────────────────────
    let store_path_str = forge_args.store_path();
    let store_path = Path::new(&store_path_str);
    let store = ObjectStore::new(store_path)?;

    // ── 3. Export mode? (manifest not required) ──────────────────────────────
    if let Some(export_id) = &forge_args.export {
        let output_dir = forge_args
            .output_dir
            .as_ref()
            .map(Path::new)
            .ok_or_else(|| BkgCaveError::InvalidInput("--export requires --output-dir".into()))?;
        store.export(export_id, output_dir)?;
        tracing::info!(id = %export_id, output = %output_dir.display(), "Object exported");
        return Ok(0);
    }

    // ── 4. Load manifest ────────────────────────────────────────────────────
    let manifest: Manifest = if let Some(ref manifest_path) = forge_args.manifest {
        tracing::info!(path = %manifest_path, "Loading manifest from file");
        let json = std::fs::read_to_string(manifest_path)?;
        Manifest::from_json(&json)?
    } else {
        // Read from stdin.
        tracing::info!("Reading manifest from stdin");
        let mut json = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut json)?;
        if json.trim().is_empty() {
            return Err(BkgCaveError::InvalidInput(
                "no manifest provided (use --manifest or pipe JSON to stdin)".into(),
            ));
        }
        Manifest::from_json(&json)?
    };

    tracing::info!(pipelines = manifest.len(), "Manifest loaded successfully");

    // ── 5. Build module index and validate ──────────────────────────────────
    let libdir_str = forge_args.libdir_path();
    let libdir = Path::new(&libdir_str);
    let _index = if libdir.is_dir() {
        let idx = Index::new(libdir)?;
        idx.validate(&manifest)?;
        tracing::info!(
            stages = idx.stages.len(),
            assemblers = idx.assemblers.len(),
            runners = idx.runners.len(),
            "Module index loaded and manifest validated"
        );
        Some(idx)
    } else {
        tracing::warn!(libdir = %libdir.display(), "Module library not found — skipping validation");
        None
    };

    // ── 6. Download sources ─────────────────────────────────────────────────
    download_sources(&manifest, &store)?;

    // ── 7. Resolve pipeline order (topological sort) ────────────────────────
    let order = manifest.depsolve()?;
    tracing::info!(order = ?order, "Pipeline execution order");

    // ── 8. Execute each pipeline ────────────────────────────────────────────
    for pipeline_name in &order {
        let pipeline = manifest.get_pipeline(pipeline_name).ok_or_else(|| {
            BkgCaveError::NotFound(format!(
                "pipeline '{pipeline_name}' not found after depsolve"
            ))
        })?;

        tracing::info!(pipeline = %pipeline_name, "Starting pipeline");

        // Create the build root for this pipeline.
        let build_dir = store_path.join("build").join(pipeline_name);
        let build_root = BuildRoot::new(&build_dir)?;

        // ── 8a. Run each stage ──────────────────────────────────────────────
        for stage in &pipeline.stages {
            let stage_id = stage.id();
            tracing::info!(
                pipeline = %pipeline_name,
                stage = %stage.name,
                id = %stage_id,
                "Running stage"
            );

            // Check cache if checkpointing is enabled.
            if forge_args.checkpoint && store.contains(&stage_id) {
                tracing::debug!(id = %stage_id, "Stage result cached — skipping");
                continue;
            }

            // Build the stage command line.
            let stage_args = build_stage_command(stage)?;

            // Run inside the build root.
            let result = build_root.run(
                &stage_args,
                &[],  // writable binds
                &[],  // readonly binds
                &[],  // capabilities
                &[],  // environment
                None, // no timeout
            )?;

            if result.returncode != 0 {
                let stderr_str = String::from_utf8_lossy(&result.stderr);
                return Err(BkgCaveError::Execution {
                    command: format!("stage '{}' in pipeline '{}'", stage.name, pipeline_name),
                    exit_code: result.returncode,
                    stderr: stderr_str.to_string(),
                });
            }

            // If checkpointing, commit the build root tree to the store.
            if forge_args.checkpoint {
                let obj = store.new_object(None)?;
                // Copy the build-root tree into the object.
                cave_core::util::copy_recursively(&build_root.build_dir, &obj.dir)?;
                obj.finalize()?;
                store.commit(obj, &stage_id)?;
            }
        }

        // ── 8b. Run assembler if present ────────────────────────────────────
        if let Some(ref assembler) = pipeline.assembler {
            tracing::info!(
                pipeline = %pipeline_name,
                assembler = %assembler.name,
                "Running assembler"
            );

            let asm_args = build_assembler_command(assembler)?;
            let result = build_root.run(&asm_args, &[], &[], &[], &[], None)?;

            if result.returncode != 0 {
                let stderr_str = String::from_utf8_lossy(&result.stderr);
                return Err(BkgCaveError::Execution {
                    command: format!(
                        "assembler '{}' in pipeline '{}'",
                        assembler.name, pipeline_name
                    ),
                    exit_code: result.returncode,
                    stderr: stderr_str.to_string(),
                });
            }
        }

        tracing::info!(pipeline = %pipeline_name, "Pipeline completed");
    }

    // ── 9. Export final result if --output-dir was specified ────────────────
    if let Some(ref output_dir) = forge_args.output_dir {
        // Export the last pipeline's build result.
        if let Some(last_pipeline) = order.last() {
            let build_dir = store_path.join("build").join(last_pipeline);
            let output = Path::new(output_dir);
            if build_dir.is_dir() {
                if output.exists() {
                    std::fs::remove_dir_all(output)?;
                }
                cave_core::util::copy_recursively(&build_dir, output)?;
                tracing::info!(output = %output.display(), "Final result exported");
            } else {
                tracing::warn!("No build output found for last pipeline '{last_pipeline}'");
            }
        }
    }

    tracing::info!("Forge build complete");
    Ok(0)
}

/// Build the command-line arguments for a stage.
///
/// The command is constructed as:
/// `/usr/lib/forge/stages/<stage_name>/run <options_as_json>`
fn build_stage_command(stage: &crate::pipeline::Stage) -> Result<Vec<String>, BkgCaveError> {
    let mut args = Vec::new();

    // The stage runner script.
    let stage_path = format!("/usr/lib/forge/stages/{}/run", stage.name);
    args.push(stage_path);

    // Serialize options as JSON and pass as argument.
    let options_json = serde_json::to_string(&stage.options)?;
    args.push(options_json);

    Ok(args)
}

/// Build the command-line arguments for an assembler.
fn build_assembler_command(
    assembler: &crate::pipeline::Assembler,
) -> Result<Vec<String>, BkgCaveError> {
    let mut args = Vec::new();

    let assembler_path = format!("/usr/lib/forge/assemblers/{}/run", assembler.name);
    args.push(assembler_path);

    let options_json = serde_json::to_string(&assembler.options)?;
    args.push(options_json);

    Ok(args)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forge_plugin_id() {
        let plugin = ForgePlugin::new();
        assert_eq!(plugin.id(), "cave-forge");
    }

    #[test]
    fn forge_plugin_name() {
        let plugin = ForgePlugin::new();
        assert_eq!(plugin.name(), "Forge OS Image Builder");
    }

    #[test]
    fn forge_plugin_version() {
        let plugin = ForgePlugin::new();
        assert_eq!(plugin.version(), "0.1.0");
    }

    #[test]
    fn forge_plugin_kind() {
        let plugin = ForgePlugin::new();
        assert_eq!(plugin.kind(), PluginKind::Forge);
    }

    #[test]
    fn forge_plugin_hot_reloadable() {
        let plugin = ForgePlugin::new();
        assert!(!plugin.hot_reloadable());
    }

    #[test]
    fn forge_plugin_commands() {
        let plugin = ForgePlugin::new();
        let cmds = plugin.commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name, "forge");
        assert!(cmds[0].description.contains("Build OS images"));
    }

    #[test]
    fn forge_plugin_init() {
        let mut plugin = ForgePlugin::new();
        let ctx = PluginContext {
            config_dir: "/tmp/forge-test-config".into(),
            data_dir: "/tmp/forge-test-data".into(),
        };
        // Init should succeed.
        let result = plugin.init(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn execute_with_empty_args_fails() {
        let result = execute(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should be InvalidInput because no manifest was provided.
        let err_str = err.to_string();
        assert!(
            err_str.contains("no manifest") || err_str.contains("argument"),
            "unexpected error: {err_str}"
        );
    }

    #[test]
    fn execute_with_invalid_manifest_path() {
        let result = execute(&[
            "--manifest".to_string(),
            "/tmp/nonexistent-forge-manifest-12345.json".to_string(),
        ]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("no such file")
                || err_str.contains("I/O error")
                || err_str.contains("NotFound"),
            "unexpected error: {err_str}"
        );
    }

    #[test]
    fn execute_export_without_output_dir_fails() {
        let result = execute(&["--export".to_string(), "abc123".to_string()]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("--export requires --output-dir"),
            "unexpected error: {err_str}"
        );
    }

    #[test]
    fn build_stage_command_format() {
        use crate::pipeline::Stage;
        use std::collections::HashMap;

        let stage = Stage {
            name: "org.osbuild.curl".into(),
            build: None,
            options: {
                let mut m = HashMap::new();
                m.insert(
                    "url".into(),
                    serde_json::Value::String("https://example.com".into()),
                );
                m
            },
            inputs: HashMap::new(),
            devices: HashMap::new(),
            mounts: vec![],
        };

        let cmd = build_stage_command(&stage).unwrap();
        assert_eq!(cmd.len(), 2);
        assert!(cmd[0].contains("org.osbuild.curl"));
        assert!(cmd[1].contains("https://example.com"));
    }

    #[test]
    fn build_assembler_command_format() {
        use crate::pipeline::Assembler;
        use std::collections::HashMap;

        let assembler = Assembler {
            name: "org.osbuild.qemu".into(),
            options: {
                let mut m = HashMap::new();
                m.insert("format".into(), serde_json::Value::String("qcow2".into()));
                m
            },
            inputs: None,
            devices: None,
            mounts: None,
        };

        let cmd = build_assembler_command(&assembler).unwrap();
        assert_eq!(cmd.len(), 2);
        assert!(cmd[0].contains("org.osbuild.qemu"));
        assert!(cmd[1].contains("qcow2"));
    }

    #[test]
    fn default_plugin() {
        let plugin = ForgePlugin::default();
        assert_eq!(plugin.id(), "cave-forge");
    }
}
