//! Architecture abstraction — maps between runtime arch names and
//! their QEMU binary names, kernel image paths, default machine types,
//! serial device names, and related constants.
//!
//! This is the "virtme" port: each [`Arch`] implementation knows how to
//! configure QEMU for a particular target architecture.

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Architecture-specific metadata and QEMU configuration.
pub trait Arch: Send + Sync {
    /// virtme-friendly architecture name (e.g. `"x86_64"`, `"aarch64"`).
    fn virtmename(&self) -> &str;

    /// Name of the QEMU system binary (e.g. `"qemu-system-x86_64"`).
    fn qemuname(&self) -> &str;

    /// Linux kernel architecture name (used in module paths, e.g. `"x86_64"`).
    fn linuxname(&self) -> &str;

    /// GCC cross-compilation prefix (e.g. `"x86_64-linux-gnu"`).
    fn gccname(&self) -> &str;

    /// Default `make` target for kernel configuration (e.g. `"defconfig"`).
    fn defconfig_target(&self) -> &str;

    /// Name of the serial device for the given index (e.g. `"ttyS0"`).
    fn serial_dev_name(&self, idx: u32) -> String;

    /// Default QEMU command-line arguments for this architecture.
    fn qemuargs(&self, _is_native: bool) -> Vec<String>;

    /// Virtio device type string for the given device kind.
    ///
    /// Most architectures use `"virtio-{kind}-pci"`; some (e.g. Arm/Aarch64
    /// with `-machine virt`) use `"virtio-{kind}-device"`.
    fn virtio_dev_type(&self, kind: &str) -> String;

    /// Early-console kernel arguments (e.g. `"earlyprintk=serial,ttyS0,115200"`).
    fn earlyconsole_args(&self) -> Vec<String>;

    /// Serial console kernel arguments.
    fn serial_console_args(&self) -> Vec<String>;

    /// QEMU arguments for disabling graphical display.
    fn qemu_nodisplay_args(&self) -> Vec<String>;

    /// Base kernel configuration fragments (e.g. `"CONFIG_9P_FS=y"`).
    fn config_base(&self) -> Vec<String>;

    /// Path to the kernel image **relative to the kernel build directory**.
    fn kimg_path(&self) -> String;

