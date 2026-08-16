# analysis.md — bkg-cave Referenzprojekte

> **Analyseziel:** Verhaltensgleiche Rust-Portierung der Komponenten bkg-cave-boot (ugrd) und bkg-cave-build (osbuild).
> **Quelle:** Referenzprojekte in references/ugrd/ und references/osbuild/

---

## 1. Gesamtübersicht

### 1.1 bkg-cave-boot (ugrd)

µgRD (Microgram Ramdisk) ist ein Framework zur Generierung von POSIX-kompatiblen Initramfs-Images anhand von TOML-Definitionen und Python-Funktionen. Die zentrale Komponente erzeugt ein Shell-basiertes Init-Skript sowie ein CPIO-Archiv, das als Initramfs dient.

| Merkmal | Wert |
|---|---|
| Originalsprache | Python >= 3.11 |
| Lizenz | GPLv2 |
| Autor | Desultory |
| Repository | https://github.com/desultory/ugrd |
| Version | 2.2.0 |
| Hauptmodul | bkg-cave-boot |
| Entrypoint | bkg-cave-boot.main:main |
| Kernidee | Modulbasierte Initramfs-Generierung mit Hook-System |

Archiviert unter: references/ugrd/upstream/ugrd.git/

### 1.2 bkg-cave-build (osbuild)

bkg-cave-build ist ein pipeline-basiertes Build-System fuer Betriebssystem-Images. Stages, Assembler und Sources werden in einer Manifest-Datei (JSON) beschrieben und von einem Pipeline-Executor ausgefuehrt — isoliert in einer Bubblewrap-Sandbox. Die zentrale Abstraktion ist der Stage mit eindeutiger ID basierend auf SHA-256 der Optionen, Inputs und Mounts.

| Merkmal | Wert |
|---|---|
| Originalsprache | Python 3.6+ |
| Lizenz | Apache-2.0 |
| Repository | https://github.com/bkg-cave-build/bkg-cave-build |
| Hauptmodul | bkg-cave-build |
| Entrypoint | bkg-cave-build.main_cli:bkg-cave-build_cli |
| Kernidee | Pipeline / Stages / Buildgraph / Sandbox-Ausfuehrung |

Archiviert unter: references/osbuild/upstream/bkg-cave-build.git/

---

## 2. Systemarchitektur

### 2.1 bkg-cave-boot (ugrd)

bkg-cave-boot verwendet eine Schichtenarchitektur mit einer Plugin-basierten Erweiterung:

```
CLI-Eingabe (argparse)
  |
  v
InitramfsGenerator (Kernklasse)
  |-- GeneratorHelpers (Mixin: Datei-Operationen, Shell-Ausfuehrung)
  |-- InitramfsConfig (UserDict mit typisiertem Setitem)
  |
  +-- Initialisierungsphase
  |     |-- Basis-Module laden (bkg-cave-boot.base.base oder bkg-cave-boot.base.core)
  |     |-- User-TOML-Config parsen und einlesen
  |     |-- CLI-Argumente ueberlagern
  |     -- Deferred-Parameter in _processing-Queue zwischenspeichern
  |
  +-- Build-Phase
  |     |-- run_build(): Hook-Aufrufe (build_enum, build_pre, build_tasks,
  |     |                     build_late, build_deploy, build_final)
  |     |-- Jeder Hook fuehrt registrierte Funktionen in Reihenfolge aus
  |     -- Import-Reihenfolge kann via import_order (before/after) gesteuert werden
  |
  +-- Init-Generierung
  |     |-- run_hook("functions", force_include=True)
  |     |-- run_hook("init_pre")
  |     |-- custom_init oder generate_init_main() (init_debug, init_main, init_mount)
  |     |-- run_hook("init_final")
  |     |-- generate_profile() -> /etc/profile erzeugen
  |     -- _write("init", init_script, 0o755)
  |
  +-- Pack-Phase
  |     |-- run_hook("pack", force_exclude) oder CPIO-Archiv via PyCPIO
  |     -- Ausgabe: initramfs.cpio (ggf. xz-komprimiert)
  |
  +-- Validierung & Tests
        |-- run_checks() -> hook "checks"
        -- run_tests() -> hook "tests"
```

#### Phase-basierte Verarbeitung

Die Konfiguration wird in drei Stufen verarbeitet:

| Phase | Beschreibung |
|---|---|
| early | Initiale Konfiguration, Basis-Module laden |
| late | spaete Parameter werden verarbeitet, _late_args entleert |
| final | Validierung, Setzen des validated-Flags |

#### Plugin-Architektur (bestehend)

bkg-cave-boot besitzt ein eingebautes Plugin-System:

1. **Modul-Load-Mechanismus**: imports in der TOML-Config definieren, welche Funktionen aus welchen Modulen geladen werden
2. **Hook-System**: Funktionen werden unter Hook-Namen (init_pre, init_main, init_mount, build_pre, build_tasks, build_late, build_deploy, build_final, pack, checks, tests, functions) registriert
3. **Import-Reihenfolge**: import_order mit before und after Regeln erlaubt explizite Ordnung
4. **Dynamisches Sideloading**: Module unter /var/lib/ugrd koennen zur Laufzeit geladen werden (_import_external_module)
5. **Masking**: Ueber masks koennen Hook-Funktionen deaktiviert werden
6. **Custom Processing**: Ueber _process_* Funktionen koennen benutzerdefinierte Verarbeitungslogiken registriert werden

#### Hot-Reloading-Basis (bestehend)

bkg-cave-boot unterstützt bereits eine Form von Hot-Reloading:

- **Deferred Processing**: Unbekannte Parameter werden in _processing (Queue) zwischengespeichert und verarbeitet, sobald der Typ bekannt ist
- **Late Args**: Parameter, die erst in der late-Phase verarbeitet werden, werden automatisch entleert
- **Modul-Neustladung**: Externe Module aus /var/lib/ugrd koennen geaendert und bei neuem Build geladen werden
- **Validierungsflag**: validated verhindert nachtraegliche Konfigurationsaenderungen nach Finalisierung

