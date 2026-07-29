# Plugin API Documentation

## bkg-cave — Plugin Development Guide

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [Introduction](#introduction)
2. [CavePlugin Trait](#caveplugin-trait)
3. [PluginContext](#plugincontext)
4. [RegisteredCommand](#registeredcommand)
5. [PluginKind Enum](#pluginkind-enum)
6. [Creating a Static Plugin](#creating-a-static-plugin)
7. [Creating a Dynamic Plugin (.so)](#creating-a-dynamic-plugin-so)
8. [Hot-Reload Mechanism](#hot-reload-mechanism)
9. [Example Plugin Implementations](#example-plugin-implementations)
10. [Plugin Best Practices](#plugin-best-practices)
11. [Plugin Distribution](#plugin-distribution)

---

## 1. Introduction

The bkg-cave plugin system allows extending the core functionality without modifying the main binary. Plugins can:

- Add new subcommands to the CLI
- Implement domain-specific logic (surge, kindle, forge)
- Hook into the event system
- Access shared configuration and shell utilities

There are two plugin types:

| Type        | Extension            | Use Case                             | Hot-Reload |
| ----------- | -------------------- | ------------------------------------ | ---------- |
| **Static**  | Compiled into binary | Core functionality, default plugins  | No         |
| **Dynamic** | `.so` shared library | Third-party extensions, user plugins | Yes        |

---

## 2. CavePlugin Trait

The `CavePlugin` trait is the core interface every plugin must implement.

```rust
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::error::Error;
use cave_core::event::EventBus;

pub trait CavePlugin: Send + Sync {
    /// Unique identifier for this plugin.
    /// Must be unique across all loaded plugins.
    /// Convention: "cave-{domain}-{name}" e.g. "cave-surge-qemu"
    fn name(&self) -> &str;

    /// Semantic version following SemVer 2.0.
    /// Used for compatibility checks.
    fn version(&self) -> &str;

    /// Which domain this plugin belongs to.
    /// Determines event routing and command namespace.
    fn kind(&self) -> PluginKind;

    /// Called once when the plugin is loaded.
    /// Use this to initialize internal state, subscribe to events, etc.
    /// Must be fast — do not perform blocking I/O here.
    fn init(&mut self, ctx: &PluginContext) -> Result<(), Error>;

    /// Register CLI commands this plugin provides.
    /// Each command becomes a subcommand: `cave {kind} {name}`
    fn register_commands(&self) -> Vec<RegisteredCommand>;

    /// Called when the plugin is unloaded or reloaded.
    /// Clean up resources, unsubscribe from events, etc.
    fn shutdown(&mut self) -> Result<(), Error>;
}
```

### Trait Bounds

- `Send + Sync` — plugins may be accessed from multiple threads
- `'static` — all referenced data must be owned or `'static`

---

## 3. PluginContext

`PluginContext` is passed to `init()` and provides access to shared services.

```rust
pub struct PluginContext {
    /// Event bus for publishing and subscribing to events.
    pub event_bus: EventBus,

    /// Merged configuration (system + user + project).
    pub config: Config,

    /// Shell execution utilities.
    pub shell: Shell,

    /// Reference to the parent PluginManager.
    /// Use for loading other plugins or querying state.
    pub plugin_manager: std::sync::Arc<PluginManager>,
}
```

### Usage Example

```rust
fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
    // Subscribe to events
    ctx.event_bus.subscribe(EventType::Surge, |event| {
        log::info!("Surge event: {:?}", event);
    });

    // Read configuration
    let qemu_path = ctx.config.surge.qemu_path.clone();

    // Execute shell commands
    ctx.shell.run("modprobe", &["kvm"])?;

    Ok(())
}
```

---

## 4. RegisteredCommand

Maps a CLI subcommand to a handler function.

```rust
pub struct RegisteredCommand {
    /// Command name (used as subcommand).
    /// Example: "run" → `cave surge run`
    pub name: &'static str,

    /// Short description shown in `--help`.
    pub description: &'static str,

    /// Handler function called when the command is invoked.
    /// Receives the plugin context and clap ArgMatches.
    pub handler: fn(&PluginContext, &clap::ArgMatches) -> Result<(), Error>,
}
```

### Command Handler Signature

```rust
fn handle_run_command(ctx: &PluginContext, matches: &clap::ArgMatches) -> Result<(), Error> {
    let arch = matches.get_one::<String>("arch").unwrap();
    let kernel = matches.get_one::<PathBuf>("kernel").unwrap();

    // Implementation here
    Ok(())
}
```

---

## 5. PluginKind Enum

Determines the plugin's domain and command namespace.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginKind {
    /// VM execution plugins.
    /// Commands: `cave surge <command>`
    Surge,

    /// Initramfs generation plugins.
    /// Commands: `cave kindle <command>`
    Kindle,

    /// OS image building plugins.
    /// Commands: `cave forge <command>`
    Forge,

    /// Core infrastructure plugins.
    /// Commands: `cave <command>` (top-level)
    Core,

    /// Cross-cutting utility plugins.
    /// Commands: `cave util <command>`
    Utility,
}
```

---

## 6. Creating a Static Plugin

Static plugins are compiled directly into the `cave` binary. They are registered at compile time.

### 6.1 Define the Plugin Struct

```rust
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::error::Error;
use std::sync::Arc;

pub struct MySurgePlugin {
    initialized: bool,
    config: Arc<cave_core::config::SurgeConfig>,
}

impl MySurgePlugin {
    pub fn new() -> Self {
        Self {
            initialized: false,
            config: Arc::new(cave_core::config::SurgeConfig::default()),
        }
    }
}
```

### 6.2 Implement CavePlugin

```rust
impl CavePlugin for MySurgePlugin {
    fn name(&self) -> &str {
        "cave-surge-myplugin"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn kind(&self) -> PluginKind {
        PluginKind::Surge
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
        self.config = Arc::new(ctx.config.surge.clone());
        self.initialized = true;

        ctx.event_bus.emit(CaveEvent::PluginLoaded {
            name: self.name().to_string(),
            kind: self.kind(),
        });

        Ok(())
    }

    fn register_commands(&self) -> Vec<RegisteredCommand> {
        vec![
            RegisteredCommand {
                name: "my-action",
                description: "My custom surge action",
                handler: Self::handle_my_action,
            },
        ]
    }

    fn shutdown(&mut self) -> Result<(), Error> {
        self.initialized = false;
        Ok(())
    }
}

// Handler must be a free function (not a method)
impl MySurgePlugin {
    fn handle_my_action(
        ctx: &PluginContext,
        matches: &clap::ArgMatches,
    ) -> Result<(), Error> {
        let verbose = matches.get_flag("verbose");
        if verbose {
            log::info!("Running my-action with config: {:?}", self.config);
        }
        // Implementation
        Ok(())
    }
}
```

### 6.3 Register the Plugin

In `cave/src/main.rs` or `cave/src/plugins.rs`:

```rust
use cave_surge::plugins::MySurgePlugin;

fn build_plugin_manager() -> PluginManager {
    let mut pm = PluginManager::new();

    // Register static plugins
    pm.register_static(Box::new(MySurgePlugin::new()));

    // Register other static plugins...
    pm.register_static(Box::new(QemuPlugin::new()));
    pm.register_static(Box::new(ModuleLoaderPlugin::new()));

    pm
}
```

---

## 7. Creating a Dynamic Plugin (.so)

Dynamic plugins are compiled as separate shared libraries and loaded at runtime via `dlopen`.

### 7.1 Crate Structure

Create a new crate for your plugin:

```
my-surge-plugin/
├── Cargo.toml
└── src/
    └── lib.rs
```

### 7.2 Cargo.toml

```toml
[package]
name = "my-surge-plugin"
version = "0.1.0"
edition = "2021"
crate-type = ["cdylib"]

[dependencies]
cave-core = { path = "../../crates/cave-core" }
```

**Important:** `crate-type = ["cdylib"]` is required to produce a `.so` file.

### 7.3 Plugin Implementation

```rust
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::error::Error;

pub struct MyDynamicPlugin {
    initialized: bool,
}

impl MyDynamicPlugin {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl CavePlugin for MyDynamicPlugin {
    fn name(&self) -> &str {
        "cave-surge-my-dynamic"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn kind(&self) -> PluginKind {
        PluginKind::Surge
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
        log::info!("Dynamic plugin {} initializing", self.name());
        self.initialized = true;
        Ok(())
    }

    fn register_commands(&self) -> Vec<RegisteredCommand> {
        vec![
            RegisteredCommand {
                name: "dynamic-action",
                description: "Action from dynamic plugin",
                handler: Self::handle_dynamic_action,
            },
        ]
    }

    fn shutdown(&mut self) -> Result<(), Error> {
        self.initialized = false;
        Ok(())
    }
}

impl MyDynamicPlugin {
    fn handle_dynamic_action(
        ctx: &PluginContext,
        _matches: &clap::ArgMatches,
    ) -> Result<(), Error> {
        ctx.event_bus.emit(CaveEvent::SurgeStarted {
            arch: "dynamic".to_string(),
            kernel: std::path::PathBuf::from("/dev/null"),
        });
        Ok(())
    }
}

// Required: export a constructor symbol for dlopen
#[no_mangle]
pub extern "C" fn cave_plugin_create() -> *mut dyn CavePlugin {
    let plugin: Box<dyn CavePlugin> = Box::new(MyDynamicPlugin::new());
    Box::into_raw(plugin)
}

// Required: export a destructor symbol
#[no_mangle]
pub extern "C" fn cave_plugin_destroy(plugin: *mut dyn CavePlugin) {
    unsafe {
        if !plugin.is_null() {
            let _ = Box::from_raw(plugin);
        }
    }
}
```

### 7.4 Build the Plugin

```bash
cd my-surge-plugin
cargo build --release
# Output: target/release/libmy_surge_plugin.so
```

### 7.5 Load the Plugin

```bash
# Copy to plugin directory
cp target/release/libmy_surge_plugin.so /usr/lib/cave/plugins/

# Load via CLI
cave plugin load my-surge-plugin.so

# Or configure auto-load in config.toml
# [plugins]
# auto_load = true
# directories = ["/usr/lib/cave/plugins"]
```

---

## 8. Hot-Reload Mechanism

The hot-reload system allows reloading dynamic plugins without restarting the `cave` process.

### 8.1 Triggering a Reload

```bash
# Reload a specific plugin
cave plugin reload my-surge-plugin.so

# Reload all plugins of a kind
cave plugin reload --kind surge

# Reload all plugins
cave plugin reload --all
```

### 8.2 Signal-Based Reload

Send `SIGHUP` to the cave process:

```bash
kill -HUP $(pidof cave)
```

This triggers a full plugin reload cycle.

### 8.3 Reload Sequence

```
1. PluginManager receives reload request
2. Lock plugin registry (mutex)
3. Call plugin.shutdown() on the old instance
4. dlclose() the old .so
5. dlopen() the .so again
6. Resolve cave_plugin_create / cave_plugin_destroy symbols
7. Call cave_plugin_create() to get new instance
8. Call plugin.init(ctx) on the new instance
9. Re-register commands in CLI router
10. Unlock registry
11. Emit PluginReloaded event
```

### 8.4 Safety Guarantees

| Guarantee                 | Mechanism                                  |
| ------------------------- | ------------------------------------------ |
| No crash on failed reload | Old plugin stays loaded if new init fails  |
| No command gap            | Commands are swapped atomically under lock |
| Event consistency         | Events are queued during reload            |
| Memory safety             | Rust ownership + dlclose safety checks     |

---

## 9. Example Plugin Implementations

### 9.1 Minimal Static Plugin

```rust
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::error::Error;

pub struct HelloPlugin;

impl CavePlugin for HelloPlugin {
    fn name(&self) -> &str { "cave-hello" }
    fn version(&self) -> &str { "0.1.0" }
    fn kind(&self) -> PluginKind { PluginKind::Utility }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), Error> {
        log::info!("HelloPlugin initialized");
        Ok(())
    }

    fn register_commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "hello",
            description: "Print a greeting",
            handler: Self::handle_hello,
        }]
    }

    fn shutdown(&mut self) -> Result<(), Error> {
        log::info!("HelloPlugin shutting down");
        Ok(())
    }
}

impl HelloPlugin {
    fn handle_hello(
        _ctx: &PluginContext,
        matches: &clap::ArgMatches,
    ) -> Result<(), Error> {
        let name = matches.get_one::<String>("name")
            .map(|s| s.as_str())
            .unwrap_or("world");
        println!("Hello, {}! Welcome to bkg-cave.", name);
        Ok(())
    }
}
```

### 9.2 QEMU Surge Plugin (Simplified)

```rust
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::error::Error;
use std::process::Command;

pub struct QemuPlugin {
    qemu_path: std::path::PathBuf,
}

impl QemuPlugin {
    pub fn new() -> Self {
        Self {
            qemu_path: std::path::PathBuf::from("/usr/bin/qemu-system-x86_64"),
        }
    }
}

impl CavePlugin for QemuPlugin {
    fn name(&self) -> &str { "cave-surge-qemu" }
    fn version(&self) -> &str { "0.1.0" }
    fn kind(&self) -> PluginKind { PluginKind::Surge }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
        self.qemu_path = ctx.config.surge.qemu_path.clone();
        Ok(())
    }

    fn register_commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "run",
            description: "Run a kernel in QEMU",
            handler: Self::handle_run,
        }]
    }

    fn shutdown(&mut self) -> Result<(), Error> { Ok(()) }
}

impl QemuPlugin {
    fn handle_run(
        ctx: &PluginContext,
        matches: &clap::ArgMatches,
    ) -> Result<(), Error> {
        let kernel = matches.get_one::<std::path::PathBuf>("kernel").unwrap();
        let memory = matches.get_one::<String>("memory").unwrap();

        ctx.event_bus.emit(CaveEvent::SurgeStarted {
            arch: "x86_64".to_string(),
            kernel: kernel.clone(),
        });

        let status = Command::new(&self.qemu_path)
            .args(&[
                "-kernel", kernel.to_str().unwrap(),
                "-m", memory,
                "-nographic",
            ])
            .status()?;

        ctx.event_bus.emit(CaveEvent::SurgeStopped {
            exit_code: status.code().unwrap_or(-1),
        });

        Ok(())
    }
}
```

### 9.3 Kindle Hook Plugin

```rust
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::error::Error;

pub struct HookPlugin {
    hook_dir: std::path::PathBuf,
}

impl CavePlugin for HookPlugin {
    fn name(&self) -> &str { "cave-kindle-hooks" }
    fn version(&self) -> &str { "0.1.0" }
    fn kind(&self) -> PluginKind { PluginKind::Kindle }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
        self.hook_dir = ctx.config.kindle.hook_dir.clone();
        Ok(())
    }

    fn register_commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "run-hooks",
            description: "Execute initramfs hooks",
            handler: Self::handle_run_hooks,
        }]
    }

    fn shutdown(&mut self) -> Result<(), Error> { Ok(()) }
}

impl HookPlugin {
    fn handle_run_hooks(
        ctx: &PluginContext,
        _matches: &clap::ArgMatches,
    ) -> Result<(), Error> {
        for entry in std::fs::read_dir(&self.hook_dir)? {
            let path = entry?.path();
            if path.extension().map(|e| e == "sh").unwrap_or(false) {
                ctx.shell.run(path.to_str().unwrap(), &[])?;
            }
        }
        Ok(())
    }
}
```

---

## 10. Plugin Best Practices

### 10.1 Error Handling

```rust
// Good: Return typed errors
fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
    if !ctx.config.surge.qemu_path.exists() {
        return Err(Error::Surge(SurgeError::QemuNotFound {
            path: ctx.config.surge.qemu_path.clone(),
        }));
    }
    Ok(())
}

