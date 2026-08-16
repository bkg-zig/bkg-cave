//! SurgePlugin — the CavePlugin implementation for the cave-surge VM runner.
//!
//! The plugin registers the `surge` command and implements the full
//! orchestration: determine architecture, locate the kernel, build the
//! initramfs, construct the QEMU command line, and finally exec the QEMU
//! process.

use clap::Parser;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

use cave_core::error::BkgCaveError;
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};

use crate::arch::{arch_from_name, Arch};
use crate::cli::SurgeArgs;
use crate::initramfs::{self, AccessMode, InitramfsConfig};
use crate::modaliases::MODALIASES;
use crate::modules;
use crate::qemu::Qemu;

// ---------------------------------------------------------------------------
// SurgePlugin
// ---------------------------------------------------------------------------

/// The `cave-surge` VM runner plugin.
pub struct SurgePlugin;

impl SurgePlugin {
    /// Create a new SurgePlugin instance.
    pub fn new() -> Self {
        SurgePlugin
    }
}

impl Default for SurgePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CavePlugin for SurgePlugin {
    fn id(&self) -> &'static str {
        "cave-surge"
    }

    fn name(&self) -> &str {
        "Surge VM Runner"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn kind(&self) -> PluginKind {
        PluginKind::Surge
    }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), BkgCaveError> {
        info!("Surge plugin initialised");
        Ok(())
    }

    fn commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "surge",
            description: "Start a VM with the host rootfs via 9p virtfs (port of virtme)",
            run: |args| execute(args),
        }]
    }

    fn hot_reloadable(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// Top-level execution
// ---------------------------------------------------------------------------

/// Execute the surge command.
///
/// This is the main entry point called when the `surge` command is dispatched.
pub fn execute(args: &[String]) -> Result<i32, BkgCaveError> {
    let surge_args = SurgeArgs::parse_from(args);
    debug!("Surge args: {surge_args:#?}");

    // 1. Resolve architecture.
    let arch_name = resolve_arch_name(&surge_args);
    let arch = arch_from_name(&arch_name);
    info!(
        "Target architecture: {} (qemu: {})",
        arch.virtmename(),
        arch.qemuname()
    );

    // 2. Discover QEMU binary.
    let mut qemu = Qemu::new(arch.as_ref());
    qemu.probe()?;
    info!(
        "Using QEMU: {} (version: {:?})",
        qemu.qemubin.display(),
        qemu.version
    );

    // 3. Determine kernel image path.
    let kimg = resolve_kernel(&surge_args, arch.as_ref())?;
    info!("Kernel image: {}", kimg.display());

    // 4. Determine kernel version.
    let kver = detect_kver(&surge_args, &kimg)?;

    // 5. Build kernel command line.
    let cmdline = build_cmdline(&surge_args, arch.as_ref())?;
    debug!("Kernel cmdline: {cmdline}");

    // 6. Resolve modules.
    let modfiles = resolve_modules(&surge_args, arch.as_ref(), &kver)?;
    debug!("Module files ({}): {modfiles:?}", modfiles.len());

    // 7. Build initramfs.
    let initramfs_data = build_initramfs_if_needed(&surge_args, arch.as_ref(), &kver, &modfiles)?;

    // 8. Build the QEMU command line.
    let mut qemuargs = build_qemu_command(
        &surge_args,
        arch.as_ref(),
        &qemu,
        &kimg,
        &cmdline,
        initramfs_data.as_deref(),
    )?;

    // 9. Handle display.
    if !surge_args.graphics {
        qemuargs.extend(arch.qemu_nodisplay_args());
    }

    // 10. Handle networking.
    qemuargs.extend(networking_args(&surge_args));

    // 11. Handle disks.
    qemuargs.extend(disk_args(&surge_args));

    // 12. Handle memory and CPUs.
    if let Some(ref mem) = surge_args.memory {
        qemuargs.push("-m".into());
        qemuargs.push(mem.clone());
    }
    if let Some(ref cpus) = surge_args.cpus {
        qemuargs.push("-smp".into());
        qemuargs.push(cpus.clone());
    }

    // 13. Extra QEMU options.
    qemuargs.extend(surge_args.qemu_opt.clone());
    if let Some(ref opts) = surge_args.qemu_opts {
        // Simple shell-like splitting (space-separated, respect quotes).
        for opt in shlex_split(opts) {
            qemuargs.push(opt);
        }
    }

    // 14. VM name.
    if let Some(ref name) = surge_args.name {
        qemuargs.push("-name".into());
        qemuargs.push(name.clone());
    }

    // 15. Save initramfs if requested.
    if let Some(ref save_path) = surge_args.save_initramfs {
        if let Some(ref data) = initramfs_data {
            std::fs::write(save_path, data).map_err(BkgCaveError::Io)?;
            info!("Saved initramfs to {}", save_path);
        }
    }

    // 16. Print or execute.
    if surge_args.dry_run || surge_args.show_command {
        print_qemu_command(&qemu.qemubin, &qemuargs);
        if surge_args.dry_run {
            return Ok(0);
        }
    }

    // 17. Set working directory if specified.
    if let Some(ref cwd) = surge_args.cwd {
        std::env::set_current_dir(cwd).map_err(BkgCaveError::Io)?;
    }

    // 18. Exec QEMU (replaces the current process).
    info!(
        "Starting QEMU: {} {}",
        qemu.qemubin.display(),
        qemuargs.join(" ")
    );
    let err = std::process::Command::new(&qemu.qemubin)
        .args(&qemuargs)
        .exec();

    // exec() only returns on error.
    error!("Failed to exec QEMU: {err}");
    Err(BkgCaveError::Io(err))
}

// ---------------------------------------------------------------------------
// Architecture resolution
// ---------------------------------------------------------------------------

fn resolve_arch_name(args: &SurgeArgs) -> String {
    if let Some(ref arch) = args.arch {
        return arch.clone();
    }
    // Detect host architecture.
    let output = std::process::Command::new("uname")
        .arg("-m")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_owned())
            } else {
                None
            }
        });
    output.unwrap_or_else(|| "x86_64".to_owned())
}