### 2.2 bkg-cave-build (osbuild)

bkg-cave-build verwendet eine Pipeline-Architektur mit Sandbox-Ausfuhrung:

```
CLI-Eingabe (argparse, main_cli.py)
  |
  v
Meta.Index (meta.py)
  |-- Index: stages/, assemblers/, schemas/ dynamisch laden
  |-- Format-Erkennung (Manifest JSON pruefen)
  -- Runner-Identifikation (os-release des Hosts)
  |
  v
Manifest laden (pipeline.py)
  |-- Quelle: JSON-Datei oder stdin
  |-- sources definieren
  |-- pipelines definieren (Liste von Pipeline-Objekten)
  -- Jede Pipeline enthaelt Stages in Abhaengigkeitsreihenfolge
  |
  v
ObjectStore oeffnen (objectstore.py)
  |-- Content-addressed Store unter .bkg-cave-build/store/
  |-- FsCache fuer staging/commit
  -- HostTree (Read-Only View des Host-Dateisystems)
  |
  v
Pipeline-Ausfuehrung
  |-- Abhaengigkeitsgraph topological sort
  |-- Fuer jede Stage nacheinander:
  |     |-- Stage.id berechnen (SHA-256 aus name+options+inputs+mounts)
  |     |-- Inputs aufloesen (von vorherigen Stages)
  |     |-- Devices und Mounts konfigurieren
  |     +-- stage.run():
  |           |-- BuildRoot.__enter__() (Bubblewrap-Sandbox)
  |           |     |-- Unshare: PID, IPC, Netzwerk
  |           |     |-- Read-only Mounts (/usr, /boot)
  |           |     |-- tmpfs fuer /dev, /tmp, /run, /var
  |           |     |-- Stage-Binary read-only einhaengen
  |           |     -- Capabilities setzen
  |           |-- In Sandbox: Stage-Binary ausfuehren
  |           |-- Input/Device/Mount mappen
  |           +-- Ergebnis sammeln (StageResult)
  |           |-- BuildRoot.__exit__() (Sandbox aufraeumen)
  |
  |-- Bei Fehler: Pipeline fehlschlagen
  -- Bei Erfolg: StageResult im ObjectStore committen
  |
  v
Assembler (finales Image-Format erzeugen)
  |
  v
Optional: Export (--export) oder VM-Test (--in-vm)
```

#### Plugin-Architektur (bestehend)

bkg-cave-build besitzt eine eingebaute Modul/Plugin-Struktur:

1. **Stage-Definitionen**: Jedes Stage ist ein Python-Modul im stages/ Verzeichnis mit meta.json und Implementierung
2. **Assembler**: Assembler kombinieren mehrere Stage-Ergebnisse zum finalen Image-Format
3. **Sources**: Sources laden Eingabedaten (Container-Images, Tree-Repositories, etc.)
4. **Devices**: geraetetreiber fuer Block Devices (LVM, RAID, Luks, etc.)
5. **Index-System**: meta.py liest stages/, assemblers/ und schemas/ dynamisch ein
6. **Manifest-Format**: JSON-basiert, versioniert (v1, v2)
7. **Solver-System**: DNF/DNF5-basiertes Paket-Aufloesungsmodul

#### Sandbox-Ausfuehrung

Jede Stage wird in einer Bubblewrap-Sandbox ausgefuehrt:
- Unshare: IPC, PID, Netzwerk
- Read-only Mounts fuer /usr, /boot
- tmpfs fuer /dev, /tmp, /run, /var
- Capabilities: CAP_SYS_ADMIN, CAP_CHOWN, CAP_DAC_OVERRIDE, etc.
- Proprietaere Stage.id = SHA-256(name + build + options + inputs + mounts)

---

## 3. Modulstruktur

### 3.1 bkg-cave-boot (ugrd)

