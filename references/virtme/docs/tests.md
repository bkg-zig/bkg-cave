# tests.md — virtme Tests

## Originale Tests

Keine Test-Dateien im Repository vorhanden.

## Geplante Tests (Rust-Port)

### Unit Tests

| Bereich       | Test                      | Beschreibung                         |
| ------------- | ------------------------- | ------------------------------------ |
| CpioWriter    | `test_cpio_basic`         | CPIO-Archiv schreiben und validieren |
| CpioWriter    | `test_cpio_mkdir`         | Verzeichnis-Eintrag im CPIO          |
| CpioWriter    | `test_cpio_symlink`       | Symlink-Eintrag im CPIO              |
| CpioWriter    | `test_cpio_chardev`       | Device-Node im CPIO                  |
| CpioWriter    | `test_cpio_trailer`       | TRAILER!!!-Eintrag + Padding         |
| Architectures | `test_arch_get_known`     | Bekannte Architekturen auflösbar     |
| Architectures | `test_arch_get_unknown`   | Unbekannte Arch → Arch_unknown       |
| Architectures | `test_arch_x86_qemuargs`  | x86-QEMU-Argumente korrekt           |
| Architectures | `test_arch_arm_kimg_path` | ARM zImage-Pfad                      |
| QemuHelpers   | `test_quote_optarg`       | Komma-Quoting korrekt                |
| QemuHelpers   | `test_has_multidevs`      | Version-Scan für multidevs           |
| QemuHelpers   | `test_cannot_overmount`   | Version-Scan für virtfs              |
| Util          | `test_find_binary`        | Binary-Suche in Pfaden               |

### Integration Tests

| Test                              | Beschreibung                             |
| --------------------------------- | ---------------------------------------- |
| `test_kernel_args_build`          | Kernel-Argumente korrekt zusammengesetzt |
| `test_virtfs_export`              | 9p-Freigabe-Kommandozeile                |
| `test_disk_args`                  | --disk Parsing + Validierung             |
| `test_script_mode`                | Script-Modus-Kommandozeilenaufbau        |
| `test_network_args`               | Netzwerk-Optionen (user/bridge)          |
| `test_kernel_discovery_installed` | find_kernel für installed-kernel         |
| `test_qemu_version_parsing`       | QEMU-Version aus --version-String        |

### Golden Tests

| Test                       | Beschreibung                      |
| -------------------------- | --------------------------------- |
| `test_golden_initramfs`    | CPIO-Archiv-Inhalt gegen Referenz |
| `test_golden_qemu_cmdline` | QEMU-Kommandozeile gegen Referenz |

## Noch fehlend (nach Port)

- End-to-End-Test mit tatsächlichem QEMU-Start
- Modul-Auflösung via modprobe (Mock)
- Shell-Skript-Korrektheit (virtme-init)
