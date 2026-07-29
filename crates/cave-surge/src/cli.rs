//! CLI argument parser for the `cave-surge` VM runner.
//!
//! Uses `clap` derive to define the full set of options accepted by the
//! `surge` subcommand.  The struct can be parsed directly from a slice
//! of string arguments.

use clap::Parser;

/// Start a virtual machine that shares the host's root filesystem via 9p
/// virtfs.  The VM runs with the same rootfs as the host, making it
/// possible to test kernels, debug initramfs, or run integration tests
/// without a separate disk image.
///
/// This is a port of `virtme` for the bkg-cave project.
#[derive(Parser, Debug, Clone)]
#[command(name = "surge", verbatim_doc_comment)]
pub struct SurgeArgs {
    // ── Kernel source (mutex group: exactly one of these) ─────────────────────
    /// Use an installed distro kernel (e.g. "5.15.0" or "latest").
    ///
    /// Surge will look for `/boot/vmlinuz-<version>` and use the matching
    /// `/lib/modules/<version>/` for module resolution.
    #[arg(long, group = "kernel", short = 'i')]
    pub installed_kernel: Option<String>,

    /// Path to a kernel image file (e.g. `arch/x86_64/boot/bzImage`).
    #[arg(long, group = "kernel", short = 'k')]
    pub kimg: Option<String>,

    /// Path to a kernel build directory.
    ///
    /// Surge will look for the kernel image under the arch-specific path
    /// inside this directory.
    #[arg(long, group = "kernel", short = 'K')]
    pub kdir: Option<String>,

    // ── Module handling ───────────────────────────────────────────────────────
    /// Module resolution strategy.
    ///
    /// - `auto`:  auto-detect the needed modules via modaliases.
    /// - `none`:  do not bundle any modules.
    /// - `all`:   bundle all modules from the installed module tree.
    /// - `<path>`: comma-separated list of specific module names/aliases.
    #[arg(long, default_value = "auto")]
    pub mods: String,

    // ── Kernel options ────────────────────────────────────────────────────────
    /// Extra kernel command-line options.
    ///
    /// Can be specified multiple times; each value is appended to the kernel
    /// command line.
    #[arg(long, short = 'o')]
    pub kopt: Vec<String>,

    /// Enable Xen guest support (adds `xen` and relevant drivers).
    #[arg(long)]
    pub xen: bool,

    // ── Root filesystem ───────────────────────────────────────────────────────
    /// Path to a root filesystem image (ext4, squashfs, etc.).
    ///
    /// If not specified, Surge uses 9p virtfs to share the host root.
    #[arg(long, short = 'r')]
    pub root: Option<String>,

    /// Mount the root filesystem read-write (only relevant with --root or
    /// when using 9p without an initramfs).
    #[arg(long)]
    pub rw: bool,

    /// Enable graphics display (instead of headless serial console).
    #[arg(long, short = 'g')]
    pub graphics: bool,

    // ── Networking ────────────────────────────────────────────────────────────
    /// Network backend configuration.
    ///
    /// - `user`:     SLiRP user-mode networking (default if not specified).
    /// - `tap`:      tap networking (requires privileges).
    /// - `none`:     no networking.
    /// - `<iface>`:  bridge to a specific host interface.
    #[arg(long)]
    pub net: Option<String>,

    /// Balloon device configuration.
    ///
    /// - `none`:     no balloon device.
    /// - `virtio`:   virtio-balloon (default).
    #[arg(long)]
    pub balloon: Option<String>,

    // ── Disk / storage ────────────────────────────────────────────────────────
    /// Attach a disk image (raw file, qcow2, etc.).
    ///
    /// Format: `<path>[:<format>[:<bus>]]`.
    /// Example: `--disk /tmp/test.img:raw:virtio`
    #[arg(long, short = 'd')]
    pub disk: Vec<String>,

    /// Attach a block device (host block device passthrough).
    /// Format: `<devpath>[:<bus>]`.
    #[arg(long)]
    pub blk_disk: Vec<String>,

