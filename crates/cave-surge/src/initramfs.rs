//! Initramfs builder — constructs a minimal CPIO archive that can serve as
//! a Linux initramfs for booting via QEMU with a 9p root filesystem.
//!
//! The generated initramfs contains:
//!
//! * Basic directory layout (`/dev`, `/proc`, `/sys`, `/tmp`, `/etc`,
//!   `/sbin`, `/bin`, `/usr`, …).
//! * Device nodes (`/dev/null`, `/dev/kmsg`, `/dev/console`).
//! * A `busybox` binary with common utility symlinks.
//! * A minimal `/init` script that mounts 9p and switches root.
//! * Optional kernel modules in `/lib/modules/<kver>/`.

use std::io::Write;
use std::path::{Path, PathBuf};

use cave_core::cpio::{CpioEntryType, CpioWriter};
use cave_core::error::BkgCaveError;
use cave_core::util::find_binary;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Access mode for the 9p root filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Read-only root mount (overlayfs for writable paths).
    Ro,
    /// Read-write root mount.
    Rw,
}

/// Configuration for building an initramfs CPIO archive.
pub struct InitramfsConfig<'a> {
    /// List of kernel module files to include (paths relative to the module
    /// root, e.g. `"kernel/fs/9p/9p.ko"`).
    pub modfiles: &'a [String],
    /// Path to the virtme data directory (used for module storage).
    pub virtme_data: &'a Path,
    /// Optional path to a busybox binary. If `None`, auto-detect.
    pub busybox: Option<PathBuf>,
    /// Whether to mount the root filesystem read-only or read-write.
    pub access: AccessMode,
    /// The guest-side root path (usually `"/"`).
    pub guest_root: &'a str,
}

// ---------------------------------------------------------------------------
// Busybox discovery
// ---------------------------------------------------------------------------

/// Find a suitable busybox binary.
///
/// Searches:
/// 1. `root` + the provided path (if `root` is `/`, this is the absolute path).
/// 2. `PATH` for `busybox`.
/// 3. Common system locations (`/usr/bin/busybox`, `/bin/busybox`).
pub fn find_busybox(root: &Path, _is_native: bool) -> Option<PathBuf> {
    // Check common locations first.
    let candidates = [
        root.join("usr").join("bin").join("busybox"),
        root.join("bin").join("busybox"),
        root.join("sbin").join("busybox"),
        root.join("usr").join("sbin").join("busybox"),
    ];

    for c in &candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }

    // Search PATH.
    if let Some(p) = find_binary("busybox") {
        return Some(p);
    }

    None
}

// ---------------------------------------------------------------------------
// Layout builders
// ---------------------------------------------------------------------------

/// Create the base directory layout in the CPIO archive.
pub fn make_base_layout(cw: &mut CpioWriter<impl Write>) -> Result<(), BkgCaveError> {
    let dirs = [
        "dev", "proc", "sys", "tmp", "etc", "sbin", "bin", "lib", "usr", "usr/bin", "usr/sbin",
        "usr/lib", "root", "mnt", "run",
    ];
    for d in &dirs {
        cw.add_dir(d, 0o755)?;
    }
    // /lib/modules must exist for module loading.
    cw.add_dir("lib/modules", 0o755)?;
    Ok(())
}

