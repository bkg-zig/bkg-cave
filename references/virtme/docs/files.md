# files.md — virtme Dateiinventar

## Übersicht

| Datei                               | Typ                    | Zeilen | Zweck                                |
| ----------------------------------- | ---------------------- | ------ | ------------------------------------ |
| `virtme-run`                        | Shell-Wrapper (Python) | 18     | CLI-Entrypoint: VM starten           |
| `virtme-configkernel`               | Shell-Wrapper (Python) | 18     | CLI-Entrypoint: Kernel konfigurieren |
| `virtme-mkinitramfs`                | Shell-Wrapper (Python) | 15     | CLI-Entrypoint: Initramfs bauen      |
| `bin/virtme-prep-kdir-mods`         | Shell-Skript           | 38     | Kernel-Modul-Verzeichnis vorbereiten |
| `virtme/__init__.py`                | Python                 | 0      | Package-Marker                       |
| `virtme/commands/__init__.py`       | Python                 | 0      | Package-Marker                       |
| `virtme/commands/run.py`            | Python                 | 645    | Hauptlogik: virtme-run               |
| `virtme/commands/configkernel.py`   | Python                 | 150    | Kernel-Konfiguration                 |
| `virtme/commands/mkinitramfs.py`    | Python                 | 56     | Initramfs CLI                        |
| `virtme/architectures.py`           | Python                 | 307    | Architektur-Abstraktion              |
| `virtme/qemu_helpers.py`            | Python                 | 44     | QEMU-Erkennung                       |
| `virtme/mkinitramfs.py`             | Python                 | 172    | Initramfs-Erzeugung                  |
| `virtme/modfinder.py`               | Python                 | 65     | Kernel-Modul-Auflösung               |
| `virtme/virtmods.py`                | Python                 | 27     | Module-Alias-Daten                   |
| `virtme/cpiowriter.py`              | Python                 | 86     | CPIO-Archiv-Writer                   |
| `virtme/util.py`                    | Python                 | 40     | Hilfsfunktionen                      |
| `virtme/resources.py`               | Python                 | 42     | Resource-Pfade                       |
| `virtme/guest/__init__.py`          | Python                 | 0      | Package-Marker                       |
| `virtme/guest/virtme-init`          | Bash                   | 251    | Gast-Init (PID 1)                    |
| `virtme/guest/virtme-udhcpc-script` | Bash                   | 24     | DHCP-Client-Skript                   |
| `setup.py`                          | Python                 | 52     | Build/Package-Konfig                 |
| `make-release-tarball.sh`           | Shell                  | —      | Release-Skript                       |

## CLI-Entrypoints

| CLI                     | Hauptfunktion                         | Aufruf                                    |
| ----------------------- | ------------------------------------- | ----------------------------------------- |
| `virtme-run`            | `virtme.commands.run:main()`          | `python3 -m virtme.commands.run`          |
| `virtme-configkernel`   | `virtme.commands.configkernel:main()` | `python3 -m virtme.commands.configkernel` |
| `virtme-mkinitramfs`    | `virtme.commands.mkinitramfs:main()`  | `python3 -m virtme.commands.mkinitramfs`  |
| `virtme-prep-kdir-mods` | Shell-Skript                          | Direktaufruf                              |

## Python-Package-Struktur

```
virtme/                          ← Package
├── __init__.py
├── architectures.py             ← Arch-Abstraktion (x86, ARM, RISC-V, ...)
├── commands/
│   ├── __init__.py
│   ├── configkernel.py           ← Kernel-Konfiguration
│   ├── mkinitramfs.py            ← Initramfs-CLI
│   └── run.py                    ← Kerns: VM-Bau + Start
├── cpiowriter.py                ← CPIO-Schreiber
├── guest/
│   ├── __init__.py
│   ├── virtme-init              ← Shell: Gast-Init
│   └── virtme-udhcpc-script     ← Shell: DHCP
├── mkinitramfs.py               ← Initramfs-Konfig + Bau
├── modfinder.py                 ← Modul-Auflösung
├── qemu_helpers.py              ← QEMU-Findung + Version
├── resources.py                 ← Pfad-Auflösung
├── util.py                      ← Binary-Suche
└── virtmods.py                  ← Module-Alias-Daten
```

## Import-Graph

```
run.py
  ├── virtmods          (Module-Aliase)
  ├── modfinder         (Modul-Auflösung)
  ├── mkinitramfs       (Initramfs-Konfig + Bau)
  ├── qemu_helpers      (QEMU-Erkennung)
  ├── architectures     (Arch-Abstraktion)
  └── resources         (Pfad-Suche)

configkernel.py
  └── architectures     (Arch-Abstraktion)

mkinitramfs.py (CLI)
  ├── modfinder
  ├── virtmods
  └── mkinitramfs       (Initramfs-Konfig + Bau)

mkinitramfs.py (Lib)
  ├── cpiowriter        (CPIO-Schreiben)
  └── util              (Binary-Suche)

modfinder.py
  └── util              (Binary-Suche)
```