// ---------------------------------------------------------------------------
// Kernel resolution
// ---------------------------------------------------------------------------

fn resolve_kernel(args: &SurgeArgs, arch: &dyn Arch) -> Result<PathBuf, BkgCaveError> {
    // Priority: --kimg > --kdir > --installed-kernel
    if let Some(ref kimg) = args.kimg {
        let p = Path::new(kimg);
        if !p.exists() {
            return Err(BkgCaveError::NotFound(format!(
                "kernel image not found: {}",
                p.display()
            )));
        }
        return Ok(p.to_owned());
    }

    if let Some(ref kdir) = args.kdir {
        let kimg_path = Path::new(kdir).join(arch.kimg_path());
        if kimg_path.exists() {
            return Ok(kimg_path);
        }
        // Try alternative names.
        let alt = Path::new(kdir).join("vmlinux");
        if alt.exists() {
            return Ok(alt);
        }
        return Err(BkgCaveError::NotFound(format!(
            "no kernel image found in {} (looked for {}, vmlinux)",
            kdir,
            arch.kimg_path()
        )));
    }

    if let Some(ref version) = args.installed_kernel {
        let boot = Path::new("/boot");
        // Try multiple naming conventions.
        let candidates = [
            boot.join(format!("vmlinuz-{version}")),
            boot.join(format!("vmlinux-{version}")),
            boot.join(format!("bzImage-{version}")),
            boot.join(format!("Image-{version}")),
        ];
        for c in &candidates {
            if c.exists() {
                return Ok(c.clone());
            }
        }
        return Err(BkgCaveError::NotFound(format!(
            "installed kernel version '{version}' not found in /boot"
        )));
    }

    // Default: try the running kernel.
    let output = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .map_err(|e| BkgCaveError::Execution {
            command: "uname -r".into(),
            exit_code: -1,
            stderr: e.to_string(),
        })?;
    let kver = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !kver.is_empty() {
        let boot = Path::new("/boot");
        let candidates = [
            boot.join(format!("vmlinuz-{kver}")),
            boot.join(format!("vmlinux-{kver}")),
            boot.join("vmlinuz"),
            boot.join("bzImage"),
        ];
        for c in &candidates {
            if c.exists() {
                return Ok(c.clone());
            }
        }
    }

    Err(BkgCaveError::NotFound(
        "no kernel found. Specify --kimg, --kdir, or --installed-kernel".into(),
    ))
}