/// Create essential device nodes in the CPIO archive.
pub fn make_dev_nodes(cw: &mut CpioWriter<impl Write>) -> Result<(), BkgCaveError> {
    cw.add_chardev("dev/null", 1, 3, 0o666)?;
    cw.add_chardev("dev/kmsg", 1, 11, 0o600)?;
    cw.add_chardev("dev/console", 5, 1, 0o600)?;
    cw.add_chardev("dev/zero", 1, 5, 0o666)?;
    cw.add_chardev("dev/random", 1, 8, 0o644)?;
    cw.add_chardev("dev/urandom", 1, 9, 0o644)?;
    cw.add_chardev("dev/tty", 5, 0, 0o666)?;
    // Add /dev/ptmx for pseudo-terminals.
    cw.add_chardev("dev/ptmx", 5, 2, 0o666)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Binary installation
// ---------------------------------------------------------------------------

/// Install a busybox binary into the CPIO archive and create the standard
/// set of BusyBox applet symlinks.
pub fn install_busybox(
    cw: &mut CpioWriter<impl Write>,
    root: &Path,
    busybox: &Path,
) -> Result<(), BkgCaveError> {
    // Resolve the busybox path relative to root.
    let bb_path = if busybox.is_absolute() {
        busybox.to_owned()
    } else {
        root.join(busybox)
    };

    cw.add_file("bin/busybox", &bb_path)?;

    // Create symlinks for common applets.
    // The busybox binary detects which applet to run based on argv[0].
    let applets = &[
        "sh",
        "mount",
        "umount",
        "insmod",
        "ls",
        "cat",
        "echo",
        "cp",
        "mv",
        "rm",
        "mkdir",
        "chmod",
        "chown",
        "ln",
        "dmesg",
        "kill",
        "ps",
        "grep",
        "sed",
        "sleep",
        "test",
        "pidof",
        "poweroff",
        "reboot",
        "switch_root",
        "depmod",
        "modprobe",
        "modinfo",
        "lsmod",
        "rmmod",
        "mknod",
        "mountpoint",
        "clear",
        "reset",
        "printenv",
        "env",
        "which",
        "cut",
        "sort",
        "uniq",
        "head",
        "tail",
        "tee",
        "wc",
        "find",
        "xargs",
        "expr",
        "tr",
        "od",
        "hexdump",
        "true",
        "false",
        "nuke",
    ];

    for applet in applets {
        if *applet == "sh" {
            // Make /bin/sh a symlink to busybox.
            cw.add_symlink("bin/sh", "/bin/busybox")?;
        } else {
            // Create symlinks in /sbin and /bin as appropriate.
            let target = "/bin/busybox".to_string();
            // Most utilities go to /bin.
            cw.add_symlink(&format!("bin/{applet}"), &target)?;
            // Some also need /sbin entries.
            let sbin_applets = &[
                "insmod",
                "modprobe",
                "rmmod",
                "lsmod",
                "depmod",
                "modinfo",
                "switch_root",
                "reboot",
                "poweroff",
            ];
            if sbin_applets.contains(applet) {
                cw.add_symlink(&format!("sbin/{applet}"), &target)?;
            }
        }
    }

    Ok(())
}

/// Install a minimal `/sbin/modprobe` script into the CPIO archive.
///
/// This is used when busybox is not available or its modprobe applet is not
/// compiled in. It simply calls `busybox modprobe` or falls back.
pub fn install_modprobe(cw: &mut CpioWriter<impl Write>) -> Result<(), BkgCaveError> {
    let script = b"#!/bin/sh
# Minimal modprobe wrapper - calls busybox modprobe
export PATH=/sbin:/bin:/usr/sbin:/usr/bin
if command -v modprobe >/dev/null 2>&1; then
    exec modprobe \"$@\"
fi
# Fallback: try insmod manually
for mod in \"$@\"; do
    m=$(find /lib/modules -name \"${mod}.ko\" 2>/dev/null | head -1)
    [ -n \"$m\" ] && insmod \"$m\" 2>/dev/null || true
done
";
    cw.add_entry("sbin/modprobe", CpioEntryType::File, script)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Module installation
// ---------------------------------------------------------------------------

/// Install kernel module files into the CPIO archive under
/// `/lib/modules/<kver>/`.
pub fn install_modules(
    cw: &mut CpioWriter<impl Write>,
    modfiles: &[String],
    virtme_data: &Path,
) -> Result<(), BkgCaveError> {
    for modpath in modfiles {
        let source = virtme_data.join(modpath);
        let target = format!("lib/modules/{}", modpath);
        cw.add_file(&target, &source)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Init script generator
// ---------------------------------------------------------------------------

/// Generate the contents of the `/init` script for the initramfs.
///
/// The resulting bytes can be written into a CPIO archive entry.
pub fn generate_init(access: AccessMode, guest_root: &str) -> Vec<u8> {
    let ro_flag = match access {
        AccessMode::Ro => "ro,",
        AccessMode::Rw => "",
    };

    // Trim trailing slash from guest_root.
    let root = guest_root.trim_end_matches('/');

    // We need a guest_root path for switch_root.
    let switch_root_target = if root.is_empty() || root == "/" {
        "/rootfs".to_owned()
    } else {
        root.to_owned()
    };

    let script = format!(
        r#"#!/bin/sh
# ── cave-surge initramfs init ────────────────────────────────────────────────
# Mount essential virtual filesystems
mount -t proc none /proc
mount -t sysfs none /sys
mount -t tmpfs none /tmp

# Mount /dev (devtmpfs if available, otherwise tmpfs + mknod fallback)
if ! mount -t devtmpfs none /dev 2>/dev/null; then
    mount -t tmpfs none /dev
    mknod /dev/null c 1 3
    mknod /dev/console c 5 1
    mknod /dev/kmsg c 1 11
    mknod /dev/zero c 1 5
    mknod /dev/tty c 5 0
fi

# Ensure PATH is set
export PATH=/sbin:/bin:/usr/sbin:/usr/bin

# Create mount point for rootfs
mkdir -p {switch_root_target}

# Load 9p and virtio modules
depmod -a 2>/dev/null || true
modprobe 9p 2>/dev/null || true
modprobe 9pnet 2>/dev/null || true
modprobe 9pnet_virtio 2>/dev/null || true
modprobe virtio_pci 2>/dev/null || true
modprobe virtio_console 2>/dev/null || true

# Mount host root via 9p virtfs
echo "Mounting 9p root filesystem (tag: host_root)..." > /dev/kmsg
mount -t 9p -o trans=virtio,version=9p2000.L,msize=262144,{ro_flag}access=any host_root {switch_root_target} 2>&1 || {{
    echo "9p mount with access=any failed, trying without..." > /dev/kmsg
    mount -t 9p -o trans=virtio,version=9p2000.L,msize=262144,{ro_flag}host_root {switch_root_target} 2>&1 || {{
        echo "9p mount failed entirely! Dropping to shell." > /dev/kmsg
        echo "9p mount failed. Check kernel config for 9P_FS, NET_9P_VIRTIO." > /dev/console
        exec /bin/sh
    }}
}}

# Check what init binary is available
for init in /sbin/init /bin/init /usr/sbin/init /usr/bin/init; do
    if [ -x "{switch_root_target}$init" ]; then
        echo "Found init at $init" > /dev/kmsg
        break
    fi
done

# Switch to the new root and exec init
echo "Switching root to {switch_root_target}..." > /dev/kmsg
exec switch_root {switch_root_target} /sbin/init
"#
    );

    script.into_bytes()
}

// ---------------------------------------------------------------------------
// Top-level builder
// ---------------------------------------------------------------------------

/// Build a complete initramfs CPIO archive according to the given config.
///
/// Returns the raw bytes of the CPIO archive.
pub fn mkinitramfs(config: &InitramfsConfig) -> Result<Vec<u8>, BkgCaveError> {
    let mut buf = Vec::new();
    {
        let mut cw = CpioWriter::new(&mut buf);

        // 1. Base directory layout.
        make_base_layout(&mut cw)?;

        // 2. Device nodes.
        make_dev_nodes(&mut cw)?;

        // 3. Busybox (if available).
        if let Some(ref bb) = config.busybox {
            let root = Path::new("/");
            install_busybox(&mut cw, root, bb)?;
        } else {
            // Fallback: try to find busybox automatically.
            if let Some(bb) = find_busybox(Path::new("/"), false) {
                install_busybox(&mut cw, Path::new("/"), &bb)?;
            } else {
                // Install the minimal modprobe script.
                install_modprobe(&mut cw)?;
            }
        }

        // 4. Kernel modules.
        if !config.modfiles.is_empty() {
            install_modules(&mut cw, config.modfiles, config.virtme_data)?;
        }

        // 5. The /init script.
        let init_data = generate_init(config.access, config.guest_root);
        cw.add_entry("init", CpioEntryType::File, &init_data)?;
        // Ensure /init is executable (add with 755)
        // Note: add_entry already sets 0o777 for files, so it's fine.

        // 6. Finish the archive (write TRAILER!!!).
        cw.finish()?;
    }

    Ok(buf)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn make_base_layout_creates_dirs() {
        let mut buf = Vec::new();
        {
            let mut cw = CpioWriter::new(&mut buf);
            make_base_layout(&mut cw).unwrap();
            cw.finish().unwrap();
        }
        // The archive should contain CPIO entries.
        assert!(buf.starts_with(b"070701"));
        // Should contain our directories — look for "dev" NUL in the content.
        assert!(
            buf.windows(4).any(|w| w == b"dev\0"),
            "should contain 'dev' directory entry"
        );
    }

    #[test]
    fn make_dev_nodes_creates_devices() {
        let mut buf = Vec::new();
        {
            let mut cw = CpioWriter::new(&mut buf);
            make_dev_nodes(&mut cw).unwrap();
            cw.finish().unwrap();
        }
        assert!(buf.starts_with(b"070701"));
        // Check for null device.
        assert!(
            buf.windows(5).any(|w| w == b"null\0"),
            "should contain dev/null"
        );
        assert!(
            buf.windows(8).any(|w| w == b"console\0"),
            "should contain dev/console"
        );
    }

    #[test]
    fn generate_init_contains_key_commands() {
        let init = generate_init(AccessMode::Rw, "/");
        let script = String::from_utf8_lossy(&init);
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("mount -t 9p"));
        assert!(script.contains("switch_root"));
        assert!(script.contains("host_root"));
        assert!(script.contains("/sbin/init"));
    }

    #[test]
    fn generate_init_ro_mode() {
        let init = generate_init(AccessMode::Ro, "/");
        let script = String::from_utf8_lossy(&init);
        assert!(script.contains("ro,"));
    }

    #[test]
    fn generate_init_rw_mode() {
        let init = generate_init(AccessMode::Rw, "/");
        let script = String::from_utf8_lossy(&init);
        // Should not contain "ro," in the 9p mount options
        assert!(!script.contains("-o trans=virtio,version=9p2000.L,msize=262144,ro,"));
    }

    #[test]
    fn generate_init_custom_guest_root() {
        let init = generate_init(AccessMode::Rw, "/custom/root");
        let script = String::from_utf8_lossy(&init);
        assert!(script.contains("/custom/root"));
    }

    #[test]
    fn mkinitramfs_empty_modfiles() {
        let tmp = TempDir::new().unwrap();
        let config = InitramfsConfig {
            modfiles: &[],
            virtme_data: tmp.path(),
            busybox: None,
            access: AccessMode::Rw,
            guest_root: "/",
        };
        let result = mkinitramfs(&config);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.starts_with(b"070701"));
        // Should contain the init script.
        assert!(
            data.windows(5).any(|w| w == b"init\0"),
            "should contain init entry"
        );
    }

    #[test]
    fn mkinitramfs_with_modfiles() {
        let tmp = TempDir::new().unwrap();
        // Create a fake module file.
        let mod_dir = tmp.path().join("kernel/fs/9p");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("9p.ko"), b"fake module content").unwrap();

        let modfiles = vec!["kernel/fs/9p/9p.ko".to_string()];
        let config = InitramfsConfig {
            modfiles: &modfiles,
            virtme_data: tmp.path(),
            busybox: None,
            access: AccessMode::Rw,
            guest_root: "/",
        };
        let result = mkinitramfs(&config);
        assert!(result.is_ok());
        let data = result.unwrap();
        // The archive should contain the module file path.
        assert!(
            data.windows(12).any(|w| w == b"lib/modules/"),
            "should contain lib/modules/ prefix"
        );
    }

    #[test]
    fn find_busybox_in_path() {
        // On most systems busybox might not be installed; just ensure no panic.
        let result = find_busybox(Path::new("/"), true);
        // Either Some or None is fine — we just shouldn't crash.
        let _ = result;
    }

    #[test]
    fn mkinitramfs_archive_parses_as_cpio() {
        let tmp = TempDir::new().unwrap();
        let config = InitramfsConfig {
            modfiles: &[],
            virtme_data: tmp.path(),
            busybox: None,
            access: AccessMode::Rw,
            guest_root: "/",
        };
        let data = mkinitramfs(&config).unwrap();

        // Basic CPIO validation: starts with magic and contains TRAILER!!!
        assert!(data.starts_with(b"070701"));
        let trailer_pos = data.len().saturating_sub(128);
        assert!(
            data[trailer_pos..].windows(10).any(|w| w == b"TRAILER!!!"),
            "should have TRAILER!!! at end"
        );
    }

    #[test]
    fn install_modprobe_creates_script() {
        let mut buf = Vec::new();
        {
            let mut cw = CpioWriter::new(&mut buf);
            install_modprobe(&mut cw).unwrap();
            cw.finish().unwrap();
        }
        assert!(buf.starts_with(b"070701"));
        // Find sbin/modprobe entry
        assert!(
            buf.windows(14).any(|w| w == b"sbin/modprobe\0"),
            "should contain sbin/modprobe"
        );
    }
}