```
bkg-cave-boot/
├── __init__.py                    # Paket-Initialisierung
├── main.py                        # CLI-Entrypoint (230 Zeilen)
├── exceptions.py                  # ValidationError, AutodetectError (5 Zeilen)
├── config_helpers.py              # Modul-Pfade, Typen-Aufloesung (65 Zeilen)
├── generator_helpers.py           # Mixin: Datei-Operationen, Hook-Sortierung (387 Zeilen)
├── initramfs_protocol.py          # Protocol-Definition (33 Zeilen)
├── initramfs_config.py            # InitramfsConfig (554 Zeilen)
├── initramfs_generator.py         # InitramfsGenerator (276 Zeilen)
│
├── base/                          # Basis-Module (bkg-cave-boot.base.*)
│   ├── base.py                    # Basis-Konfiguration (switch_root_target, loglevel, shebang)
│   ├── core.py                    # Kerneinstellungen (hostonly, validate, tmpdir, build_dir)
│   ├── cmdline.py                 # Kernel-Cmdline-Verarbeitung
│   ├── kmod.py                    # Kernel-Modul-Konfiguration
│   ├── mounts.py                  # Mount-Definitionen
│   ├── cpio.py                    # CPIO-Archiv-Konfiguration
│   ├── console.py                 # Konsolen-Konfiguration (agetty, etc.)
│   ├── banner.py                  # Begrueszungsbanner
│   ├── bell.py                    # System-Bell
│   ├── debug.py                   # Debug-Konfiguration
│   ├── fonts.py                   # Konsolen-Fonts
│   ├── plymouth.py                # Plymouth-Konfiguration
│   ├── udev.py                    # Udev-Regeln
│   ├── keymap.py                  # Tastatur-Layouts
│   ├── checks.py                  # Validierungs-Checks
│   └── test.py                    # Test-Modul
│   └── *.toml                     # TOML-Konfigurationsdateien je Modul
│
├── kmod/                          # Kernel-Modul-Module (bkg-cave-boot.kmod.*)
│   ├── kmod.py                    # Kernel-Modul-Hauptlogik
│   ├── input.py                   # Input-Geräte-Module
│   ├── platform.py                # Plattform-Module
│   ├── kconfig.py                 # Kernel-Config-Analyse
│   ├── standard_mask.py           # Standard-Kmod-Masken (nosound, nonetwork, novideo, etc.)
│   ├── nvme.toml                  # NVMe-Support
│   ├── usb.toml                   # USB-Support
│   ├── fat.toml                   # FAT-Support
│   ├── mmc.toml                   # MMC/SD-Support
│   ├── nonetwork.toml             # Netzwerk deaktivieren
│   ├── nosound.toml               # Sound deaktivieren
│   └── ...                        # Weitere kmod-Module
│
├── fs/                            # Dateisystem-Module (bkg-cave-boot.fs.*)
│   ├── ext4.py                    # ext4-Dateisystem
│   ├── btrfs.py                   # Btrfs-Dateisystem
│   ├── f2fs.py                    # F2FS-Dateisystem
│   ├── lvm.py                     # LVM-Unterstuetzung
│   ├── mdraid.py                  # RAID-Unterstuetzung
│   ├── overlayfs.py               # OverlayFS-Unterstuetzung
│   ├── zfs.py                     # ZFS-Unterstuetzung
│   ├── bcachefs.py                # BCacheFS-Unterstuetzung
│   ├── resume.py                  # Hibernation/Resume
│   ├── livecd.py                  # Live-CD-Unterstuetzung
│   ├── fakeudev.py                # Fake-Udev fuer Tests
│   ├── cpio.py                    # CPIO-Archiv-Logik
│   ├── mounts.py                  # Mount-Konfiguration
│   ├── test_image.py              # Test-Image-Generierung
│   └── *.toml                     # TOML-Konfigurationen pro Dateisystem
│
├── net/                           # Netzwerk-Module (bkg-cave-boot.net.*)
│   ├── net.py                     # Netzwerk-Hauptlogik
│   ├── dhcpcd.py                  # DHCP-Client-Konfiguration
│   ├── static.py                  # Static-Netzwerk-Konfiguration
│   └── *.toml                     # Netzwerk-Konfigurationen (VLAN, etc.)
│
├── crypto/                        # Krypto-Module (bkg-cave-boot.crypto.*)
│   ├── cryptsetup.py              # LUKS-Verschluesselung
│   ├── gpg.py                     # GPG-Integration
│   ├── smartcard.py               # Smartcard (YubiKey)
│   └── *.toml                     # Krypto-Konfigurationen
│
└── *.toml                         # Top-Level-Konfigurationen
```

### 3.2 bkg-cave-build (osbuild)

```
bkg-cave-build/
├── __init__.py                    # Paket-Initialisierung
├── __main__.py                    # python -m bkg-cave-build Entrypoint
├── main_cli.py                    # CLI-Entrypoint (argparse)
├── api.py                         # JSON-IPC-API fuer Sandbox-Execution
│
├── pipeline.py                    # Manifest, Pipeline, Stage, Executor, Runner (732 Zeilen)
├── buildroot.py                   # Bubblewrap-Sandbox (393 Zeilen)
├── objectstore.py                 # Content-Store (635 Zeilen)
├── meta.py                        # Index, Schema, Validierung (815 Zeilen)
│
├── sources.py                     # Source-Handler
├── devices.py                     # geraetetreiber
├── inputs.py                      # Input-Manager
├── mounts.py                      # Mount-Manager
├── mixins.py                      # Mixin-Klassen fuer Stages
├── host.py                        # Host-System-Detektion
├── loop.py                        # Loop-Device-Handling
├── qemu.py                        # QEMU-Integration
├── vm.py                          # VM-Ausfuehrung
├── monitor.py                     # Build-Monitoring
│
├── util/                          # Hilfsmodule
│   ├── fscache.py                 # Dateisystem-Cache (1278 Zeilen)
│   ├── lvm2.py                    # LVM2-Hilfsroutinen (625 Zeilen)
│   ├── linux.py                   # Linux-spezifische Hilfsmittel (437 Zeilen)
│   ├── lorax.py                   # Lorax-Integration (315 Zeilen)
│   ├── ostree.py                  # OSTree-Integration
│   ├── rhsm.py                    # Red Hat Subscription Manager
│   ├── containers.py              # Container-Unterstuetzung
│   ├── checksum.py                # Checksummen-Berechnung
│   ├── chroot.py                  # Chroot-Hilfsfunktionen
│   ├── mnt.py                     # Mount-Hilfsfunktionen
│   ├── term.py                    # Terminal-Formatierung
│   ├── makeefi.py                 # EFI-Image-Erstellung
│   ├── pe32p.py                   # PE32+ binary parsing
│   ├── toml.py                    # TOML-Parsing
│   ├── parsing.py                 # Argument-Parsing
│   ├── osrelease.py               # OS-Release-Auswertung
│   ├── host.py                    # Host-System-Detektion
│   ├── run.py                     # Runner-Hilfsfunktionen
│   ├── path.py                    # Pfad-Hilfen
│   ├── rmrf.py                    # Recursive Remove
│   ├── ctx.py                     # Context-Manager
│   ├── replace.py                 # String-Replacement
│   ├── experimentalflags.py        # Experimental Features
│   ├── types.py                   # Typ-Definitionen
│   └── sbom/                      # SBOM-Unterstuetzung
│       ├── model.py               # SBOM-Datenmodell
│       ├── dnf.py                 # DNF SBOM
│       ├── dnf5.py                # DNF5 SBOM
│       ├── spdx.py                # SPDX-Berichterstellung
│       └── spdx2/
│           ├── model.py           # SPDX-2 Datenstruktur (397 Zeilen)
│           └── __init__.py
│
├── stages/                        # 351 Stage-Definitionen
│   ├── org.bkg-cave-build.copy.meta.json
│   ├── org.bkg-cave-build.copy
│   ├── org.bkg-cave-build.chmod
│   ├── org.bkg-cave-build.chown
│   ├── org.bkg-cave-build.ln
│   ├── org.bkg-cave-build.mkdir
│   ├── org.bkg-cave-build.fstab
│   ├── org.bkg-cave-build.dracut
│   ├── org.bkg-cave-build.mkinitcpio
│   ├── org.bkg-cave-build.grub2
│   ├── org.bkg-cave-build.zipl
│   ├── org.bkg-cave-build.grub2.d
│   ├── org.bkg-cave-build.kernel-cmdline
│   └── ... (351 Dateien)
│
├── assemblers/                    # 7 Assembler (org.bkg-cave-build.error, noop, oci-archive,
│   │                             #  ostree.commit, qemu, rawfs, tar)
│   ├── org.bkg-cave-build.error
│   ├── org.bkg-cave-build.noop
│   ├── org.bkg-cave-build.oci-archive
│   ├── org.bkg-cave-build.ostree.commit
│   ├── org.bkg-cave-build.qemu
│   ├── org.bkg-cave-build.rawfs
│   └── org.bkg-cave-build.tar
│
├── devices/                       # geraetetreiber (loopback, luks2, lvm2.lv)
│   ├── org.bkg-cave-build.loopback
│   ├── org.bkg-cave-build.loopback.meta.json
│   ├── org.bkg-cave-build.luks2
│   ├── org.bkg-cave-build.luks2.meta.json
│   ├── org.bkg-cave-build.lvm2.lv
│   └── org.bkg-cave-build.lvm2.lv.meta.json
│
├── inputs/                        # Input-Handler (containers, containers-storage, files, noop, ostree, tree)
│   ├── org.bkg-cave-build.containers
│   ├── org.bkg-cave-build.containers-storage
│   ├── org.bkg-cave-build.files
│   ├── org.bkg-cave-build.noop
│   ├── org.bkg-cave-build.ostree
│   ├── org.bkg-cave-build.ostree.checkout
│   └── org.bkg-cave-build.tree
│
├── formats/                       # Manifest-Formate (v1, v2)
│   ├── __init__.py
│   ├── v1.py
│   └── v2.py
│
├── solver/                        # Abhaengigkeitsauflösung
│   ├── model.py                   # Abhaengigkeitsmodell (620 Zeilen)
│   ├── dnf.py                     # DNF4-Solver (522 Zeilen)
│   ├── dnf5.py                    # DNF5-Solver (560 Zeilen)
│   ├── api/
│   │   ├── v1.py                  # REST-API v1 (303 Zeilen)
│   │   ├── v2.py                  # REST-API v2 (476 Zeilen)
│   │   └── __init__.py
│   ├── exceptions.py
│   └── __init__.py
│
├── runners/                       # Laeufer-Definitionen (os-release-basiert)
│   ├── org.bkg-cave-build.centos9
│   ├── org.bkg-cave-build.fedora30
│   ├── org.bkg-cave-build.fedora38
│   ├── org.bkg-cave-build.linux
│   ├── org.bkg-cave-build.rhel7
│   ├── org.bkg-cave-build.rhel81
│   ├── org.bkg-cave-build.rhel82
│   ├── org.bkg-cave-build.ubuntu1804
│   └── ... (symlinks zu Distributionen)
│
├── data/                          # Statische Daten
├── schemas/                       # JSON-Schema-Definitionen (bkg-cave-build1.json, bkg-cave-build2.json)
└── selinux/                       # SELinux-Policy-Dateien
```