    /// Optional path to a device-tree blob relative to the kernel build dir.
    fn dtb_path(&self) -> Option<String> {
        None
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Create an [`Arch`] instance by name.
///
/// Unknown names return [`ArchUnknown`], which provides safe defaults.
pub fn arch_from_name(name: &str) -> Box<dyn Arch> {
    match name {
        "x86_64" | "i386" | "x86" => Box::new(ArchX86),
        "arm" => Box::new(ArchArm),
        "aarch64" | "arm64" => Box::new(ArchAarch64),
        "ppc64" | "ppc64le" | "powerpc64" => Box::new(ArchPpc64),
        "riscv64" | "riscv" => Box::new(ArchRiscv64),
        "sparc64" | "sparc" => Box::new(ArchSparc64),
        "s390x" | "s390" => Box::new(ArchS390x),
        _ => Box::new(ArchUnknown),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_earlyconsole(dev: &str) -> Vec<String> {
    vec![format!("earlyprintk=serial,{dev},115200")]
}

fn default_serial_console(dev: &str) -> Vec<String> {
    vec![format!("console={dev},115200"), "console=tty0".into()]
}

fn default_nodisplay() -> Vec<String> {
    vec!["-nographic".into()]
}

fn default_config_base() -> Vec<String> {
    vec![
        "CONFIG_9P_FS=y".into(),
        "CONFIG_9P_FS_POSIX_ACL=y".into(),
        "CONFIG_NET_9P=y".into(),
        "CONFIG_NET_9P_VIRTIO=y".into(),
        "CONFIG_VIRTIO_PCI=y".into(),
        "CONFIG_VIRTIO_CONSOLE=y".into(),
        "CONFIG_DEVTMPFS=y".into(),
        "CONFIG_DEVTMPFS_MOUNT=y".into(),
        "CONFIG_TMPFS=y".into(),
        "CONFIG_TMPFS_POSIX_ACL=y".into(),
        "CONFIG_EXT4_FS=y".into(),
        "CONFIG_PROC_FS=y".into(),
        "CONFIG_SYSFS=y".into(),
    ]
}

fn default_virtio_pci(kind: &str) -> String {
    format!("virtio-{kind}-pci")
}

// ---------------------------------------------------------------------------
// ArchX86 – x86_64 AND i386
// ---------------------------------------------------------------------------

pub struct ArchX86;

impl Arch for ArchX86 {
    fn virtmename(&self) -> &str {
        "x86_64"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-x86_64"
    }
    fn linuxname(&self) -> &str {
        "x86_64"
    }
    fn gccname(&self) -> &str {
        "x86_64-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyS{idx}")
    }
    fn qemuargs(&self, is_native: bool) -> Vec<String> {
        let mut args = vec!["-machine".into(), "type=pc,accel=kvm:tcg".into()];
        if is_native {
            args.push("-cpu".into());
            args.push("host".into());
        }
        args
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        default_virtio_pci(kind)
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        default_earlyconsole("ttyS0")
    }
    fn serial_console_args(&self) -> Vec<String> {
        default_serial_console("ttyS0")
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_8250=y".into());
        cfg.push("CONFIG_SERIAL_8250_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/x86_64/boot/bzImage".into()
    }
}

// ---------------------------------------------------------------------------
// ArchArm – 32-bit ARM
// ---------------------------------------------------------------------------

pub struct ArchArm;

impl Arch for ArchArm {
    fn virtmename(&self) -> &str {
        "arm"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-arm"
    }
    fn linuxname(&self) -> &str {
        "arm"
    }
    fn gccname(&self) -> &str {
        "arm-linux-gnueabihf"
    }
    fn defconfig_target(&self) -> &str {
        "vexpress_defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyAMA{idx}")
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec![
            "-machine".into(),
            "virt".into(),
            "-cpu".into(),
            "cortex-a15".into(),
        ]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        default_virtio_pci(kind)
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        default_earlyconsole("ttyAMA0")
    }
    fn serial_console_args(&self) -> Vec<String> {
        vec!["console=ttyAMA0,115200".into(), "console=tty0".into()]
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_AMBA_PL011=y".into());
        cfg.push("CONFIG_SERIAL_AMBA_PL011_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/arm/boot/zImage".into()
    }
    fn dtb_path(&self) -> Option<String> {
        Some("arch/arm/boot/dts/vexpress-v2p-ca9.dtb".into())
    }
}

// ---------------------------------------------------------------------------
// ArchAarch64 – 64-bit ARM (AArch64)
// ---------------------------------------------------------------------------

pub struct ArchAarch64;

impl Arch for ArchAarch64 {
    fn virtmename(&self) -> &str {
        "aarch64"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-aarch64"
    }
    fn linuxname(&self) -> &str {
        "arm64"
    }
    fn gccname(&self) -> &str {
        "aarch64-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyAMA{idx}")
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec![
            "-machine".into(),
            "virt,gic-version=max".into(),
            "-cpu".into(),
            "max".into(),
        ]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        // Aarch64 on `-machine virt` uses the plain device variant.
        format!("virtio-{kind}-device")
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        default_earlyconsole("ttyAMA0")
    }
    fn serial_console_args(&self) -> Vec<String> {
        vec!["console=ttyAMA0,115200".into(), "console=tty0".into()]
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_AMBA_PL011=y".into());
        cfg.push("CONFIG_SERIAL_AMBA_PL011_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/arm64/boot/Image".into()
    }
    fn dtb_path(&self) -> Option<String> {
        None
    }
}

// ---------------------------------------------------------------------------
// ArchPpc64 – 64-bit PowerPC (PPC64 / PPC64LE)
// ---------------------------------------------------------------------------

pub struct ArchPpc64;

impl Arch for ArchPpc64 {
    fn virtmename(&self) -> &str {
        "ppc64"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-ppc64"
    }
    fn linuxname(&self) -> &str {
        "powerpc"
    }
    fn gccname(&self) -> &str {
        "powerpc64-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "pseries_defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyS{idx}")
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec![
            "-machine".into(),
            "pseries,accel=kvm:tcg".into(),
            "-cpu".into(),
            "host".into(),
        ]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        default_virtio_pci(kind)
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        default_earlyconsole("ttyS0")
    }
    fn serial_console_args(&self) -> Vec<String> {
        default_serial_console("ttyS0")
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_8250=y".into());
        cfg.push("CONFIG_SERIAL_8250_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/powerpc/boot/zImage".into()
    }
}

// ---------------------------------------------------------------------------
// ArchRiscv64 – 64-bit RISC-V
// ---------------------------------------------------------------------------

pub struct ArchRiscv64;

impl Arch for ArchRiscv64 {
    fn virtmename(&self) -> &str {
        "riscv64"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-riscv64"
    }
    fn linuxname(&self) -> &str {
        "riscv"
    }
    fn gccname(&self) -> &str {
        "riscv64-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyS{idx}")
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec![
            "-machine".into(),
            "virt".into(),
            "-cpu".into(),
            "rv64".into(),
        ]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        // riscv64 on `-machine virt` uses the plain device variant.
        format!("virtio-{kind}-device")
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        default_earlyconsole("ttyS0")
    }
    fn serial_console_args(&self) -> Vec<String> {
        default_serial_console("ttyS0")
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_8250=y".into());
        cfg.push("CONFIG_SERIAL_8250_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/riscv/boot/Image".into()
    }
}

// ---------------------------------------------------------------------------
// ArchSparc64 – 64-bit SPARC
// ---------------------------------------------------------------------------

pub struct ArchSparc64;

impl Arch for ArchSparc64 {
    fn virtmename(&self) -> &str {
        "sparc64"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-sparc64"
    }
    fn linuxname(&self) -> &str {
        "sparc"
    }
    fn gccname(&self) -> &str {
        "sparc64-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyS{idx}")
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec!["-machine".into(), "sun4u,accel=tcg".into()]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        default_virtio_pci(kind)
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        default_earlyconsole("ttyS0")
    }
    fn serial_console_args(&self) -> Vec<String> {
        default_serial_console("ttyS0")
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_8250=y".into());
        cfg.push("CONFIG_SERIAL_8250_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/sparc/boot/image".into()
    }
}

// ---------------------------------------------------------------------------
// ArchS390x – IBM S/390 (64-bit)
// ---------------------------------------------------------------------------

pub struct ArchS390x;

impl Arch for ArchS390x {
    fn virtmename(&self) -> &str {
        "s390x"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-s390x"
    }
    fn linuxname(&self) -> &str {
        "s390"
    }
    fn gccname(&self) -> &str {
        "s390x-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        // s390x uses ttyS1 for the first serial console.
        format!("ttyS{}", idx + 1)
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec!["-machine".into(), "s390-ccw-virtio,accel=kvm:tcg".into()]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        default_virtio_pci(kind)
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        vec!["earlyprintk=sclp".into()]
    }
    fn serial_console_args(&self) -> Vec<String> {
        vec!["console=ttyS1,115200".into()]
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        // s390x has no display; just use -nographic
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        let mut cfg = default_config_base();
        cfg.push("CONFIG_SERIAL_8250=y".into());
        cfg.push("CONFIG_SERIAL_8250_CONSOLE=y".into());
        cfg
    }
    fn kimg_path(&self) -> String {
        "arch/s390/boot/bzImage".into()
    }
}

// ---------------------------------------------------------------------------
// ArchUnknown – fallback
// ---------------------------------------------------------------------------

pub struct ArchUnknown;

impl Arch for ArchUnknown {
    fn virtmename(&self) -> &str {
        "unknown"
    }
    fn qemuname(&self) -> &str {
        "qemu-system-unknown"
    }
    fn linuxname(&self) -> &str {
        "unknown"
    }
    fn gccname(&self) -> &str {
        "unknown-linux-gnu"
    }
    fn defconfig_target(&self) -> &str {
        "defconfig"
    }
    fn serial_dev_name(&self, idx: u32) -> String {
        format!("ttyS{idx}")
    }
    fn qemuargs(&self, _is_native: bool) -> Vec<String> {
        vec!["-nographic".into()]
    }
    fn virtio_dev_type(&self, kind: &str) -> String {
        default_virtio_pci(kind)
    }
    fn earlyconsole_args(&self) -> Vec<String> {
        vec!["earlyprintk=serial,ttyS0,115200".into()]
    }
    fn serial_console_args(&self) -> Vec<String> {
        vec!["console=ttyS0,115200".into()]
    }
    fn qemu_nodisplay_args(&self) -> Vec<String> {
        default_nodisplay()
    }
    fn config_base(&self) -> Vec<String> {
        default_config_base()
    }
    fn kimg_path(&self) -> String {
        "arch/unknown/boot/bzImage".into()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_from_name_x86_64() {
        let a = arch_from_name("x86_64");
        assert_eq!(a.virtmename(), "x86_64");
        assert_eq!(a.qemuname(), "qemu-system-x86_64");
        assert_eq!(a.linuxname(), "x86_64");
    }

    #[test]
    fn arch_from_name_i386() {
        let a = arch_from_name("i386");
        assert_eq!(a.virtmename(), "x86_64");
        assert_eq!(a.qemuname(), "qemu-system-x86_64");
    }

    #[test]
    fn arch_from_name_aarch64() {
        let a = arch_from_name("aarch64");
        assert_eq!(a.virtmename(), "aarch64");
        assert_eq!(a.qemuname(), "qemu-system-aarch64");
        assert_eq!(a.linuxname(), "arm64");
    }

    #[test]
    fn arch_from_name_arm64() {
        let a = arch_from_name("arm64");
        assert!(a.virtmename() == "aarch64");
    }

    #[test]
    fn arch_from_name_arm() {
        let a = arch_from_name("arm");
        assert_eq!(a.virtmename(), "arm");
        assert_eq!(a.serial_dev_name(0), "ttyAMA0");
    }

    #[test]
    fn arch_from_name_ppc64() {
        let a = arch_from_name("ppc64");
        assert_eq!(a.virtmename(), "ppc64");
        assert_eq!(a.kimg_path(), "arch/powerpc/boot/zImage");
    }

    #[test]
    fn arch_from_name_riscv64() {
        let a = arch_from_name("riscv64");
        assert_eq!(a.virtmename(), "riscv64");
        assert_eq!(a.kimg_path(), "arch/riscv/boot/Image");
    }

    #[test]
    fn arch_from_name_sparc64() {
        let a = arch_from_name("sparc64");
        assert_eq!(a.virtmename(), "sparc64");
    }

    #[test]
    fn arch_from_name_s390x() {
        let a = arch_from_name("s390x");
        assert_eq!(a.virtmename(), "s390x");
        // s390x serial devices are numbered from ttyS1
        assert_eq!(a.serial_dev_name(0), "ttyS1");
    }

    #[test]
    fn arch_from_name_unknown() {
        let a = arch_from_name("mips");
        assert_eq!(a.virtmename(), "unknown");
    }

    #[test]
    fn arch_x86_serial() {
        let a = ArchX86;
        assert_eq!(a.serial_dev_name(0), "ttyS0");
        assert_eq!(a.serial_dev_name(3), "ttyS3");
    }

    #[test]
    fn arch_arm_serial() {
        let a = ArchArm;
        assert_eq!(a.serial_dev_name(0), "ttyAMA0");
    }

    #[test]
    fn arch_x86_kimg() {
        let a = ArchX86;
        assert_eq!(a.kimg_path(), "arch/x86_64/boot/bzImage");
    }

    #[test]
    fn arch_nodisplay_args() {
        let a = ArchX86;
        let args = a.qemu_nodisplay_args();
        assert!(args.contains(&"-nographic".to_string()));
    }

    #[test]
    fn arch_earlyconsole() {
        let a = ArchX86;
        let args = a.earlyconsole_args();
        assert!(args[0].contains("earlyprintk=serial"));
    }

    #[test]
    fn arch_virtio_dev_type_pci() {
        let a = ArchX86;
        assert_eq!(a.virtio_dev_type("9p"), "virtio-9p-pci");
    }

    #[test]
    fn arch_virtio_dev_type_device() {
        let a = ArchAarch64;
        assert_eq!(a.virtio_dev_type("9p"), "virtio-9p-device");
    }

    #[test]
    fn arch_s390x_serial_offset() {
        let a = ArchS390x;
        assert_eq!(a.serial_dev_name(0), "ttyS1");
        assert_eq!(a.serial_dev_name(1), "ttyS2");
    }

    #[test]
    fn arch_dtb_path_some() {
        let a = ArchArm;
        assert!(a.dtb_path().is_some());
    }

    #[test]
    fn arch_dtb_path_none() {
        let a = ArchX86;
        assert!(a.dtb_path().is_none());
    }

    #[test]
    fn arch_x86_qemuargs_native() {
        let a = ArchX86;
        let args = a.qemuargs(true);
        assert!(args.contains(&"-cpu".to_string()));
        assert!(args.contains(&"host".to_string()));
        assert!(args.contains(&"-machine".to_string()));
    }

    #[test]
    fn config_base_contains_9p() {
        let a = ArchX86;
        let cfg = a.config_base();
        assert!(cfg.iter().any(|s| s.contains("CONFIG_9P_FS=y")));
        assert!(cfg.iter().any(|s| s.contains("CONFIG_VIRTIO_PCI=y")));
    }
}
