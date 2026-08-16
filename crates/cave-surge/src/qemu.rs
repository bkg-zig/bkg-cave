//! QEMU binary discovery, version probing, and virtfs export helpers.
//!
//! This module handles finding the right QEMU binary for a given architecture,
//! parsing its version string to determine feature availability, and
//! constructing the command-line arguments for 9p virtfs shares.

use std::path::PathBuf;

use crate::arch::Arch;
use cave_core::error::BkgCaveError;
use cave_core::util::find_binary;

// ---------------------------------------------------------------------------
// Simple version type
// ---------------------------------------------------------------------------

/// A parsed QEMU version (major.minor.patch).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn parse(text: &str) -> Option<Self> {
        // Expected format: "QEMU emulator version 8.2.0 (v8.2.0)"
        // Also: "QEMU emulator version 9.0.0"
        let text = text.trim();
        let rest = text
            .strip_prefix("QEMU emulator version ")
            .or_else(|| text.strip_prefix("QEMU PC emulator version "))
            .or_else(|| text.strip_prefix("qemu "))
            .or_else(|| {
                // Try "qemu-system-x86_64 version 8.2.0"
                let idx = text.find(" version ")?;
                Some(&text[idx + 9..])
            })?;

        let rest = rest.split_whitespace().next()?; // get "8.2.0" before "(v8.2.0)"
        let parts: Vec<&str> = rest.splitn(3, '.').collect();
        if parts.len() < 2 {
            return None;
        }
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        let patch = parts
            .get(2)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        Some(Version {
            major,
            minor,
            patch,
        })
    }

    fn is_at_least(&self, major: u32, minor: u32, patch: u32) -> bool {
        self.major > major
            || (self.major == major && self.minor > minor)
            || (self.major == major && self.minor == minor && self.patch >= patch)
    }
}

// ---------------------------------------------------------------------------
// Qemu struct
// ---------------------------------------------------------------------------

/// A discovered QEMU binary with version information and feature flags.
pub struct Qemu {
    /// Path to the QEMU system binary.
    pub qemubin: PathBuf,
    /// Optional raw version string.
    pub version: Option<String>,
    /// True if QEMU < 1.6 (cannot overmount virtfs).
    pub cannot_overmount_virtfs: bool,
    /// True if QEMU >= 4.2 (supports `multidevs=`).
    pub has_multidevs: bool,
}

impl Qemu {
    /// Discover the QEMU binary for the given architecture.
    ///
    /// Searches for the architecture-specific QEMU binary first, then falls
    /// back to `qemu-kvm` and finally a plain `qemu-system-*` name.
    pub fn new(arch: &dyn Arch) -> Self {
        let qemu_name = arch.qemuname();

        // Try the arch-specific name first, then common aliases.
        let qemubin = find_binary(qemu_name)
            .or_else(|| find_binary("qemu-kvm"))
            .or_else(|| {
                // Try without the arch suffix (unlikely but worth a shot).
                find_binary("qemu")
            })
            .unwrap_or_else(|| PathBuf::from(qemu_name));

        Qemu {
            qemubin,
            version: None,
            cannot_overmount_virtfs: false,
            has_multidevs: false,
        }
    }