---

## 4. Datenfluss

### 4.1 bkg-cave-boot (ugrd)

```
CLI-Args (sys.argv)
  |
  v
get_args_n_logger() --> args (Namespace), logger
  |
  v
get_kwargs_from_args(args) --> kwargs dict
  |
  v
InitramfsGenerator(**kwargs)
  |
  +-- __init__()
  |     +-- init_logger()
  |     +-- InitramfsConfig(...)
  |           +-- builtin_parameters definieren (modules, imports, import_order, etc.)
  |           +-- NO_BASE Flag pruefen --> bkg-cave-boot.base.base oder bkg-cave-boot.base.core laden
  |           +-- User-Config TOML parsen --> _load_module()
  |           |     +-- imports --> hook registration
  |           |     +-- needs --> dependency check
  |           |     +-- provides --> tag registration
  |           |     +-- custom_parameters --> deferred processing
  |           |     +-- _enqueue() fuer unbekannte Parameter
  |           +-- CLI-Arguments importieren (import_args)
  |           +-- Deferred Values in _processing Queue behalten
  |
  +-- build()
        +-- Stage = "late" --> _process_late_values()
        +-- run_build()
        |     +-- run_hook("build_enum", force_exclude)
        |     +-- run_hook("build_pre", force_exclude)
        |     +-- run_hook("build_tasks", force_exclude)
        |     +-- run_hook("build_late", force_exclude)
        |     +-- run_hook("build_deploy", force_exclude)
        |     +-- run_hook("build_final", force_exclude)
        +-- Stage = "final" --> _validate()
        +-- generate_init()
        |     +-- run_hook("functions", force_include=True)
        |     +-- run_hook("init_pre")
        |     +-- custom_init oder generate_init_main()
        |     |     +-- run_init_hook("init_debug")
        |     |     +-- run_init_hook("init_main")
        |     |     +-- run_init_hook("init_mount")
        |     +-- run_hook("init_final")
        |     +-- generate_profile() --> /etc/profile schreiben
        |     +-- _write("init", init_script, 0o755)
        +-- pack_build()
        |     +-- run_hook("pack", force_exclude) oder CPIO via PyCPIO
        +-- run_checks() --> hook "checks"
        +-- run_tests() --> hook "tests"
  |
  v
Ausgabe: initramfs.cpio (ggf. xz-komprimiert)
```

### 4.2 bkg-cave-build (osbuild)

