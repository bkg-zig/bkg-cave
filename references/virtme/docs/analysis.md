# analysis.md — virtme Architekturanalyse

## Gesamtübersicht

virtme ist ein Toolkit zum Starten eines virtualisierten Linux-Kernels, der die Host-Distribution als RootFS verwendet — ohne separates Disk-Image. Der Fokus liegt auf einfacher Kernel-Entwicklung und -Tests.

## Architektur

```
┌──────────────────────────────────────────────────────────────────┐
│                        virtme-run (CLI)                          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ argparse     │  │ find_kernel  │  │ mkinitramfs          │  │
│  │ → args       │  │ → Kernel     │  │ → CpioWriter         │  │
│  └──────────────┘  └──────┬───────┘  │ → busybox            │  │
│                           │           │ → modules            │  │
│                           │           │ → /init (Shell)      │  │
│                           ▼           └──────────┬───────────┘  │
│  ┌──────────────────────────────────────────────┐│              │
│  │           QEMU-Argumente-Builder               │              │
│  │  ┌─────────┐ ┌───────────┐ ┌──────────────┐ │              │
│  │  │ VirtFS  │ │ KVM/Accel│ │ Console/Net  │ │              │
│  │  └─────────┘ └───────────┘ └──────────────┘ │              │
│  └──────────────────────┬───────────────────────┘              │
│                         ▼                                      │
│              os.execv(qemu, args)                              │
└─────────────────────────┬────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────┐
│                      QEMU (Gast)                                 │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │         Kernel + virtme-init (PID 1)                     │   │
│  │                                                          │   │
│  │  ┌────────┐  ┌───────────┐  ┌────────────────────────┐  │   │
│  │  │ sysfs  │  │ 9p-Mounts │  │ udev + hotplug        │  │   │
│  │  │ proc   │  │ /dev/root │  │  ─ trigger + settle   │  │   │
│  │  │ tmpfs  │  │ guesttools│  │  ─ modules laden      │  │   │
│  │  └────────┘  │ mounts    │  └────────────────────────┘  │   │
│  │               └───────────┘                              │   │
│  │                                                          │   │
│  │  ┌────────────┐  ┌──────────────┐  ┌────────────────┐  │   │
│  │  │ Console    │  │ Networking   │  │ Script-Mode    │  │   │
│  │  │ (serial)   │  │ (DHCP)       │  │ → poweroff     │  │   │
│  │  └────────────┘  └──────────────┘  └────────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

## Module

| Modul                      | Verantwortlichkeit                                                                 | SLOC |
| -------------------------- | ---------------------------------------------------------------------------------- | ---- |
| `commands/run.py`          | CLI-Parsing, Kernel-Discovery, QEMU-Builder, Initramfs-Integration                 | 645  |
| `commands/configkernel.py` | Kernel .config generieren/aktualisieren                                            | 150  |
| `commands/mkinitramfs.py`  | Initramfs-CLI (Wrapper um mkinitramfs.py)                                          | 56   |
| `architectures.py`         | 8 Architektur-Profile (x86, ARM, aarch64, ppc64, riscv64, sparc64, s390x, unknown) | 307  |
| `qemu_helpers.py`          | QEMU-Binary-Findung, Version-Erkennung, Argument-Quote                             | 44   |
| `mkinitramfs.py`           | Initramfs-Konfiguration, Aufbau (Layout, busybox, module, /init)                   | 172  |
| `modfinder.py`             | Kernel-Modul-Auflösung via modprobe                                                | 65   |
| `cpiowriter.py`            | CPIO-Archiv-Schreiber (newc-Format)                                                | 86   |
| `virtmods.py`              | Module-Alias-Liste (statisch)                                                      | 27   |
| `util.py`                  | Binary-Suche in Pfaden                                                             | 40   |
| `resources.py`             | Resource-Discovery (Gast-Tools, Scripts)                                           | 42   |

## Datenfluss

```
CLI-Args
  │
  ▼
argparse (run.py:make_parser)
  │
  ▼
architectures.get(args.arch) → Arch-Objekt
  │
  ▼
qemu_helpers.Qemu(arch.qemuname) → Qemu-Objekt
  │
  ▼
find_kernel_and_mods(arch, args) → Kernel-Objekt
  │  ├── --installed-kernel → /lib/modules/<ver> + /boot/vmlinuz
  │  ├── --kdir → <kdir>/arch/.../bzImage + .virtme_mods
  │  └── --kimg → direkter Pfad (keine Module)
  │
  ▼
args.qemu_opt + args.qemu_opts → Sammeln von Zusatzoptionen
  │
  ▼
Initramfs bauen (falls benötigt)
  │  ├── CpioWriter: CPIO-Archiv erstellen
  │  ├── busybox + Symlinks + Module + /init
  │  └── os.execv ersetzt Prozess durch QEMU
  │
  ▼
QEMU startet → Kernel bootet → virtme-init (PID 1)
```

## Buildprozess

- Python-Code, keine Kompilierung nötig
- Versand via `setup.py` (setuptools)
- Entrypoints: `virtme-run`, `virtme-configkernel`, `virtme-mkinitramfs`
- Package-Daten: Gast-Tools (`virtme-init`, `virtme-udhcpc-script`)
- Keine externen Python-Abhängigkeiten (nur stdlib + setuptools/pkg_resources)

## Externe Bibliotheken

- **Keine** externen Python-Pakete zur Laufzeit
- `pkg_resources` (setuptools) nur für Resource-Discovery
- Laufzeit-Binaries: QEMU, modprobe, busybox (optional)

## Bekannte Einschränkungen

1. **Python 3.3+** erforderlich (keine älteren Versionen)
2. **QEMU < 1.6**: Kein `multidevs=remap`, eingeschränktes VirtFS
3. **QEMU < 4.2**: Kein `multidevs=remap` für korrekte hardlink-Behandlung
4. **Busybox** nur bei initramfs-Bedarf gefunden
5. **Xen**-Support experimentell (multiboot)
6. **Bridge-Netzwerk** experimentell (ohne WDS-Support i.d.R. funktionslos)
7. **Kein DTB-Support** für --kimg (nur --kdir)
8. **PS/2-Tastatur-Erkennung** auf einigen Systemen langsam
9. **Architektur-unbekannt** Fallback ohne spezifische Optimierungen
