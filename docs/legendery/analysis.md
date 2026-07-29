# bkg-cave — Architekturanalyse (Full Rebrand)

> **Datum**: 2026-07-28
> **Status**: Verbindlich — ersetzt alle vorherigen Analyse-Dokumente
> **Prinzip**: Ein Rust-Projekt, Plugin-Architektur, Cave-Theme-Naming

---

## 1. Projektziel

bkg-cave ist eine **modulare Rust-Applikation** für Linux-Systementwicklung:

| Subcommand    | Funktion                          | Ehemals |
| ------------- | --------------------------------- | ------- |
| `cave surge`  | VM starten mit Host-RootFS        | virtme  |
| `cave kindle` | Initramfs-Generator               | ugrd    |
| `cave forge`  | Pipeline-basiertes OS-Image-Build | osbuild |

**Drei Interfaces**:

1. **CLI** (`cave surge/kindle/forge`) — primär
2. **Web** (Dioxus/WASM, später) — gleiche Engine via Browser
3. **Plugin-API** — alle Domänen als `CavePlugin`-Trait

---

## 2. Systemarchitektur

### 2.1 Schichten

```
┌──────────────────────────────────────────────────────────────┐
│                      UI-Schicht                               │
│  CLI (clap)  │  Web (Dioxus/WASM)  │  TUI (Ratatui, später)  │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                   Applikations-Schicht                        │
│                                                               │
│  PluginManager                                                │
│  ├── load_static / load_dynamic / unload / reload             │
│  ├── find_command / execute                                   │
│  └── Hot-Reload-Watcher (inotify + libloading)               │
│                                                               │
│  EventBus  (Plugin ↔ Plugin, async-frei)                      │
│  ConfigRegistry  (zentral, versioniert, watchable)            │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                   Domänen-Schicht (Plugins)                    │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │  cave-surge  │  │ cave-kindle  │  │   cave-forge     │    │
│  │  (VM-Plugin) │  │ (Boot-Plug.) │  │  (Build-Plugin)  │    │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘    │
└─────────┼──────────────────┼───────────────────┼──────────────┘
          │                  │                   │
┌─────────▼──────────────────▼───────────────────▼──────────────┐
│                   Infrastruktur-Schicht                        │
│                                                               │
│  cave-core                                                     │
│  ├── cpio::CpioWriter (newc-Format, binary-identisch)         │
│  ├── util::find_binary, quote_karg, sanitize_disk_args        │
│  ├── shell::ShellCodeGenerator (für Init-Skripte)             │
│  ├── config::ConfigRegistry (TOML/JSON/Env)                   │
│  ├── event::EventBus (synchron, in-proc)                      │
│  └── error::BkgCaveError (thiserror, 9+ Varianten)            │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Plugin-Architektur

```rust
// Jedes Plugin implementiert dieses Trait:
pub trait CavePlugin: Send + Sync {
    fn id(&self) -> &'static str;          // "cave-surge"
    fn name(&self) -> &str;                // "Surge VM Runner"
    fn version(&self) -> &str;             // "0.1.0"
    fn kind(&self) -> PluginKind;          // Surge | Kindle | Forge | Utility
    fn init(&mut self, ctx: &PluginContext) -> Result<()>;
    fn commands(&self) -> Vec<RegisteredCommand>;
    fn hot_reloadable(&self) -> bool { true }
}

// PluginManager orchestriert alle Plugins:
pub struct PluginManager {
    plugins: HashMap<&'static str, PluginState>,
    event_bus: EventBus,
    config: ConfigRegistry,
    watcher: Option<FileWatcher>,
}

enum PluginState {
    Static(Box<dyn CavePlugin>),
    Dynamic { plugin: Box<dyn CavePlugin>, library: libloading::Library, path: PathBuf },
}
```

### 2.3 Hot-Reload (von Anfang an)

| Mechanismus        | Beschreibung                                                                                             |
| ------------------ | -------------------------------------------------------------------------------------------------------- |
| **Statisch**       | Default: 3 Plugins einkompiliert. Kein Overhead, keine Unsicherheit.                                     |
| **Dynamisch**      | Plugin als `cdylib` mit `#[no_mangle] pub extern "C" fn cave_plugin_create()`. Via `libloading` geladen. |
| **File-Watcher**   | `notify`-Crate überwacht `{plugin_dir}/`. Bei `.so`-Änderung → `unload()` + `load_dynamic()`.            |
| **State-Preserve** | ConfigRegistry bleibt erhalten. Plugin kann bei `init()` alten State übernehmen.                         |