```
CLI-Args (Manifest-Pfad)
  |
  v
main_cli.py bkg-cave-build_cli()
  +-- parse_manifest(manifest_path) --> dict (JSON)
  +-- argparse-Konfiguration (cache, libdir, checkpoint, export, etc.)
  |
  v
Meta.Index (meta.py)
  +-- Index: stages/, assemblers/, schemas/ dynamisch laden
  +-- Format-Detektion (Manifest JSON pruefen)
  +-- Runner-Identifikation (os-release des Hosts)
  |
  v
Manifest laden (pipeline.py)
  +-- sources definieren
  +-- pipelines definieren (Liste von Pipeline-Objekten)
  +-- Jede Pipeline enthaelt Stages in Abhaengigkeitsreihenfolge
  |
  v
ObjectStore oeffnen (objectstore.py)
  +-- Content-addressed Store unter .bkg-cave-build/store/
  +-- FsCache fuer staging/commit
  +-- HostTree (Read-Only View des Host-Dateisystems)
  |
  v
Pipeline-Ausfuehrung
  +-- Abhaengigkeitsgraph topological sort
  +-- Fuer jede Stage nacheinander:
  |     +-- Stage.id berechnen (SHA-256 aus name+options+inputs+mounts)
  |     +-- Inputs aufloesen (von vorherigen Stages)
  |     +-- Devices und Mounts konfigurieren
  |     +-- stage.run():
  |           +-- BuildRoot.__enter__() (Bubblewrap-Sandbox)
  |           |     +-- Unshare: PID, IPC, Netzwerk
  |           |     +-- Read-only Mounts (/usr, /boot)
  |           |     +-- tmpfs fuer /dev, /tmp, /run, /var
  |           |     +-- Stage-Binary read-only einhaengen
  |           |     +-- Capabilities setzen
  |           +-- In Sandbox: Stage-Binary ausfuehren
  |           +-- Input/Device/Mount mappen
  |           +-- Ergebnis sammeln (StageResult)
  |           +-- BuildRoot.__exit__() (Sandbox aufraeumen)
  |
  +-- Bei Fehler: Pipeline fehlschlagen
  +-- Bei Erfolg: StageResult im ObjectStore committen
  |
  v
Assembler (finales Image-Format erzeugen)
  |
  v
Optional: Export (--export) oder VM-Test (--in-vm)
```

---

## 5. Initialisierung und Startsequenzen

### 5.1 bkg-cave-boot (ugrd) — Build-Initialisierung

```
1. Argument-Parsing (bkg-cave-boot.main:main -> argparse)
2. Logger initialisieren (zenlib.util.get_args_n_logger)
3. Config-Datei laden (/etc/ugrd/config.toml oder -c Argument)
4. InitramfsConfig wird erstellt:
   a. builtin_parameters definieren (modules, imports, import_order, etc.)
   b. NO_BASE Flag pruefen --> bkg-cave-boot.base.base oder bkg-cave-boot.base.core laden
   c. User-Config TOML parsen --> _load_module()
   d. CLI-Arguments importieren --> import_args()
   e. Deferred Values in _processing Queue behalten
5. Stage auf "early" setzen
6. Alle Module laden, import_order verarbeiten
7. Bei build():
   a. Stage --> "late" --> _process_late_values()
   b. Run Build Tasks (hooks: build_enum, build_pre, build_tasks, build_late, build_deploy, build_final)
   c. Stage --> "final" --> _validate() --> Alle Queues leer? --> validated = True
   d. generate_init():
      - run_hook("functions", force_include=True)
      - run_hook("init_pre")
      - custom_init oder generate_init_main()
      - run_hook("init_final")
      - generate_profile()
      - _write("init", ...)
   e. pack_build()
   f. run_checks()
   g. run_tests()
```

### 5.2 bkg-cave-build (osbuild) — Build-Initialisierung

```
1. CLI-Parsing (argparse, main_cli.py:bkg-cave-build_cli)
2. Manifest laden (parse_manifest --> JSON aus Datei oder stdin)
3. Meta.Index erstellen (stages, assemblers, schemas laden)
4. Manifest validieren (JSON-Schema via jsonschema)
5. ObjectStore oeffnen oder erstellen (.bkg-cave-build/store/)
6. Runner identifizieren (os-release-basiert)
7. Pipeline ausfuehren:
   a. Fuer jede Pipeline im Manifest:
   b. Pipeline-Abhaengigkeitsgraph aufloesen
   c. Topological Sort der Stages
   d. Jede Stage nacheinander ausfuehren:
      - BuildRoot (bubblewrap) aufsetzen
      - Stage in Sandbox ausfuehren
      - Ergebnis in ObjectStore committen
   e. Assembler ausfuehren (finales Image)
8. Optional: Export oder VM-Test
```

---

## 6. Build-Prozesse

### 6.1 bkg-cave-boot (ugrd)

Der Build-Prozess ist eine sequenzielle Hook-Ausfuehrung:

| Hook | Phase | force_exclude | Beschreibung |
|---|---|---|---|
| build_enum | build | yes | Module aufzaehlen |
| build_pre | build | yes | Vorbereitung |
| build_tasks | build | yes | Hauptaufgaben (Dateien kopieren) |
| build_late | build | yes | Spaete Anpassungen |
| build_deploy | build | yes | Deployment-Aufgaben |
| build_final | build | yes | Abschlussarbeiten |
| pack | pack | yes | Archivierung (oder custom) |
| checks | check | yes | Validierungspruefungen |
| tests | tests | yes | Testausfuehrung |
| functions | init | yes | Shell-Funktionen in Profile aufnehmen |
| init_pre | init | no | Initialisierungs-Schritte vor main |
| init_main | init | no | Hauptinitialisierung |
| init_mount | init | no | Mount-Punkte einrichten |
| init_final | init | no | Abschliessende Init-Schritte |

### 6.2 bkg-cave-build (osbuild)

Der Build-Prozess ist eine Pipeline von Stages:

| Phase | Beschreibung |
|---|---|
| Source Fetch | Quellen laden (Container-Images, Trees, etc.) |
| Stage Execution | Jede Stage in Sandbox ausfuehren |
| Input Resolution | Outputs vorheriger Stages als Inputs |
| Device Mount | Block-Devices konfigurieren |
| Assembly | Assembler erzeugen finales Image-Format |
| Export | Optionaler Export des Ergebnisses |

