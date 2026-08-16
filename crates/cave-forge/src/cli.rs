//! Command-line argument parsing for `cave-forge`.
//!
//! Uses [`clap`](https://docs.rs/clap) with the derive API to define a
//! structured argument set matching osbuild's command-line interface.

/// CLI arguments for the forge pipeline builder.
///
/// These arguments control the manifest to build, the object store location,
/// caching behaviour, output paths, and debugging options.
#[derive(clap::Parser, Debug, Clone)]
#[command(name = "cave-forge", about = "Pipeline OS image builder")]
pub struct ForgeArgs {
    /// Path to the pipeline manifest JSON file.
    ///
    /// If not specified, the manifest is read from stdin.
    #[arg(long)]
    pub manifest: Option<String>,

    /// Path to the content-addressed object store.
    ///
    /// Defaults to `./forge-store/` if not specified.
    #[arg(long)]
    pub store: Option<String>,

    /// Directory to export the final assembled result into.
    ///
    /// If not set, the result is left in the store.
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Content ID of a specific object to export (instead of building).
    ///
    /// When set, no build is performed — the object is simply copied from the
    /// store to `output_dir`.
    #[arg(long)]
    pub export: Option<String>,

    /// Enable checkpointing: after each stage, commit the result tree to the
    /// store so it can be reused on subsequent runs.
    #[arg(long, default_value_t = false)]
    pub checkpoint: bool,

    /// Run stages inside a virtual machine instead of a bubblewrap sandbox.
    #[arg(long, default_value_t = false)]
    pub in_vm: bool,

    /// Path to the forge module library directory.
    ///
    /// Defaults to `/usr/lib/forge/modules/` if not specified.
    #[arg(long)]
    pub libdir: Option<String>,

    /// Maximum cache size (human-readable, e.g. "10G", "500M").
    ///
    /// When the store exceeds this size, old entries are evicted.
    #[arg(long)]
    pub cache_max: Option<String>,

    /// Monitoring mode.  One of: "log", "json", "prometheus".
    #[arg(long, default_value = "log")]
    pub monitor: String,

    /// Break (pause) before executing the first stage for debugging.
    #[arg(long, default_value_t = false)]
    pub r#break: bool,
}

impl ForgeArgs {
    /// Return the store directory path, falling back to the default.
    pub fn store_path(&self) -> String {
        self.store
            .clone()
            .unwrap_or_else(|| "./forge-store".to_string())
    }

    /// Return the libdir path, falling back to the default.
    pub fn libdir_path(&self) -> String {
        self.libdir
            .clone()
            .unwrap_or_else(|| "/usr/lib/forge/modules".to_string())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_defaults() {
        let args = ForgeArgs::try_parse_from(["cave-forge"]).unwrap();
        assert!(args.manifest.is_none());
        assert!(args.store.is_none());
        assert!(args.output_dir.is_none());
        assert!(args.export.is_none());
        assert!(!args.checkpoint);
        assert!(!args.in_vm);
        assert_eq!(args.monitor, "log");
        assert!(!args.r#break);
    }

    #[test]
    fn parse_manifest_flag() {
        let args =
            ForgeArgs::try_parse_from(["cave-forge", "--manifest", "/path/to/manifest.json"])
                .unwrap();
        assert_eq!(args.manifest.as_deref(), Some("/path/to/manifest.json"));
    }

    #[test]
    fn parse_store_flag() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--store", "/custom/store"]).unwrap();
        assert_eq!(args.store.as_deref(), Some("/custom/store"));
    }

    #[test]
    fn parse_output_dir_flag() {
        let args =
            ForgeArgs::try_parse_from(["cave-forge", "--output-dir", "/output/path"]).unwrap();
        assert_eq!(args.output_dir.as_deref(), Some("/output/path"));
    }

    #[test]
    fn parse_export_flag() {
        let args = ForgeArgs::try_parse_from([
            "cave-forge",
            "--export",
            "abc123def456",
            "--output-dir",
            "/out",
        ])
        .unwrap();
        assert_eq!(args.export.as_deref(), Some("abc123def456"));
    }

    #[test]
    fn parse_checkpoint_flag() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--checkpoint"]).unwrap();
        assert!(args.checkpoint);
    }

    #[test]
    fn parse_in_vm_flag() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--in-vm"]).unwrap();
        assert!(args.in_vm);
    }

    #[test]
    fn parse_libdir_flag() {
        let args =
            ForgeArgs::try_parse_from(["cave-forge", "--libdir", "/opt/forge/modules"]).unwrap();
        assert_eq!(args.libdir.as_deref(), Some("/opt/forge/modules"));
    }

    #[test]
    fn parse_cache_max() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--cache-max", "10G"]).unwrap();
        assert_eq!(args.cache_max.as_deref(), Some("10G"));
    }

    #[test]
    fn parse_monitor_flag() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--monitor", "json"]).unwrap();
        assert_eq!(args.monitor, "json");
    }

    #[test]
    fn parse_break_flag() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--break"]).unwrap();
        assert!(args.r#break);
    }

    #[test]
    fn store_path_default() {
        let args = ForgeArgs::try_parse_from(["cave-forge"]).unwrap();
        assert_eq!(args.store_path(), "./forge-store");
    }

    #[test]
    fn store_path_custom() {
        let args = ForgeArgs::try_parse_from(["cave-forge", "--store", "/data/store"]).unwrap();
        assert_eq!(args.store_path(), "/data/store");
    }

    #[test]
    fn libdir_path_default() {
        let args = ForgeArgs::try_parse_from(["cave-forge"]).unwrap();
        assert_eq!(args.libdir_path(), "/usr/lib/forge/modules");
    }

    #[test]
    fn libdir_path_custom() {
        let args =
            ForgeArgs::try_parse_from(["cave-forge", "--libdir", "/opt/forge/modules"]).unwrap();
        assert_eq!(args.libdir_path(), "/opt/forge/modules");
    }

    #[test]
    fn parse_all_flags() {
        let args = ForgeArgs::try_parse_from([
            "cave-forge",
            "--manifest",
            "m.json",
            "--store",
            "s",
            "--output-dir",
            "o",
            "--export",
            "abc",
            "--checkpoint",
            "--in-vm",
            "--libdir",
            "l",
            "--cache-max",
            "1G",
            "--monitor",
            "prometheus",
            "--break",
        ])
        .unwrap();

        assert_eq!(args.manifest.as_deref(), Some("m.json"));
        assert_eq!(args.store.as_deref(), Some("s"));
        assert_eq!(args.output_dir.as_deref(), Some("o"));
        assert_eq!(args.export.as_deref(), Some("abc"));
        assert!(args.checkpoint);
        assert!(args.in_vm);
        assert_eq!(args.libdir.as_deref(), Some("l"));
        assert_eq!(args.cache_max.as_deref(), Some("1G"));
        assert_eq!(args.monitor, "prometheus");
        assert!(args.r#break);
    }

    #[test]
    fn invalid_monitor_rejected() {
        // clap won't reject invalid monitor values since it's just a String.
        // This test verifies we accept any string.
        let args = ForgeArgs::try_parse_from(["cave-forge", "--monitor", "invalid-mode"]).unwrap();
        assert_eq!(args.monitor, "invalid-mode");
    }
}