// ---------------------------------------------------------------------------
// Kernel version detection
// ---------------------------------------------------------------------------

fn detect_kver(args: &SurgeArgs, kimg: &Path) -> Result<String, BkgCaveError> {
    // If the user specified a version explicitly via --installed-kernel, use it.
    if let Some(ref v) = args.installed_kernel {
        if v != "latest" {
            return Ok(v.clone());
        }
    }

    // Try to extract from the image path (e.g. /boot/vmlinuz-5.15.0).
    if let Some(file_name) = kimg.file_name().and_then(|s| s.to_str()) {
        let parts: Vec<&str> = file_name.splitn(2, '-').collect();
        if parts.len() == 2 && !parts[1].is_empty() {
            let version = parts[1].to_owned();
            // Quick sanity: first char is a digit.
            if version.starts_with(|c: char| c.is_ascii_digit()) {
                return Ok(version);
            }
        }
    }

    // Fallback: query the installed module tree.
    modules::detect_kernel_version(Path::new("/"))
}

// ---------------------------------------------------------------------------
// Kernel command-line builder
// ---------------------------------------------------------------------------

fn build_cmdline(args: &SurgeArgs, arch: &dyn Arch) -> Result<String, BkgCaveError> {
    let mut parts: Vec<String> = Vec::new();

    // Base: arch-specific console args.
    if !args.graphics {
        parts.extend(arch.serial_console_args().into_iter().map(|a| a.to_owned()));
        parts.extend(arch.earlyconsole_args().into_iter().map(|a| a.to_owned()));
    }

    // Root filesystem.
    if let Some(ref root) = args.root {
        parts.push(format!("root={root}"));
        if args.rw {
            parts.push("rw".into());
        } else {
            parts.push("ro".into());
        }
    } else {
        // 9p root — the initramfs handles mounting.
        parts.push("root=/dev/root".into());
        parts.push("rootfstype=9p".into());
        if args.rw {
            parts.push("rw".into());
        } else {
            parts.push("ro".into());
        }
    }

    // Extra kernel options.
    parts.extend(args.kopt.clone());

    // Xen support.
    if args.xen {
        parts.push("xen".into());
    }

    // Common options.
    parts.push("panic=-1".into());
    parts.push("oops=panic".into());

    Ok(parts.join(" "))
}

// ---------------------------------------------------------------------------
// Module resolution
// ---------------------------------------------------------------------------

fn resolve_modules(
    args: &SurgeArgs,
    arch: &dyn Arch,
    kver: &str,
) -> Result<Vec<String>, BkgCaveError> {
    match args.mods.as_str() {
        "none" => Ok(Vec::new()),
        "all" => {
            // Collect all modules from the installed module tree.
            let mod_dir = modules::find_mod_dir(arch, kver, Path::new("/"));
            if mod_dir.is_dir() {
                let files = cave_core::util::walk_files(&mod_dir)?;
                let rels: Vec<String> = files
                    .iter()
                    .filter_map(|p| p.strip_prefix(&mod_dir).ok())
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                Ok(rels)
            } else {
                warn!("Module directory {} not found", mod_dir.display());
                Ok(Vec::new())
            }
        }
        _ => {
            // "auto" or comma-separated list of aliases.
            let aliases: Vec<&str> = if args.mods == "auto" {
                MODALIASES.to_vec()
            } else {
                args.mods.split(',').collect()
            };

            let result = modules::find_from_install(&aliases, Path::new("/"), kver)?;
            if result.is_empty() {
                warn!("No modules resolved from aliases (kernel version: {kver})");
            }
            Ok(result)
        }
    }
}

// ---------------------------------------------------------------------------
// Initramfs builder
// ---------------------------------------------------------------------------

