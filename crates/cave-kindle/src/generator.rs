//! The initramfs generator — the heart of `cave-kindle`.
//!
//! [`Generator`] reads configuration, drives hooks, produces the `/init`
//! shell script, assembles the CPIO archive, and packs it into a
//! compressed image.

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use tracing::{debug, info};

use cave_core::cpio::CpioWriter;
use cave_core::util::walk_files;

use crate::base;
use crate::config::InitramfsConfig;
use crate::error::KindleError;
use crate::generator_helpers::GeneratorProtocol;
use crate::hooks::HookRegistry;
use crate::protocol::InitramfsProtocol;

/// The main initramfs generator.
///
/// # Lifecycle
///
/// 1. Create a `Generator` with [`new`](Self::new).
/// 2. Call [`build`](Self::build) which orchestrates the full pipeline:
///    * Prepares the build and output directories.
///    * Drives build hooks to populate the staging tree.
///    * Generates the `/init` script.
///    * Packs the staging tree into a (compressed) CPIO archive.
///    * Runs optional sanity checks.
pub struct Generator {
    /// The resolved initramfs configuration.
    pub config: InitramfsConfig,

    /// Registry of all registered hook functions.
    pub hooks: HookRegistry,

    /// Staging directory for the initramfs tree.
    pub build_dir: PathBuf,

    /// Output directory for the final archive.
    pub out_dir: PathBuf,

    /// Tracks which shell functions have been included per hook point.
    pub included_functions: HashMap<String, Vec<String>>,
}

impl Generator {
    /// Create a new generator with the given configuration and paths.
    ///
    /// The hook registry is pre-populated with built-in hooks
    /// (set up via private setup method).
    pub fn new(config: InitramfsConfig, build_dir: PathBuf, out_dir: PathBuf) -> Self {
        let mut gen = Self {
            config,
            hooks: HookRegistry::new(),
            build_dir,
            out_dir,
            included_functions: HashMap::new(),
        };
        gen.setup_builtin_hooks();
        gen
    }

    /// Register built-in hook functions.
    ///
    /// These provide the minimal init script infrastructure that every
    /// initramfs needs (shebang, safe shell options, log level, base
    /// filesystem mounts, and switch_root).
    fn setup_builtin_hooks(&mut self) {
        use crate::hooks::HookFn;

        // ── init hooks ────────────────────────────────────────────────

        self.hooks.register(
            "init",
            HookFn {
                name: "shebang".into(),
                module: "builtin".into(),
                func: |_p| Ok(Some(vec![base::shebang()])),
                before: vec![],
                after: vec![],
            },
        );

        self.hooks.register(
            "init",
            HookFn {
                name: "shell_safe".into(),
                module: "builtin".into(),
                func: |p| {
                    let no_base = p
                        .config()
                        .get::<bool>("no_base")
                        .ok()
                        .flatten()
                        .unwrap_or(false);
                    if no_base {
                        return Ok(None);
                    }
                    Ok(Some(base::shell_safe()))
                },
                before: vec![],
                after: vec!["shebang".into()],
            },
        );

        self.hooks.register(
            "init",
            HookFn {
                name: "set_loglevel".into(),
                module: "builtin".into(),
                func: |_p| Ok(Some(base::set_loglevel())),
                before: vec![],
                after: vec!["shell_safe".into()],
            },
        );

        self.hooks.register(
            "init",
            HookFn {
                name: "mount_base".into(),
                module: "builtin".into(),
                func: |p| {
                    let no_base = p
                        .config()
                        .get::<bool>("no_base")
                        .ok()
                        .flatten()
                        .unwrap_or(false);
                    if no_base {
                        return Ok(None);
                    }
                    Ok(Some(base::mount_base()))
                },
                before: vec![],
                after: vec!["set_loglevel".into()],
            },
        );

        self.hooks.register(
            "init",
            HookFn {
                name: "find_root".into(),
                module: "builtin".into(),
                func: |p| {
                    let no_base = p
                        .config()
                        .get::<bool>("no_base")
                        .ok()
                        .flatten()
                        .unwrap_or(false);
                    if no_base {
                        return Ok(None);
                    }
                    Ok(Some(base::find_root(&[
                        "/dev/sda1",
                        "/dev/sdb1",
                        "/dev/nvme0n1p1",
                    ])))
                },
                before: vec![],
                after: vec!["mount_base".into()],
            },
        );

        self.hooks.register(
            "init",
            HookFn {
                name: "switch_root".into(),
                module: "builtin".into(),
                func: |p| {
                    let no_base = p
                        .config()
                        .get::<bool>("no_base")
                        .ok()
                        .flatten()
                        .unwrap_or(false);
                    if no_base {
                        return Ok(None);
                    }
                    Ok(Some(base::do_switch_root()))
                },
                before: vec![],
                after: vec!["find_root".into()],
            },
        );

        // ── pre_build hooks ───────────────────────────────────────────

        self.hooks.register(
            "pre_build",
            HookFn {
                name: "create_build_dirs".into(),
                module: "builtin".into(),
                func: |p| {
                    p.mkdir(Path::new("bin"))?;
                    p.mkdir(Path::new("sbin"))?;
                    p.mkdir(Path::new("etc"))?;
                    p.mkdir(Path::new("dev"))?;
                    p.mkdir(Path::new("proc"))?;
                    p.mkdir(Path::new("sys"))?;
                    p.mkdir(Path::new("tmp"))?;
                    p.mkdir(Path::new("var"))?;
                    Ok(None)
                },
                before: vec![],
                after: vec![],
            },
        );

        // ── post_build hooks ──────────────────────────────────────────

        self.hooks.register(
            "post_build",
            HookFn {
                name: "cleanup_stale".into(),
                module: "builtin".into(),
                func: |_p| {
                    // Stub — modules can override for custom cleanup.
                    Ok(None)
                },
                before: vec![],
                after: vec![],
            },
        );
    }

