# Architecture Documentation

## bkg-cave — System Architecture

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Layered Architecture](#layered-architecture)
3. [Plugin System Design](#plugin-system-design)
4. [Data Flow per Domain](#data-flow-per-domain)
5. [CPIO Format Specification](#cpio-format-specification)
6. [Event System](#event-system)
7. [Configuration System](#configuration-system)
8. [WASM / Dioxus Readiness](#wasm--dioxus-readiness)
9. [Crate Dependency Graph](#crate-dependency-graph)
10. [Error Handling Strategy](#error-handling-strategy)

---

## 1. System Overview

`bkg-cave` is a modular, plugin-driven Rust workspace for building, running, and packaging operating-system artifacts. It unifies three historically separate toolchains under a single binary:

| Domain       | Legacy Tool | Crate         | Purpose                                     |
| ------------ | ----------- | ------------- | ------------------------------------------- |
| VM Execution | `virtme`    | `cave-surge`  | Boot kernels in QEMU with dynamic initramfs |
| Initramfs    | `ugrd`      | `cave-kindle` | Generate initramfs images from TOML config  |
| OS Image     | `osbuild`   | `cave-forge`  | Build OS images via pipeline stages         |

The project is structured as a **Cargo workspace** with five crates:

```
bkg-cave/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── cave-core/          # Foundation library
│   ├── cave-surge/         # VM runner
│   ├── cave-kindle/        # Initramfs generator
│   ├── cave-forge/         # OS image builder
│   └── cave/               # Main binary
└── docs/
```

---

## 2. Layered Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI Layer (cave/)                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │  surge   │ │  kindle  │ │  forge   │ │  plugin  │              │
│  │  subcmd  │ │  subcmd  │ │  subcmd  │ │  mgmt    │              │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘              │
│       └────────────┴────────────┴────────────┘                     │
│                         │ PluginManager                             │
├─────────────────────────┼───────────────────────────────────────────┤
│                    Plugin Layer                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │  Static  │ │ Dynamic  │ │  WASM    │ │  Hot-    │              │
│  │  Plugins │ │  (.so)   │ │  Plugins │ │  Reload  │              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
├─────────────────────────┼───────────────────────────────────────────┤
│                   Domain Layer                                      │
│  ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐    │
│  │   cave-surge     │ │   cave-kindle    │ │   cave-forge     │    │
│  │   (VM Runner)    │ │ (Initramfs Gen)  │ │ (OS Image Build) │    │
│  └────────┬─────────┘ └────────┬─────────┘ └────────┬─────────┘    │
│           └────────────────────┼────────────────────┘               │
├────────────────────────────────┼───────────────────────────────────┤
│                         cave-core                                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │CpioWriter│ │PluginMgr │ │ EventBus │ │  Config  │              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                            │
│  │  Shell   │ │   Util   │ │  Error   │                            │
│  └──────────┘ └──────────┘ └──────────┘                            │
└─────────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer            | Responsibility                                         |
| ---------------- | ------------------------------------------------------ |
| **CLI Layer**    | Argument parsing, subcommand routing, user interaction |
| **Plugin Layer** | Extensible behavior via static/dynamic/WASM plugins    |
| **Domain Layer** | Business logic for surge, kindle, forge                |
| **cave-core**    | Shared primitives, no domain logic                     |

---

## 3. Plugin System Design

### 3.1 CavePlugin Trait

The central abstraction for all plugins:

```rust
pub trait CavePlugin: Send + Sync {
    /// Unique identifier for this plugin (e.g. "cave-surge-qemu")
    fn name(&self) -> &str;

    /// Semantic version of the plugin
    fn version(&self) -> &str;

    /// Which domain(s) this plugin belongs to
    fn kind(&self) -> PluginKind;

    /// Initialize the plugin with the given context
    fn init(&mut self, ctx: &PluginContext) -> Result<(), Error>;

    /// Register commands this plugin provides
    fn register_commands(&self) -> Vec<RegisteredCommand>;

    /// Shutdown hook (called on unload/reload)
    fn shutdown(&mut self) -> Result<(), Error>;
}
```

### 3.2 PluginContext

Provides plugins access to shared services:

```rust
pub struct PluginContext {
    pub event_bus: EventBus,
    pub config: Config,
    pub shell: Shell,
    pub plugin_manager: Arc<PluginManager>,
}
```

### 3.3 RegisteredCommand

Maps a CLI subcommand to plugin logic:

```rust
pub struct RegisteredCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: fn(&PluginContext, &ArgMatches) -> Result<(), Error>,
}
```

### 3.4 PluginKind Enum

```rust
pub enum PluginKind {
    Surge,   // VM execution plugins
    Kindle,  // Initramfs generation plugins
    Forge,   // OS image building plugins
    Core,    // Core infrastructure plugins
    Utility, // Cross-cutting utilities
}
```

### 3.5 PluginManager

The `PluginManager` is responsible for:

1. **Discovery** — scanning plugin directories for `.so` files
2. **Loading** — `dlopen` + symbol resolution
3. **Registration** — wiring commands into the CLI router
4. **Hot-reload** — unloading and reloading plugins at runtime
5. **Lifecycle** — calling `init()` and `shutdown()` hooks

```
PluginManager
├── plugins: HashMap<String, Box<dyn CavePlugin>>
├── plugin_dirs: Vec<PathBuf>
├── load_plugin(path) -> Result<()>
├── unload_plugin(name) -> Result<()>
├── reload_plugin(name) -> Result<()>
├── list_plugins() -> Vec<PluginInfo>
└── dispatch(command, args) -> Result<()>
```

### 3.6 Hot-Reload Mechanism

```
┌─────────────┐     SIGHUP      ┌──────────────┐
│   Process   │ ──────────────► │  Signal       │
│             │                 │  Handler      │
└─────────────┘                 └──────┬───────┘
                                        │
                                        ▼
                              ┌──────────────────┐
                              │ PluginManager     │
                              │ .reload_plugin()  │
                              └────┬─────────────┘
                                   │
                          ┌────────┴────────┐
                          │ 1. shutdown()    │
                          │ 2. dlclose()     │
                          │ 3. dlopen()      │
                          │ 4. resolve syms  │
                          │ 5. init()        │
                          └─────────────────┘
```

**Safety guarantees:**

- Plugins are `Send + Sync` for thread-safe access
- `shutdown()` is always called before `dlclose`
- Failed reloads leave the old plugin loaded
- EventBus is notified of all load/unload events

---

## 4. Data Flow per Domain

### 4.1 cave-surge (VM Runner)

```
User Input: cave surge run --arch x86_64 --kernel /path/to/vmlinuz
    │
    ▼
CLI Parser (clap)
    │
    ▼
PluginManager::dispatch("surge", "run", args)
    │
    ▼
SurgePlugin::run()
    │
    ├──► Config::load() ──► TOML config file
    │
    ├──► ArchPlugin::detect() ──► x86_64 / aarch64 / riscv64 / ...
    │
    ├──► QemuBuilder::new()
    │       ├── cpu_model
    │       ├── memory
    │       ├── devices
    │       └── kernel_args
    │
    ├──► InitramfsBuilder::new()
    │       ├──► cave-kindle integration
    │       └──► CpioWriter::write()
    │
    ├──► KernelModuleLoader::load()
    │       └──► /lib/modules/$(uname -r)/
    │
    ▼
QEMU Process spawned
    │
    ▼
EventBus: SurgeStarted, SurgeStopped, SurgeError
```

### 4.2 cave-kindle (Initramfs Generator)

```
User Input: cave kindle build --config init.toml --output initramfs.cpio
    │
    ▼
CLI Parser
    │
    ▼
KindlePlugin::build()
    │
    ├──► Config::load("init.toml")
    │       ├── [hooks] section
    │       ├── [files] section
    │       └── [modules] section
    │
    ├──► HookRunner::execute_prepare()
    │       └── shell scripts or WASM hooks
    │
    ├──► FileCollector::collect()
    │       ├── binaries (busybox, etc.)
    │       ├── libraries (ldd resolution)
    │       ├── config files
    │       └── kernel modules
    │
    ├──► CpioWriter::new(Format::Newc)
    │       ├── add_file("/init", init_script)
    │       ├── add_file("/bin/busybox", ...)
    │       ├── add_dir("/dev")
    │       ├── add_symlink("/init", "/bin/busybox")
    │       └── write_trailer()
    │
    ▼
Output: initramfs.cpio.gz
```

### 4.3 cave-forge (OS Image Builder)

```
User Input: cave forge build --pipeline pipeline.toml --output disk.img
    │
    ▼
CLI Parser
    │
    ▼
ForgePlugin::build()
    │
    ├──► Pipeline::load("pipeline.toml")
    │       └── Vec<Stage> (ordered)
    │
    ├──► ObjectStore::new("/var/cache/cave-forge")
    │       └── content-addressed storage
    │
    ├──► StageExecutor::run(pipeline)
    │       │
    │       ├── Stage 1: org.osbuild.mount
    │       │   └── Input: base tree
    │       │   └── Output: mounted dir
    │       │
    │       ├── Stage 2: org.osbuild.packages
    │       │   └── Input: mount point
    │       │   └── Output: mounted dir + packages
    │       │
    │       ├── Stage 3: org.osbuild.grub2
    │       │   └── Input: mounted dir
    │       │   └── Output: disk.img
    │       │
    │       └── ...
    │
    ▼
Output: disk.img (qcow2 / raw / iso)
```

---

## 5. CPIO Format Specification

`cave-core` implements the **newc (SVR4)** CPIO format, the most widely used variant for initramfs.

### 5.1 Header Structure (110 bytes)

```
Offset  Size  Field         Description
------  ----  ------------  -------------------------------------------
0       6     magic         "070701" (ASCII)
6       8     ino           inode number (hex)
14      8     mode          file mode + type (hex)
22      8     uid           user ID (hex)
30      8     gid           group ID (hex)
38      8     nlink         link count (hex)
46      8     mtime         modification time (hex)
54      8     filesize      file size in bytes (hex, 0 for dirs)
62      8     dev_major     major device number (hex)
70      8     dev_minor     minor device number (hex)
78      8     rdev_major    major device for special files (hex)
86      8     rdev_minor    minor device for special files (hex)
94      8     namesize      filename length + null terminator (hex)
102     8     check         checksum / always 0 (hex)
```

### 5.2 File Type Bits

```
S_IFMT   0o170000  mask for file type
S_IFREG  0o100000  regular file
S_IFDIR  0o040000  directory
S_IFLNK  0o120000  symbolic link
S_IFCHR  0o020000  character device
S_IFBLK  0o060000  block device
S_IFIFO  0o010000  FIFO / named pipe
S_IFSOCK 0o140000  socket
```

### 5.3 Trailer

The archive ends with the special entry `"TRAILER!!!"` (namesize = 12) which signals end-of-archive.

### 5.4 CpioWriter API

```rust
pub struct CpioWriter {
    format: CpioFormat,
    entries: Vec<CpioEntry>,
}

impl CpioWriter {
    pub fn new(format: CpioFormat) -> Self;
    pub fn add_file(&mut self, path: &str, data: &[u8]) -> Result<()>;
    pub fn add_dir(&mut self, path: &str, mode: u32) -> Result<()>;
    pub fn add_symlink(&mut self, path: &str, target: &str) -> Result<()>;
    pub fn add_device(&mut self, path: &str, major: u32, minor: u32, mode: u32) -> Result<()>();
    pub fn write(&self, writer: &mut dyn Write) -> Result<u64>;
    pub fn write_gzipped(&self, path: &Path) -> Result<()>;
}
```

---

## 6. Event System

The `EventBus` provides decoupled, typed communication between components.

### 6.1 Event Types

```rust
pub enum CaveEvent {
    // Plugin lifecycle
    PluginLoaded { name: String, kind: PluginKind },
    PluginUnloaded { name: String },
    PluginError { name: String, error: String },

    // Surge events
    SurgeStarted { arch: String, kernel: PathBuf },
    SurgeStopped { exit_code: i32 },
    SurgeError { message: String },

    // Kindle events
    KindleBuildStarted { config: PathBuf },
    KindleBuildCompleted { output: PathBuf, size: u64 },
    KindleBuildFailed { error: String },

    // Forge events
    ForgeBuildStarted { pipeline: PathBuf },
    ForgeStageCompleted { stage: String },
    ForgeBuildCompleted { output: PathBuf },
    ForgeBuildFailed { error: String },
}
```

### 6.2 EventBus API

```rust
pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<EventType, Vec<BoxedCallback>>>>,
}

impl EventBus {
    pub fn new() -> Self;
    pub fn subscribe<F>(&mut self, event_type: EventType, callback: F) -> SubscriptionId
    where
        F: Fn(&CaveEvent) + Send + Sync + 'static;

    pub fn unsubscribe(&mut self, id: SubscriptionId) -> Result<()>;
    pub fn emit(&self, event: CaveEvent);
    pub fn emit_async(&self, event: CaveEvent) -> JoinHandle<()>;
}
```

### 6.3 Event Flow

```
Component A                    EventBus                   Component B
    │                              │                          │
    │  emit(SurgeStarted{...})     │                          │
    │─────────────────────────────►│                          │
    │                              │  notify(subscribers)     │
    │                              │─────────────────────────►│
    │                              │                          │  handle()
```

---

## 7. Configuration System

### 7.1 Config Structure

```rust
pub struct Config {
    pub global: GlobalConfig,
    pub surge: SurgeConfig,
    pub kindle: KindleConfig,
    pub forge: ForgeConfig,
    pub plugins: PluginConfig,
}

pub struct GlobalConfig {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_level: String,
    pub max_parallel_jobs: usize,
}

pub struct SurgeConfig {
    pub qemu_path: PathBuf,
    pub default_memory: String,
    pub default_cpu: String,
    pub kernel_dir: PathBuf,
    pub module_dir: PathBuf,
}

pub struct KindleConfig {
    pub compressor: Compressor,
    pub compression_level: u32,
    pub hook_dir: PathBuf,
    pub default_binaries: Vec<String>,
}

pub struct ForgeConfig {
    pub object_store: PathBuf,
    pub build_root: PathBuf,
    pub default_output_format: OutputFormat,
}
```

### 7.2 Configuration File Locations

| Scope       | Path                               |
| ----------- | ---------------------------------- |
| System-wide | `/etc/cave/config.toml`            |
| User        | `~/.config/cave/config.toml`       |
| Project     | `./cave.toml`                      |
| Environment | `CAVE_CONFIG=/path/to/config.toml` |

### 7.3 TOML Example

```toml
[global]
data_dir = "/var/lib/cave"
cache_dir = "/var/cache/cave"
log_level = "info"
max_parallel_jobs = 4

[surge]
qemu_path = "/usr/bin/qemu-system-x86_64"
default_memory = "2G"
default_cpu = "host"
kernel_dir = "/boot"
module_dir = "/lib/modules"

[kindle]
compressor = "gzip"
compression_level = 6
hook_dir = "/etc/cave/hooks"
default_binaries = ["busybox"]

[forge]
object_store = "/var/cache/cave-forge"
build_root = "/var/lib/cave-forge/buildroot"
default_output_format = "qcow2"

[plugins]
directories = ["/usr/lib/cave/plugins", "~/.local/lib/cave/plugins"]
auto_load = true
```

---

## 8. WASM / Dioxus Readiness

The architecture is designed to support future WASM and Dioxus-based UI extensions.

### 8.1 Current Readiness

| Feature                      | Status             | Notes                            |
| ---------------------------- | ------------------ | -------------------------------- |
| `wasmtime` / `wasmtime-wasi` | Optional dep       | For WASM plugin sandboxing       |
| `dioxus`                     | Not yet integrated | Planned for M10                  |
| `wasi-http`                  | Optional dep       | For network-isolated plugins     |
| Plugin ABI stability         | Designed           | `CavePlugin` trait is ABI-stable |

### 8.2 WASM Plugin Model (Planned)

```
┌─────────────────────────────────────────────┐
│              cave binary                     │
│  ┌───────────────────────────────────────┐  │
│  │         WasmPluginHost                │  │
│  │  ┌─────────────┐  ┌───────────────┐  │  │
│  │  │  wasmtime   │  │  WASI Store   │  │  │
│  │  │  Instance   │  │  (sandboxed)  │  │  │
│  │  └──────┬──────┘  └───────────────┘  │  │
│  │         │ import: CavePluginHost       │  │
│  │         ▼                              │  │
│  │  ┌────────────────────────────────┐   │  │
│  │  │  Plugin WASM Module            │   │  │
│  │  │  - name()                      │   │  │
│  │  │  - init()                      │   │  │
│  │  │  - register_commands()         │   │  │
│  │  │  - shutdown()                  │   │  │
│  │  └────────────────────────────────┘   │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 8.3 Dioxus UI (Planned M10)

The CLI layer will be abstracted behind a `UiBackend` trait, allowing Dioxus to provide a graphical frontend:

```rust
pub trait UiBackend {
    fn run_command(&self, command: &str, args: &[String]) -> Result<Output>;
    fn show_progress(&self, message: &str, progress: f32);
    fn prompt(&self, message: &str) -> Result<String>;
}
```

---

## 9. Crate Dependency Graph

```
cave (binary)
├── cave-core
├── cave-surge
│   └── cave-core
├── cave-kindle
│   └── cave-core
└── cave-forge
    └── cave-core

cave-core (no internal deps beyond std + libc)
```

### Feature Flags

| Crate       | Feature           | Description                       |
| ----------- | ----------------- | --------------------------------- |
| cave-core   | `dynamic-plugins` | Enable `dlopen`/`dlclose` support |
| cave-core   | `wasm-plugins`    | Enable WASM plugin sandboxing     |
| cave-core   | `gzip`            | Gzip compression support          |
| cave-core   | `xz`              | XZ compression support            |
| cave-core   | `zstd`            | Zstandard compression support     |
| cave-surge  | `qemu`            | QEMU integration (default)        |
| cave-surge  | `all-archs`       | Enable all 8 architecture plugins |
| cave-kindle | `hooks`           | Shell hook execution              |
| cave-forge  | `buildroot`       | BuildRoot stage support           |

---

## 10. Error Handling Strategy

All crates use a unified `Error` type from `cave-core`:

```rust
pub enum Error {
    Io(io::Error),
    Config(ConfigError),
    Plugin(PluginError),
    Cpio(CpioError),
    Surge(SurgeError),
    Kindle(KindleError),
    Forge(ForgeError),
    Other(anyhow::Error),
}
```

**Principles:**

- `thiserror` for typed, structured errors
- `anyhow` for propagation in application code
- All errors implement `std::error::Error`
- Plugin errors are isolated — a failing plugin cannot crash the host
- EventBus logs all errors without propagating to subscribers

---

_End of Architecture Documentation_