```rust
// Plugin als dynamische Bibliothek:
#[no_mangle]
pub extern "C" fn cave_plugin_create() -> *mut dyn CavePlugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}

// Laden:
let lib = unsafe { libloading::Library::new(path) }?;
let create: libloading::Symbol<extern fn() -> *mut dyn CavePlugin> =
    unsafe { lib.get(b"cave_plugin_create") }?;
let plugin = unsafe { Box::from_raw(create()) };
```

### 2.4 WASM/Dioxus-Vorbereitung

Architektur ist von Anfang an WASM-kompatibel:

- **`cave-core`** hat keine CLI-Abhängigkeit — keine `clap`-Dependency in core
- **Plugins** sind reine Trait-Implementierungen — kein `std::process` in der Schnittstelle
- **EventBus** arbeitet synchron — kein `async` nötig
- **`cave-web`** (später): Dioxus-App die PluginEngine aufruft:

```rust
// Spätere Dioxus-App (skizziert):
#[derive(Props, PartialEq)]
struct SurgeForm { kernel: String, args: Vec<String> }

fn App(cx: Scope) -> Element {
    let engine = use_plugin_engine(cx);
    render! {
        div { class: "cave-dashboard",
            Tab { label: "Surge", form: SurgeForm {...} }
            Tab { label: "Kindle", form: KindleForm {...} }
            Tab { label: "Forge", form: ForgeForm {...} }
            pre { class: "output" }
        }
    }
}
```

---

## 3. Cave-Theme-Naming

| Konzept    | Name            | Begründung                                                      |
| ---------- | --------------- | --------------------------------------------------------------- |
| Foundation | **cave-core**   | Zentrum der Höhle — alles baut darauf auf                       |
| VM starten | **cave-surge**  | "Surge" = Stoss, Welle, plötzliche Kraft — VM startet mit Wucht |
| Boot       | **cave-kindle** | "Kindle" = entzünden, anzünden — Boot-Prozess zünden            |
| Build      | **cave-forge**  | "Forge" = Schmiede — Images werden geschmiedet                  |
| App        | **cave**        | Die Höhle selbst — alle Werkzeuge vereint                       |

Submodule innerhalb von Crates folgen dem Theme:

- `cave-kindle/src/base/` — Basis-Module (am Höhlenboden)
- `cave-kindle/src/fs/` — Dateisystem-Module (Höhlenwände)
- `cave-forge/src/stages/` — Stage-Definitionen (Schmiedeschritte)

---

## 4. Domänen im Detail

### 4.1 cave-surge (ehemals virtme)

**Quelle**: https://github.com/amluto/virtme, Commit 3dfc002, GPLv2

**Kern**: VM starten der Host-Distribution als RootFS via QEMU/KVM.

```
cave surge --kdir /path/to/kernel --dry-run --show-command

1. Architektur bestimmen (Arch-Trait, 8 Impls)
2. QEMU finden + proben (Version, Capabilities)
3. Kernel finden (installiert / kdir / kimg)
4. Module auflösen (modprobe + MODALIASES)
5. QEMU-Argumente bauen:
   ├── VirtFS: /dev/root (9p) + guesttools
   ├── KVM: accel=kvm:tcg
   ├── Arch-spezifisch: CPU, Watchdog, Console
   ├── Networking: user/bridge
   ├── Disks: virtio-scsi / virtio-blk
   ├── Scripting: virtserialport
   └── Kernel-Args: init=/bin/sh -- -c "..."
6. Initramfs bauen (CPIO mit busybox + modules + /init)
7. os.execv(qemu, args)
```

**Plugin-Umfang**:

| Rust-Modul          | Python-Original              | SLOC (Orig) |
| ------------------- | ---------------------------- | ----------- |
| `arch.rs`           | `architectures.py`           | 307         |
| `qemu.rs`           | `qemu_helpers.py`            | 44          |
| `modules.rs`        | `modfinder.py`               | 65          |
| `modaliases.rs`     | `virtmods.py`                | 27          |
| `initramfs.rs`      | `mkinitramfs.py`             | 172         |
| `cpio.rs` (in core) | `cpiowriter.py`              | 86          |
| `cli.rs`            | `run.py` (argparse)          | ~160        |
| `lib.rs` (Orch.)    | `run.py` (do_it)             | ~485        |
| `guest/init.sh`     | `guest/virtme-init`          | 251         |
| `guest/udhcpc.sh`   | `guest/virtme-udhcpc-script` | 24          |

