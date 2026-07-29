//! [`GeneratorProtocol`] — the concrete [`InitramfsProtocol`] implementation
//! used by [`Generator`] during the build.
//!
//! Methods operate relative to the generator's `build_dir` so that hooks can
//! create files and directories in the initramfs staging tree.

use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tracing::debug;

use crate::config::InitramfsConfig;
use crate::error::KindleError;
use crate::generator::Generator;
use crate::protocol::InitramfsProtocol;

/// Bridges [`Generator`] and [`InitramfsProtocol`].
///
/// Wraps a mutable reference to a `Generator` so that protocol methods
/// can read config and manipulate the build directory.
pub struct GeneratorProtocol<'a> {
    /// The generator being driven.
    pub generator: &'a mut Generator,
}

impl InitramfsProtocol for GeneratorProtocol<'_> {
    fn config(&self) -> &InitramfsConfig {
        &self.generator.config
    }

    fn included_functions(&self) -> &HashMap<String, Vec<String>> {
        &self.generator.included_functions
    }

    fn run_hook(
        &self,
        name: &str,
        force_include: bool,
        force_exclude: &[&str],
    ) -> Result<Vec<String>, KindleError> {
        // We need to call HookRegistry::run which takes &self on the registry.
        // Access through the generator reference (immutable borrow).
        self.generator
            .hooks
            .run(name, self, force_include, force_exclude)
    }

    fn mkdir(&self, path: &Path) -> Result<(), KindleError> {
        let target = self.generator.build_dir.join(path);
        debug!(
            target = %target.display(),
            "GeneratorProtocol::mkdir"
        );
        std::fs::create_dir_all(&target).map_err(|e| {
            KindleError::Build(format!(
                "failed to create directory '{}': {e}",
                target.display()
            ))
        })
    }

    fn write(&self, name: &str, content: &str, mode: u32) -> Result<(), KindleError> {
        let target = self.generator.build_dir.join(name);
        debug!(
            target = %target.display(),
            mode = format!("{mode:o}"),
            "GeneratorProtocol::write"
        );

        // Ensure parent directory exists.
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                KindleError::Build(format!(
                    "failed to create parent directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        std::fs::write(&target, content).map_err(|e| {
            KindleError::Build(format!("failed to write '{}': {e}", target.display()))
        })?;

        // Set permissions.
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode)).map_err(|e| {
            KindleError::Build(format!(
                "failed to set permissions on '{}': {e}",
                target.display()
            ))
        })?;

        Ok(())
    }

    fn copy(&self, src: &Path, dst: &Path) -> Result<(), KindleError> {
        let target = self.generator.build_dir.join(dst);
        debug!(
            src = %src.display(),
            dst = %target.display(),
            "GeneratorProtocol::copy"
        );

        // Ensure parent directory exists.
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                KindleError::Build(format!(
                    "failed to create parent directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        if src.is_dir() {
            cave_core::util::copy_recursively(src, &target).map_err(|e| {
                KindleError::Build(format!("failed to copy directory '{}': {e}", src.display()))
            })?;
        } else {
            std::fs::copy(src, &target).map_err(|e| {
                KindleError::Build(format!("failed to copy '{}': {e}", src.display()))
            })?;
        }

        Ok(())
    }

    fn symlink(&self, target: &str, link: &Path) -> Result<(), KindleError> {
        let link_path = self.generator.build_dir.join(link);
        debug!(
            target = %target,
            link = %link_path.display(),
            "GeneratorProtocol::symlink"
        );

        // Ensure parent directory exists.
        if let Some(parent) = link_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                KindleError::Build(format!(
                    "failed to create parent directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        // Remove existing file/link if present.
        if link_path.exists() || link_path.is_symlink() {
            std::fs::remove_file(&link_path).map_err(|e| {
                KindleError::Build(format!(
                    "failed to remove existing link '{}': {e}",
                    link_path.display()
                ))
            })?;
        }

        std::os::unix::fs::symlink(target, &link_path).map_err(|e| {
            KindleError::Build(format!(
                "failed to create symlink '{}' → '{target}': {e}",
                link_path.display()
            ))
        })?;

        Ok(())
    }

    fn run(&self, cmd: &str) -> Result<String, KindleError> {
        debug!(cmd = %cmd, "GeneratorProtocol::run");
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| KindleError::Build(format!("failed to execute command '{cmd}': {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(KindleError::Build(format!(
                "command '{cmd}' failed with exit code {}: {stderr}",
                output.status.code().unwrap_or(-1)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InitramfsConfig;
    use crate::hooks::HookFn;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Create a minimal generator for testing.
    fn test_generator() -> (TempDir, TempDir, Generator) {
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
    fn protocol_mkdir_creates_directory() {
        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        proto.mkdir(Path::new("sub/dir")).unwrap();
        let created = gen.build_dir.join("sub/dir");
        assert!(created.is_dir());
    }

    #[test]
    fn protocol_write_creates_file() {
        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        proto.write("test.txt", "hello world", 0o644).unwrap();
        let path = gen.build_dir.join("test.txt");
        assert!(path.is_file());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");

        // Check permissions.
        let meta = std::fs::metadata(&path).unwrap();
        #[cfg(unix)]
        assert_eq!(meta.permissions().mode() & 0o777, 0o644);
    }

    #[test]
    fn protocol_write_creates_parent_dirs() {
        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        proto.write("a/b/c/nested.txt", "nested", 0o644).unwrap();
        let path = gen.build_dir.join("a/b/c/nested.txt");
        assert!(path.is_file());
    }

    #[test]
    fn protocol_copy_file() {
        let src_dir = TempDir::new().unwrap();
        let src_file = src_dir.path().join("source.txt");
        std::fs::write(&src_file, "src content").unwrap();

        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        proto.copy(&src_file, Path::new("dest.txt")).unwrap();
        let dest = gen.build_dir.join("dest.txt");
        assert!(dest.is_file());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "src content");
    }

    #[test]
    fn protocol_symlink_creates_link() {
        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        proto.symlink("/target/path", Path::new("mylink")).unwrap();
        let link = gen.build_dir.join("mylink");
        assert!(link.is_symlink());
        let target = std::fs::read_link(&link).unwrap();
        assert_eq!(target, PathBuf::from("/target/path"));
    }

    #[test]
    fn protocol_run_executes_command() {
        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        let output = proto.run("echo hello").unwrap();
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn protocol_run_failing_command() {
        let (_b, _o, mut gen) = test_generator();
        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        let result = proto.run("false");
        assert!(result.is_err());
    }

    #[test]
    fn protocol_config_returns_config() {
        let mut cfg = InitramfsConfig::new();
        cfg.set("test_key", toml::Value::String("val".into()));

        let build_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let mut gen = Generator::new(cfg, build_dir.path().to_owned(), out_dir.path().to_owned());

        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        let val: Option<String> = proto.config().get("test_key").unwrap();
        assert_eq!(val, Some("val".into()));
    }

    #[test]
    fn protocol_run_hook_invokes_registered_hooks() {
        let build_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let mut gen = Generator::new(
            InitramfsConfig::new(),
            build_dir.path().to_owned(),
            out_dir.path().to_owned(),
        );

        // Register a test hook.
        gen.hooks.register(
            "test_hook",
            HookFn {
                name: "test_fn".into(),
                module: "test".into(),
                func: |_p| Ok(Some(vec!["echo 'from hook'".into()])),
                before: vec![],
                after: vec![],
            },
        );

        let proto = GeneratorProtocol {
            generator: &mut gen,
        };

        let lines = proto.run_hook("test_hook", false, &[]).unwrap();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("from hook"));
    }
}
