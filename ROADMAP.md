# bkg-cave — Project Roadmap

> **Version:** 0.2.0
> **Last Updated:** 2026-07-29

---

## 1. Completed Milestones

### M1: cave-core Foundation ✅

- Cargo workspace (edition 2024)
- Plugin API (`CavePlugin` trait, `PluginContext`, `PluginManager`)
- `PluginKind` enum (Static, Dynamic, HotReload)
- `RegisteredCommand` struct
- `ConfigRegistry` (namespaced configuration)
- `EventBus` (pub/sub event routing)
- `CpioWriter` (newc format, streaming)
- Shell utilities (code generation, helpers)
- Unified error handling (`thiserror`)

### M2: cave-surge VM Runtime ✅

- virtme architecture port
- QEMU/KVM integration
- 8 architecture implementations (x86_64, i386, arm64, arm, riscv64, ppc64le, s390x, mips64)
- Kernel discovery (`/boot`, `/lib/modules`)
- Module handling (load, unload, dependencies)
- Initramfs generation (integrated with cave-kindle)

### M3: cave-kindle Initramfs Engine ✅

- TOML configuration (`InitramfsConfig`)
- Hook system (pre-init, post-init, pre-mount, post-mount)
- Dependency resolution
- Binary collection (ELF detection, library tracking)
- Compression support (none, gzip, zstd, xz)
- Shell code generation (busybox, systemd, dracut)

### M4: cave-forge Image Builder ✅

- Pipeline engine (stage execution, dependency ordering)
- BuildRoot isolation (bubblewrap)
- ObjectStore (content-addressed storage)
- Stage execution (run, script, tree, fs, archive)
- Image exporters (tar, cpio, ext4, squashfs)
- Source handling (local, URL, git)

### M5: cave Runtime Integration ✅

- Unified `cave` binary
- CLI routing (clap with subcommands)
- Plugin discovery (static + dynamic)
- Plugin lifecycle management (init, execute, shutdown)
- Static plugin registration

### M6: Dynamic Plugin System ✅

- `libloading` integration
- Dynamic `.so` plugin loading
- ABI version checks
- Hot reload (file watcher via `notify`)
- Plugin lifecycle events (loaded, unloaded, error)

### M7: QA Release Gate ✅

- `cargo fmt` — clean
- `cargo clippy -- -D warnings` — clean
- `cargo test --workspace` — 348 tests, 0 failures
- `cargo doc --workspace --no-deps` — 0 warnings
- Documentation complete (ARCHITECTURE.md, PLUGIN_API.md, BUILD.md, TESTING.md, CONTRIBUTING.md, CHANGELOG.md, MIGRATION.md, FAQ.md, ROADMAP.md, USER_GUIDE.md)

### M8: bkg-cave 0.1.0 Release ✅

- Initial platform release
- 3 domain plugins (surge, kindle, forge)
- Plugin system (static + dynamic + hot-reload)
- Complete CLI
- Full documentation

### M9: P2P Architecture Planning ✅

- bkg-p2p v0.3.0 analyzed (37 modules, ~12k LOC)
- Feature inventory complete (INVENTORY_PART1/2/3.md)
- Plugin integration strategy defined
- Runtime container architecture defined (thin container, not god object)
- Feature preservation strategy documented
- Deferred subsystem boundaries identified
- Milestone-based port plan created (PLAN.md, TASKS.md, MIGRATION.md)

---

## 2. Current Status

### Version 0.2.0 — P2P Foundation Phase

**Status: Active Development**

| Component     | Status      | Notes                                |
| ------------- | ----------- | ------------------------------------ |
| cave-core     | ✅ Stable   | Plugin API, EventBus, Config         |
| cave-surge    | ✅ Stable   | QEMU VM runner                       |
| cave-kindle   | ✅ Stable   | Initramfs generator                  |
| cave-forge    | ✅ Stable   | OS image builder                     |
| cave CLI      | ✅ Stable   | Plugin management + commands         |
| Plugin system | ✅ Stable   | Static, dynamic, hot reload          |
| cave-p2p      | 🚧 Planned  | P2P runtime plugin port              |
| Documentation | 🔄 Updating | P2P architecture documentation added |

**Test Status**:

- 5 crates
- 348 tests
- 0 failures
- 0 clippy warnings

---

## 3. Known Limitations

| Limitation                  | Severity | Planned Fix |
| --------------------------- | -------- | ----------- |
| No P2P distributed runtime  | High     | M11         |
| No resource marketplace     | High     | M12         |
| No remote execution layer   | High     | M12         |
| No provider discovery       | Medium   | M12         |
| No distributed job handling | Medium   | M13         |
| No WASM sandbox             | Medium   | M15         |
| No Web/TUI frontend         | Low      | M16         |

---