### 4.2 cave-kindle (ehemals ugrd)

**Quelle**: https://github.com/desultory/ugrd, Version 2.2.0, GPLv2

**Kern**: TOML-getriebener Initramfs-Generator mit Hook-System.

```
cave kindle --config my-system.toml

1. TOML-Config parsen
2. InitramfsConfig erstellen (Queue, Stage, Typ-System)
3. Module laden (imports, hooks, provides, needs)
4. Build-Hooks ausführen:
   ├── build_enum: Host-Erkennung
   ├── build_pre: Vorbereitung
   ├── build_tasks: Hauptlogik (Binaries, Dependencies, Symlinks)
   ├── build_late: Module auflösen
   ├── build_deploy: Struktur generieren
   └── build_final: fstab, ld.so.cache
5. Init-Skript generieren:
   ├── functions (Shell-Funktionen für Profile)
   ├── init_pre (Basis-Setup)
   ├── init_main (Crypt, Mounts)
   ├── init_mount (Root-Mount)
   └── init_final (switch_root)
6. CPIO packen (xz-komprimiert)
7. Checks + Tests
```

**Schlüsselkonzepte**:

- **Config-Queue**: Unbekannte Parameter werden in `_processing[key]` zwischengespeichert bis der Typ registriert ist
- **Phase-System**: `early → late → final`, `_late_args` werden erst in late verarbeitet
- **Hook-Sortierung**: `before/after`-Konvergenz (max 100 Iterationen)
- **Shell-Code**: Wird als Python-Strings generiert, zur Build-Zeit parametrisiert

**Plugin-Umfang**:

| Rust-Modul             | Python-Original                | SLOC (Orig) |
| ---------------------- | ------------------------------ | ----------- |
| `config.rs`            | `initramfs_config.py`          | 554         |
| `hooks.rs`             | `generator_helpers.py` (Hooks) | ~200        |
| `helpers.rs`           | `generator_helpers.py` (Files) | ~187        |
| `protocol.rs`          | `initramfs_protocol.py`        | 33          |
| `modules.rs`           | `config_helpers.py`            | 65          |
| `error.rs`             | `exceptions.py`                | 5           |
| `base/base.rs`         | `base/base.py`                 | 427         |
| `base/core.rs`         | `base/core.py`                 | 740         |
| `base/cmdline.rs`      | `base/cmdline.py`              | 183         |
| `base/debug.rs`        | `base/debug.py`                | 75          |
| `base/kmod.rs`         | `base/kmod.py`                 | ~100        |
| `base/mounts.rs`       | `base/mounts.py`               | ~200        |
| `base/cpio.rs`         | `base/cpio.py`                 | ~80         |
| `fs/mounts.rs`         | `fs/mounts.py`                 | 1164        |
| `kmod/kmod.rs`         | `kmod/kmod.py`                 | 693         |
| `net/net.rs`           | `net/net.py`                   | 80          |
| `crypto/cryptsetup.rs` | `crypto/cryptsetup.py`         | 642         |
| `cli.rs`               | `main.py`                      | 230         |

### 4.3 cave-forge (ehemals osbuild)

**Quelle**: https://github.com/osbuild/osbuild, Apache-2.0

**Kern**: Pipeline-basiertes Build-System für OS-Images, isoliert in Bubblewrap-Sandbox.

```
cave forge --manifest os-image.json

1. JSON-Manifest parsen
2. Meta.Index erstellen (stages, assemblers, schemas laden)
3. Schema validieren
4. ObjectStore öffnen (content-addressed, FsCache)
5. Depsolve (topologische Sortierung der Pipelines)
6. Sources downloaden
7. Für jede Pipeline:
   a. Für jede Stage:
      ├── BuildRoot: bwrap-Sandbox aufsetzen (unshare, tmpfs, binds)
      ├── Inputs/Devices/Mounts über Host-Service
      ├── Stage-Binary in Sandbox ausführen
      ├── Ergebnis in ObjectStore committen
   b. Assembler: finales Image-Format erzeugen
8. Export
```

**Schlüsselkonzepte**:

- **Content-Addressed IDs**: `Stage.id = SHA256(name + build + options + inputs + mounts)` → deterministic caching
- **Copy-on-Write**: `cp --reflink=auto -a` für instantane Tree-Kopien
- **Host-Services**: Subprozesse via Unix-Socket-IPC für Sources/Devices/Mounts
- **Source-Date-Epoch**: `clamp_mtime()` für reproduzierbare Builds