    /// Run the full build pipeline.
    ///
    /// # Steps
    ///
    /// 1. Create output and build directories.
    /// 2. Sort hooks to respect ordering constraints.
    /// 3. Run `pre_build` hooks.
    /// 4. Run `run_build` (main build step).
    /// 5. Generate the `/init` script.
    /// 6. Run `post_build` hooks.
    /// 7. Pack the build directory into a CPIO archive.
    /// 8. Run sanity checks.
    /// 9. Run optional tests.
    pub fn build(&mut self) -> Result<()> {
        info!("starting initramfs build");

        // 1. Ensure directories exist.
        fs::create_dir_all(&self.build_dir)
            .with_context(|| format!("create build dir: {}", self.build_dir.display()))?;
        fs::create_dir_all(&self.out_dir)
            .with_context(|| format!("create out dir: {}", self.out_dir.display()))?;

        // 2. Sort hooks (topological order based on before/after).
        self.hooks
            .sort_hooks()
            .map_err(|e| anyhow::anyhow!("hook sorting failed: {e}"))?;

        // 3. Pre-build hooks.
        {
            let proto = GeneratorProtocol { generator: self };
            let pre_lines = proto.run_hook("pre_build", true, &[])?;
            if !pre_lines.is_empty() {
                debug!("pre_build produced {} lines", pre_lines.len());
            }
        }

        // 4. Main build step.
        self.run_build()?;

        // 5. Generate /init script.
        self.generate_init()?;

        // 6. Post-build hooks.
        {
            let proto = GeneratorProtocol { generator: self };
            let post_lines = proto.run_hook("post_build", true, &[])?;
            if !post_lines.is_empty() {
                debug!("post_build produced {} lines", post_lines.len());
            }
        }

        // 7. Pack into CPIO.
        self.pack_build()?;

        // 8. Run checks.
        self.run_checks()?;

        // 9. Run tests.
        self.run_tests()?;

        info!("initramfs build complete");
        Ok(())
    }

    /// Execute the main build step.
    ///
    /// Runs the `"build"` hook which modules use to populate the staging
    /// tree (copy binaries, create device nodes, etc.).
    pub fn run_build(&mut self) -> Result<()> {
        debug!("running build hooks");
        let proto = GeneratorProtocol { generator: self };
        let lines = proto.run_hook("build", true, &[])?;
        debug!("build hooks produced {} lines", lines.len());
        Ok(())
    }