fn build_initramfs_if_needed(
    args: &SurgeArgs,
    arch: &dyn Arch,
    kver: &str,
    modfiles: &[String],
) -> Result<Option<Vec<u8>>, BkgCaveError> {
    // Always build an initramfs unless --root is given (and --force-initramfs
    // is not set).
    let needs_initramfs = args.root.is_none() || args.force_initramfs;

    if !needs_initramfs {
        return Ok(None);
    }

    // Resolve busybox.
    let busybox = if let Some(ref bb) = args.busybox {
        Some(PathBuf::from(bb))
    } else {
        initramfs::find_busybox(Path::new("/"), arch.virtmename() == "x86_64")
    };

    let access = if args.rw {
        AccessMode::Rw
    } else {
        AccessMode::Ro
    };

    // Module staging directory: we use /tmp/cave-surge/modules/<kver>/ as
    // virtme_data (we need to have the module files available for CPIO).
    let virtme_data = Path::new("/tmp")
        .join("cave-surge")
        .join("modules")
        .join(kver);
    if !modfiles.is_empty() && !virtme_data.exists() {
        // Copy modules to staging area.
        let mod_dir = modules::find_mod_dir(arch, kver, Path::new("/"));
        if mod_dir.is_dir() {
            std::fs::create_dir_all(&virtme_data).map_err(BkgCaveError::Io)?;
            for m in modfiles {
                let src = mod_dir.join(m);
                let dst = virtme_data.join(m);
                if src.is_file() && !dst.exists() {
                    if let Some(parent) = dst.parent() {
                        std::fs::create_dir_all(parent).map_err(BkgCaveError::Io)?;
                    }
                    std::fs::copy(&src, &dst).map_err(BkgCaveError::Io)?;
                }
            }
        }
    }

    let config = InitramfsConfig {
        modfiles,
        virtme_data: &virtme_data,
        busybox,
        access,
        guest_root: "/",
    };

    info!("Building initramfs (modfiles: {})", modfiles.len());
    let data = initramfs::mkinitramfs(&config)?;
    info!("Initramfs built ({} bytes)", data.len());

    Ok(Some(data))
}

// ---------------------------------------------------------------------------
// QEMU command-line builder
// ---------------------------------------------------------------------------

fn build_qemu_command(
    args: &SurgeArgs,
    arch: &dyn Arch,
    qemu: &Qemu,
    kimg: &Path,
    cmdline: &str,
    initramfs: Option<&[u8]>,
) -> Result<Vec<String>, BkgCaveError> {
    let mut qemuargs = Vec::new();

    // Architecture-specific base args.
    let is_native = arch.virtmename() == resolve_arch_name(args);
    qemuargs.extend(arch.qemuargs(is_native));

    // Kernel.
    qemuargs.push("-kernel".into());
    qemuargs.push(kimg.to_string_lossy().into_owned());

    // Append kernel command line.
    qemuargs.push("-append".into());
    qemuargs.push(cmdline.to_owned());

    // Initramfs.
    if let Some(data) = initramfs {
        // Write initramfs to a temporary file.
        let tmp_dir = Path::new("/tmp").join("cave-surge");
        std::fs::create_dir_all(&tmp_dir).map_err(BkgCaveError::Io)?;
        let initramfs_path = tmp_dir.join("initramfs.cpio");
        std::fs::write(&initramfs_path, data).map_err(BkgCaveError::Io)?;

        qemuargs.push("-initrd".into());
        qemuargs.push(initramfs_path.to_string_lossy().into_owned());
    }

    // 9p virtfs root share (unless --root is given without --force-initramfs).
    // If the user provides --root, we boot from that image and don't need 9p.
    if args.root.is_none() || args.force_initramfs {
        // Export the host root via 9p.
        let ro = !args.rw;
        qemu.export_virtfs(&mut qemuargs, "/", "host_root", ro, true, "");
    }

    // Additional shared directories.
    for dir_spec in &args.rwdir {
        if let Some((path, tag)) = dir_spec.split_once(':') {
            qemu.export_virtfs(&mut qemuargs, path, tag, false, false, "");
        } else {
            qemu.export_virtfs(&mut qemuargs, dir_spec, dir_spec, false, false, "");
        }
    }
    for dir_spec in &args.rodir {
        if let Some((path, tag)) = dir_spec.split_once(':') {
            qemu.export_virtfs(&mut qemuargs, path, tag, true, false, "");
        } else {
            qemu.export_virtfs(&mut qemuargs, dir_spec, dir_spec, true, false, "");
        }
    }

    Ok(qemuargs)
}

