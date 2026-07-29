# bkg-p2p → cave-p2p Migration Guide

> **Purpose**: Detailed guide for migrating from bkg-p2p monolith to cave-p2p plugin.
> **Audience**: Developers maintaining or extending the P2P functionality.

---

## 1. OVERVIEW

bkg-p2p is a monolithic Rust crate (~37 modules, ~12k LOC) that implements a decentralized P2P AI agent network. cave-p2p is the same functionality ported as a plugin to the bkg-cave plugin architecture.

**Key Differences**:

- bkg-p2p: Single binary (`bkg-peer`) with all functionality built-in
- cave-p2p: Plugin (`cave-p2p`) loaded by the `cave` binary via `PluginManager`
- bkg-p2p: Own CLI (`bkg-peer run`, `bkg-peer serve`, etc.)
- cave-p2p: CLI subcommands (`cave p2p serve`, `cave p2p peers`, etc.)

---

## 2. MIGRATION STRATEGY

### 2.1 Direct Port (Recommended)

Port each module from bkg-p2p to cave-p2p with minimal changes:

1. Copy source file from bkg-p2p
2. Update module paths (e.g., `crate::p2p::` → `cave_p2p::p2p::`)
3. Update external crate references if needed
4. Add to cave-p2p's `lib.rs`
5. Run tests

### 2.2 Dependency Mapping

All bkg-p2p dependencies map 1:1 to cave-p2p dependencies (see INVENTORY_PART2.md §28).

**Exception**: `vectx`, `wasmtime`, `llama-cpp-2`, `candle-*`, `hf-hub` are deferred to v1.0.

### 2.3 Protocol Preservation

**CRITICAL**: Do NOT change these protocol strings:

- `/bkg-p2p/a2a-rpc/1.0.0` (A2A JSON-RPC)
- `/bkg-p2p/kad/1.0.0` (Kademlia DHT)
- `/bkg-p2p/1.0.0` (Identify protocol)
- `bkg-p2p/resources/v1` (resource manifests)
- `bkg-p2p/providers/v1` (provider advertisements)
- `bkg-p2p/tools/v1` (tool manifests)
- `bkg-p2p/skills/v1` (skill announcements)
- `bkg-p2p/a2a/v1` (agent cards)
- `bkg-p2p/crew/tasks/v1` (crew tasks)
- `bkg-p2p/crew/pod/v1` (pod artifacts)
- `bkg-p2p/world/{campaign}/v1` (campaign milestones)
- `bkg-p2p/jobs/requests/v1` (job requests)
- `bkg-p2p/jobs/bids/v1` (job bids)
- `bkg-p2p/jobs/status/v1` (job status)

---

## 3. MODULE-BY-MODULE MIGRATION

### 3.1 identity/mod.rs

**Changes**: None (direct port)
**Tests**: Copy all 3 tests

### 3.2 p2p/behaviour.rs

**Changes**: None (direct port)
**Tests**: Copy `test_build_swarm`

### 3.3 p2p/events.rs

**Changes**: None (direct port)

### 3.4 p2p/resource.rs

**Changes**: None (direct port)
**Tests**: Copy all 3 tests

### 3.5 p2p/provider.rs

**Changes**: None (direct port)
**Tests**: Copy all 2 tests

### 3.6 executor/resource.rs

**Changes**: None (direct port)
**Tests**: Copy all 4 tests

### 3.7 executor/router.rs

**Changes**: None (direct port)
**Tests**: Copy all 3 tests

### 3.8 executor/task.rs

**Changes**: None (direct port)
**Tests**: Copy all 5 tests

### 3.9 executor/remote.rs

**Changes**: None (direct port)
**Tests**: Copy `test_remote_executor_config`

### 3.10 executor/mod.rs

**Changes**: None (direct port)
**Tests**: Copy all 3 tests

### 3.11 wallet/mod.rs

**Changes**: None (direct port)
**Tests**: Copy all 5 tests

### 3.12 job/mod.rs

**Changes**: None (direct port)
**Tests**: Copy all 2 tests

### 3.13 node.rs

**Changes**: Update `crate::` references to `cave_p2p::`
**Tests**: Copy `test_node_creation`

### 3.14 runtime.rs

**Changes**: Update `crate::` references to `cave_p2p::`
**Tests**: Copy all 2 tests