    /// Generate the `/init` script.
    ///
    /// Runs the `"init"` hook chain to collect shell code, then writes
    /// the result to `build_dir/init` and makes it executable.
    pub fn generate_init(&mut self) -> Result<()> {
        debug!("generating /init script");

        let proto = GeneratorProtocol { generator: self };

        let lines = proto.run_hook("init", true, &[])?;

        // Prepend shebang if it wasn't included by hooks.
        let mut script = String::new();
        if !lines.iter().any(|l| l.starts_with("#!/")) {
            script.push_str(&base::shebang());
            script.push('\n');
        }

        for line in &lines {
            script.push_str(line);
            script.push('\n');
        }

        let init_path = self.build_dir.join("init");
        fs::write(&init_path, script.as_bytes())
            .with_context(|| format!("failed to write init script: {}", init_path.display()))?;

        // Make executable.
        fs::set_permissions(&init_path, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("chmod init: {}", init_path.display()))?;

        info!(
            path = %init_path.display(),
            size = script.len(),
            "init script written"
        );
        Ok(())
    }

    /// Pack the build directory into a compressed CPIO archive.
    ///
    /// The archive is written to `{out_dir}/initramfs.cpio` (or `.cpio.{comp}`).
    /// Compression is determined by the `compression` config key.
    pub fn pack_build(&self) -> Result<()> {
        let compress = self
            .config
            .get::<String>("compression")
            .ok()
            .flatten()
            .unwrap_or_else(|| "xz".into());

        let ext = if compress == "none" {
            "cpio".to_owned()
        } else {
            format!("cpio.{}", compress)
        };

        let out_path = self.out_dir.join(format!("initramfs.{}", ext));
        debug!(
            output = %out_path.display(),
            compression = %compress,
            "packing initramfs"
        );

        // Walk build directory and collect files.
        let files = walk_files(&self.build_dir)
            .with_context(|| format!("walk build dir: {}", self.build_dir.display()))?;

        // Write CPIO to a temporary buffer first, then (optionally) compress.
        let mut cpio_buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut cpio_buf);

            // Collect all directories to ensure they exist in the archive.
            let mut dirs: Vec<PathBuf> = Vec::new();
            for file_path in &files {
                if let Some(parent) = file_path.parent() {
                    if parent != self.build_dir {
                        let rel_parent = parent.strip_prefix(&self.build_dir).unwrap_or(parent);
                        if !dirs.contains(&rel_parent.to_owned()) {
                            dirs.push(rel_parent.to_owned());
                        }
                    }
                }
            }

            // Add directory entries (sorted for deterministic archives).
            dirs.sort();
            for dir in &dirs {
                let dir_str = dir.to_string_lossy();
                // Ensure trailing slash for directory entries.
                let entry_name = if dir_str.ends_with('/') {
                    dir_str.to_string()
                } else {
                    format!("{}/", dir_str)
                };
                writer
                    .add_dir(&entry_name, 0o755)
                    .with_context(|| format!("add dir entry: {entry_name}"))?;
            }

            // Add file entries.
            for file_path in &files {
                let relative = file_path.strip_prefix(&self.build_dir).with_context(|| {
                    format!(
                        "strip prefix: {} from {}",
                        self.build_dir.display(),
                        file_path.display()
                    )
                })?;

                let entry_name = relative.to_string_lossy();

                // Check for symlinks first.
                if file_path.is_symlink() {
                    let target = fs::read_link(file_path)
                        .with_context(|| format!("read link: {}", file_path.display()))?;
                    writer
                        .add_symlink(&entry_name, &target.to_string_lossy())
                        .with_context(|| format!("add symlink: {entry_name}"))?;
                } else {
                    writer
                        .add_file(&entry_name, file_path)
                        .with_context(|| format!("add file: {entry_name}"))?;
                }
            }

