# analysis.md — ugrd (MicrogRAM disk)

> **Analyseziel**: Verhaltensgleiche Rust-Portierung als `bkg-cave-boot`.
> **Quelle**: https://github.com/desultory/ugrd (GPL-2.0)
> **Version im Mirror**: HEAD (Quelltext gelesen aus `references/ugrd/upstream/ugrd.git/src/`)

---

## 1. Gesamtübersicht

ugrd (MicrogRAM Disk) ist ein generischer, TOML-konfigurierter Initramfs-Generator für Linux.
Im Gegensatz zu klassischen `mkinitramfs`-Tools nutzt ugrd ein **modulares Plugin-System**:
Konfigurationsmodule (`ugrd.base.*`) registrieren Parameter, liefern Import-Funktionen für
Shell-Hooks und können voneinander abhängen (`needs`, `provides`).

| Merkmal | Wert |
|---------|------|
| Originalsprache | Python ≥ 3.11 |
| Lizenz | GPL-2.0 |
| Zeilen gesamt (Python) | ~7.700 (ohne Tests/Hooks) |
| Hauptmodul | `ugrd` |
| Entrypoint | `ugrd.main:main` |
| Kernidee | Konfig-Module → Shell-Code → CPIO |

---

## 2. Architektur

### 2.1 Schichten

```
┌─────────────────────────────────────────────────────────┐
│ main.py                                                   │  CLI-Entrypoint
│ ├── argparse (30+ Flags)                                 │
│ ├── --test / --livecd / --validate / ...                 │
│ └── InitramfsGenerator(**kwargs)                         │
└────────────────────────┬──────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────┐
│ initramfs_generator.py                                   │  Orchestrierung
│ ├── build()                                              │
│ │   ├── run_build()       (build_* hooks)               │
│ │   ├── generate_init()   (init_* hooks)                │
│ │   ├── pack_build()      (pack hook)                   │
│ │   └── run_checks/run_tests                            │
│ ├── run_hook() / run_func()                              │
│ ├── generate_profile()  → /etc/profile (Shell-Funktionen)│
│ └── generate_init_main() → init-Skript (Shell)           │
└────────────────────────┬──────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────┐
│ initramfs_config.py                                      │  Konfiguration
│ ├── InitramfsConfig (UserDict)                           │
│ │   ├── builtin_parameters (modules, imports, …)        │
│ │   ├── stages: early → late → final                    │
│ │   ├── Queues: _processing, _late_args                 │
│ │   └── _process_* (Typ-Handler pro Parameter)          │
│ └── _load_module() → TOML-Parsing + dynamisches Laden   │
└────────────────────────┬──────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────┐
│ generator_helpers.py                                     │  Gemeinsame Basis
│ └── GeneratorHelpers                                      │
│     ├── shell_functions (einfo/ewarn/eerror/edebug/…)   │
│     ├── _mkdir/_copy/_symlink/_write                     │
│     └── _run (subprocess im Build-Dir)                   │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Datenfluss

```
TOML-Config
│
├── base.base (Standardmodul, Minimal-Set)
│   └── registriert: shebang, switch_root, shell-functions
│
├── Module laden (_process_modules)
│   ├── imports: {hook → [function, …]}
│   ├── needs/provides: Abhängigkeit zwischen Modulen
│   └── custom_parameters: zusätzliche Config-Schlüssel
│
├── Build-Phase (run_build)
│   ├── build_enum / build_pre / build_tasks
│   ├── build_late / build_deploy / build_final
│   └── Jeder Hook gibt Shell-Code (str | list[str]) zurück
│
├── Init-Phase (generate_init)
│   ├── functions → in /etc/profile eingefügt
│   ├── init_pre / init_debug / init_main / init_mount
│   ├── init_final (switch_root als letzter Schritt)
│   └── → /init (ausführbares Shell-Skript)
│
└── Packen
    ├── CPIO-Archiv (via pycpio)
    └── Optionale Komprimierung (xz, gz, zstd)