### 3.15 web/mod.rs

**Changes**: Update `crate::` references to `cave_p2p::`
**Tests**: None in bkg-p2p (add integration tests)

### 3.16 config.rs

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.17 bootstrap.rs

**Changes**: Update paths from `~/.bkg-peer` to `~/.cave-p2p` (or use cave-core's path system)

### 3.18 db.rs

**Changes**: Update paths from `~/.bkg-peer/data/` to cave-p2p data directory

### 3.19 observability.rs

**Changes**: None (direct port)

### 3.20 swarm/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.21 a2a/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.22 reputation/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.23 routines/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.24 run/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.25 workspace/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.26 prompts/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.27 proxy/

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.28 agent_library.rs

**Changes**: Update `crate::` references to `cave_p2p::`

### 3.29 cli.rs

**Changes**: Update command names from `bkg-peer` to `cave p2p`
**Example**: `bkg-peer serve` → `cave p2p serve`

---

## 4. PLUGIN INTEGRATION

### 4.1 CavePlugin Trait Implementation

```rust
// cave-p2p/src/plugin.rs
use cave_core::plugin::{CavePlugin, PluginContext, PluginCommand};

pub struct CaveP2PPlugin;

impl CavePlugin for CaveP2PPlugin {
    fn name(&self) -> &'static str {
        "cave-p2p"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn initialize(&mut self, ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize P2P node, network, etc.
        Ok(())
    }

    fn commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand::new("serve", "Start P2P node with web dashboard"),
            PluginCommand::new("peers", "List connected peers"),
            PluginCommand::new("wallet", "Wallet operations"),
            // ... more commands
        ]
    }

    fn execute(&self, cmd: &str, args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        // Handle CLI commands
        Ok(format!("Executed: {} {:?}", cmd, args))
    }
}
```

### 4.2 Plugin Registration

```rust
// cave/src/main.rs
use cave_p2p::CaveP2PPlugin;

fn main() {
    let mut manager = PluginManager::new();
    manager.register_static(CaveP2PPlugin);
    // ... rest of main
}
```

### 4.3 CLI Integration

```rust
// cave/src/main.rs
.clap_subcommand(
    SubCommand::with_name("p2p")
        .about("P2P networking commands")
        .subcommand(SubCommand::with_name("serve").args(&[
            Arg::with_name("web").long("web").takes_value(true),
            Arg::with_name("ollama").long("ollama"),
            // ... more args
        ]))
        .subcommand(SubCommand::with_name("peers").subcommand(SubCommand::with_name("list")))
        .subcommand(SubCommand::with_name("wallet").subcommand(SubCommand::with_name("balance")))
        // ... more subcommands
)
```

---

## 5. CONFIGURATION MIGRATION

### 5.1 bkg-p2p config.toml

```toml
[p2p]
listen_addresses = ["/ip4/0.0.0.0/tcp/0"]
bootstrap_peers = []
mdns_enabled = true
kademlia_enabled = true
advertise_interval_secs = 300

[web]
enabled = false
listen_addr = "127.0.0.1:8080"

# ... rest of config
```

### 5.2 cave-p2p config (via cave's ConfigRegistry)

```toml
[p2p]
listen_addresses = ["/ip4/0.0.0.0/tcp/0"]
bootstrap_peers = []
mdns_enabled = true
kademlia_enabled = true
advertise_interval_secs = 300

[p2p.web]
enabled = false
listen_addr = "127.0.0.1:8080"

# ... rest of config
```

**Note**: cave-p2p config is namespaced under `[p2p]` to avoid conflicts with other plugins.

---

## 6. DATA MIGRATION

### 6.1 Identity

- bkg-p2p: `~/.bkg-peer/identity.key`
- cave-p2p: `~/.cave-p2p/identity.key` (or use cave-core's identity system)

**Migration**: Copy `identity.key` to new location.

### 6.2 Database

- bkg-p2p: `~/.bkg-peer/data/bkg-peer.redb`
- cave-p2p: `~/.cave-p2p/data/cave-p2p.redb`

**Migration**: Copy redb file or use migration script.

### 6.3 Models

- bkg-p2p: `~/.bkg-peer/models/`
- cave-p2p: `~/.cave-p2p/models/` (deferred to v1.0)

**Migration**: Copy models directory (when inference is ported).

### 6.4 Skills

- bkg-p2p: `~/.bkg-peer/skills/`
- cave-p2p: `~/.cave-p2p/skills/` (deferred to v1.0)

**Migration**: Copy skills directory (when skills are ported).

### 6.5 Tools

- bkg-p2p: `~/.bkg-peer/tools/`
- cave-p2p: `~/.cave-p2p/tools/` (deferred to v1.0)

**Migration**: Copy tools directory (when WASM is ported).

### 6.6 Prompts

- bkg-p2p: `~/.bkg-peer/prompts/`
- cave-p2p: `~/.cave-p2p/prompts/`

**Migration**: Copy prompts directory.

---

## 7. TESTING MIGRATION

### 7.1 Unit Tests

All bkg-p2p unit tests are ported directly to cave-p2p. Run:

```bash
cargo test -p cave-p2p
```

### 7.2 Integration Tests

New integration tests for cave-p2p plugin:

```bash
cargo test --workspace
```

### 7.3 Manual Testing

1. Start cave-p2p node: `cave p2p serve --web 127.0.0.1:8080`
2. Open browser: `http://127.0.0.1:8080`
3. Verify chat UI loads
4. Verify console dashboard loads
5. Verify WebSocket connects
6. Verify peer discovery works
7. Verify resource advertisement works

---

## 8. COMMON PITFALLS

### 8.1 Path Changes

- bkg-p2p uses `~/.bkg-peer/` as base directory
- cave-p2p should use `~/.cave-p2p/` or integrate with cave-core's path system
- **Action**: Update all `bootstrap::base_dir()` calls

### 8.2 Crate Name Changes

- bkg-p2p uses `bkg_p2p` as crate name
- cave-p2p uses `cave_p2p` as crate name
- **Action**: Update all `extern crate` and `use` statements

### 8.3 Module Path Changes

- bkg-p2p: `crate::p2p::Network`
- cave-p2p: `cave_p2p::p2p::Network`
- **Action**: Use search-replace for `crate::` → `cave_p2p::`

### 8.4 CLI Command Changes

- bkg-p2p: `bkg-peer serve`
- cave-p2p: `cave p2p serve`
- **Action**: Update all CLI command strings

### 8.5 Plugin Context

- cave-p2p runs inside the `cave` binary
- Must use `PluginContext` for shared state
- **Action**: Design plugin interface carefully

---

## 9. ROLLBACK PLAN

If migration fails:

1. Keep bkg-p2p repository intact
2. cave-p2p is a separate crate in the workspace
3. No changes to cave-core, cave-surge, cave-kindle, cave-forge
4. Can revert by removing cave-p2p from workspace

---

## 10. SUCCESS CRITERIA

1. **Feature Parity**: 100% of MUST PORT features implemented (see INVENTORY_PART1/2/3.md)
2. **Test Coverage**: All unit tests pass; integration tests for plugin loading, P2P discovery, job marketplace
3. **QA Gates**: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, `cargo doc` all clean
4. **Plugin Integration**: cave-p2p loads as a static plugin in the `cave` binary
5. **Zero Regressions**: All existing cave-core, cave-surge, cave-kindle, cave-forge tests still pass
6. **Documentation**: Complete docs for architecture, plugin API, configuration, CLI, web API, P2P protocol

---

## 11. TIMELINE

| Week | Phase                     | Deliverable                                                   |
| ---- | ------------------------- | ------------------------------------------------------------- |
| 1-2  | Phase 1: Foundation       | Basic P2P node with identity, swarm, events                   |
| 3-4  | Phase 2: Resource Sharing | Resource advertisement, provider discovery                    |
| 5-6  | Phase 3: Job Marketplace  | Full job lifecycle with token payments                        |
| 7-8  | Phase 4: Web Dashboard    | Complete admin UI with WebSocket + REST API                   |
| 9    | Phase 5: CLI & Config     | Complete CLI and configuration system                         |
| 10   | Phase 6: Testing & QA     | All tests pass, docs complete                                 |
| v1.0 | Phase 7: Deferred         | Inference, vector DB, WASM, tools, skills, MCP, agent runtime |

**Total Estimated Time**: 10 weeks for core plugin, + ongoing for v1.0 features.