// ---------------------------------------------------------------------------
// Networking arguments
// ---------------------------------------------------------------------------

fn networking_args(args: &SurgeArgs) -> Vec<String> {
    let mut qemuargs = Vec::new();

    match args.net.as_deref() {
        None | Some("user") => {
            qemuargs.push("-nic".into());
            qemuargs.push("user,model=virtio-net-pci".into());
        }
        Some("none") => {
            qemuargs.push("-nic".into());
            qemuargs.push("none".into());
        }
        Some("tap") => {
            qemuargs.push("-nic".into());
            qemuargs.push("tap,model=virtio-net-pci".into());
        }
        Some(bridge) => {
            // Treat as a bridge name.
            qemuargs.push("-nic".into());
            qemuargs.push(format!("bridge,model=virtio-net-pci,br={bridge}"));
        }
    }

    qemuargs
}

// ---------------------------------------------------------------------------
// Disk arguments
// ---------------------------------------------------------------------------

fn disk_args(args: &SurgeArgs) -> Vec<String> {
    let mut qemuargs = Vec::new();

    for disk_spec in &args.disk {
        let parts: Vec<&str> = disk_spec.split(':').collect();
        let path = parts[0];
        let format = parts.get(1).copied().unwrap_or("raw");
        let bus = parts.get(2).copied().unwrap_or("virtio");

        qemuargs.push("-drive".into());
        qemuargs.push(format!("file={path},format={format},if={bus}"));
    }

    for blk_spec in &args.blk_disk {
        let parts: Vec<&str> = blk_spec.split(':').collect();
        let devpath = parts[0];
        let bus = parts.get(1).copied().unwrap_or("virtio");

        qemuargs.push("-drive".into());
        qemuargs.push(format!("file={devpath},format=raw,if={bus}"));
    }

    qemuargs
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Simple shell-like string splitter (respects double quotes).
fn shlex_split(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if ch.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Print the full QEMU command line (for --dry-run / --show-command).
fn print_qemu_command(qemubin: &Path, args: &[String]) {
    let cmd = std::iter::once(qemubin.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" \\\n    ");
    println!("QEMU command:\n{cmd}");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shlex_split_empty() {
        assert!(shlex_split("").is_empty());
    }

    #[test]
    fn test_shlex_split_simple() {
        let result = shlex_split("abc def ghi");
        assert_eq!(result, vec!["abc", "def", "ghi"]);
    }

    #[test]
    fn test_shlex_split_with_quotes() {
        let result = shlex_split("abc \"def ghi\" jkl");
        assert_eq!(result, vec!["abc", "def ghi", "jkl"]);
    }

    #[test]
    fn test_shlex_split_escaped() {
        let result = shlex_split(r#"abc def\ ghi"#);
        assert_eq!(result, vec!["abc", "def ghi"]);
    }

    #[test]
    fn test_networking_user() {
        let args = SurgeArgs::parse_from(&["surge", "--net", "user"]);
        let result = networking_args(&args);
        assert_eq!(result, vec!["-nic", "user,model=virtio-net-pci"]);
    }

    #[test]
    fn test_networking_none() {
        let args = SurgeArgs::parse_from(&["surge", "--net", "none"]);
        let result = networking_args(&args);
        assert_eq!(result, vec!["-nic", "none"]);
    }

    #[test]
    fn test_networking_default() {
        let args = SurgeArgs::parse_from(&["surge"]);
        let result = networking_args(&args);
        assert!(result.contains(&"-nic".to_string()));
    }

    #[test]
    fn test_disk_args_simple() {
        let args = SurgeArgs::parse_from(&["surge", "--disk", "/tmp/test.img:raw"]);
        let result = disk_args(&args);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "-drive");
        assert!(result[1].contains("file=/tmp/test.img"));
        assert!(result[1].contains("format=raw"));
    }

    #[test]
    fn test_disk_args_with_bus() {
        let args = SurgeArgs::parse_from(&["surge", "--disk", "/tmp/test.img:qcow2:virtio"]);
        let result = disk_args(&args);
        assert_eq!(result.len(), 2);
        assert!(result[1].contains("if=virtio"));
    }

    #[test]
    fn test_build_cmdline_with_root() {
        let arch = crate::arch::ArchX86;
        let args = SurgeArgs::parse_from(&["surge", "--root", "/dev/sda1"]);
        let cmdline = build_cmdline(&args, &arch).unwrap();
        assert!(cmdline.contains("root=/dev/sda1"));
        assert!(cmdline.contains("ro"));
        assert!(!cmdline.contains("rootfstype=9p"));
    }

    #[test]
    fn test_build_cmdline_no_root() {
        let arch = crate::arch::ArchX86;
        let args = SurgeArgs::parse_from(&["surge"]);
        let cmdline = build_cmdline(&args, &arch).unwrap();
        assert!(cmdline.contains("root=/dev/root"));
        assert!(cmdline.contains("rootfstype=9p"));
    }

    #[test]
    fn test_build_cmdline_rw() {
        let arch = crate::arch::ArchX86;
        let args = SurgeArgs::parse_from(&["surge", "--rw"]);
        let cmdline = build_cmdline(&args, &arch).unwrap();
        assert!(cmdline.contains("rw"));
    }

    #[test]
    fn test_build_cmdline_with_kopt() {
        let arch = crate::arch::ArchX86;
        let args = SurgeArgs::parse_from(&["surge", "-o", "quiet", "-o", "loglevel=4"]);
        let cmdline = build_cmdline(&args, &arch).unwrap();
        assert!(cmdline.contains("quiet"));
        assert!(cmdline.contains("loglevel=4"));
    }

    #[test]
    fn test_build_cmdline_xen() {
        let arch = crate::arch::ArchX86;
        let args = SurgeArgs::parse_from(&["surge", "--xen"]);
        let cmdline = build_cmdline(&args, &arch).unwrap();
        assert!(cmdline.contains("xen"));
    }

    #[test]
    fn test_resolve_arch_name_default() {
        let args = SurgeArgs::parse_from(&["surge"]);
        let name = resolve_arch_name(&args);
        // Should detect host arch or fall back to x86_64.
        assert!(!name.is_empty());
    }

    #[test]
    fn test_resolve_arch_name_override() {
        let args = SurgeArgs::parse_from(&["surge", "--arch", "aarch64"]);
        let name = resolve_arch_name(&args);
        assert_eq!(name, "aarch64");
    }

    #[test]
    fn test_surge_plugin_traits() {
        let plugin = SurgePlugin::new();
        assert_eq!(plugin.id(), "cave-surge");
        assert_eq!(plugin.name(), "Surge VM Runner");
        assert_eq!(plugin.kind(), PluginKind::Surge);
        assert!(!plugin.hot_reloadable());
        assert_eq!(plugin.commands().len(), 1);
        assert_eq!(plugin.commands()[0].name, "surge");
    }

    #[test]
    fn test_module_resolution_auto() {
        let args = SurgeArgs::parse_from(&["surge", "--mods", "auto"]);
        // We don't actually call resolve_modules in test because it requires
        // system state. Just verify the parsing.
        assert_eq!(args.mods, "auto");
    }

    #[test]
    fn test_module_resolution_none() {
        let args = SurgeArgs::parse_from(&["surge", "--mods", "none"]);
        assert_eq!(args.mods, "none");
    }

    #[test]
    fn test_module_resolution_custom() {
        let args = SurgeArgs::parse_from(&["surge", "--mods", "fs-9p,virtio-pci"]);
        assert_eq!(args.mods, "fs-9p,virtio-pci");
    }

    #[test]
    fn test_dry_run_returns_immediately() {
        // This test verifies the clap parsing for dry-run.
        let args = SurgeArgs::parse_from(&["surge", "--dry-run"]);
        assert!(args.dry_run);
    }

    #[test]
    fn test_show_command_flag() {
        let args = SurgeArgs::parse_from(&["surge", "--show-command"]);
        assert!(args.show_command);
    }

    #[test]
    fn test_print_qemu_command_does_not_panic() {
        let qemubin = Path::new("/usr/bin/qemu-system-x86_64");
        let args = vec!["-kernel".into(), "bzImage".into()];
        // Just verify no panic.
        print_qemu_command(qemubin, &args);
    }
}