Jede Stage hat einen deterministischen ID-Algorithmus:
```
Stage.id = SHA-256(name + sort_keys(options) + sort_keys(inputs) + sort_keys(mounts))
```

---

## 7. Dateistruktur

### 7.1 bkg-cave-boot (ugrd) — Quellstruktur

```
bkg-cave-boot/
├── src/ugrd/
│   ├── __init__.py
│   ├── main.py
│   ├── exceptions.py
│   ├── config_helpers.py
│   ├── generator_helpers.py
│   ├── initramfs_protocol.py
│   ├── initramfs_config.py
│   ├── initramfs_generator.py
│   │
│   ├── base/
│   │   ├── __init__.py
│   │   ├── base.py, core.py, cmdline.py, kmod.py, mounts.py
│   │   ├── cpio.py, console.py, banner.py, bell.py, debug.py
│   │   ├── fonts.py, plymouth.py, udev.py, keymap.py, checks.py, test.py
│   │   └── *.toml
│   │
│   ├── kmod/
│   │   ├── __init__.py
│   │   ├── kmod.py, input.py, platform.py, kconfig.py, standard_mask.py
│   │   └── *.toml (nvme.toml, usb.toml, fat.toml, mmc.toml, etc.)
│   │
│   ├── fs/
│   │   ├── ext4.py, btrfs.py, f2fs.py, lvm.py, mdraid.py
│   │   ├── overlayfs.py, zfs.py, bcachefs.py, resume.py, livecd.py
│   │   ├── fakeudev.py, cpio.py, mounts.py, test_image.py
│   │   └── *.toml
│   │
│   ├── net/
│   │   ├── net.py, dhcpcd.py, static.py
│   │   └── *.toml (net.toml, dhcpcd.toml, static.toml, vlan.toml)
│   │
│   └── crypto/
│       ├── cryptsetup.py, gpg.py, smartcard.py
│       └── *.toml (cryptsetup.toml, gpg.toml, smartcard.toml)
│
├── tests/
│   ├── test_config.py
│   ├── test_filesystems.py
│   ├── test_kmod.py
│   ├── test_compression.py
│   ├── test_hibernation.py
│   ├── test_mount_options.py
│   ├── test_output_paths.py
│   ├── fullauto.toml
│   ├── bad_config.toml
│   ├── no_kmods.toml
│   ├── cryptsetup_included_key.toml
│   └── tests/
│
├── docs/
│   ├── configuration.md
│   ├── runtime_usage.md
│   ├── dev_manual.md
│   ├── usage.md
│   ├── benchmarks.md
│   ├── bootloader_config.md
│   ├── readme.md
│   └── installation.md
│
├── examples/
│   ├── example.toml, kmod.toml, subvol.toml, subvol_selector.toml
│   ├── livegui.toml, gpg_keyfile.toml, detached_headers.toml
│   ├── vm-opensc.toml, keymap.toml, luks.toml, yubikey.toml
│   └── raid_crypt_serial.toml
│
├── hooks/
│   ├── installkernel/52-ugrd.install
│   ├── kernel-install/52-ugrd.install
│   └── alpm/ (91-ugrd-install.hook, 60-ugrd-remove.hook, etc.)
│
├── completion/
│   ├── ugrd (bash completion)
│   └── _ugrd (zsh completion)
│
├── pyproject.toml
├── setup.py
├── MANIFEST.in
├── LICENSE (GPLv2)
├── readme.md
└── pylintrc
```

### 7.2 bkg-cave-build (osbuild) — Quellstruktur

