//! Command-line interface for `cave-kindle`.
//!
//! Uses `clap` derive to parse arguments passed to the `kindle` subcommand.

use clap::Parser;

/// Arguments for the `kindle` initramfs generator command.
#[derive(Debug, Clone, Parser)]
#[command(name = "kindle", about = "Generate an initramfs from TOML config")]
pub struct KindleArgs {
    /// Path to the TOML configuration file.
    #[arg(long, short = 'c', value_name = "FILE")]
    pub config: Option<String>,

    /// Skip base initramfs functionality (shebang, mounts, switch_root).
    #[arg(long)]
    pub no_base: bool,

    /// Only validate the configuration, do not build.
    #[arg(long)]
    pub validate_only: bool,

    /// Output path for the generated initramfs image.
    #[arg(long, short = 'o', value_name = "FILE")]
    pub output: Option<String>,

    /// Compression algorithm for the initramfs image.
    #[arg(long, default_value = "xz", value_name = "ALGO")]
    pub compress: String,

    /// Enable debug logging.
    #[arg(long)]
    pub debug: bool,

    /// Perform a dry run without writing the output.
    #[arg(long)]
    pub dry_run: bool,

    /// Directory containing module TOML files.
    #[arg(long, value_name = "DIR")]
    pub modules_dir: Option<String>,

    /// Additional binary to include in the initramfs (may be repeated).
    #[arg(long, value_name = "BIN")]
    pub include_binary: Vec<String>,

    /// Additional directory to include in the initramfs (may be repeated).
    #[arg(long, value_name = "DIR")]
    pub include_dir: Vec<String>,

    /// Additional file to include in the initramfs (may be repeated).
    #[arg(long, value_name = "FILE")]
    pub include_file: Vec<String>,
}

impl KindleArgs {
    /// Convert CLI arguments into a TOML value that can be merged into
    /// the [`InitramfsConfig`](crate::config::InitramfsConfig).
    pub fn to_toml(&self) -> toml::Value {
        let mut map = toml::value::Table::new();

        if let Some(ref config) = self.config {
            map.insert("config".into(), toml::Value::String(config.clone()));
        }
        map.insert("no_base".into(), toml::Value::Boolean(self.no_base));
        map.insert(
            "validate_only".into(),
            toml::Value::Boolean(self.validate_only),
        );
        if let Some(ref output) = self.output {
            map.insert("output".into(), toml::Value::String(output.clone()));
        }
        map.insert(
            "compress".into(),
            toml::Value::String(self.compress.clone()),
        );
        map.insert("debug".into(), toml::Value::Boolean(self.debug));
        map.insert("dry_run".into(), toml::Value::Boolean(self.dry_run));
        if let Some(ref modules_dir) = self.modules_dir {
            map.insert(
                "modules_dir".into(),
                toml::Value::String(modules_dir.clone()),
            );
        }
        if !self.include_binary.is_empty() {
            let arr: Vec<toml::Value> = self
                .include_binary
                .iter()
                .map(|s| toml::Value::String(s.clone()))
                .collect();
            map.insert("include_binary".into(), toml::Value::Array(arr));
        }
        if !self.include_dir.is_empty() {
            let arr: Vec<toml::Value> = self
                .include_dir
                .iter()
                .map(|s| toml::Value::String(s.clone()))
                .collect();
            map.insert("include_dir".into(), toml::Value::Array(arr));
        }
        if !self.include_file.is_empty() {
            let arr: Vec<toml::Value> = self
                .include_file
                .iter()
                .map(|s| toml::Value::String(s.clone()))
                .collect();
            map.insert("include_file".into(), toml::Value::Array(arr));
        }

        toml::Value::Table(map)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_args() {
        let args = KindleArgs::try_parse_from(["kindle"]).unwrap();
        assert!(!args.no_base);
        assert!(!args.validate_only);
        assert!(!args.debug);
        assert!(!args.dry_run);
        assert_eq!(args.compress, "xz");
        assert!(args.config.is_none());
        assert!(args.output.is_none());
    }

    #[test]
    fn parse_all_args() {
        let args = KindleArgs::try_parse_from([
            "kindle",
            "--config",
            "/etc/cave/kindle.toml",
            "--no-base",
            "--validate-only",
            "--output",
            "/boot/initramfs.cpio.zst",
            "--compress",
            "zstd",
            "--debug",
            "--dry-run",
            "--modules-dir",
            "/usr/share/cave-kindle/modules",
            "--include-binary",
            "/bin/busybox",
            "--include-binary",
            "/bin/sh",
            "--include-dir",
            "/lib/modules",
            "--include-file",
            "/etc/fstab",
        ])
        .unwrap();

        assert_eq!(args.config.as_deref(), Some("/etc/cave/kindle.toml"));
        assert!(args.no_base);
        assert!(args.validate_only);
        assert_eq!(args.output.as_deref(), Some("/boot/initramfs.cpio.zst"));
        assert_eq!(args.compress, "zstd");
        assert!(args.debug);
        assert!(args.dry_run);
        assert_eq!(
            args.modules_dir.as_deref(),
            Some("/usr/share/cave-kindle/modules")
        );
        assert_eq!(args.include_binary.len(), 2);
        assert_eq!(args.include_dir.len(), 1);
        assert_eq!(args.include_file.len(), 1);
    }

    #[test]
    fn to_toml_roundtrip() {
        let args = KindleArgs {
            config: Some("test.toml".into()),
            no_base: true,
            validate_only: false,
            output: Some("/tmp/out.cpio".into()),
            compress: "lz4".into(),
            debug: true,
            dry_run: false,
            modules_dir: Some("/modules".into()),
            include_binary: vec!["busybox".into()],
            include_dir: vec![],
            include_file: vec![],
        };

        let toml_value = args.to_toml();
        let table = toml_value.as_table().unwrap();

        assert_eq!(
            table.get("config").and_then(|v| v.as_str()),
            Some("test.toml")
        );
        assert_eq!(table.get("compress").and_then(|v| v.as_str()), Some("lz4"));
        assert_eq!(table.get("no_base").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(table.get("debug").and_then(|v| v.as_bool()), Some(true));
        assert!(table.contains_key("modules_dir"));
    }

    #[test]
    fn default_values() {
        let args = KindleArgs::try_parse_from(["kindle"]).unwrap();
        assert_eq!(args.compress, "xz");
        assert!(!args.no_base);
        assert!(!args.validate_only);
        assert!(!args.debug);
        assert!(!args.dry_run);
        assert!(args.config.is_none());
        assert!(args.output.is_none());
        assert!(args.modules_dir.is_none());
        assert!(args.include_binary.is_empty());
        assert!(args.include_dir.is_empty());
        assert!(args.include_file.is_empty());
    }

    #[test]
    fn short_opts_work() {
        let args =
            KindleArgs::try_parse_from(["kindle", "-c", "/cfg.toml", "-o", "/out.cpio"]).unwrap();
        assert_eq!(args.config.as_deref(), Some("/cfg.toml"));
        assert_eq!(args.output.as_deref(), Some("/out.cpio"));
    }
}