**Plugin-Umfang**:

| Rust-Modul       | Python-Original  | SLOC (Orig) |
| ---------------- | ---------------- | ----------- |
| `pipeline.rs`    | `pipeline.py`    | 732         |
| `buildroot.rs`   | `buildroot.py`   | 393         |
| `objectstore.rs` | `objectstore.py` | 635         |
| `meta.rs`        | `meta.py`        | 815         |
| `sources.rs`     | `sources.py`     | 220         |
| `devices.rs`     | `devices.py`     | 164         |
| `inputs.rs`      | `inputs.py`      | 157         |
| `mounts.rs`      | `mounts.py`      | 255         |
| `host.rs`        | `host.py`        | 552         |
| `api.rs`         | `api.py`         | 211         |
| `monitor.rs`     | `monitor.py`     | 433         |

---

## 5. Externe Abhängigkeiten

### 5.1 Laufzeit-Binaries (nicht Rust)

| Binary          | Wird benötigt von       | Optional       |
| --------------- | ----------------------- | -------------- |
| `qemu-system-*` | cave-surge              | Nein           |
| `modprobe`      | cave-surge, cave-kindle | Nein           |
| `depmod`        | cave-surge              | Ja             |
| `busybox`       | cave-surge              | Ja (Initramfs) |
| `bwrap`         | cave-forge              | Nein           |
| `udevadm`       | cave-kindle             | Ja             |
| `blkid`         | cave-kindle             | Ja             |
| `lddtree`       | cave-kindle             | Ja             |

### 5.2 Rust-Crates (Whitelist)

| Crate                | Version | Verwendung                       | MS  |
| -------------------- | ------- | -------------------------------- | --- |
| `clap`               | 4       | CLI-Argument-Parsing             | M2+ |
| `tracing`            | 0.1     | Strukturiertes Logging           | M1  |
| `tracing-subscriber` | 0.3     | Log-Output-Formatierung          | M1  |
| `thiserror`          | 1       | Fehler-Derive                    | M1  |
| `anyhow`             | 1       | Fehler-Propagation               | M1  |
| `serde`              | 1       | Serialisierung (Config/Manifest) | M5+ |
| `serde_json`         | 1       | JSON-Manifest                    | M4+ |
| `serde_derive`       | 1       | Derive-Makros                    | M5+ |
| `toml`               | 0.8     | TOML-Parsing (Config)            | M3+ |
| `sha2`               | 0.10    | SHA256-Hashing (Stage-IDs)       | M4+ |
| `libloading`         | 0.8     | Hot-Reload (dlopen)              | M8  |
| `notify`             | 6       | File-Watcher (Hot-Reload)        | M8  |
| `tempfile`           | 3       | Temporäre Dateien (Tests)        | M1  |
| `hex`                | 0.4     | Hex-Formatierung (CPIO)          | M1  |

### 5.3 WASM/Dioxus (später, M10)

| Crate    | Version | Verwendung   |
| -------- | ------- | ------------ |
| `dioxus` | 0.5     | Web-Frontend |

---

## 6. Datenflüsse

### 6.1 Plugin-Manager

```
CLI-Start (main.rs)
  │
  ├── PluginManager::new()
  │     ├── EventBus::new()
  │     └── ConfigRegistry::new()
  │
  ├── load_static(cave-surge::plugin())
  │     └── plugin.init(ctx)
  │           └── commands: [RegisteredCommand { name: "surge", ... }]
  │
  ├── load_static(cave-kindle::plugin())
  │     └── plugin.init(ctx)
  │           └── commands: [RegisteredCommand { name: "kindle", ... }]
  │
  ├── load_static(cave-forge::plugin())
  │     └── plugin.init(ctx)
  │           └── commands: [RegisteredCommand { name: "forge", ... }]
  │
  ├── [Optional] load_dynamic("/usr/lib/cave/plugins/*.so")
  │     └── libloading::Library::new()
  │     └── cave_plugin_create() → Box<dyn CavePlugin>
  │     └── plugin.init(ctx)
  │
  ├── start_watcher("/usr/lib/cave/plugins/")?
  │     └── inotify → on_modify → reload()
  │
  └── plugin_manager.execute("surge", args)
        └── find_command("surge") → RegisteredCommand
        └── command.run(args)
              └── cave-surge::execute()
```

### 6.2 cave-surge