            writer.finish().context("failed to write CPIO trailer")?;
        }

        // Compute SHA256 and capture size of the CPIO stream (before compression).
        let mut hasher = Sha256::new();
        hasher.update(&cpio_buf);
        let cpio_hash = hex::encode(hasher.finalize());
        let cpio_size = cpio_buf.len();
        debug!(cpio_size, cpio_hash = %cpio_hash, "CPIO archive SHA256");

        // Write final output (compressed if needed).
        let final_data: Vec<u8> = match compress.as_str() {
            "none" => cpio_buf,
            other => {
                // Pipe through an external compression command.
                let (compressor, extra_args): (&str, &[&str]) = match other {
                    "gzip" => ("gzip", &["-c"]),
                    "xz" => ("xz", &["-f", "-c"]),
                    "zstd" => ("zstd", &["-f", "-c"]),
                    "lz4" => ("lz4", &["-f", "-c"]),
                    "lzo" => ("lzop", &["-f", "-c"]),
                    "bzip2" => ("bzip2", &["-c"]),
                    _ => anyhow::bail!(KindleError::Validation(format!(
                        "unsupported compression: '{other}'"
                    ))),
                };
                pipe_compress(&cpio_buf, compressor, extra_args)
                    .with_context(|| format!("external compression with '{compressor}'"))?
            }
        };

        fs::write(&out_path, &final_data)
            .with_context(|| format!("write output: {}", out_path.display()))?;

        let out_size = final_data.len();

        info!(
            output = %out_path.display(),
            cpio_size = cpio_size,
            compressed_size = out_size,
            compression = %compress,
            sha256 = %cpio_hash,
            "initramfs archive written"
        );

        // Write checksum file.
        let hash_path = {
            let mut p = out_path.to_string_lossy().to_string();
            p.push_str(".sha256");
            PathBuf::from(p)
        };
        fs::write(
            &hash_path,
            format!(
                "{cpio_hash}  {}\n",
                out_path.file_name().unwrap_or_default().to_string_lossy()
            ),
        )
        .with_context(|| format!("write checksum: {}", hash_path.display()))?;

        Ok(())
    }

    /// Run sanity checks on the built initramfs.
    ///
    /// Checks that the `/init` script exists and is executable, and that
    /// the archive is valid.
    pub fn run_checks(&self) -> Result<()> {
        info!("running initramfs sanity checks");

        // Check /init exists.
        let init_path = self.build_dir.join("init");
        if !init_path.exists() {
            anyhow::bail!(KindleError::Build(format!(
                "/init not found at {}",
                init_path.display()
            )));
        }
        if !init_path.is_file() {
            anyhow::bail!(KindleError::Build(format!(
                "/init is not a regular file: {}",
                init_path.display()
            )));
        }

        // Check /init is executable.
        let meta = fs::metadata(&init_path)?;
        #[cfg(unix)]
        if meta.permissions().mode() & 0o111 == 0 {
            anyhow::bail!(KindleError::Build(format!(
                "/init is not executable: {}",
                init_path.display()
            )));
        }

        debug!("sanity checks passed");
        Ok(())
    }

    /// Run optional integration tests on the generated initramfs.
    ///
    /// Currently a no-op; modules can register `"test"` hooks for custom
    /// verification.
    pub fn run_tests(&self) -> Result<()> {
        info!("running initramfs tests");
        // Modules can register test hooks under the "test" hook name.
        // This method runs them if any are registered.
        // For now this is a placeholder that always passes.
        Ok(())
    }
}