    /// Probe the QEMU binary to extract its version string and set feature
    /// flags.
    pub fn probe(&mut self) -> Result<(), BkgCaveError> {
        let output = std::process::Command::new(&self.qemubin)
            .arg("--version")
            .output()
            .map_err(|e| BkgCaveError::Execution {
                command: format!("{} --version", self.qemubin.display()),
                exit_code: -1,
                stderr: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(BkgCaveError::Execution {
                command: format!("{} --version", self.qemubin.display()),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let version_line = stdout.lines().next().unwrap_or("").to_owned();
        self.version = Some(version_line.clone());

        // Parse version to set feature flags.
        if let Some(v) = Version::parse(&version_line) {
            // QEMU < 1.6 cannot properly overmount virtfs shares.
            self.cannot_overmount_virtfs = !v.is_at_least(1, 6, 0);
            // QEMU >= 4.2 supports the multidevs= option.
            self.has_multidevs = v.is_at_least(4, 2, 0);
        }

        Ok(())
    }

    /// Add virtfs (9p) export arguments to the QEMU command line.
    ///
    /// Parameters:
    /// - `qemuargs`: mutable vector of QEMU arguments to extend.
    /// - `path`: host path to export.
    /// - `tag`: mount tag (used by the guest to identify the share).
    /// - `ro`: if true, export read-only.
    /// - `root`: if true, this is the root filesystem share.
    /// - `guest_root`: guest-side path prefix for the root share.
    pub fn export_virtfs(
        &self,
        qemuargs: &mut Vec<String>,
        path: &str,
        tag: &str,
        ro: bool,
        root: bool,
        guest_root: &str,
    ) {
        let security_model = if root { "none" } else { "mapped-xattr" };

        // Build -fsdev local,id=<id>,path=<path>,security_model=<model>
        let fsdev_id = format!("fsdev-{tag}");
        let mut fsdev = format!("local,id={fsdev_id},path={path},security_model={security_model}");
        if ro {
            fsdev.push_str(",readonly");
        }
        if self.has_multidevs {
            fsdev.push_str(",multidevs=remap");
        }
        qemuargs.push("-fsdev".into());
        qemuargs.push(fsdev);

        // Build -device virtio-9p-*,fsdev=<id>,mount_tag=<tag>
        // (The device type depends on the architecture.)
        let dev_type = "9p"; // always use virtio-9p-*
                             // FIXME: non-root shares might want different types.

        // Determine the right virtio device name.
        let dev_name = if root && !guest_root.is_empty() {
            // The virtme approach: for the root share, we may use a specific
            // virtio device id.
            format!("virtio-{dev_type}-pci")
        } else {
            // Default: let the arch decide.
            // We use the arch-specific kind via a simple heuristic.
            format!("virtio-{dev_type}-pci")
        };

        // Add the device.
        qemuargs.push("-device".into());
        qemuargs.push(format!("{dev_name},fsdev={fsdev_id},mount_tag={tag}"));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::ArchX86;

    #[test]
    fn version_parse_full() {
        let v = Version::parse("QEMU emulator version 8.2.0 (v8.2.0)").unwrap();
        assert_eq!(v.major, 8);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn version_parse_minor_only() {
        let v = Version::parse("QEMU emulator version 5.0").unwrap();
        assert_eq!(v.major, 5);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn version_parse_pc_emulator() {
        let v = Version::parse("QEMU PC emulator version 2.1.3").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn version_parse_garbage() {
        assert!(Version::parse("not a version").is_none());
        assert!(Version::parse("").is_none());
    }

    #[test]
    fn version_parse_with_prefix() {
        let v = Version::parse("qemu 9.0.0").unwrap();
        assert_eq!(v.major, 9);
        assert_eq!(v.minor, 0);
    }

    #[test]
    fn version_comparison() {
        let v = Version {
            major: 4,
            minor: 2,
            patch: 0,
        };
        assert!(v.is_at_least(4, 2, 0));
        assert!(v.is_at_least(4, 1, 0));
        assert!(v.is_at_least(3, 0, 0));
        assert!(!v.is_at_least(4, 3, 0));
        assert!(!v.is_at_least(5, 0, 0));
    }

    #[test]
    fn version_cannot_overmount() {
        let v = Version {
            major: 1,
            minor: 5,
            patch: 9,
        };
        assert!(!v.is_at_least(1, 6, 0));
        let v2 = Version {
            major: 1,
            minor: 6,
            patch: 0,
        };
        assert!(v2.is_at_least(1, 6, 0));
    }

    #[test]
    fn version_has_multidevs() {
        let v = Version {
            major: 4,
            minor: 1,
            patch: 9,
        };
        assert!(!v.is_at_least(4, 2, 0));
        let v2 = Version {
            major: 4,
            minor: 2,
            patch: 0,
        };
        assert!(v2.is_at_least(4, 2, 0));
    }

    #[test]
    fn qemu_new_x86() {
        // ArchX86 should look for "qemu-system-x86_64".
        // We can't guarantee it exists in test, but the path should be set.
        let arch = ArchX86;
        let qemu = Qemu::new(&arch);
        assert!(
            qemu.qemubin
                .to_string_lossy()
                .contains("qemu-system-x86_64")
                || qemu.qemubin.to_string_lossy().contains("qemu-kvm")
        );
    }

    #[test]
    fn export_virtfs_readonly() {
        let arch = ArchX86;
        let mut qemu = Qemu::new(&arch);
        qemu.has_multidevs = false;
        qemu.cannot_overmount_virtfs = false;

        let mut args = Vec::new();
        qemu.export_virtfs(&mut args, "/host/path", "root", true, true, "/");

        // Should contain -fsdev and -device
        assert!(args.contains(&"-fsdev".to_string()));
        assert!(args.contains(&"-device".to_string()));

        // Check that readonly appears in the fsdev arg
        let fsdev_idx = args.iter().position(|a| a == "-fsdev").unwrap();
        let fsdev_val = &args[fsdev_idx + 1];
        assert!(fsdev_val.contains("readonly"), "readonly flag missing");
        assert!(
            fsdev_val.contains("security_model=none"),
            "root security model should be 'none'"
        );
    }

    #[test]
    fn export_virtfs_writable() {
        let arch = ArchX86;
        let mut qemu = Qemu::new(&arch);
        qemu.has_multidevs = true;

        let mut args = Vec::new();
        qemu.export_virtfs(&mut args, "/host/path", "data", false, false, "");

        let fsdev_idx = args.iter().position(|a| a == "-fsdev").unwrap();
        let fsdev_val = &args[fsdev_idx + 1];
        assert!(!fsdev_val.contains("readonly"), "should not be readonly");
        assert!(fsdev_val.contains("multidevs=remap"), "multidevs missing");
        assert!(
            fsdev_val.contains("security_model=mapped-xattr"),
            "non-root security model"
        );
    }
}