```
CLI: cave surge --kdir /linux --dry-run --show-command

SurgePlugin::execute(args)
  │
  ├── 1. arch = arch_from_name(args.arch)
  │
  ├── 2. qemu = Qemu::new(arch.qemuname)
  │     └── qemu.probe() → version, cannot_overmount_virtfs, has_multidevs
  │
  ├── 3. kernel = find_kernel(args)
  │     ├── --installed-kernel: /lib/modules/{ver}/vmlinuz + modfinder
  │     ├── --kdir: .config + .virtme_mods + modmode
  │     └── --kimg: Direktpfad, keine Module
  │
  ├── 4. need_initramfs = force_initramfs || cannot_overmount_virtfs || kernel.modfiles
  │
  ├── 5. qemuargs bauen
  │     ├── -name
  │     ├── -fsdev: /dev/root (9p)
  │     ├── -fsdev: guesttools (9p)
  │     ├── -machine: accel=kvm:tcg
  │     ├── Arch-Specific: CPU, Watchdog, Console
  │     ├── -net: user/bridge
  │     ├── -drive: Disks
  │     ├── -chardev: Scripting
  │     └── -append: Kernel-Args
  │
  ├── 6. [need_initramfs]:
  │     ├── busybox finden
  │     ├── CpioWriter → CPIO-Archiv
  │     ├── make_base_layout + dev_nodes
  │     ├── busybox installieren + Symlinks
  │     ├── Module kopieren + load_all.sh
  │     └── /init generieren
  │
  ├── 7. [--show-command]: println(qemuargs)
  │
  ├── 8. [--dry-run]: return 0
  │
  └── 9. os.execv(qemu.qemubin, qemuargs)
```

### 6.3 cave-kindle

```
CLI: cave kindle --config my-system.toml

KindlePlugin::build(config_path)
  │
  ├── 1. TOML laden → Value
  │
  ├── 2. InitramfsConfig::new()
  │     ├── builtin_parameters setzen
  │     ├── _load_module(config_toml)
  │     │     ├── imports → Hook-Registrierung
  │     │     ├── custom_parameters → Typ-Registrierung
  │     │     ├── provides/needs → Dependencies prüfen
  │     │     └── unbekannte → _processing Queue
  │     └── CLI-Args importieren (überschreiben)
  │
  ├── 3. Generator::new(config)
  │
  ├── 4. generator.build()
  │     ├── stage = "late" → deferred args verarbeiten
  │     │
  │     ├── run_build()
  │     │     ├── run_hook("build_enum")   → Autodetection
  │     │     ├── run_hook("build_pre")    → Cleanup, Setup
  │     │     ├── run_hook("build_tasks")  → Dateien kopieren
  │     │     ├── run_hook("build_late")   → Module auflösen
  │     │     ├── run_hook("build_deploy") → Symlinks, Nodes
  │     │     └── run_hook("build_final")  → fstab, ld.so.cache
  │     │
  │     ├── stage = "final" → _validate()
  │     │
  │     ├── generate_init()
  │     │     ├── run_hook("functions", force_include=true)
  │     │     ├── run_hook("init_pre")
  │     │     ├── generate_init_main()
  │     │     │     ├── run_init_hook("init_debug")
  │     │     │     ├── run_init_hook("init_main")
  │     │     │     └── run_init_hook("init_mount")
  │     │     ├── run_hook("init_final")
  │     │     ├── generate_profile() → /etc/profile
  │     │     └── _write("init", init_script, 0o755)
  │     │
  │     ├── pack_build()
  │     │     └── run_hook("pack") → CpioWriter (xz)
  │     │
  │     ├── run_checks()
  │     │     └── run_hook("checks")
  │     │
  │     └── run_tests()
  │           └── run_hook("tests")
  │
  └── 5. Output: {build_dir}/initramfs.cpio.xz
```

### 6.4 cave-forge

