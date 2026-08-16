# dependencies.md — virtme Abhängigkeiten

## Python-Standardbibliothek

| Modul             | Verwendung in                                                                    |
| ----------------- | -------------------------------------------------------------------------------- |
| `argparse`        | run.py, configkernel.py, mkinitramfs.py                                          |
| `os`              | Alle Module                                                                      |
| `sys`             | Alle Module                                                                      |
| `subprocess`      | run.py, configkernel.py, modfinder.py, resources.py                              |
| `tempfile`        | run.py                                                                           |
| `shutil`          | configkernel.py, qemu_helpers.py, util.py                                        |
| `shlex`           | run.py, configkernel.py, mkinitramfs.py                                          |
| `fcntl`           | run.py                                                                           |
| `re`              | run.py, qemu_helpers.py, modfinder.py                                            |
| `io`              | mkinitramfs.py                                                                   |
| `errno`           | run.py                                                                           |
| `itertools`       | run.py, modfinder.py, mkinitramfs.py, util.py                                    |
| `multiprocessing` | configkernel.py                                                                  |
| `typing`          | run.py, architectures.py, qemu_helpers.py, mkinitramfs.py, modfinder.py, util.py |

## Externe Python-Pakete

| Paket           | Verwendung   | Zweck                       |
| --------------- | ------------ | --------------------------- |
| `pkg_resources` | resources.py | Resource-Pfade (setuptools) |
| `setuptools`    | setup.py     | Build/Package               |

## Externe Laufzeit-Binaries

| Binary               | Pfad/Bedingung                       | Verwendung in   |
| -------------------- | ------------------------------------ | --------------- |
| `qemu-system-<arch>` | `shutil.which()`                     | qemu_helpers.py |
| `qemu-kvm`           | Fallback bei nativer Arch            | qemu_helpers.py |
| `modprobe`           | `/usr/sbin/modprobe`                 | modfinder.py    |
| `busybox`            | Suche in Root-PATH + Standard-Pfaden | mkinitramfs.py  |
| `depmod`             | (via virtme-prep-kdir-mods)          | Shell-Skript    |
| `udevadm`            | (im Gast)                            | virtme-init     |
| `busybox udhcpc`     | (im Gast)                            | virtme-init     |
| `mount`/`umount`     | (im Gast)                            | virtme-init     |

## Kernel-Requirements (Gast-Kernel)

| Option                | Typ             | Zweck                      |
| --------------------- | --------------- | -------------------------- |
| CONFIG_VIRTIO         | built-in/module | VirtIO-Framework           |
| CONFIG_VIRTIO_PCI     | built-in/module | VirtIO-PCI-Transport       |
| CONFIG_NET_9P         | built-in/module | 9p over network            |
| CONFIG_NET_9P_VIRTIO  | built-in/module | 9p over VirtIO             |
| CONFIG_9P_FS          | built-in/module | 9p-Dateisystem             |
| CONFIG_VIRTIO_NET     | built-in/module | VirtIO-Netzwerk            |
| CONFIG_VIRTIO_CONSOLE | built-in/module | VirtIO-Serial (Scripting)  |
| CONFIG_SCSI_VIRTIO    | built-in/module | VirtIO-SCSI (Disk)         |
| CONFIG_BINFMT_SCRIPT  | built-in        | Shell-Script-Unterstützung |
| CONFIG_TMPFS          | built-in        | Temporäre Dateisysteme     |
| CONFIG_DEVTMPFS       | built-in        | Device-Verwaltung          |

## Host-Requirements

| Komponente         | Version                          | Optional                                  |
| ------------------ | -------------------------------- | ----------------------------------------- |
| Python             | >= 3.3                           | Nein                                      |
| QEMU               | >= 1.6 (1.4 mit Einschränkungen) | Nein                                      |
| KVM                | Beliebig                         | Ja                                        |
| busybox (statisch) | Beliebig                         | Ja (für initramfs)                        |
| Bubblewrap         | —                                | Wird nicht verwendet (anders als osbuild) |