// Bad: Panic or unwrap
fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
    let path = ctx.config.surge.qemu_path.clone();
    // DON'T: panic!("QEMU not found at {:?}", path);
    // DON'T: let _ = path.exists().unwrap();
    Ok(())
}
```

### 10.2 Event Emission

```rust
// Always emit events for significant actions
fn handle_run(&self, ctx: &PluginContext, matches: &clap::ArgMatches) -> Result<(), Error> {
    ctx.event_bus.emit(CaveEvent::SurgeStarted { ... });
    // ... do work ...
    ctx.event_bus.emit(CaveEvent::SurgeStopped { ... });
    Ok(())
}
```

### 10.3 Resource Cleanup

```rust
fn shutdown(&mut self) -> Result<(), Error> {
    // Always clean up in shutdown
    if let Some(handle) = self.child.take() {
        let _ = handle.kill();
    }
    self.event_subscription = None;
    Ok(())
}
```

### 10.4 Configuration Defaults

```rust
// Provide sensible defaults, allow override via config
fn init(&mut self, ctx: &PluginContext) -> Result<(), Error> {
    self.memory = ctx.config.surge.default_memory.clone();
    self.cpu = ctx.config.surge.default_cpu.clone();
    Ok(())
}
```

---

## 11. Plugin Distribution

### 11.1 Packaging

Package plugins as system packages:

```
# Debian/Ubuntu
cave-plugin-myplugin_0.1.0_amd64.deb
  └── /usr/lib/cave/plugins/libmyplugin.so

# RPM (Fedora/RHEL)
cave-plugin-myplugin-0.1.0.x86_64.rpm
  └── /usr/lib64/cave/plugins/libmyplugin.so
```

### 11.2 Installation

```bash
# System-wide
sudo cp libmyplugin.so /usr/lib/cave/plugins/
sudo cave plugin reload myplugin

# User-local
mkdir -p ~/.local/lib/cave/plugins
cp libmyplugin.so ~/.local/lib/cave/plugins/
cave plugin load ~/.local/lib/cave/plugins/libmyplugin.so
```

### 11.3 Version Compatibility

Plugins should declare their compatible `cave-core` version:

```toml
# In plugin's Cargo.toml
[package.metadata.cave]
cave_core_version = "0.1"
api_version = 1
```

The `PluginManager` checks compatibility at load time and rejects incompatible plugins.

---

_End of Plugin API Documentation_
