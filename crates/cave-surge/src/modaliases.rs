//! Static module alias table used to discover the minimal set of kernel
//! modules needed to boot a VM via QEMU + 9p virtfs.
//!
//! Each entry is a modalias string that `modprobe --show-depends` can resolve
//! against the installed module tree (`/lib/modules/$(uname -r)/`).
//!
//! The list covers:
//! - 9p2000 filesystem and network modules for the virtfs root mount.
//! - Virtio transport (PCI, MMIO) for device access.
//! - Basic input (PS/2, AT keyboard, USB HID).
//! - ACPI for power management.
//! - Storage (PATA, SATA, AHCI, NVMe).
//! - Network (virtio-net, e1000, rtl8139).
//! - Console/serial (virtio-console, 8250).
//! - Miscellaneous (fuse, loop, ext4, btrfs, squashfs).

/// Modalias strings for auto-detecting required kernel modules.
pub const MODALIASES: &[&str] = &[
    // ── 9p / virtfs ───────────────────────────────────────────────────────────
    "fs-9p",
    "virtio:d00000001v00001AF4",
    "virtio:pci00000000.device",
    // 9pnet
    "virtio:d00000009v00001AF4",
    // 9pnet_virtio
    "virtio:d00000002v00001AF4",
    // PCI vendor 0x1AF4 (Red Hat / QEMU) virtio devices
    "pci:v00001AF4d00001009sv00001AF4sd00001009bc00sc00i00",
    "pci:v00001AF4d00001001sv00001AF4sd00001001bc01sc01i00",
    "pci:v00001AF4d00001002sv*bc*sd*",
    "pci:v00001AF4d00001003sv*bc*sd*",
    "pci:v00001AF4d00001004sv*bc*sd*",
    // ── Virtio transport ──────────────────────────────────────────────────────
    "virtio:pci*",
    "virtio:mmio*",
    // ── Input ─────────────────────────────────────────────────────────────────
    "i8042",
    "atkbd",
    "serio:ty01pr00fc00",
    "psmouse",
    "input*",
    // ── ACPI ──────────────────────────────────────────────────────────────────
    "acpi",
    "acpi:*:PNP0C0C:*", // power button
    "acpi:*:PNP0C0E:*", // sleep button
    "acpi:*:PNP0303:*", // i8042 keyboard
    "acpi:*:PNP0501:*", // 16550A-compatible serial
    // ── Storage ───────────────────────────────────────────────────────────────
    "pci:v00008086d00007010sv*bc*sd*", // PIIX4 PATA
    "pci:v00008086d00007110sv*bc*sd*", // SATA AHCI
    "pci:v00001AF4d00001042sv*bc*sd*", // virtio-blk
    "pci:v00001AF4d00001048sv*bc*sd*", // virtio-scsi
    "scsi:t-0x00",                     // SCSI disk
    // ── Network ───────────────────────────────────────────────────────────────
    "pci:v00001AF4d00001000sv*bc*sd*", // virtio-net
    "pci:v00008086d0000100Esv*bc*sd*", // e1000
    "pci:v000010ECd00008139sv*bc*sd*", // rtl8139
    // ── Console / serial ──────────────────────────────────────────────────────
    "virtio:d00000003v00001AF4", // virtio-console
    "pnp:dPNP0501",              // 16550A UART
    "pnp:dPNP0500",              // 16450 UART
    // ── USB ───────────────────────────────────────────────────────────────────
    "usb:v1D6Bp0001d*", // USB 1.1 (UHCI)
    "usb:v1D6Bp0002d*", // USB 2.0 (EHCI)
    "usb:v1D6Bp0003d*", // USB 3.0 (xHCI)
    "usb-storage",
    // ── Filesystem ────────────────────────────────────────────────────────────
    "fs-ext4",
    "fs-btrfs",
    "fs-squashfs",
    "fs-fuse",
    "fs-iso9660",
    "fs-vfat",
    // ── Misc ──────────────────────────────────────────────────────────────────
    "unix",
    "loop",
];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modalias_list_not_empty() {
        assert!(!MODALIASES.is_empty(), "MODALIASES should not be empty");
    }

    #[test]
    fn modalias_contains_9p() {
        assert!(MODALIASES.contains(&"fs-9p"));
    }

    #[test]
    fn modalias_contains_virtio() {
        assert!(MODALIASES.iter().any(|m| m.contains("virtio")));
    }

    #[test]
    fn modalias_contains_input() {
        assert!(MODALIASES.contains(&"i8042"));
        assert!(MODALIASES.contains(&"atkbd"));
    }

    #[test]
    fn modalias_contains_acpi() {
        assert!(MODALIASES.contains(&"acpi"));
    }

    #[test]
    fn modalias_no_empty_strings() {
        for m in MODALIASES {
            assert!(!m.is_empty(), "modalias entry should not be empty");
        }
    }

    #[test]
    fn modalias_all_unique() {
        let mut seen = std::collections::HashSet::new();
        for m in MODALIASES {
            assert!(seen.insert(m), "duplicate modalias: {m}");
        }
    }
}
