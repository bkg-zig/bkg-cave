# OpenCode Agent-Plan — bkg-cave

## Grundsatzentscheidung

**Alte Namen verboten** — kein `virtme`, `ugrd`, `osbuild` in Crates, Binaries, Modulen, Code.

**Cave-Theme-Naming** — alle Crates setzen das Höhlen-Motiv fort:

| Rolle          | Crate         | Binary        | Subcommand    |
| -------------- | ------------- | ------------- | ------------- |
| Fundament      | `cave-core`   | —             | —             |
| VM starten     | `cave-surge`  | `cave-surge`  | `cave surge`  |
| Initramfs boot | `cave-kindle` | `cave-kindle` | `cave kindle` |
| Images bauen   | `cave-forge`  | `cave-forge`  | `cave forge`  |

**Eine App, drei Interfaces**:

- `cave` CLI (primary): `cave surge/kindle/forge`
- `cave` Web (via Dioxus/WASM, später): gleiche Engine übers Web bedienbar
- `cave` TUI (optional): Terminal-UI via Ratatui

**Von Anfang an modular**: Plugin-System + Hot-Reload in Rust.
Jede Domäne ist ein Plugin, das zur Laufzeit geladen/getauscht werden kann.

---

## Architektur: Plugin-System + Hot-Reload

```
┌─────────────────────────────────────────────────────────┐
│                      cave CLI/Web                        │
│  (clap oder Dioxus, ruft PluginEngine::execute auf)      │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                  cave-core (Library)                     │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ PluginManager                                     │  │
│  │  ├── load_static(Box<dyn CavePlugin>)             │  │
│  │  ├── load_dynamic(path) → dlopen + init           │  │
│  │  ├── unload(name) → Plugin entladen               │  │
│  │  ├── reload(name) → Hot-Reload (inotify watch)    │  │
│  │  └── list() → aktive Plugins                      │  │
│  │                                                   │  │
│  │  EventBus (Plugin → Plugin Kommunikation)          │  │
│  │  ConfigRegistry (zentrale, versionierte Config)    │  │
│  │  ErrorTypes (cave-core::error::* )                 │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  CPIO-Writer (cave-core::cpio::CpioWriter)              │
│  Util (cave-core::util::find_binary, ...)               │
│  Shell-Code-Generator (cave-core::shell)                │
└─────────────────────────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
┌───────────────┐ ┌───────────┐ ┌──────────────┐
│  cave-surge   │ │cave-kindle│ │ cave-forge   │
│  VM-Plugin    │ │Boot-Plugin│ │ Build-Plugin │
│               │ │           │ │              │
│ Arch-Trait    │ │ Initramfs │ │ Pipeline     │
│ QEMU-Discov.  │ │ Hook-Sys  │ │ BuildRoot    │
│ Modul-Finder  │ │ FS/KMod   │ │ ObjectStore  │
│ Initramfs     │ │ Crypto    │ │ Meta/Index   │
│ Gast-Tools    │ │ CPIO-Pack │ │ Services     │
└───────────────┘ └───────────┘ └──────────────┘
        │                │                │
        └────────────────┼────────────────┘
                         ▼
┌─────────────────────────────────────────┐
│         externals (kein Rust)           │
│  QEMU, modprobe, depmod, bwrap, busybox │
└─────────────────────────────────────────┘
```

---

## Plugin-Interface

```rust
// cave-core/src/plugin.rs
pub trait CavePlugin: Send + Sync {
    fn id(&self) -> &'static str;          // "cave-surge"
    fn name(&self) -> &str;                // "Surge VM Runner"
    fn version(&self) -> &str;             // "0.1.0"
    fn kind(&self) -> PluginKind;          // Surge | Kindle | Forge | Utility
    fn init(&mut self, ctx: &PluginContext) -> Result<()>;
    fn commands(&self) -> Vec<RegisteredCommand>;
    fn hot_reloadable(&self) -> bool { true }
}

pub struct RegisteredCommand {
    pub name: &'static str,        // "surge"
    pub about: &'static str,       // "Start a virtual machine"
    pub run: fn(Args) -> Result<i32>,
}

pub struct PluginContext {
    pub config: ConfigRegistry,
    pub events: EventBus,
    pub plugin_dir: PathBuf,
}

// Hot-Reload via shared library
// Jedes Plugin kann als .so kompiliert werden:
#[no_mangle]
pub extern "C" fn cave_plugin_create() -> *mut dyn CavePlugin { ... }
```

---

## Hot-Reload-Strategie