    /// Memory size (e.g. "2G", "512M").
    #[arg(long, short = 'm')]
    pub memory: Option<String>,

    /// Number of CPUs / SMP topology (e.g. "4", "cpus=4,sockets=2").
    #[arg(long, short = 'c')]
    pub cpus: Option<String>,

    /// VM name (for display and process management).
    #[arg(long)]
    pub name: Option<String>,

    // ── Scripting / automation ────────────────────────────────────────────────
    /// Path to a shell script to run inside the VM.
    ///
    /// The script is copied into the initramfs and executed after boot.
    #[arg(long)]
    pub script_sh: Option<String>,

    /// Path to a binary to execute inside the VM.
    #[arg(long)]
    pub script_exec: Option<String>,

    // ── Architecture ──────────────────────────────────────────────────────────
    /// Target architecture.
    ///
    /// Supported: x86_64, i386, arm, aarch64, ppc64, riscv64, sparc64, s390x.
    /// Default: host architecture (via `uname -m`).
    #[arg(long)]
    pub arch: Option<String>,

    /// Path to the busybox binary to include in the initramfs.
    ///
    /// If not specified, Surge searches common locations.
    #[arg(long)]
    pub busybox: Option<String>,

    // ── QEMU control ──────────────────────────────────────────────────────────
    /// Extra QEMU arguments (can be specified multiple times).
    #[arg(long, allow_hyphen_values = true)]
    pub qemu_opt: Vec<String>,

    /// Extra QEMU arguments as a single string (parsed with shell-like
    /// splitting).
    #[arg(long)]
    pub qemu_opts: Option<String>,

    // ── Flags ─────────────────────────────────────────────────────────────────
    /// Always build an initramfs, even when `--root` is specified.
    #[arg(long)]
    pub force_initramfs: bool,

    /// Dry-run: print the QEMU command line without executing it.
    #[arg(long)]
    pub dry_run: bool,

    /// Show the QEMU command line (similar to dry-run but also runs).
    #[arg(long)]
    pub show_command: bool,

    /// Save the generated initramfs to a file for inspection.
    #[arg(long)]
    pub save_initramfs: Option<String>,

    /// Show boot console output (alias for `--graphics`).
    #[arg(long)]
    pub show_boot_console: bool,

    // ── Working directory ─────────────────────────────────────────────────────
    #[arg(long)]
    pub pwd: Option<String>,

    #[arg(long)]
    pub cwd: Option<String>,

    // ── Shared folders ────────────────────────────────────────────────────────
    /// Additional read-write shared directories (9p mounts).
    ///
    /// Can be specified multiple times. Format: `<host_path>:<mount_tag>`.
    #[arg(long)]
    pub rwdir: Vec<String>,

    /// Additional read-only shared directories (9p mounts).
    ///
    /// Can be specified multiple times. Format: `<host_path>:<mount_tag>`.
    #[arg(long)]
    pub rodir: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal() {
        // No arguments — should parse with defaults.
        let args = SurgeArgs::parse_from(&["surge"]);
        assert_eq!(args.mods, "auto");
        assert!(!args.rw);
        assert!(!args.graphics);
        assert!(!args.xen);
        assert!(!args.dry_run);
        assert!(!args.force_initramfs);
        assert!(!args.show_command);
        assert!(args.installed_kernel.is_none());
        assert!(args.kimg.is_none());
        assert!(args.kdir.is_none());
        assert!(args.root.is_none());
    }

    #[test]
    fn parse_installed_kernel() {
        let args = SurgeArgs::parse_from(&["surge", "--installed-kernel", "5.15.0"]);
        assert_eq!(args.installed_kernel.as_deref(), Some("5.15.0"));
        assert!(args.kimg.is_none());
        assert!(args.kdir.is_none());
    }

    #[test]
    fn parse_kimg() {
        let args = SurgeArgs::parse_from(&["surge", "--kimg", "/boot/bzImage"]);
        assert_eq!(args.kimg.as_deref(), Some("/boot/bzImage"));
    }

