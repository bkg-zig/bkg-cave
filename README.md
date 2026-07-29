# bkg-cave

> Modular Rust toolchain for Linux system development.

```text
cave surge  — Start a VM with the host rootfs (QEMU/KVM)
cave kindle — Generate initramfs from TOML config
cave forge  — Build OS images from pipeline manifests
```

## Architecture

```text
cave (main binary)
├── PluginManager ──────────────── orchestrates all plugins
│   ├── Static Plugins
│   │   ├── cave-surge ───────── VM runner (port of virtme)
│   │   ├── cave-kindle ──────── Initramfs generator (port of ugrd)
│   │   └── cave-forge ───────── OS image builder (port of osbuild)
│   └── Dynamic Plugins (.so via libloading + hot-reload)
└── cave-core
    ├── Plugin API (CavePlugin trait, PluginContext, PluginManager)
    ├── EventBus (pub/sub event routing)
    ├── ConfigRegistry (namespaced configuration)
    ├── CpioWriter (newc format, streaming)
    └── Utilities (shell code generation, helpers)
```

## Quick Start

```bash
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# List plugins
cargo run -- plugin list

# VM runner (dry-run)
cargo run -- surge --dry-run --kimg /path/to/vmlinuz

# Initramfs generator (dry-run)
cargo run -- kindle --dry-run --compress none

# OS image builder
cargo run -- forge --manifest manifest.json
```

## Plugin System

Plugins implement `CavePlugin` trait and can be:

- **Static**: Compiled into the binary (default)
- **Dynamic**: Loaded from `/usr/lib/cave/plugins/*.so` via `libloading`
- **Hot-Reload**: File watcher triggers reload on `.so` change

```rust
pub trait CavePlugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn kind(&self) -> PluginKind;
    fn init(&mut self, ctx: &PluginContext) -> Result<()>;
    fn commands(&self) -> Vec<RegisteredCommand>;
    fn execute(&self, command: &str, args: &[String]) -> Result<String>;
}
```

## Project Structure

```text
code/
├── Cargo.toml              # Workspace root (edition 2024)
├── crates/
│   ├── cave-plugin-api/    # Plugin ABI (CavePlugin, PluginContext, PluginManager)
│   ├── cave-core/          # Foundation: Plugin API, EventBus, Config, CpioWriter
│   ├── cave-surge/         # VM runner (virtme port)
│   ├── cave-kindle/        # Initramfs generator (ugrd port)
│   ├── cave-forge/         # OS image builder (osbuild port)
│   └── cave/               # Main binary
├── docs/                   # Documentation
│   └── p2p/               # cave-p2p migration plan
└── AGENTS.md               # Agent workspace rules
```

## Test Status

Current snapshot:

- 5 crates
- 348 tests
- 0 failures
- 0 clippy warnings

| Crate       | Tests | Coverage                                           |
| ----------- | ----- | -------------------------------------------------- |
| cave        | 5     | CLI routing, plugin loading                        |
| cave-core   | 57    | CPIO, PluginManager, Config, EventBus, Shell, Util |
| cave-surge  | 103   | Arch (8 impls), QEMU, Modules, Initramfs           |
| cave-kindle | 99    | TOML config, Hook system, Generator, Shell code    |
| cave-forge  | 84    | Pipeline, BuildRoot, ObjectStore, Meta, Sources    |

## Milestones

| MS  | Branch | Crate                    | Status  |
| --- | ------ | ------------------------ | ------- |
| M1  | a.0.1  | cave-core foundation     | DONE    |
| M2  | b.0.2  | cave-surge initial port  | DONE    |
| M3  | c.0.3  | cave-kindle initial port | DONE    |
| M4  | d.0.4  | cave-forge initial port  | DONE    |
| M5  | e.0.5  | cave-surge parity        | DONE    |
| M6  | f.0.6  | cave-kindle parity       | DONE    |
| M7  | g.0.7  | cave-forge parity        | DONE    |
| M8  | h.0.8  | cave runtime + plugins   | DONE    |
| M9  | i.0.9  | QA release gate          | DONE    |
| M10 | j.0.10 | Dioxus/WASM UI           | PLANNED |

## Roadmap

Current plugins:

- cave-surge — VM runner
- cave-kindle — Initramfs generator
- cave-forge — OS image builder

Planned:

- cave-p2p — decentralized agent network (P2P, GPU/CPU/RAM sharing, token economy)
- cave-beam — GPU fabric
- cave-agent — local agent runtime

## License

Apache-2.0