```
CLI: cave forge --manifest os-image.json

ForgePlugin::build(manifest_path)
  │
  ├── 1. JSON parsen → Manifest (serde_json)
  │
  ├── 2. Index::new(libdir)
  │     ├── stages/, assemblers/, schemas/ durchsuchen
  │     └── ModuleInfo + Schema + RunnerInfo aufbauen
  │
  ├── 3. Schema validieren
  │
  ├── 4. ObjectStore::new(store_dir)
  │
  ├── 5. Manifest::depsolve(store, exports)
  │     ├── Pipeline-Reihenfolge: Abhängigkeiten vor Nutzern
  │     └── Gecachte Pipelines überspringen (by SHA256)
  │
  ├── 6. Manifest::download(store, monitor)
  │     └── Sources via ServiceManager laden
  │
  ├── 7. Für jede Pipeline:
  │     ├── Cache-Hit? → überspringen
  │     ├── Letzten Checkpoint finden (rückwärts)
  │     ├── Object::init(checkpoint) → CoW-Tree
  │     ├── Für jede Stage:
  │     │     ├── BuildRoot::new() → bwrap
  │     │     ├── InputManager::map() → Inputs auflösen
  │     │     ├── DeviceManager::open() → Devices öffnen
  │     │     ├── MountManager::mount() → Mounts einrichten
  │     │     ├── API + LoopServer registrieren
  │     │     ├── stage.run() → bwrap container
  │     │     └── Object::finalize() → clamp_mtime
  │     ├── ObjectStore::commit(object)
  │     └── Assembler ausführen
  │
  └── 8. Export (cp --reflink=auto -a)
```

---

## 7. Qualitätssicherung

### 7.1 Test-Strategie

| Test-Typ         | Beschreibung                                    | Werkzeug                        |
| ---------------- | ----------------------------------------------- | ------------------------------- |
| **Golden Tests** | Rust-Ausgabe == Python-Ausgabe binary-identisch | `include_bytes!` + `assert_eq!` |
| **Unit Tests**   | Einzelfunktionen isoliert                       | `#[test]`                       |
| **Integration**  | Subcommands vollständig                         | `std::process::Command`         |
| **Behavioural**  | Edge Cases + Fehlerpfade                        | Parametrisierte Tests           |
| **Shell-Code**   | Generierte Shell-Skripte validieren             | `sh -n` Syntax-Check            |

### 7.2 QA-Gates (jeder Milestone)

```
cargo fmt --check           # Keine Änderungen
cargo clippy --all -D warnings  # Null Warnings
cargo test --all            # Alle Tests grün
cargo doc --workspace --no-deps  # Null broken links
```

### 7.3 Cross-Referenzen

Jede Rust-Funktion muss auf ihre Python-Originalquelle verweisen:

```rust
/// Surge: Kernel-Module aus installiertem Kernel auflösen
///
/// ## Source
/// - Repository: https://github.com/amluto/virtme
/// - Commit: 3dfc002
/// - Datei: virtme/modfinder.py
/// - Funktion: find_modules_from_install
/// - Zeilen: 56-65
pub fn find_modules_from_install(...) -> Result<Vec<String>> { ... }
```

---

## 8. Vergleich: Alt → Neu

| Aspekt         | Alt (virtme/ugrd/osbuild) | Neu (bkg-cave)                  |
| -------------- | ------------------------- | ------------------------------- |
| Sprache        | Python 3                  | Rust (Edition 2021)             |
| Build          | setup.py                  | Cargo Workspace                 |
| Isolation      | Keine / bwrap             | Plugin-System + Hot-Reload      |
| CLI            | 3 separate Binaries       | 1 Binary, 3 Subcommands         |
| UI             | Nur CLI                   | CLI + WASM/Dioxus (später)      |
| Naming         | Projekt-spezifisch        | Cave-Theme (surge/kindle/forge) |
| Architektur    | Monolithisch pro Projekt  | Modular, Plugin-basiert         |
| Abhängigkeiten | Python-Stdlib + crates.io | Rust-Stdlib + Whitelist         |
| Lizenz         | GPLv2 / Apache-2.0        | Apache-2.0                      |

---

## 9. Implementierungsreihenfolge (Begründung)

1. **cave-core (M1)**: Fundament. CPIO wird von surge UND kindle benötigt. Plugin-System muss von Anfang an stehen.

2. **cave-surge (M2, M5)**: Geringste Komplexität (~1.500 SLOC). Keine externen Python-Deps. Dient als Proof-of-Concept für Plugin-Architektur.

3. **cave-kindle (M3, M6)**: Mittlere Komplexität (~7.000 SLOC). Hook-System ist die größte Herausforderung. Baut auf CPIO aus core auf.

4. **cave-forge (M4, M7)**: Höchste Komplexität (~4.800 SLOC). Bubblewrap, Content-Store, Host-Service-IPC. Wird erst begonnen wenn surge + kindle laufen.

5. **Integration (M8)**: Alle Plugins im `cave` Binary. Hot-Reload. File-Watcher.

6. **QA (M9)**: vollständige Qualitätssicherung.

7. **Web (M10, später)**: Dioxus/WASM-Frontend.