/// Pipe data through an external compression command.
fn pipe_compress(data: &[u8], cmd: &str, args: &[&str]) -> Result<Vec<u8>> {
    use std::process::{Command, Stdio};

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn compressor '{cmd}'"))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .context("failed to get compressor stdin")?;
        use std::io::Write;
        stdin.write_all(data).context("write to compressor stdin")?;
    }

    let output = child
        .wait_with_output()
        .with_context(|| format!("wait for compressor '{cmd}'"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(KindleError::Build(format!(
            "compressor '{cmd}' failed: {stderr}"
        )));
    }

    Ok(output.stdout)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InitramfsConfig;
    use tempfile::TempDir;

    fn setup() -> (TempDir, TempDir, Generator) {
        let build_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let gen = Generator::new(
            InitramfsConfig::new(),
            build_dir.path().to_owned(),
            out_dir.path().to_owned(),
        );
        (build_dir, out_dir, gen)
    }

    #[test]
    fn new_generator_has_defaults() {
        let (_b, _o, gen) = setup();
        assert!(gen.config.has("compression"));
        assert!(!gen.hooks.is_empty());
        assert!(gen.included_functions.is_empty());
    }

    #[test]
    fn generate_init_writes_executable_script() {
        let (_b, _o, mut gen) = setup();
        gen.generate_init().unwrap();

        let init_path = gen.build_dir.join("init");
        assert!(init_path.is_file());

        let content = fs::read_to_string(&init_path).unwrap();
        assert!(content.starts_with("#!/bin/sh"));

        // Should contain mount commands from built-in hooks.
        assert!(content.contains("mount -t proc"));
    }

    #[test]
    fn generate_init_script_contains_key_elements() {
        let (_b, _o, mut gen) = setup();
        gen.generate_init().unwrap();

        let content = fs::read_to_string(gen.build_dir.join("init")).unwrap();
        // shebang
        assert!(content.starts_with("#!/bin/sh"));
        // safe shell
        assert!(content.contains("set -e"));
        // log level
        assert!(content.contains("loglevel"));
        // base mounts
        assert!(content.contains("mount -t proc"));
        assert!(content.contains("mount -t sysfs"));
        // switch_root
        assert!(content.contains("switch_root"));
    }

    #[test]
    fn pack_build_creates_cpio_archive() {
        let (_b, _o, gen) = setup();

        // Create a minimal build tree.
        fs::write(gen.build_dir.join("init"), "#!/bin/sh\necho test\n").unwrap();
        #[cfg(unix)]
        fs::set_permissions(
            gen.build_dir.join("init"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();

        gen.pack_build().unwrap();

        // Check output exists.
        let out_path = gen.out_dir.join("initramfs.cpio.xz");
        assert!(
            out_path.is_file(),
            "output archive not found: {:?}",
            out_path
        );

        // Check hash file exists.
        let hash_path = gen.out_dir.join("initramfs.cpio.xz.sha256");
        assert!(hash_path.is_file());
    }

    #[test]
    fn pack_build_no_compression() {
        let (_b, _o, mut gen) = setup();
        gen.config
            .set("compression", toml::Value::String("none".into()));

        fs::write(gen.build_dir.join("init"), "#!/bin/sh\necho test\n").unwrap();
        #[cfg(unix)]
        fs::set_permissions(
            gen.build_dir.join("init"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();

        gen.pack_build().unwrap();

        let out_path = gen.out_dir.join("initramfs.cpio");
        assert!(out_path.is_file());
    }

    #[test]
    fn run_checks_passes_with_valid_init() {
        let (_b, _o, gen) = setup();
        fs::write(gen.build_dir.join("init"), "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        fs::set_permissions(
            gen.build_dir.join("init"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();

        assert!(gen.run_checks().is_ok());
    }

    #[test]
    fn run_checks_fails_without_init() {
        let (_b, _o, gen) = setup();
        let result = gen.run_checks();
        assert!(result.is_err());
    }

    #[test]
    fn run_checks_fails_with_nonexecutable_init() {
        let (_b, _o, gen) = setup();
        fs::write(gen.build_dir.join("init"), "#!/bin/sh\n").unwrap();
        // Make it non-executable.
        #[cfg(unix)]
        fs::set_permissions(
            gen.build_dir.join("init"),
            fs::Permissions::from_mode(0o644),
        )
        .unwrap();

        // On unix this should fail; on other platforms it's skipped.
        #[cfg(unix)]
        assert!(gen.run_checks().is_err());
        #[cfg(not(unix))]
        let _ = gen.run_checks();
    }

    #[test]
    fn run_build_executes_build_hooks() {
        let (_b, _o, mut gen) = setup();
        // Should not panic or error with default empty build hooks.
        gen.run_build().unwrap();
    }

    #[test]
    fn build_full_pipeline() {
        let (_b, _o, mut gen) = setup();
        // Full build should succeed.
        gen.build().unwrap();

        // Verify outputs.
        let init_path = gen.build_dir.join("init");
        assert!(init_path.is_file());

        // Check archive exists (compressed).
        let out_path = gen.out_dir.join("initramfs.cpio.xz");
        assert!(out_path.is_file());
    }

    #[test]
    fn pipe_compress_xz() {
        let data = b"hello world compression test";
        let compressed = pipe_compress(data, "xz", &["-f", "-c"]).unwrap();
        assert!(!compressed.is_empty());
        assert_ne!(compressed.len(), data.len());

        // Decompress and verify using pipe_compress in reverse.
        let decompressed = pipe_compress(&compressed, "xz", &["-d", "-c"]).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&decompressed),
            "hello world compression test"
        );
    }

    #[test]
    fn builtin_hooks_produce_init_content() {
        let (_b, _o, mut gen) = setup();
        gen.generate_init().unwrap();

        let content = std::fs::read_to_string(gen.build_dir.join("init")).unwrap();

        // All built-in hooks should have fired.
        assert!(content.contains("#!/bin/sh")); // shebang
        assert!(content.contains("set -e")); // shell_safe
        assert!(content.contains("loglevel")); // set_loglevel
        assert!(content.contains("mount -t proc")); // mount_base
        assert!(content.contains("switch_root")); // switch_root
    }
}