| Mechanismus            | Beschreibung                                                                                |
| ---------------------- | ------------------------------------------------------------------------------------------- |
| **Statische Plugins**  | Default: alle 3 Plugins sind einkompiliert. Kein Overhead.                                  |
| **Dynamische Plugins** | Optional: Plugin-Crates als `cdylib` bauen, via `libloading` laden.                         |
| **File-Watcher**       | `inotify` auf Plugin-Verzeichnis → bei Änderung `dlclose` + `dlopen`.                       |
| **State-Preserve**     | Config und Events bleiben erhalten. Plugin meldet via `init()` ob es State übernehmen kann. |

---

## Crate-Namen: Cave-Theme

| Crate         | Bedeutung                           | Binary        | Beschreibung                        |
| ------------- | ----------------------------------- | ------------- | ----------------------------------- |
| `cave-core`   | Höhlenzentrum                       | —             | Plugins, Events, CPIO, Util, Config |
| `cave-surge`  | Stoss, Welle (VM startet mit Wucht) | `cave-surge`  | VM-Orchestrierung (ehemals virtme)  |
| `cave-kindle` | Entzünden (Boot-Prozess zünden)     | `cave-kindle` | Initramfs-Generator (ehemals ugrd)  |
| `cave-forge`  | Schmiede (Images schmieden)         | `cave-forge`  | Pipeline-Build (ehemals osbuild)    |
| `cave`        | Hauptapp                            | `cave`        | CLI + Web-Bridge                    |

Unterverzeichnisse folgen dem Theme:

- `cave-kindle/src/base/` → Module
- `cave-kindle/src/fs/` → Dateisysteme
- `cave-forge/src/stages/` → Stage-Definitionen

---

## Plugin-Lebenszyklus (von Anfang an)

```
1. PluginManager::new()
2.   .load_static(cave_surge::plugin())
3.   .load_static(cave_kindle::plugin())
4.   .load_static(cave_forge::plugin())
5.   .load_dir("/usr/lib/cave/plugins/")?  // dynamische Plugins
6.
7. // Hot-Reload: File-Watcher auf /usr/lib/cave/plugins/
8. // Bei Dateiänderung: plugin.unload() → dlclose → dlopen → plugin.init()
9.
10. // CLI-Routing:
11. PluginManager::find_command("surge")?  // → RegisteredCommand
12. command.run(args)
```

---

## WASM/Dioxus (Vorbereitung, später aktiv)

Architektur von Anfang an WASM-fähig:

- `cave-core` hat **keine** CLI-Abhängigkeit (kein `clap` in core)
- Plugins sind `CavePlugin`-Trait, nicht an CLI gebunden
- `cave-core` compiliert zu WASM (`#![cfg_attr(target_arch = "wasm32", no_std)]`?)

```rust
// Später: cave-web als Dioxus-App
// Gleiche PluginEngine, andere UI-Schicht
fn App(cx: Scope) -> Element {
    let engine = use_plugin_engine(cx);
    render! {
        button { onclick: move |_| engine.execute("surge", args),
            "VM starten" }
    }
}
```

---

## Workspace-Struktur

```
bkg-cave/
├── Cargo.toml                     # [workspace] resolver = "2"
├── crates/
│   ├── cave-core/                 # Fundament + Plugin-System
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── plugin.rs           # CavePlugin-Trait + PluginManager
│   │       ├── event.rs            # EventBus
│   │       ├── config.rs           # ConfigRegistry
│   │       ├── cpio.rs             # CpioWriter
│   │       ├── shell.rs            # Shell-Code-Generator-Basis
│   │       └── util.rs             # find_binary, quote_karg, ...
│   │
│   ├── cave-surge/                # VM-Orchestrierung (Plugin)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # pub fn plugin() -> impl CavePlugin
│   │       ├── arch.rs             # Arch-Trait + 8 Impls
│   │       ├── qemu.rs             # QEMU-Discovery
│   │       ├── modules.rs          # Modul-Finder
│   │       ├── modaliases.rs       # Statische MODALIASES
│   │       ├── initramfs.rs        # Initramfs-Builder
│   │       ├── cli.rs             # clap-Argumente für Subcommand
│   │       └── guest/
│   │           ├── init.sh
│   │           └── udhcpc.sh
│   │
│   ├── cave-kindle/               # Initramfs-Generator (Plugin)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # pub fn plugin() -> impl CavePlugin
│   │       ├── cli.rs
│   │       ├── config.rs           # InitramfsConfig (TOML, Queue, Stage)
│   │       ├── hooks.rs            # Hook-Registry + Sortierung
│   │       ├── helpers.rs          # GeneratorHelpers
│   │       ├── protocol.rs         # InitramfsProtocol-Trait
│   │       ├── error.rs
│   │       ├── base/ (base, core, cmdline, debug, ...)
│   │       ├── fs/     (ext4, btrfs, mounts, ...)
│   │       ├── kmod/   (kmod, ...)
│   │       ├── net/    (net, ...)
│   │       └── crypto/ (cryptsetup, ...)
│   │
│   ├── cave-forge/                # Pipeline-Build (Plugin)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # pub fn plugin() -> impl CavePlugin
│   │       ├── cli.rs
│   │       ├── pipeline.rs         # Manifest, Pipeline, Stage, Runner
│   │       ├── buildroot.rs        # Bubblewrap-Sandbox
│   │       ├── objectstore.rs      # Content-Addressed Store
│   │       ├── meta.rs             # Index, Schema
│   │       ├── sources.rs
│   │       ├── devices.rs
│   │       ├── inputs.rs
│   │       ├── mounts.rs
│   │       ├── host.rs
│   │       ├── api.rs
│   │       └── monitor.rs
│   │
│   └── cave/                       # Hauptbinary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs             # CLI: clap → PluginEngine
│           └── bridge.rs           # (später) WASM-Bridge für Dioxus
│
├── tests/
├── docs/
│   └── agents/opencode/
│       ├── PLAN.md
│       └── TASKS.md
└── references/                     # Git-Mirrors (unverändert)
```