## 4. Future Plans

### M11: cave-p2p Foundation (v0.2.0)

**Target**: Q4 2026

**Goal**: Port bkg-p2p core into a first-class cave plugin.

| Feature           | Description                    |
| ----------------- | ------------------------------ |
| Identity          | Ed25519 node identity          |
| libp2p swarm      | TCP + Noise + Yamux networking |
| GossipSub         | Distributed event propagation  |
| Discovery         | mDNS + Kademlia                |
| Provider tracking | Remote resource discovery      |
| Resource monitor  | CPU/RAM/GPU advertisement      |
| Runtime container | Thin subsystem orchestration   |
| cave plugin       | Native integration             |

### M12: Distributed Execution Layer (v0.3.0)

**Target**: Q1 2027

| Feature          | Description               |
| ---------------- | ------------------------- |
| Wallet           | Token accounting          |
| Escrow           | Job payment lifecycle     |
| Job marketplace  | Request/bid/execute flow  |
| Remote executor  | P2P task offloading       |
| Task router      | Local vs remote execution |
| Resource pricing | Provider economics        |

### M13: A2A and Web Control Plane (v0.4.0)

**Target**: Q2 2027

| Feature           | Description         |
| ----------------- | ------------------- |
| Agent cards       | A2A discovery       |
| JSON-RPC          | Agent communication |
| WebSocket control | Live node events    |
| REST API          | Network management  |
| Dashboard         | Node visualization  |

### M14: Cave Ecosystem Integration (v0.5.0)

**Target**: Q3 2027

| Feature             | Description              |
| ------------------- | ------------------------ |
| Plugin marketplace  | Distributed plugins      |
| Remote builders     | Network OS builds        |
| Shared object store | Distributed artifacts    |
| Build workers       | Remote forge execution   |
| GPU workers         | Distributed acceleration |

### M15: Advanced Runtime Features (v0.6.0)

**Target**: Q4 2027

| Feature           | Description            |
| ----------------- | ---------------------- |
| WASM sandbox      | Secure plugins         |
| Inference engines | GGUF/Ollama/Candle     |
| Vector memory     | Distributed memory     |
| Tools             | Remote tool execution  |
| MCP               | Model Context Protocol |
| Skills            | Skill marketplace      |
| Agent runtime     | ReAct execution        |

### M16: Web/TUI Interface (v1.0.0)

**Target**: 2028

| Feature             | Description              |
| ------------------- | ------------------------ |
| Dioxus Web UI       | Modern reactive frontend |
| Dioxus TUI          | Terminal UI              |
| Browser management  | Remote administration    |
| Real-time dashboard | WebSocket, SSE           |

---

## 5. Release Schedule

| Version | Target     | Focus                               |
| ------- | ---------- | ----------------------------------- |
| 0.1.0   | 2026-07-29 | Core release (surge, kindle, forge) |
| 0.2.0   | Q4 2026    | cave-p2p foundation                 |
| 0.3.0   | Q1 2027    | Jobs + remote execution             |
| 0.4.0   | Q2 2027    | A2A + dashboard                     |
| 0.5.0   | Q3 2027    | Ecosystem integration               |
| 0.6.0   | Q4 2027    | Advanced runtime                    |
| 1.0.0   | TBD        | Stable distributed platform         |

---

## 6. Long-term Vision

bkg-cave evolves into a distributed Linux development platform.

The goal is combining:

- OS image generation
- Virtual machine execution
- Initramfs construction
- Distributed build workers
- P2P resource sharing
- Remote execution
- Plugin-based extensions

**Architecture**:

```text
bkg-cave
├── cave-surge    VM execution
├── cave-kindle   Boot generation
├── cave-forge    OS construction
├── cave-p2p      Distributed nodes
├── cave-beam     Compute fabric
└── cave-agent    Local AI runtime
```

**Timeline**:

- **Year 1**: Core system tooling ✅ + P2P foundation 🚧
- **Year 2**: Distributed execution, remote builders, resource marketplace, web control plane
- **Year 3**: Full ecosystem: plugin marketplace, distributed GPU workers, agent collaboration, enterprise deployment

**Ultimate Goal**: bkg-cave becomes a modular distributed operating-system engineering fabric: a platform where machines, builders, resources and plugins cooperate through a secure P2P architecture.

---

## 7. Design Principles

1. **Port before refactor**: Existing code is ported 1:1, then modernized after feature parity
2. **One task = one commit**: Small, reviewable, rollback-friendly commits
3. **QA after every task**: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
4. **Reference tracking**: Every task traces back to its bkg-p2p source
5. **Thin containers**: Runtime is a container, not a god object
6. **Protocol preservation**: Network protocols are never renamed (DNS analogy)
7. **Plugin-first architecture**: Plugin API before domain implementation