```

### 2.3 Wichtige Klassen und Module

| Klasse/Modul | Datei | Verantwortung |
|-------------|-------|----------------|
| `InitramfsConfig` | `initramfs_config.py` | Konfigurations-Dict mit Queue |
| `InitramfsGenerator` | `initramfs_generator.py` | Haupt-Orchestrator |
| `GeneratorHelpers` | `generator_helpers.py` | Shell-Templates + Datei-Ops |
| `InitramfsProtocol` | `initramfs_protocol.py` | Provider-Protokoll |
| `base.base` | `base/base.py` | Basis-Modul (Shell-Funktionen, boot-logic) |
| `base.core` | `base/core.py` | Minimal-Core (ohne UI) |
| `base.udev` | `base/udev.py` | udev-Einbindung |
| `base.test` | `base/test.py` | Test-Modul (QEMU-Boot) |
| `fs.mounts` | `fs/mounts.py` | Mount-Management (1164 Zeilen, größte Datei) |
| `kmod.kmod` | `kmod/kmod.py` | Kernel-Modul-Auflösung |
| `fs.cpio` | `fs/cpio.py` | CPIO-Paketierung |
| `fs.overlayfs` | `fs/overlayfs.py` | OverlayFS-Unterstützung |
| `crypto.cryptsetup` | `crypto/cryptsetup.py` | LUKS-Entsperrung |

### 2.4 Shell-Hooks

In `base.py` und `core.py` werden Shell-Funktionen als Python-Strings definiert:

- `einfo`, `ewarn`, `eerror`, `edebug` — Logging mit optionalem Plymouth
- `readvar` / `setvar` — Variablen in `/run/ugrd/<name>`
- `check_var` — Kernel-Cmdline auswerten
- `prompt_user` / `wait_for_space` — Interaktive Prompts
- `retry` — Kommando-Wiederholung
- `do_switch_root` — Finaler Root-Switch
- `rd_fail` / `rd_restart` — Fehlerbehandlung

### 2.5 Konfigurationsmodell

ugrd arbeitet mit **drei Config-Stages**:

1. **early** — Basismodule laden, CLI-Args verarbeiten
2. **late** — Module einbinden, `_late_args` verarbeiten
3. **final** — Validierung, kein weiteres Setzen erlaubt

Jeder Parameter hat einen Typ (`list`, `dict`, `bool`, `int`, `str`, `Path`, `PyCPIO`, …).
Unbekannte Werte werden in eine `Queue` gestellt, bis der Typ bekannt ist.

---

## 3. Module

### 3.1 Core-Module

| Pfad | Zeilen | Aufgabe |
|------|--------|---------|
| `ugrd/main.py` | 230 | CLI, argparse, Generator starten |
| `ugrd/initramfs_generator.py` | 276 | Haupt-Orchestrator |
| `ugrd/initramfs_config.py` | 554 | Konfig-Dict, Module laden, Queues |
| `ugrd/initramfs_protocol.py` | — | Protokoll-Typen |
| `ugrd/config_helpers.py` | — | Hilfsfunktionen für Config |
| `ugrd/generator_helpers.py` | 387 | Shell-Templates + Datei-Ops |
| `ugrd/exceptions.py` | — | `AutodetectError`, `ValidationError` |

### 3.2 Filesystem-Module (`ugrd/fs/`)

| Modul | Aufgabe |
|-------|---------|
| `mounts.py` | Mount-Management (größte Datei, 1164 Z.) |
| `cpio.py` | CPIO-Archiv-Erzeugung |
| `overlayfs.py` | OverlayFS-Setup |
| `btrfs.py` | Btrfs-Unterstützung |
| `ext4.py` | ext4-Erkennung |
| `f2fs.py` | F2FS-Erkennung |
| `lvm.py` | LVM-Integration |
| `mdraid.py` | MD-RAID-Integration |
| `zfs.py` | ZFS-Integration |
| `resume.py` | Resume-from-Disk |
| `livecd.py` | LiveCD-Image-Support |
| `fakeudev.py` | Experimentelles udev |
| `test_image.py` | Test-Image-Erzeugung |

### 3.3 Kmod-Module (`ugrd/kmod/`)

| Modul | Aufgabe |
|-------|---------|
| `kmod.py` | Kernel-Modul-Handling |
| `input.py` | Input-Modul (lspci/lsmod) |
| `kconfig.py` | Kernel-Config-Parsing |
| `platform.py` | Plattform-Erkennung |
| `standard_mask.py` | Standard-Masken |

### 3.4 Crypto-Module (`ugrd/crypto/`)

| Modul | Aufgabe |
|-------|---------|
| `cryptsetup.py` | LUKS/LUKS2-Entsperrung |
| `gpg.py` | GPG-Entschlüsselung |
| `smartcard.py` | Smartcard/PKCS#11 |

### 3.5 Base-Module (`ugrd/base/`)

| Modul | Aufgabe |
|-------|---------|
| `base.py` | Basis-Shell-Funktionen |
| `core.py` | Minimal-Core (ohne Plymouth/UI) |
| `udev.py` | udev-Einbindung |
| `plymouth.py` | Plymouth-Splash-Support |
| `keymap.py` | Tastaturlayout |
| `console.py` | Konsolen-Setup |
| `fonts.py` | Fonts / Framebuffer |
| `banner.py` | Boot-Banner |
| `bell.py` | Boot-Beep |
| `cmdline.py` | Kernel-Cmdline-Parsing |
| `debug.py` | Debug-Optionen |
| `checks.py` | Build-Prüfungen |
| `test.py` | QEMU-Test-Runner |

### 3.6 Netzwerk (`ugrd/net/`)

| Modul | Aufgabe |
|-------|---------|
| `net.py` | Netzwerk-Grundlagen |
| `dhcpcd.py` | DHCP-Client |
| `static.py` | Statische IP |

---

## 4. Buildprozess

1. `ugrd.main:main()` → argparse → `kwargs`
2. `InitramfsGenerator(**kwargs)` → lädt Config-Module
3. `generator.build()`:
   - `run_build()` → baut Dateistruktur im Build-Dir
   - `generate_init()` → erzeugt `/init` + `/etc/profile`
   - `pack_build()` → CPIO-Archiv (ggf. komprimiert)
   - `run_checks()` → Prüfungen
   - `run_tests()` → Tests
4. Ergebnis: `initramfs.img` (oder konfigurierter Name)

---

## 5. Externe Abhängigkeiten

| Abhängigkeit | Zweck |
|--------------|-------|
| Python ≥ 3.11 | Runtime |
| `zenlib >= 3.5.0` | Logging, TOML, Utility-Funktionen |
| `pycpio >= 1.7.0` | CPIO-Archiv-Erzeugung |
| `zstandard` (optional) | ZSTD-Kompression |
| `lddtree` (optional) | Dynamische Bibliothek-Auflösung |
| `busybox` (optional) | Statische Binaries im Initramfs |

### 5.1 zenlib-Abhängigkeiten (kritisch)

ugrd hängt stark von `zenlib` ab (extern, nicht Teil dieses Projekts):
- `zenlib.util.get_args_n_logger` — CLI + Logger-Setup
- `zenlib.util.colorize` — Terminal-Farben
- `zenlib.util.parse_toml` — TOML-Konfiguration lesen
- `zenlib.util.handle_plural` — Plural-Handler-Decorator
- `zenlib.logging.LoggerMixIn` — Logging-Mixin
- `zenlib.types.NoDupFlatList` — Deduplizierte Liste
- `zenlib.types.Protocol` — Basis-Protokoll (PyCPIO)

→ **Rust-Port muss diese Funktionalität nativ ersetzen**.

---

## 6. Bekannte Einschränkungen

1. **Python ≥ 3.11** erforderlich
2. **zenlib** als zentraler externer Baustein — vollständig ersetzen
3. **pycpio** durch Rust-CPIO-Library ersetzen
4. **Shell-Code-Generierung**: Alle Hook-Funktionen geben Shell-Strings zurück
   — diese müssen in Rust als `String` generiert und geschrieben werden
5. **Dynamisches Modul-Laden**: `importlib` + `/var/lib/ugrd/` sideloading
   → In Rust: Plugin-System oder Compile-Time-Module
6. **TOML-Konfiguration**: Großer Teil der Logik steckt in TOML-Modulen
7. **Experimentelle Features**: fakeudev, Xen-ähnliche Konzepte