    #[test]
    fn parse_kdir() {
        let args = SurgeArgs::parse_from(&["surge", "--kdir", "/home/user/linux"]);
        assert_eq!(args.kdir.as_deref(), Some("/home/user/linux"));
    }

    #[test]
    fn parse_kopt_multiple() {
        let args = SurgeArgs::parse_from(&["surge", "-o", "quiet", "-o", "loglevel=4"]);
        assert_eq!(args.kopt, vec!["quiet", "loglevel=4"]);
    }

    #[test]
    fn parse_disk() {
        let args = SurgeArgs::parse_from(&["surge", "--disk", "/tmp/test.img:raw"]);
        assert_eq!(args.disk, vec!["/tmp/test.img:raw"]);
    }

    #[test]
    fn parse_rwdir() {
        let args = SurgeArgs::parse_from(&["surge", "--rwdir", "/mnt/data:data"]);
        assert_eq!(args.rwdir, vec!["/mnt/data:data"]);
    }

    #[test]
    fn parse_rodir() {
        let args = SurgeArgs::parse_from(&["surge", "--rodir", "/mnt/iso:iso"]);
        assert_eq!(args.rodir, vec!["/mnt/iso:iso"]);
    }

    #[test]
    fn parse_dry_run() {
        let args = SurgeArgs::parse_from(&["surge", "--dry-run"]);
        assert!(args.dry_run);
    }

    #[test]
    fn parse_show_command() {
        let args = SurgeArgs::parse_from(&["surge", "--show-command"]);
        assert!(args.show_command);
    }

    #[test]
    fn parse_force_initramfs() {
        let args = SurgeArgs::parse_from(&["surge", "--force-initramfs"]);
        assert!(args.force_initramfs);
    }

    #[test]
    fn parse_memory() {
        let args = SurgeArgs::parse_from(&["surge", "--memory", "2G"]);
        assert_eq!(args.memory.as_deref(), Some("2G"));
    }

    #[test]
    fn parse_cpus() {
        let args = SurgeArgs::parse_from(&["surge", "--cpus", "4"]);
        assert_eq!(args.cpus.as_deref(), Some("4"));
    }

    #[test]
    fn parse_arch() {
        let args = SurgeArgs::parse_from(&["surge", "--arch", "aarch64"]);
        assert_eq!(args.arch.as_deref(), Some("aarch64"));
    }

    #[test]
    fn parse_qemu_opt() {
        let args = SurgeArgs::parse_from(&[
            "surge",
            "--qemu-opt",
            "-enable-kvm",
            "--qemu-opt",
            "-cpu host",
        ]);
        assert_eq!(args.qemu_opt, vec!["-enable-kvm", "-cpu host"]);
    }

    #[test]
    fn parse_name() {
        let args = SurgeArgs::parse_from(&["surge", "--name", "test-vm"]);
        assert_eq!(args.name.as_deref(), Some("test-vm"));
    }

    #[test]
    fn parse_graphics() {
        let args = SurgeArgs::parse_from(&["surge", "--graphics"]);
        assert!(args.graphics);
    }

    #[test]
    fn parse_net() {
        let args = SurgeArgs::parse_from(&["surge", "--net", "user"]);
        assert_eq!(args.net.as_deref(), Some("user"));
    }

    #[test]
    fn parse_balloon() {
        let args = SurgeArgs::parse_from(&["surge", "--balloon", "none"]);
        assert_eq!(args.balloon.as_deref(), Some("none"));
    }

    #[test]
    fn parse_xen() {
        let args = SurgeArgs::parse_from(&["surge", "--xen"]);
        assert!(args.xen);
    }

    #[test]
    fn parse_save_initramfs() {
        let args = SurgeArgs::parse_from(&["surge", "--save-initramfs", "/tmp/init.cpio"]);
        assert_eq!(args.save_initramfs.as_deref(), Some("/tmp/init.cpio"));
    }

    #[test]
    fn parse_script_sh() {
        let args = SurgeArgs::parse_from(&["surge", "--script-sh", "/tmp/test.sh"]);
        assert_eq!(args.script_sh.as_deref(), Some("/tmp/test.sh"));
    }
}