```
bkg-cave-build/
├── bkg-cave-build/                     # Hauptpaket
│   ├── __init__.py
│   ├── __main__.py
│   ├── main_cli.py              # CLI-Entrypoint (232 Zeilen)
│   ├── api.py                   # JSON-IPC-API
│   ├── pipeline.py              # Manifest, Pipeline, Stage, Runner (732 Zeilen)
│   ├── buildroot.py             # Bubblewrap-Sandbox (393 Zeilen)
│   ├── objectstore.py           # Content-Store (635 Zeilen)
│   ├── meta.py                  # Index, Schema, Validierung (815 Zeilen)
│   ├── sources.py               # Source-Handler
│   ├── devices.py               # geraetetreiber
│   ├── inputs.py                # Input-Manager
│   ├── mounts.py                # Mount-Manager
│   ├── mixins.py                # Mixin-Klassen
│   ├── host.py                  # Host-System-Detektion
│   ├── loop.py                  # Loop-Device-Handling
│   ├── qemu.py                  # QEMU-Integration
│   ├── vm.py                    # VM-Ausfuehrung
│   ├── monitor.py               # Build-Monitoring
│   ├── remoteloop.py            # Remote Loop-Device
│   └── util/
│       ├── __init__.py
│       ├── fscache.py           # Dateisystem-Cache (1278 Zeilen)
│       ├── lvm2.py              # LVM2-Hilfsroutinen (625 Zeilen)
│       ├── linux.py             # Linux-Hilfsmittel (437 Zeilen)
│       ├── lorax.py             # Lorax-Integration (315 Zeilen)
│       ├── ostree.py            # OSTree-Integration
│       ├── rhsm.py              # RHSM-Integration
│       ├── containers.py        # Container-Unterstuetzung
│       ├── checksum.py          # Checksummen
│       ├── chroot.py            # Chroot-Hilfen
│       ├── mnt.py               # Mount-Hilfen
│       ├── term.py              # Terminal-Formatierung
│       ├── makeefi.py           # EFI-Erstellung
│       ├── pe32p.py             # PE32+ Parsing
│       ├── toml.py              # TOML-Parsing
│       ├── parsing.py           # Argument-Parsing
│       ├── osrelease.py         # OS-Release-Auswertung
│       ├── host.py              # Host-Detektion
│       ├── run.py               # Runner-Hilfen
│       ├── path.py              # Pfad-Hilfen
│       ├── rmrf.py              # Recursive Remove
│       ├── ctx.py               # Context-Manager
│       ├── replace.py           # String-Replacement
│       ├── experimentalflags.py # Experimental Features
│       ├── types.py             # Typ-Definitionen
│       └── sbom/
│           ├── __init__.py
│           ├── model.py         # SBOM-Datenmodell
│           ├── dnf.py           # DNF SBOM
│           ├── dnf5.py          # DNF5 SBOM
│           ├── spdx.py          # SPDX-Berichterstellung
│           └── spdx2/
│               ├── __init__.py
│               └── model.py     # SPDX-2 Datenstruktur (397 Zeilen)
│
├── stages/                      # 351 Stage-Definitionen
│   ├── org.bkg-cave-build.copy.meta.json
│   ├── org.bkg-cave-build.copy
│   ├── org.bkg-cave-build.chmod
│   ├── org.bkg-cave-build.chown
│   ├── org.bkg-cave-build.ln
│   ├── org.bkg-cave-build.mkdir
│   ├── org.bkg-cave-build.fstab
│   ├── org.bkg-cave-build.dracut
│   ├── org.bkg-cave-build.mkinitcpio
│   ├── org.bkg-cave-build.grub2
│   ├── org.bkg-cave-build.zipl
│   ├── org.bkg-cave-build.grub2.d
│   ├── org.bkg-cave-build.kernel-cmdline
│   └── ... (351 Dateien)
│
├── assemblers/                  # 7 Assembler
│   ├── org.bkg-cave-build.error
│   ├── org.bkg-cave-build.noop
│   ├── org.bkg-cave-build.oci-archive
│   ├── org.bkg-cave-build.ostree.commit
│   ├── org.bkg-cave-build.qemu
│   ├── org.bkg-cave-build.rawfs
│   └── org.bkg-cave-build.tar
│
├── devices/                     # geraetetreiber
│   ├── org.bkg-cave-build.loopback
│   ├── org.bkg-cave-build.loopback.meta.json
│   ├── org.bkg-cave-build.luks2
│   ├── org.bkg-cave-build.luks2.meta.json
│   ├── org.bkg-cave-build.lvm2.lv
│   └── org.bkg-cave-build.lvm2.lv.meta.json
│
├── inputs/                      # Input-Handler
│   ├── org.bkg-cave-build.containers
│   ├── org.bkg-cave-build.containers-storage
│   ├── org.bkg-cave-build.files
│   ├── org.bkg-cave-build.noop
│   ├── org.bkg-cave-build.ostree
│   ├── org.bkg-cave-build.ostree.checkout
│   └── org.bkg-cave-build.tree
│
├── formats/                     # Manifest-Formate
│   ├── __init__.py
│   ├── v1.py
│   └── v2.py
│
├── solver/                      # Abhaengigkeitsauflösung
│   ├── model.py                 # Abhaengigkeitsmodell (620 Zeilen)
│   ├── dnf.py                   # DNF4-Solver (522 Zeilen)
│   ├── dnf5.py                  # DNF5-Solver (560 Zeilen)
│   ├── api/
│   │   ├── __init__.py
│   │   ├── v1.py                # REST-API v1 (303 Zeilen)
│   │   └── v2.py                # REST-API v2 (476 Zeilen)
│   └── exceptions.py
│
├── runners/                     # Laeufer-Definitionen
│   ├── org.bkg-cave-build.centos9
│   ├── org.bkg-cave-build.fedora30
│   ├── org.bkg-cave-build.fedora38
│   ├── org.bkg-cave-build.linux
│   ├── org.bkg-cave-build.rhel7
│   ├── org.bkg-cave-build.rhel81
│   ├── org.bkg-cave-build.rhel82
│   ├── org.bkg-cave-build.ubuntu1804
│   └── ... (symlinks)
│
├── data/                        # Statische Daten
├── schemas/                     # JSON-Schema (bkg-cave-build1.json, bkg-cave-build2.json)
├── selinux/                     # SELinux-Policy
├── test/                        # Testdaten und Testmanifests
├── conftest.py
├── setup.py
├── setup.cfg
├── Makefile
├── README.md
├── LICENSE (Apache-2.0)
├── requirements.txt
└── tox.ini
```


---

## 8. Externe Bibliotheken

### 8.1 bkg-cave-boot (ugrd)

| Bibliothek | Verwendung | Lizenz | Migrationshinweis |
|---|---|---|---|
| `pycpio >= 1.7.0` | CPIO-Archiv lesen/schreiben, Komprimierung | GPLv2 | CPIO-Writer in Rust neu implementieren; `bytes` + `xz2` oder `zstd` nutzen |
| `zenlib >= 3.5.0` | Logging-Mixin, Hilfsfunktionen (parse_toml, pretty_print, colorize) | GPLv2 | Funktionalität in `bkg-cave-core` übernehmen; Logging via `tracing`, TOML via `toml`-Crate |
| `zstandard` (optional) | Zstd-Komprimierung | MIT | Optional: `zstd`-Feature in Rust implementieren |
| Python-Stdlib | `pathlib`, `subprocess`, `importlib`, `queue`, `shutil`, `uuid`, `collections` | PSF | Durch Rust-Std ersetzen; `subprocess` → `std::process`; `importlib` → dynamisches Plugin-System |

### 8.2 bkg-cave-build (osbuild)

| Bibliothek | Verwendung | Lizenz | Migrationshinweis |
|---|---|---|---|
| `jsonschema` | JSON-Schema-Validierung | MIT | `jsonschema`-Validierung in Rust; `schemars` + `jsonschema`-Rust-Crate prüfen |
| `bubblewrap` | Sandbox-Ausführung | LGPL-2.1-or-later | `bubblewrap`-syscall-Portierung oder `capsule`/`landlock`-Alternative; Capabilities via `capabilities`-Crate |
| `dnf` / `dnf5` | Solver-Backend | GPL | Solver-Logik dokumentieren; Abhängigkeitsauflösung in Rust portieren |
| `requests`/`urllib3` | HTTP-Quellen laden | Apache-2.0/PSF | `reqwest`-Crate verwenden |
| `selinux` | SELinux-Kontext-Handling | GPLv2 | SELinux-Funktionen via `selinux`-Crate oder syscall |
| Python-Stdlib | `json`, `pathlib`, `subprocess`, `hashlib`, `importlib`, `os`, `sys` | PSF | Durch Rust-Std + Crates ersetzen |