---

## Milestones

### Phase 1: Fundament

| MS  | Branch | Crate         | Scope                                                        | Lieferobjekt                                |
| --- | ------ | ------------- | ------------------------------------------------------------ | ------------------------------------------- |
| M1  | a.0.1  | `cave-core`   | error, util, config, cpio, shell, plugin (Trait + Manager)   | `cargo test` grün                           |
| M2  | b.0.2  | `cave-surge`  | Plugin-Integration, Arch-Trait, QEMU, Module, Initramfs      | `cave surge --dry-run`                      |
| M3  | c.0.3  | `cave-kindle` | Plugin-Integration, Config-Engine, Hook-System, Basis-Module | `cave kindle --config test.toml --dry-run`  |
| M4  | d.0.4  | `cave-forge`  | Plugin-Integration, Pipeline-Core, BuildRoot, ObjectStore    | `cave forge --manifest test.json --dry-run` |

### Phase 2: Feature-Vollständigkeit

| MS  | Branch | Crate         | Scope                                                | Lieferobjekt                    |
| --- | ------ | ------------- | ---------------------------------------------------- | ------------------------------- |
| M5  | e.0.5  | `cave-surge`  | Alle 35 CLI-Args, Gast-Tools, Integrationstests      | VM startet (QEMU exec)          |
| M6  | f.0.6  | `cave-kindle` | FS/KMod/Net/Crypto Module, init-Script, CPIO-Ausgabe | Initramfs-Erzeugung vollständig |
| M7  | g.0.7  | `cave-forge`  | Meta/Index, Services, Assembler, CLI                 | Pipeline-Build vollständig      |

### Phase 3: Integration

| MS  | Branch | Crate  | Scope                                   | Lieferobjekt                           |
| --- | ------ | ------ | --------------------------------------- | -------------------------------------- |
| M8  | h.0.8  | `cave` | Hauptbinary, Plugin-Loading, Hot-Reload | `cave surge/kindle/forge` alle nutzbar |
| M9  | i.0.9  | —      | QA: clippy, fmt, test, doc — alles grün | CI-Grün                                |

### Phase 4 (später): Web

| MS  | Branch | Scope                | Lieferobjekt                           |
| --- | ------ | -------------------- | -------------------------------------- |
| M10 | j.0.10 | Dioxus-WASM-Frontend | `cave-web` — gleiche Engine im Browser |

---

## Erlaubte Crates

| Crate                                   | Zweck                       | MS  |
| --------------------------------------- | --------------------------- | --- |
| `clap` 4                                | CLI                         | M2+ |
| `tracing` + `tracing-subscriber`        | Logging                     | M1  |
| `thiserror`                             | Error-Derive                | M1  |
| `anyhow`                                | Error-Propagation           | M1  |
| `serde` + `serde_json` + `serde_derive` | Serialisierung              | M5+ |
| `toml` 0.8                              | TOML-Parsing                | M3+ |
| `sha2` 0.10                             | SHA256                      | M4+ |
| `tempfile`                              | Tests                       | M1  |
| `hex`                                   | Hex-Formatierung            | M1  |
| `libloading` 0.8                        | Hot-Reload (dlopen)         | M8  |
| `notify` 6                              | File-Watcher für Hot-Reload | M8  |
| `dioxus` 0.5                            | WASM-UI (später)            | M10 |

## Verboten

- Alte Namen (`virtme`, `ugrd`, `osbuild`)
- `todo!()`, `unreachable!()`, `unimplemented!()`
- Mock-Objekte für behaviorale Logik
- `async` ohne Begründung
- Crates außerhalb Whitelist
- Abhängigkeiten zwischen surge/kindle/forge