---

## 9. Bekannte Einschränkungen

### 9.1 bkg-cave-boot (ugrd)

- **keine externen Python-Pakete** zur Laufzeit außer `pycpio` und `zenlib`
- **Python >= 3.11** erforderlich; `match`-Statements und moderne Typ-Annotationen werden verwendet
- **Host-only-Modus** als Standard: viele Auto-Detections funktionieren nur gegen den Build-Host
- **LUKS-Szenarien** sind eingeschränkt auf getestete Kombinationen; detached headers, YubiKey und LUKS-under-LVM/MDRAID sind validiert, aber nicht alle Variationen
- **Xen-Support** experimentell; wird in der Portierung nicht vorausgesetzt
- **Bridge-Netzwerk** experimentell und ohne WDS-Support eingeschränkt nutzbar
- **Kein DTB-Support** für `--kimg`; nur für `--kdir` verfügbar
- **PS/2-Tastatur** kann auf einigen Systemen langsam erkannt werden
- **Architektur-unbekannt-Fallback** ohne spezifische Optimierungen
- **Busybox** nur bei Initramfs-Bedarf gefunden; kein Fallback-Busybox-Bau integriert

### 9.2 bkg-cave-build (osbuild)

- **Python 3.6+** erforderlich; nutzt `match`-Statements und moderne Typ-Annotationen
- **Bubblewrap-Abhängigkeit** zwingend; ohne `bwrap` ist keine Sandbox-Ausführung möglich
- **Root-Rechte** erforderlich für viele Operationen (Mounts, Devices, Loop-Devices)
- **Linux-only**: nutzt Linux-spezifische Capabilities, Mount-Namespaces und SELinux
- **351+ Stages** sind extern definiert; in Rust müssen alle Stages als Module oder generierte Dateien abgebildet werden
- **DNF/DNF5-Abhängigkeit** für Solver; RPM-basierte Distributionen vorausgesetzt
- **Manifest-Format** versioniert (v1/v2); Abwärtskompatibilität muss erhalten bleiben
- **JSON-Schema-Validierung** zentral; die Schemas (`bkg-cave-build1.json`, `bkg-cave-build2.json`) sind exakt abzubilden
- **Runner-System** ist os-release-basiert; Distributionen müssen explizit konfiguriert werden
- **ObjectStore** ist content-addressed und persistent; Locking und Cache-Invalidierung müssen korrekt portiert werden

---

## 10. Plattform- und Betriebssystem-Informationen

### 10.1 bkg-cave-boot (ugrd)

- **Betriebssysteme**: Gentoo (primär entwickelt), Garuda Linux, CachyOS, Debian 12, Ubuntu 22.04
- **Shells**: bash (primär), POSIX-kompatibel (dash, ksh)
- **Dateisysteme**: BTRFS, EXT4, XFS, F2FS, NILFS2 (voll); BCACHEFS, ZFS (eingeschränkt)
- **Architekturen**: amd64 (primär), arm64 (Raspberry Pi 4/5, Quartz64, Radxa Zero3E), riscv64 (StarFive VisionFive 2)

### 10.2 bkg-cave-build (osbuild)

- **Betriebssysteme**: Red Hat Enterprise Linux, Fedora, Ubuntu, Debian, Arch Linux
- **Laufzeit**: Linux 4.15+ (Namespace- und Capability-Anforderungen)
- **Paketmanager**: DNF4/DNF5 (RPM), optional YUM (legacy)
- **Architekturen**: x86_64, aarch64, s390x, ppc64le (je nach Distribution)

---

## 11. Migration-Ready-Bewertung

### bkg-cave-boot (ugrd)

| Kriterium | Bewertung |
|---|---|
| Verhaltensgleichheit erreichbar | Hoch — Shell-Ausgabe und CPIO sind exakt nachvollziehbar |
| Plugin-Architektur portierbar | Hoch — TOML + Hook-System ist bereits modular |
| Hot-Reloading-Basis erhalten | Mittel — Deferred-Processing und Late-Args sind übertragbar |
| Testabdeckung möglich | Mittel — Pytests sind vorhanden, aber nicht vollständig |
| Risiko bei Portierung | Mittel — Python-dynamische Konstrukte und Metaklassen erfordern explizite Rust-Typen |

### bkg-cave-build (osbuild)

| Kriterium | Bewertung |
|---|---|
| Verhaltensgleichheit erreichbar | Mittel — Sandbox, Solver und JSON-Manifest sind komplex |
| Plugin-Architektur portierbar | Hoch — Stage/Assembler/Input/Device sind bereits modular |
| Hot-Reloading-Basis erhalten | Niedrig — Kein eingebauter Mechanismus; extern über Manifest-Versionierung |
| Testabdeckung möglich | Mittel — Umfangreiche Pytests vorhanden, aber nicht vollständig |
| Risiko bei Portierung | Hoch — Bubblewrap, Solver und SELinux sind systemnah und komplex |

---

## 12. Nächste Schritte

1. **Review freigeben**: `docs/analysis.md` mit Stakeholdern abstimmen.
2. **Mapping erstellen**: `docs/migration.md` mit Datei-zu-Rust-Modul-Zuordnung befüllen.
3. **Herkunft dokumentieren**: `docs/sources.md` für jede Funktion mit Commit, Repository und Zeilenbereich füllen.
4. **Phase 1 starten**: Workspace + `bkg-cave-core` + Toolchain.
5. **Masterplan aktualisieren**: `MASTERPLAN.md` und `TASKS.md` prüfen und mit Review-Ergebnissen abgleichen.
