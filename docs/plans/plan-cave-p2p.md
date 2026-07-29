# cave-p2p Implementation Plan — Milestone Based

> Goal: Port bkg-p2p v0.3.0 into the bkg-cave plugin architecture.
> Principle: 100% feature preservation before refactoring.
> Additional Core Capability:
> Distributed Compute Fabric:
>
> - CPU Sharing
> - GPU Sharing
> - RAM Sharing
> - Storage Sharing
> - Remote Worker Execution

---

# Architecture Principle

## Runtime is a container, not a god object

The runtime only wires independent subsystems. Communication happens through:

- EventBus
- typed events
- protocol messages

No subsystem directly controls another subsystem.

---

# Runtime Components

Runtime
├── EventBus
│   └── internal event routing
├── Identity
│   ├── Ed25519 keys
│   └── PeerId
├── Network
│   ├── libp2p swarm
│   ├── Kademlia
│   ├── GossipSub
│   ├── mDNS
│   └── Identify
├── A2AState
│   ├── Agent Cards
│   ├── JSON-RPC
│   └── Discovery
├── ResourceMonitor
│   ├── CPU
│   ├── RAM
│   ├── GPU
│   └── Storage
├── ComputeFabric
│   ├── Worker Registry
│   ├── Scheduler
│   ├── Resource Leasing
│   ├── GPU Sharing
│   ├── CPU Sharing
│   └── Remote Workers
├── ProviderTracker
│   ├── LLM Providers
│   ├── Model Discovery
│   └── Pricing
├── JobManager
│   ├── Requests
│   ├── Bids
│   ├── Acceptance
│   ├── Execution
│   └── Settlement
├── Wallet
│   ├── Credits
│   ├── Escrow
│   └── Staking
├── TaskExecutor
│   ├── Local execution
│   └── Remote execution
├── Web
│   ├── REST API
│   ├── WebSocket
│   └── Dashboard
└── Metrics

---

# Protocol Preservation

Protocols must remain unchanged.

 /bkg-p2p/a2a-rpc/1.0.0
 /bkg-p2p/kad/1.0.0
 /bkg-p2p/1.0.0
 bkg-p2p/resources/v1
 bkg-p2p/providers/v1
 bkg-p2p/tools/v1
 bkg-p2p/skills/v1
 bkg-p2p/a2a/v1
 bkg-p2p/jobs/requests/v1
 bkg-p2p/jobs/bids/v1
 bkg-p2p/jobs/status/v1
 bkg-p2p/compute/workers/v1
 bkg-p2p/compute/resources/v1
 bkg-p2p/compute/jobs/v1

---

# Golden Rules

## Rule 1: Port before refactor

- Preserve APIs
- Preserve serialization
- Preserve protocols
- Preserve behavior
- Preserve tests

Refactoring happens only after feature parity.

---

## Rule 2: One Task = One Commit

Commit format:

feat(p2p): port <module> from bkg-p2p
test(p2p): add <module> tests
docs(p2p): update documentation

---

## Rule 3: QA after every task

Required:

cargo build -p cave-p2p
cargo test -p cave-p2p
cargo clippy -p cave-p2p -- -D warnings
cargo fmt -p cave-p2p -- --check

---

# Milestone 0

# Workspace Bootstrap

Goal: Create empty cave-p2p plugin crate.

Tasks:

- Create crate
- Add dependencies
- Add module structure
- Add plugin skeleton
- Integrate cave-core

Exit criteria:

- cave-p2p builds
- tests run
- plugin loads

---

# Milestone 1

# Identity System

Features:

- Ed25519 key generation
- PeerId generation
- Signing
- Verification
- Save/load identity

Structures:

NodeIdentity
IdentityInfo

Tests:

- generate identity
- save/load
- PeerId derivation

---

# Milestone 2

# Network Core

Features:

- libp2p swarm
- TCP transport
- Noise encryption
- Yamux
- Kademlia
- GossipSub
- Identify

Modules:

network/
behaviour.rs
events.rs
mod.rs
resource.rs

Exit gate: Two node communication test.

---

# Milestone 3

# A2A Core

Features:

- Agent Cards
- JSON-RPC
- Agent discovery
- Gossip announcements

Modules:

a2a/
mod.rs
jsonrpc.rs
gossip.rs

---

# Milestone 4

# Discovery and Compute Fabric

Goal: Distributed resources become usable network resources.

---

## Resource Discovery

Resources:

CPU:

- cores
- threads
- architecture
- frequency
- load

RAM:

- total memory
- free memory

GPU:

- vendor
- model
- VRAM
- driver
- compute capability

Storage:

- capacity
- free space

---

# Compute Fabric

New module:

compute_fabric/
mod.rs
worker.rs
scheduler.rs
capability.rs
lease.rs
resource_pool.rs
heartbeat.rs
protocol.rs

---

# Worker System

A node can become:

- CPU Worker
- GPU Worker
- Build Worker
- AI Worker
- Storage Worker

Worker states:

Available
Busy
Reserved
Offline

---

# GPU Sharing

Features:

- GPU discovery
- VRAM tracking
- capability detection
- worker advertisement
- GPU job scheduling
- resource reservation

Supported:

- NVIDIA
- future AMD support

---

# CPU Sharing

Features:

- CPU worker registration
- core reservation
- load monitoring
- compile jobs
- CPU inference

---

# RAM Sharing

Features:

- memory advertisement
- memory limits
- job requirements

---

# Storage Sharing

Features:

- artifact storage
- distributed cache preparation
- capacity advertisement

---

# Milestone 5

# Economy and Distributed Jobs

Job lifecycle:

Request
↓
Discovery
↓
Bid
↓
Accept
↓
Resource Lease
↓
Execution
↓
Result
↓
Settlement

---

Job types:

Inference
GPU Compute
CPU Compute
Build
Storage Artifact
Cache
WASM Execution

---

Components:

executor/
task.rs
router.rs
remote.rs
worker.rs
scheduler.rs

---

# Wallet

Features:

- balance
- credit
- debit
- escrow
- staking
- settlement

---

# Milestone 6

# Web Dashboard Core

Features:

- Axum server
- WebSocket control plane
- peer view
- resource view
- job view

---

# Milestone 7

# Full Web API

Preserved:

- 60+ REST endpoints
- OpenAI compatible API
- Wallet API
- Job API
- Peer API
- Resource API
- Metrics API
- Swarm visualization

OpenAI:

/v1/chat/completions
/v1/models
/v1/embeddings

---

# Milestone 8

# CLI and Plugin Integration

Commands:

cave p2p serve
cave p2p peers
cave p2p wallet
cave p2p jobs
cave p2p resources
cave p2p workers
cave p2p gpu
cave p2p network
cave p2p doctor

---

# Milestone 9

# QA and Documentation

Required:

cargo build --workspace
cargo test --workspace
cargo clippy --all -- -D warnings
cargo fmt --all --check
cargo doc --workspace --no-deps

Documentation:

docs/p2p/
README.md
ARCHITECTURE.md
PROTOCOL.md
COMPUTE_FABRIC.md
WEB_API.md
CLI_REFERENCE.md

---

# Milestone 10

# Deferred Features

Not part of first port:

- Inference engines
- Vector database
- WASM sandbox
- MCP
- Skills
- Tools
- Agent runtime
- Crew
- Flow
- Safety
- Messaging channels

---

# Integration Tests

## Two Node Network Test

Verify:

- discovery
- GossipSub
- A2A exchange
- resource exchange

---

## Compute Fabric Test

Node A:

GPU Worker
CPU Worker
Resource Provider

Node B:

Submit Compute Job

Verify:

- discovery
- scheduling
- lease
- execution
- result delivery

---

# Success Criteria

- 100% bkg-p2p core feature parity
- Plugin loads in cave
- CPU sharing works
- GPU sharing works
- Remote execution works
- Job marketplace works
- Wallet settlement works
- Existing cave tests remain green
- No protocol breaking changes

# bkg-p2p → cave-p2p Feature Inventory — Part 3

---

## 26. FEATURE PRESERVATION CHECKLIST

### P2P Networking

- [x] libp2p swarm with TCP + Noise + Yamux
- [x] QUIC transport support
- [x] Kademlia DHT
- [x] mDNS local discovery
- [x] GossipSub pub/sub
- [x] Identify protocol
- [x] A2A JSON-RPC over request-response
- [x] Ed25519 peer identity
- [x] Peer discovery events
- [x] Connection tracking
- [x] Bootstrap peer dialing
- [x] Periodic resource advertisement

### Resource Sharing

- [x] CPU core detection
- [x] RAM monitoring (real-time)
- [x] GPU detection (Nvidia/AMD/Apple/Intel)
- [x] GPU usage probing (nvidia-smi, ioreg)
- [x] VRAM tracking
- [x] Storage advertisement
- [x] Bandwidth advertisement
- [x] Capability flags (WasmSandbox, Inference, Storage, Relay, WebProxy, LlmProvider)
- [x] Resource manifest signing
- [x] Resource manifest gossip

### LLM Provider Sharing

- [x] Provider manifest advertisement
- [x] Model offerings (name, context, price, quantization, backend)
- [x] Rate limits (requests/hour, tokens/day, concurrent)
- [x] Provider tracker
- [x] Ollama backend
- [x] GGUF backend
- [x] OpenAI-compatible backend
- [x] Provider manifest signing
- [x] Price multiplier config

### Job Marketplace

- [x] Request → Bid → Accept → Execute → Settle flow
- [x] Resource type inference (model, tokens)
- [x] Resource type GPU (duration, VRAM)
- [x] Resource type CPU (cores, duration)
- [x] Resource type Storage (bytes, operation)
- [x] Resource type Embedding
- [x] Resource type ImageGeneration
- [x] Resource type WebFetch
- [x] Resource type VectorSearch
- [x] Resource type WasmTool
- [x] Escrow creation
- [x] Escrow release/refund
- [x] Spending limits (per-tx, hourly, daily)
- [x] Pricing strategies (Fixed, Dynamic, Auction)
- [x] Bid scoring (price, latency, reputation)
- [x] Job status tracking
- [x] Job result storage
- [x] GossipSub broadcast for all job messages

### Token Economy

- [x] PCLAW/BKG token (6 decimals, μBKG)
- [x] Wallet credit/debit
- [x] Balance snapshot (available, in_escrow, staked, total)
- [x] Escrow system (Active, Released, Refunded, Expired)
- [x] Staking/unstaking
- [x] Transaction history
- [x] Spending limits
- [x] Reserve balance
- [x] Expired escrow sweeping
- [x] Ed25519 signing for transactions

### Task Execution

- [x] Unified TaskExecutor
- [x] Smart routing (local vs network)
- [x] Resource-based routing
- [x] Utilization thresholds
- [x] Concurrent task limits
- [x] Local fallback on network failure
- [x] Inference task (GGUF + Ollama)
- [x] Web fetch task (GET/POST/PUT/DELETE/PATCH/HEAD)
- [x] Web search task (DuckDuckGo)
- [x] WASM task execution
- [x] Streaming inference
- [x] Batch aggregation
- [x] Model loading/unloading
- [x] Resource state tracking

### Web Dashboard

- [x] Axum HTTP server
- [x] WebSocket control plane (`/ws`)
- [x] SSE streaming
- [x] OpenAI-compatible API (`/v1/chat/completions`, `/v1/models`, `/v1/embeddings`)
- [x] Embedded chat UI (`/`, `/chat`)
- [x] Console dashboard (`/console`)
- [x] SPA fallback (`web/dist/`)
- [x] 60+ REST API endpoints
- [x] Agent library management
- [x] Workflow/crew/flow validation and kickoff
- [x] Task management (create, list, detail, stop)
- [x] Network topology view
- [x] Peer dialing
- [x] Job marketplace UI
- [x] Tool execution UI
- [x] Skills management
- [x] MCP configuration
- [x] Chat with session history
- [x] Provider settings
- [x] Model download/pull
- [x] Channel management
- [x] Wallet balance/transactions
- [x] Vector memory CRUD
- [x] A2A peer discovery
- [x] Metrics endpoint
- [x] HITL approvals
- [x] Runs/events (redb-backed)

### Admin UI Features

- [x] Real-time status updates (WebSocket)
- [x] Task progress streaming
- [x] Tool call event streaming
- [x] Agent library CRUD
- [x] Crew/flow run management
- [x] Network peer visualization
- [x] Resource monitoring (CPU/RAM/GPU)
- [x] Job marketplace integration
- [x] Wallet integration
- [x] Skills studio (AI generation)
- [x] MCP console
- [x] Setup wizard (onboarding steps)
- [x] Ollama integration
- [x] Model management

### CLI

- [x] `run <model>` - interactive chat
- [x] `run <model> "prompt"` - single query
- [x] `chat` - slash commands
- [x] `models list/download/pull`
- [x] `serve [--web] [--ollama] [--share-inference] [--agent]`
- [x] `network status`
- [x] `peers list`
- [x] `vector create/insert/search`
- [x] `skill list/install/search`
- [x] `tool list/info/build`
- [x] `wallet balance/send/history`
- [x] `job submit/status/list`
- [x] `doctor` - diagnostics

### Configuration

- [x] TOML config file
- [x] Environment variables
- [x] Prompt override directory
- [x] All sections: p2p, web, resources, database, agent, inference, vector, safety, executor, wasm, economy, provider_sharing, skills, mcp, orchestration, prompts

### Security

- [x] Noise encryption (all P2P traffic)
- [x] Ed25519 signatures (all messages)
- [x] Capability-based access
- [x] WASM sandbox (wasmtime)
- [x] Spending limits
- [x] Escrow system
- [x] Signature verification on manifests

### Observability

- [x] Tracing spans (react_pass, tool_execute, crew_task, job_phase)
- [x] Metrics (counters, gauges, histograms)
- [x] Prometheus-style `/api/metrics`
- [x] Structured logging (tracing + tracing-subscriber)
- [x] JSON log format

---

## 27. ARCHITECTURE MAPPING: bkg-p2p → cave-p2p

```
bkg-p2p (monolith)          cave-p2p (plugin)
─────────────────           ──────────────────
src/node.rs        →        cave-p2p/src/node.rs
src/runtime.rs     →        cave-p2p/src/runtime.rs
src/p2p/           →        cave-p2p/src/p2p/
src/executor/       →        cave-p2p/src/executor/
src/web/            →        cave-p2p/src/web/
src/wallet/         →        cave-p2p/src/wallet/
src/job/            →        cave-p2p/src/job/
src/identity/       →        cave-p2p/src/identity/
src/config.rs       →        cave-p2p/src/config.rs
src/bootstrap.rs    →        cave-p2p/src/bootstrap.rs
src/db.rs           →        cave-p2p/src/db.rs
src/observability.rs →      cave-p2p/src/observability.rs
src/swarm/          →        cave-p2p/src/swarm/
src/a2a/            →        cave-p2p/src/a2a/
src/reputation/     →        cave-p2p/src/reputation/
src/routines/       →        cave-p2p/src/routines/
src/run/            →        cave-p2p/src/run/
src/workspace/      →        cave-p2p/src/workspace/
src/prompts/      →        cave-p2p/src/prompts/
src/proxy/          →        cave-p2p/src/proxy/
src/agent_library.rs →      cave-p2p/src/agent_library.rs
src/cli.rs          →        cave-p2p/src/cli.rs
```

**Plugin Integration Points**:

- `CavePlugin` trait implementation in `cave-p2p/src/plugin.rs`
- `PluginManager::register_static(cave_p2p_plugin)`
- CLI subcommands: `cave p2p serve`, `cave p2p peers`, `cave p2p wallet`, etc.
- EventBus integration for P2P events
- ConfigRegistry integration for P2P config sections

---

## 28. DEPENDENCY MAPPING

| bkg-p2p Dep                    | cave-p2p Dep | Notes                  |
| ------------------------------ | ------------ | ---------------------- |
| `libp2p = "0.54"`              | same         | Core P2P               |
| `axum = "0.7"`                 | same         | Web server             |
| `tokio = "1"`                  | same         | Async runtime          |
| `redb = "2"`                   | same         | Database               |
| `ed25519-dalek = "2"`          | same         | Crypto                 |
| `blake3 = "1"`                 | same         | Hashing                |
| `serde + rmp-serde`            | same         | Serialization          |
| `clap = "4"`                   | same         | CLI                    |
| `tracing + tracing-subscriber` | same         | Logging                |
| `sysinfo = "0.31"`             | same         | Resource monitoring    |
| `thiserror = "2"`              | same         | Error handling         |
| `vectx`                        | DEFER        | Vector DB (v1.0)       |
| `wasmtime = "28"`              | DEFER        | WASM sandbox (v1.0)    |
| `llama-cpp-2`                  | DEFER        | Inference (v1.0)       |
| `candle-*`                     | DEFER        | Inference (v1.0)       |
| `hf-hub`                       | DEFER        | Model downloads (v1.0) |
| `askama = "0.12"`              | same         | Templates              |
| `rustyline = "14"`             | same         | CLI editor             |
| `parking_lot = "0.12"`         | same         | Locking                |
| `cel = "0.13"`                 | same         | Expression language    |
| `cron = "0.13"`                | same         | Scheduling             |

---

## 29. TEST COVERAGE (from bkg-p2p)

| Module              | Tests                                                                         |
| ------------------- | ----------------------------------------------------------------------------- |
| `identity`          | 3 tests (generate, sign/verify, save/load, peer_id derivation)                |
| `p2p/behaviour`     | 1 test (build_swarm)                                                          |
| `p2p/resource`      | 3 tests (manifest creation, signing, default)                                 |
| `p2p/provider`      | 2 tests (manifest creation, serialization)                                    |
| `executor/task`     | 4 tests (parse model size, cloud minimal, unknown name, task type)            |
| `executor/router`   | 3 tests (route lightweight, job requirements satisfied/not)                   |
| `executor/resource` | 4 tests (can_handle sufficient/insufficient/no_gpu, capacity)                 |
| `executor/mod`      | 3 tests (web fetch, inference placeholder, resource state)                    |
| `executor/remote`   | 1 test (config default)                                                       |
| `wallet`            | 5 tests (credit/balance, debit insufficient, escrow lifecycle, stake/unstake) |
| `job`               | 2 tests (create request, bid flow)                                            |
| `node`              | 1 test (node creation)                                                        |
| `runtime`           | 2 tests (runtime creation, stats)                                             |

**Total**: ~30 unit tests in bkg-p2p

---

## 30. CRITICAL IMPLEMENTATION NOTES

### 30.1 GPU Detection

- **Linux**: `nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits`
- **macOS**: `ioreg -r -d 1 -c IOAccelerator` parsing
- **Windows**: Not implemented (returns None)
- Must preserve this cross-platform behavior

### 30.2 Model Size Parsing

```rust
fn parse_model_size(model: &str) -> Option<f32> {
    // "llama-3.2-8b" → Some(8.0)
    // "mistral-7b-v0.1" → Some(7.0)
    // "glm-4.7:cloud" → None (remote)
    // "gpt-4o" → None (remote)
}
```

### 30.3 VRAM Estimation

```rust
// Q4 quantization estimates:
// <= 3B params: 2000 MB VRAM
// <= 8B params: 5000 MB VRAM
// <= 13B params: 8000 MB VRAM
// <= 34B params: 20000 MB VRAM
// <= 70B params: 40000 MB VRAM
// > 70B params: 80000 MB VRAM
// + 1000 MB for KV cache overhead
```

### 30.4 GossipSub Configuration

- Heartbeat interval: 10s
- Validation mode: Strict
- Message authenticity: Signed
- Must preserve for security

### 30.5 A2A Protocol

- Protocol: `/bkg-p2p/a2a-rpc/1.0.0`
- Kademlia protocol: `/bkg-p2p/kad/1.0.0`
- Identify protocol: `/bkg-p2p/1.0.0`
- Must preserve protocol strings

### 30.6 Token Precision

- 1 BKG = 1,000,000 μBKG (6 decimal places)
- All monetary values in μBKG internally
- Convert for display only

### 30.7 Initial Balance

- New wallets receive 1000 BKG (1,000,000,000 μBKG) as initial_balance
- Must preserve for testnet/development

### 30.8 Job Timeouts

- Bid collection: 10s
- Result waiting: 300s
- Bid validity: 60s

### 30.9 Resource Monitor Defaults

- Update interval: 1s
- Reserve RAM: 500 MB
- Max concurrent inference: 2
- Max concurrent WASM: 10

### 30.10 Router Defaults

- Local utilization threshold: 0.8
- Offload threshold: 0.9
- Network offload allowed: true
- Max concurrent inference: 2
- Max concurrent WASM: 10
- Local preference factor: 1.2

---

## 31. REFERENCES

- **bkg-p2p Repository**: https://github.com/1bitshit/bkg-p2p
- **bkg-p2p Version**: v0.3.0
- **bkg-p2p Commits**: 30
- **bkg-p2p License**: MIT OR Apache-2.0
- **bkg-p2p Rust Edition**: 2021
- **bkg-p2p Rust Version**: 1.80

---

## 32. NEXT STEPS

1. Review this inventory with the team
2. Create `docs/p2p/PLAN.md` with implementation phases
3. Create `docs/p2p/TASKS.md` with granular tasks
4. Begin Phase 1: Core P2P Plugin (identity, networking, events)
5. Preserve ALL features listed above — no exceptions
bkg-p2p → cave-p2p Feature Inventory — Part 2

---

## 4. WEB DASHBOARD (src/web/)

### 4.1 WebState

**50+ fields** including:

- `local_peer_id`, `resource_monitor`, `wallet_balance`
- `connected_peers`, `active_jobs`, `completed_jobs`
- `job_list: Vec<WebJobInfo>`
- `inference_tx`, `job_submit_tx` (channels)
- `swarm_manager: Option<Arc<SwarmManager>>`
- `task_store: Vec<WebTask>`
- `agent_task_cancels: HashMap<String, Arc<AtomicBool>>`
- `provider_tracker`, `agent_task_tx`
- `ws_control_tx: broadcast::Sender<serde_json::Value>`
- `skills`, `chat_sessions`, `session_store`
- `mcp_config`, `mcp_manager`
- `tools`, `node_tool_tx`
- `peer_dial_tx`, `p2p_network_hints`
- `inference`, `channel_registry`
- `wallet`, `vector_store`
- `database`, `metrics`, `hitl_store`
- `a2a`, `crew_store`, `flow_store`
- `prompts`, `agent_library_path`, `agent_library_user`

### 4.2 WebSocket Control Plane (`/ws`)

**Message Types**:

- `{ "type": "status", "data": {...} }` - resource + peers + jobs
- `{ "type": "tasks_changed" }` - task list updated
- `{ "type": "task_stream_delta", "task_id": "...", "text": "..." }` - live LLM chunks
- `{ "type": "task_tool_call", "task_id": "...", "tool_name": "...", "status": "started|completed", "args": "...", "result": "..." }` - tool call events
- `{ "type": "agents_library_changed" }` - agent list updated
- `{ "type": "crews_changed" }` - crew runs updated
- `{ "type": "flows_changed" }` - flow runs updated
- `{ "type": "flow_log", "run_id": "...", "line": "...", "status": "..." }` - flow log streaming

### 4.3 REST API Endpoints (60+ routes)

**Tasks**:

- `POST /api/tasks` - create task
- `GET /api/tasks` - list tasks
- `GET /api/tasks/:id` - task detail
- `POST /api/tasks/:id/stop` - cancel task

**Workflows/Crews/Flows**:

- `POST /api/workflows/validate` - validate spec
- `POST /api/workflows/kickoff` - start workflow
- `GET /api/workflows/runs` - list runs
- `GET /api/workflows/runs/:id` - run status
- `GET /api/workflows/runs/:id/stream` - SSE progress
- `POST /api/workflows/runs/:id/stop` - cancel run
- Legacy: `/api/crews/*`, `/api/flows/*`

**Agents**:

- `GET /api/agents/library` - list agents
- `POST /api/agents/library` - save agent
- `DELETE /api/agents/library/:id` - delete agent

**Network**:

- `GET /api/status` - node status
- `GET /api/onboarding` - setup wizard steps
- `GET /api/network/multiaddrs` - listen addresses
- `POST /api/peers/dial` - dial peer
- `GET /api/peers/network` - network topology
- `GET /api/peers` - peer list

**Jobs**:

- `GET /api/jobs` - list jobs
- `POST /api/jobs/submit` - submit job

**Tools**:

- `GET /api/tools` - list tools
- `GET /api/tools/:name` - tool detail
- `POST /api/tools/execute` - execute tool

**Skills**:

- `GET /api/skills/local` - local skills
- `GET /api/skills/network` - network skills
- `GET /api/skills/meta` - skills metadata
- `POST /api/skills/scan` - scan skills
- `GET /api/skills/templates` - skill templates
- `POST /api/skills/:name/toggle` - toggle skill
- `POST /api/skills/studio/ai` - AI skill generation
- `GET /api/skills/studio` - list studio skills
- `GET /api/skills/studio/:slug` - get skill
- `PUT /api/skills/studio/:slug` - update skill

**MCP**:

- `GET /api/mcp/status` - MCP status
- `PUT /api/mcp/config` - update config

**Chat**:

- `POST /api/chat` - chat (non-streaming)
- `POST /api/chat/stream` - chat (streaming)

**Sessions**:

- `GET /api/sessions` - list chat sessions

**Providers**:

- `GET /api/providers` - list LLM providers
- `GET /api/providers/config` - get config
- `POST /api/providers/config` - set config

**Nodes**:

- `GET /api/nodes/:id` - node detail

**Runs**:

- `GET /api/runs` - list runs (redb-backed)
- `GET /api/runs/:id` - run detail
- `GET /api/runs/:id/events` - run events

**Swarm**:

- `GET /api/swarm/agents` - swarm agents
- `GET /api/swarm/topology` - network topology
- `GET /api/swarm/timeline` - event timeline

**Inference**:

- `GET /api/inference/settings` - get settings
- `PUT /api/inference/settings` - update settings
- `POST /api/models/download` - download model
- `GET /api/ollama/models` - list Ollama models
- `POST /api/ollama/pull` - pull Ollama model

**Channels**:

- `GET /api/channels` - list channels
- `POST /api/channels` - register channel
- `DELETE /api/channels/:id` - remove channel
- `POST /api/channels/:id/test` - test channel

**Wallet**:

- `GET /api/wallet` - balance
- `GET /api/wallet/transactions` - transaction history

**Vector Memory**:

- `GET /api/vector/collections` - list collections
- `POST /api/vector/collections` - create collection
- `DELETE /api/vector/collections/:name` - delete collection
- `POST /api/vector/search` - semantic search
- `POST /api/vector/insert` - insert text

**A2A**:

- `GET /api/a2a/peers` - discovered agent cards
- `GET /.well-known/agent-card.json` - agent card
- `POST /a2a` - JSON-RPC interface

**Metrics**:

- `GET /api/metrics` - Prometheus-style metrics

**Approvals (HITL)**:

- `GET /api/approvals` - list pending approvals
- `POST /api/approvals/:id/approve` - approve
- `POST /api/approvals/:id/reject` - reject

**OpenAI-Compatible**:

- `POST /v1/chat/completions` - chat completions (SSE streaming)
- `GET /v1/models` - list models
- `POST /v1/embeddings` - embeddings

### 4.4 Static UI

- `/` → `chat.html` (embedded)
- `/chat` → `chat.html`
- `/console` → `dashboard.html`
- `/embed/chat` → legacy chat
- `/embed/console` → legacy console
- SPA fallback from `web/dist/` when built

---

## 5. WALLET & TOKEN ECONOMY (src/wallet/)

### 5.1 Wallet

```rust
pub const TOKEN_DECIMALS: u32 = 6;
pub const MICRO_BKG: u64 = 1_000_000;

pub struct Wallet {
    identity: Arc<NodeIdentity>,
    state: RwLock<WalletState>,
    config: WalletConfig,
    database: Database,
}

pub struct WalletState {
    available: u64,           // μBKG
    in_escrow: u64,
    staked: u64,
    escrows: HashMap<EscrowId, Escrow>,
    transactions: Vec<Transaction>,
    hourly_spend: HashMap<u64, u64>,
    daily_spend: HashMap<u64, u64>,
}
```

**Methods**:

- `new(identity, config, database)` → create/load wallet
- `balance()` → `BalanceSnapshot { available, in_escrow, staked, total }`
- `credit(amount, reason)` → add tokens
- `debit(amount, reason)` → spend tokens (with limits)
- `check_spending_limits(amount)` → per-tx/hour/day limits
- `create_escrow(amount, recipient, job_id, timeout)` → lock funds
- `release_escrow(escrow_id)` → pay provider
- `refund_escrow(escrow_id)` → refund requester
- `stake(amount)` / `unstake(amount)` → provider bond
- `transactions(limit)` → recent history
- `active_escrows()` → active escrow list
- `sweep_expired_escrows()` → cleanup expired
- `sign(message)` / `verify(message, sig)` → Ed25519

**WalletConfig**:

- `max_spend_per_tx: 100 BKG`
- `max_spend_per_hour: 500 BKG`
- `max_spend_per_day: 2000 BKG`
- `reserve_balance: 100 BKG`

### 5.2 Escrow System

```rust
pub struct Escrow {
    id: EscrowId,
    amount: u64,
    recipient: String,
    job_id: String,
    created_at: u64,
    expires_at: u64,
    status: EscrowStatus,  // Active, Released, Refunded, Expired
}
```

### 5.3 Transactions

```rust
pub struct Transaction {
    id: TransactionId,
    tx_type: TransactionType,
    amount: u64,
    timestamp: u64,
    reason: String,
    counterparty: Option<String>,
    job_id: Option<String>,
}
```

---

## 6. JOB MARKETPLACE (src/job/)

### 6.1 Job Manager

```rust
pub struct JobManager {
    local_peer_id: String,
    requests: RwLock<HashMap<JobId, JobRequest>>,
    bids: RwLock<HashMap<JobId, Vec<JobBid>>>,
    active_jobs: RwLock<HashMap<JobId, Job>>,
    completed_jobs: RwLock<Vec<Job>>,
    pricing: RwLock<PricingStrategy>,
    wallet: Arc<Wallet>,
    broadcast_tx: RwLock<Option<JobBroadcastTx>>,
    resource_monitor: Option<ResourceMonitor>,
}
```

**Resource Types**:

```rust
pub enum ResourceType {
    Inference { model: String, tokens: u32 },
    Gpu { duration_secs: u64, vram_mb: u32 },
    Cpu { cores: u16, duration_secs: u64 },
    Storage { bytes: u64, operation: StorageOperation },
    Embedding { tokens: u32 },
    ImageGeneration { count: u32 },
    WebFetch { url_count: u32 },
    VectorSearch { query_count: u32 },
    WasmTool { tool_name: String, invocations: u32 },
}
```

**Methods**:

- `create_request(request)` → broadcast to network
- `receive_bid(bid)` → store bid
- `get_bids(job_id)` → list bids
- `accept_bid(job_id, bid_id)` → create escrow, notify winner
- `evaluate_request(request)` → decide if we can fulfill
- `can_fulfill(request)` → check local resources
- `submit_bid(request)` → broadcast bid
- `start_job(job_id)` → mark in_progress
- `submit_result(job_id, result)` → store result
- `settle_job(job_id, success)` → release/refund escrow
- `active_jobs()` / `completed_jobs(limit)` / `pending_requests()`
- `describe_job_for_tools(job_id)` → JSON for UI tools

### 6.2 Job Network Protocol

**Topics**:

- `JOB_REQUESTS` - new job requests
- `JOB_BIDS` - bids on requests
- `JOB_STATUS` - status updates, results, acceptances

**Messages**:

```rust
pub enum JobMessage {
    Request(JobRequestMessage),
    Bid(JobBidMessage),
    BidAccepted(BidAcceptedMessage),
    Result(JobResultMessage),
    StatusUpdate(JobStatusMessage),
}
```

### 6.3 Pricing

```rust
pub enum PricingStrategy {
    Fixed(f64),           // fixed price per unit
    Dynamic { factor: f64, base: f64 },  // dynamic based on demand
    Auction,              // let market decide
}

pub struct ResourcePricing {
    pub inference_per_1k_tokens: f64,
    pub gpu_per_hour: f64,
    pub cpu_per_hour: f64,
    pub storage_per_gb: f64,
    pub web_fetch_per_request: f64,
    pub wasm_per_invocation: f64,
}
```

---

## 7. IDENTITY & CRYPTO (src/identity/)

```rust
pub struct NodeIdentity {
    signing_key: SigningKey,  // Ed25519
    peer_id: PeerId,          // libp2p PeerId
}

// Methods:
// generate() → new random identity
// from_bytes(secret) → load from bytes
// load(path) / save(path) → file persistence
// peer_id() → libp2p PeerId
// public_key() / public_key_bytes()
// sign(message) → Ed25519 signature
// verify(message, signature) → bool
// to_libp2p_keypair() → libp2p identity
```

---

## 8. CONFIGURATION (src/config.rs)

### 8.1 Config Sections

```rust
pub struct Config {
    pub p2p: P2pConfig,
    pub web: WebConfig,
    pub resources: ResourceConfig,
    pub database: DatabaseConfig,
    pub agent: AgentConfig,
    pub inference: InferenceConfig,
    pub vector: VectorConfig,
    pub safety: SafetyConfig,
    pub executor: ExecutorConfig,
    pub wasm: WasmConfig,
    pub economy: EconomyConfig,
    pub provider_sharing: ProviderSharingConfig,
    pub skills: SkillsConfig,
    pub mcp: McpConfig,
    pub orchestration: OrchestrationConfig,
    pub prompts: PromptsConfig,
}
```

### 8.2 P2P Config

```rust
pub struct P2pConfig {
    pub listen_addresses: Vec<String>,
    pub bootstrap_peers: Vec<String>,
    pub mdns_enabled: bool,
    pub kademlia_enabled: bool,
    pub advertise_interval_secs: u64,
}
```

### 8.3 Inference Config

```rust
pub struct InferenceConfig {
    pub models_dir: PathBuf,
    pub default_model: String,
    pub max_loaded_models: u32,
    pub max_memory_mb: u32,
    pub gpu_layers: i32,
    pub context_size: u32,
    pub batch_size: u32,
    pub enable_p2p_download: bool,
    pub use_ollama: bool,
    pub ollama_url: String,
    pub use_local_gguf: bool,
    pub remote_api_enabled: bool,
    pub remote_api_base_url: String,
    pub remote_api_model: String,
    pub remote_api_key: String,
}
```

### 8.4 Economy Config

```rust
pub struct EconomyConfig {
    pub enabled: bool,
    pub inference_price_per_1k: u64,
    pub tool_price_per_call: u64,
}
```

### 8.5 Provider Sharing Config

```rust
pub struct ProviderSharingConfig {
    pub enabled: bool,
    pub advertise_models: bool,
    pub max_requests_per_hour: u32,
    pub max_tokens_per_day: u64,
    pub max_concurrent_requests: u32,
    pub price_multiplier: f64,
}
```

---

## 9. OBSERVABILITY (src/observability.rs)

```rust
pub struct Metrics {
    // Counters, gauges, histograms
}

pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Vec<f64>>,
}

// Tracing spans:
// - react_pass
// - tool_execute
// - crew_task
// - job_phase
// All with run_id field
```

---

## 10. SWARM VISUALIZATION (src/swarm/)

```rust
pub struct SwarmManager {
    event_buffer: Vec<SwarmEvent>,
    action_history: Vec<ActionRecord>,
    agents: HashMap<String, SwarmAgent>,
    track_remote_peers: bool,
}

pub struct SwarmAgent {
    pub id: String,
    pub peer_id: Option<PeerId>,
    pub profile: AgentProfile,
    pub status: AgentStatus,
    pub last_seen: u64,
}

pub struct AgentProfile {
    pub capabilities: Vec<Capability>,
    pub models: Vec<String>,
    pub resources: Resources,
}

pub enum SwarmEvent {
    PeerDiscovered,
    PeerConnected,
    PeerDisconnected,
    TaskOffered,
    TaskClaimed,
    TaskCompleted,
    ResourceAdvertised,
}

pub enum ActionType {
    CrewTaskOffer,
    CrewTaskClaim,
    CrewTaskComplete,
    JobBid,
    JobAccepted,
    JobCompleted,
}
```

---

## 11. A2A PROTOCOL (src/a2a/)

### 11.1 Agent Card

```rust
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub peer_id: String,
    pub models: Vec<String>,
    pub capabilities: AgentCapabilities,
    pub channels: Vec<String>,
}

pub struct AgentCapabilities {
    pub streaming: bool,
    pub tools: bool,
    pub crew: bool,
    pub vector_memory: bool,
}
```

### 11.2 A2A State

```rust
pub struct A2aState {
    peer_cards: RwLock<HashMap<String, AgentCard>>,
    task_store: RwLock<HashMap<String, A2aTask>>,
}

// Methods:
// upsert_peer_card(peer_id, card)
// list_peer_cards() → Vec<(String, AgentCard)>
// handle_jsonrpc(body) → serde_json::Value
```

### 11.3 Gossip Topics

- `A2A_GOSSIP_TOPIC` - agent card announcements
- `bkg-p2p/world/{campaign}/v1` - campaign milestones

---

## 12. GOSSIPSUB TOPICS (ALL MUST BE PORTED)

| Topic                         | Purpose                         |
| ----------------------------- | ------------------------------- |
| `bkg-p2p/resources/v1`        | Resource manifests              |
| `bkg-p2p/providers/v1`        | LLM provider advertisements     |
| `bkg-p2p/tools/v1`            | Tool manifest advertisements    |
| `bkg-p2p/skills/v1`           | Skill announcements             |
| `bkg-p2p/a2a/v1`              | Agent cards                     |
| `bkg-p2p/crew/tasks/v1`       | Crew task offers/claims/results |
| `bkg-p2p/crew/pod/v1`         | Pod artifacts                   |
| `bkg-p2p/world/{campaign}/v1` | Campaign milestones             |
| `bkg-p2p/jobs/requests/v1`    | Job requests                    |
| `bkg-p2p/jobs/bids/v1`        | Job bids                        |
| `bkg-p2p/jobs/status/v1`      | Job status/results              |

---

## 13. CLI COMMANDS (src/cli.rs)

### 13.1 Chat & Inference

- `bkg-peer run <model>` - interactive chat
- `bkg-peer run <model> "prompt"` - single query
- `bkg-peer chat` - chat with slash commands

### 13.2 Models

- `bkg-peer models list` - list downloaded models
- `bkg-peer models download <model>` - download from HF
- `bkg-peer pull <model>` - alias for download

### 13.3 Network

- `bkg-peer serve` - start peer node
- `bkg-peer serve --web <addr>` - with web dashboard
- `bkg-peer serve --ollama --share-inference` - share LLM
- `bkg-peer serve --ollama --agent <toml>` - with agent
- `bkg-peer network status` - network health
- `bkg-peer peers list` - connected peers

### 13.4 Vector Memory

- `bkg-peer vector create <collection>`
- `bkg-peer vector insert <collection> <text>`
- `bkg-peer vector search <collection> <query> -k 5`

### 13.5 Skills, Tools, Wallet, Jobs

- `bkg-peer skill list | install | search`
- `bkg-peer tool list | info | build`
- `bkg-peer wallet balance | send | history`
- `bkg-peer job submit | status | list`
- `bkg-peer doctor` - diagnostics

---

## 14. CONFIGURATION KEYS (config.toml)

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

[resources]
advertise_gpu = true
cpu_cores = 5
storage_bytes = 5545115
ram_mb = 16225

[database]
path = "~/.bkg-peer/data/bkg-peer.redb"

[agent]
max_agents = 10
default_model = "llama-3.2-3b"
tool_timeout_secs = 60

[inference]
models_dir = "~/.bkg-peer/models"
default_model = "llama-3.2-3b"
max_loaded_models = 5
max_memory_mb = 16000
gpu_layers = -1
context_size = 4096
batch_size = 512
enable_p2p_download = true
use_ollama = false
ollama_url = "http://localhost:11434"
use_local_gguf = true
remote_api_enabled = false
remote_api_base_url = ""
remote_api_model = ""
remote_api_key = ""

[vector]
embedding_dim = 384
use_hnsw = true
enable_bm25 = true
distance = "Cosine"

[safety]
leak_detection_enabled = true
injection_defense_enabled = true
policy_enabled = true
max_output_length = 100000
redaction_string = "[REDACTED]"
tool_timeout = 30

[executor]
local_utilization_threshold = 0.8
offload_threshold = 0.9
allow_network_offload = true
max_concurrent_inference = 2
max_concurrent_wasm = 10
max_web_response_size = 10485760
default_web_timeout_secs = 30
batch_window_ms = 50
max_batch_size = 8
min_batch_size = 4

[wasm]
tools_dir = "~/.bkg-peer/tools"
max_memory_mb = 256
default_fuel_limit = 100000000
timeout_secs = 60

[economy]
enabled = true
inference_price_per_1k = 500000
tool_price_per_call = 20000

[provider_sharing]
enabled = false
advertise_models = true
max_requests_per_hour = 60
max_tokens_per_day = 100000
max_concurrent_requests = 2
price_multiplier = 1.0

[skills]
directory = "~/.bkg-peer/skills"

[mcp]
enabled = true
timeout_secs = 30
auto_reconnect = true
servers = []

[orchestration]
crew_worker = false

[prompts]
directory = "~/.bkg-peer/prompts"
```

---

## 15. DEPENDENCIES (Cargo.toml)

### 15.1 Core Dependencies

```toml
tokio = { version = "1", features = ["full"] }
tokio-stream = { version = "0.1", features = ["sync"] }
futures = "0.3"
libp2p = { version = "0.54", features = [
    "tokio", "noise", "tcp", "quic", "mdns", "kad",
    "gossipsub", "request-response", "json", "identify",
    "macros", "serde", "yamux",
] }
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "fs", "normalize-path"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rmp-serde = "1"
toml = "0.8"
lru = "0.12"
clap = { version = "4", features = ["derive", "env"] }
rustyline = "14"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"
ed25519-dalek = { version = "2", features = ["serde", "rand_core"] }
blake3 = "1"
rand = "0.8"
hex = "0.4"
num_cpus = "1"
sysinfo = "0.31"
redb = "2"
vectx = { git = "https://github.com/antonellof/vectX.git" }
parking_lot = "0.12"
wasmtime = { version = "28", features = ["component-model"] }
wasmtime-wasi = "28"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
time = "0.3.36"
thiserror = "2"
anyhow = "1"
async-trait = "0.1"
askama = "0.12"
dirs = "5"
base64 = "0.22"
flate2 = "1"
regex = "1"
serde_yaml = "0.9"
which = "7"
url = "2"
cron = "0.13"
cel = { version = "0.13", default-features = false, features = ["json", "regex", "chrono"] }
```

### 15.2 Optional Dependencies (feature-gated)

```toml
[features]
default = ["local-inference"]
local-inference = [
    "dep:candle-core",
    "dep:candle-nn",
    "dep:candle-transformers",
    "dep:hf-hub",
    "dep:tokenizers",
    "dep:llama-cpp-2",
    "dep:llama-cpp-sys-2",
]
metal = []  # Apple Silicon GPU
cuda = []   # Nvidia GPU
```

---

## 16. RUNTIME INTEGRATION (src/runtime.rs)

### 16.1 Runtime Struct

```rust
pub struct Runtime {
    identity: Arc<NodeIdentity>,
    database: Arc<Database>,
    wallet: Arc<Wallet>,
    job_manager: Arc<RwLock<JobManager>>,
    network: Arc<RwLock<Network>>,
    executor: Arc<TaskExecutor>,
    inference: Arc<InferenceEngine>,
    model_distributor: Arc<ModelDistributor>,
    job_provider: Arc<JobProvider>,
    remote_executor: Arc<RemoteExecutor>,
    batch_aggregator: Arc<BatchAggregator>,
    tools: Arc<ToolRegistry>,
    skills: Arc<SkillRegistry>,
    provider_tracker: Arc<ProviderTracker>,
    local_peer_id: PeerId,
    config: Config,
    prompts: Arc<PromptBundle>,
    a2a: Arc<A2aState>,
    metrics: Arc<Metrics>,
    swarm_manager: RwLock<Option<Arc<SwarmManager>>>,
}
```

### 16.2 Key Methods

- `new(identity, database, config)` → initialize all subsystems
- `attach_swarm_manager(sm)` → wire swarm for visualization
- `subscribe_to_job_topics()` → subscribe to all GossipSub topics
- `advertise_provider()` → broadcast LLM provider manifest
- `advertise_tools()` → broadcast tool manifests
- `advertise_skills()` → broadcast skill announcements
- `set_pricing(strategy)` → update pricing
- `execute_task(task)` → route and execute
- `inference(prompt, model, max_tokens)` → simple inference
- `inference_batched(source, model, prompt, max_tokens, temperature)` → batch inference
- `inference_streaming_print(model, prompt, max_tokens, temperature)` → streaming to stdout
- `web_fetch(url)` → fetch web page
- `resource_state()` → current resources
- `balance()` → wallet balance
- `connected_peers_count()` → peer count
- `handle_gossip_message(topic, data, source)` → route all gossip messages
- `gossip_resource_manifest()` → broadcast resources
- `gossip_agent_card(public_base_url)` → publish A2A card
- `stats()` → `RuntimeStats`

### 16.3 Gossip Message Routing

The runtime handles these topics:

- `JOB_REQUESTS` → `job_provider.handle_request()`
- `JOB_BIDS` → `job_manager.receive_bid()`
- `JOB_STATUS` → bid acceptance, results, status updates
- `TOOL_MANIFEST_TOPIC` → discover remote tools
- `PROVIDER_TOPIC` → `provider_tracker.update_provider()`
- `SKILLS_TOPIC` → `skills.handle_announcement_batch()`
- `A2A_GOSSIP_TOPIC` → `a2a.upsert_peer_card()`
- `RESOURCES_GOSSIP_TOPIC` → log resource manifests
- `CREW_TASK_TOPIC` → crew task offers/claims/results
- `POD_TOPIC` → pod artifacts
- `bkg-p2p/world/*/v1` → campaign milestones

---

## 17. NODE ORCHESTRATOR (src/node.rs)

```rust
pub struct Node {
    config: Config,
    identity: NodeIdentity,
    database: Database,
    network: Network,
    a2a: Arc<A2aState>,
    swarm_manager: Arc<SwarmManager>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}
```

**Methods**:

- `new(config)` → bootstrap dirs, load/generate identity, open DB, create network + swarm
- `peer_id()` → local peer ID
- `database()` → database reference
- `shutdown_handle()` → shutdown channel
- `swarm_manager()` → swarm reference
- `run()` → start network, web server, event loop

**Startup Sequence**:

1. `bootstrap::ensure_dirs()` - create data directories
2. Load/generate Ed25519 identity
3. Open redb database
4. Create A2A state
5. Create P2P network
6. Create swarm manager
7. Start network listener
8. Optionally start web server
9. Register local agent in swarm
10. Main event loop: network events + Ctrl+C

---

## 18. DATABASE (src/db.rs)

```rust
pub struct Database {
    // redb-backed persistent storage
}

// Methods:
// open(path) → Database
// store_setting(key, value) → Result
// get_setting<T>(key) → Result<Option<T>>
// list_run_ids() → Vec<String>
// get_run<T>(id) → Result<Option<T>>
// list_run_events(run_id) → Vec<RunEvent>
```

---

## 19. BOOTSTRAP (src/bootstrap.rs)

```rust
// Methods:
// ensure_dirs() → create data directories
// identity_path() → Path to identity.key
// base_dir() → ~/.bkg-peer
// data_dir() → ~/.bkg-peer/data
// models_dir() → ~/.bkg-peer/models
// skills_dir() → ~/.bkg-peer/skills
// tools_dir() → ~/.bkg-peer/tools
// prompts_dir() → ~/.bkg-peer/prompts
```

---

## 20. ROUTINES (src/routines.rs)

```rust
pub struct RoutineEngine {
    routines: Vec<Routine>,
}

pub struct Routine {
    pub config: RoutineConfig,
    pub schedule: cron::Schedule,
}

pub struct RoutineConfig {
    pub name: String,
    pub cron: String,
    pub enabled: bool,
}

// Heartbeat routine for network keep-alive
```

---

## 21. REPUTATION (src/reputation.rs)

```rust
pub struct PeerReputation {
    peer_id: PeerId,
    trust_level: TrustLevel,
    score: f64,
    completed_jobs: u32,
    failed_jobs: u32,
    avg_latency_ms: f64,
}

pub enum TrustLevel {
    New,
    Established,
    Trusted,
    Verified,
}

pub struct ReputationStore {
    peers: HashMap<PeerId, PeerReputation>,
}

pub struct ReputationConfig {
    pub new_threshold: f64,
    pub established_threshold: f64,
    pub trusted_threshold: f64,
}
```

---

## 22. PROMPTS (src/prompts.rs)

```rust
pub struct PromptBundle {
    // Embedded defaults + optional overlay directory
}

// Methods:
// load_prompt_bundle(config) → Arc<PromptBundle>
// agentic_system_intro() → &str
// crew_worker_prompt(summary) → &str
// Resolution: config.toml → BKG_PEER_PROMPTS_DIR → ~/.bkg-peer/prompts
```

---

## 23. WORKSPACE (src/workspace.rs)

```rust
pub struct Workspace {
    pub config: WorkspaceConfig,
    pub root: PathBuf,
}

pub struct WorkspaceConfig {
    pub root: PathBuf,
    pub agents_dir: PathBuf,
    pub workflows_dir: PathBuf,
    pub skills_dir: PathBuf,
}
```

---

## 24. AGENT LIBRARY (src/agent_library.rs)

```rust
pub struct AgentLibraryEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub flow_spec: Option<FlowSpec>,
    pub is_builtin: bool,
}

// Built-in entries:
// - deep-researcher
// - code-reviewer
// - creative-writer
// - data-analyst
```

---

## 25. SUMMARY: MUST PORT vs DEFER

### MUST PORT (Core P2P Plugin)

1. **Identity**: Ed25519 keypair, PeerId derivation, save/load
2. **P2P Networking**: libp2p swarm, Kademlia, mDNS, GossipSub, Identify, Noise, Yamux, QUIC
3. **Network Events**: PeerDiscovered/Connected/Disconnected, GossipMessage, A2aRpcInbound
4. **Resource Manifest**: CPU/RAM/GPU/Storage advertisement, Capability flags
5. **Provider Manifest**: LLM model offerings, rate limits, pricing
6. **Provider Tracker**: Network LLM discovery
7. **Tool Manifest**: Tool advertisement via GossipSub
8. **Executor Core**: TaskExecutor, TaskRouter, ResourceMonitor
9. **Task Types**: Inference, WebFetch, WebSearch, WasmExecution
10. **Remote Executor**: Job marketplace client (request→bid→accept→execute→settle)
11. **Job Provider**: Server-side job handler
12. **Wallet**: Token accounting, escrow, staking, spending limits
13. **Job Manager**: Full marketplace lifecycle
14. **Web Dashboard**: Axum server, 60+ REST endpoints, WebSocket control plane
15. **OpenAI API**: `/v1/chat/completions`, `/v1/models`, `/v1/embeddings`
16. **A2A Protocol**: Agent cards, JSON-RPC, GossipSub discovery
17. **Swarm Manager**: Agent visualization, action history
18. **Observability**: Metrics, tracing spans
19. **Routines**: Cron-based heartbeat
20. **Reputation**: Peer trust levels
21. **Prompts**: Override system
22. **Workspace**: Directory management
23. **Agent Library**: Built-in + user agents
24. **Bootstrap**: Directory creation, paths
25. **Database**: redb persistence
26. **Config**: All TOML sections

### DEFER (v1.0)

1. **Inference Engines**: GGUF (llama-cpp-2), Ollama, Candle, HF-Hub
2. **Vector Database**: vectX HNSW+BM25
3. **WASM Sandbox**: wasmtime component model
4. **Tools**: 20+ builtin tools, WASM tools, MCP tools
5. **Skills**: SKILL.md system, registry, scanning
6. **MCP**: Model Context Protocol client
7. **Agent Runtime**: ReAct loop, tool calling, budget enforcement
8. **Crew/Flow Orchestration**: Multi-agent pipelines
9. **Safety Layer**: Leak detection, injection defense, HITL
10. **Messaging Channels**: Telegram, Webhook, REPL
11. **Models HF**: HuggingFace model downloads
12. **Documentation Agent**: Auto-docs

# bkg-p2p → cave-p2p Feature Inventory — Part 1

> **Purpose**: Line-by-line feature extraction from `bkg-p2p` v0.3.0 to ensure zero feature loss during recode as `cave-p2p` plugin.
> **Source**: https://github.com/1bitshit/bkg-p2p (30 commits, v0.3.0)

---

## 1. CRATE STRUCTURE

| bkg-p2p Module                   | cave-p2p Target                        | Status                    |
| -------------------------------- | -------------------------------------- | ------------------------- |
| `src/lib.rs` (37 public modules) | `cave-p2p/src/lib.rs`                  | MUST PORT                 |
| `src/node.rs`                    | `cave-p2p/src/node.rs`                 | MUST PORT                 |
| `src/runtime.rs`                 | `cave-p2p/src/runtime.rs`              | MUST PORT                 |
| `src/p2p/mod.rs`                 | `cave-p2p/src/p2p/mod.rs`              | MUST PORT                 |
| `src/p2p/behaviour.rs`           | `cave-p2p/src/p2p/behaviour.rs`        | MUST PORT                 |
| `src/p2p/events.rs`              | `cave-p2p/src/p2p/events.rs`          | MUST PORT                 |
| `src/p2p/resource.rs`            | `cave-p2p/src/p2p/resource.rs`         | MUST PORT                 |
| `src/p2p/provider.rs`            | `cave-p2p/src/p2p/provider.rs`         | MUST PORT                 |
| `src/p2p/provider_tracker.rs`    | `cave-p2p/src/p2p/provider_tracker.rs` | MUST PORT                 |
| `src/p2p/tool_manifest.rs`       | `cave-p2p/src/p2p/tool_manifest.rs`    | MUST PORT                 |
| `src/p2p/envelope.rs`            | `cave-p2p/src/p2p/envelope.rs`         | MUST PORT                 |
| `src/p2p/invite.rs`              | `cave-p2p/src/p2p/invite.rs`           | MUST PORT                 |
| `src/p2p/permissions.rs`         | `cave-p2p/src/p2p/permissions.rs`      | MUST PORT                 |
| `src/p2p/transfer.rs`            | `cave-p2p/src/p2p/transfer.rs`         | MUST PORT                 |
| `src/executor/mod.rs`            | `cave-p2p/src/executor/mod.rs`         | MUST PORT                 |
| `src/executor/resource.rs`       | `cave-p2p/src/executor/resource.rs`    | MUST PORT                 |
| `src/executor/remote.rs`         | `cave-p2p/src/executor/remote.rs`      | MUST PORT                 |
| `src/executor/router.rs`         | `cave-p2p/src/executor/router.rs`      | MUST PORT                 |
| `src/executor/task.rs`           | `cave-p2p/src/executor/task.rs`        | MUST PORT                 |
| `src/web/mod.rs`                 | `cave-p2p/src/web/mod.rs`              | MUST PORT                 |
| `src/wallet/mod.rs`              | `cave-p2p/src/wallet/mod.rs`           | MUST PORT                 |
| `src/job/mod.rs`                 | `cave-p2p/src/job/mod.rs`              | MUST PORT                 |
| `src/identity/mod.rs`            | `cave-p2p/src/identity/mod.rs`         | MUST PORT                 |
| `src/config.rs`                  | `cave-p2p/src/config.rs`               | MUST PORT                 |
| `src/bootstrap.rs`               | `cave-p2p/src/bootstrap.rs`            | MUST PORT                 |
| `src/db.rs`                      | `cave-p2p/src/db.rs`                   | MUST PORT                 |
| `src/observability.rs`           | `cave-p2p/src/observability.rs`        | MUST PORT                 |
| `src/safety.rs`                  | `cave-p2p/src/safety.rs`               | DEFER (v1.0)              |
| `src/inference/`                 | `cave-p2p/src/inference/`              | DEFER (v1.0)              |
| `src/vector/`                    | `cave-p2p/src/vector/`                 | DEFER (v1.0)              |
| `src/tools/`                     | `cave-p2p/src/tools/`                  | DEFER (v1.0)              |
| `src/skills/`                    | `cave-p2p/src/skills/`                 | DEFER (v1.0)              |
| `src/mcp/`                       | `cave-p2p/src/mcp/`                    | DEFER (v1.0)              |
| `src/agent/`                     | `cave-p2p/src/agent/`                  | DEFER (v1.0)              |
| `src/crew/`                      | `cave-p2p/src/crew/`                   | DEFER (v1.0)              |
| `src/flow/`                      | `cave-p2p/src/flow/`                   | DEFER (v1.0)              |
| `src/a2a/`                       | `cave-p2p/src/a2a/`                    | MUST PORT (core protocol) |
| `src/swarm/`                     | `cave-p2p/src/swarm/`                  | MUST PORT                 |
| `src/hitl/`                      | `cave-p2p/src/hitl/`                   | DEFER (v1.0)              |
| `src/wasm/`                      | `cave-p2p/src/wasm/`                   | DEFER (v1.0)              |
| `src/messaging/`                 | `cave-p2p/src/messaging/`              | DEFER (v1.0)              |
| `src/channels/`                  | `cave-p2p/src/channels/`               | DEFER (v1.0)              |
| `src/reputation/`                | `cave-p2p/src/reputation/`             | MUST PORT                 |
| `src/routines/`                  | `cave-p2p/src/routines/`               | MUST PORT                 |
| `src/run/`                       | `cave-p2p/src/run/`                    | MUST PORT                 |
| `src/workspace/`                 | `cave-p2p/src/workspace/`              | MUST PORT                 |
| `src/prompts/`                   | `cave-p2p/src/prompts/`                | MUST PORT                 |
| `src/proxy/`                     | `cave-p2p/src/proxy/`                  | MUST PORT                 |
| `src/documentation/`             | `cave-p2p/src/documentation/`          | NICE-TO-HAVE              |
| `src/models_hf/`                 | `cave-p2p/src/models_hf/`              | DEFER (v1.0)              |
| `src/cli.rs`                     | `cave-p2p/src/cli.rs`                  | MUST PORT                 |
| `src/agent_library.rs`           | `cave-p2p/src/agent_library.rs`        | MUST PORT                 |

---

## 2. P2P NETWORKING LAYER (src/p2p/)

### 2.1 Behaviour Composition (`behaviour.rs`)

```rust
#[derive(NetworkBehaviour)]
pub struct BkgPeerdBehaviour {
    pub a2a_rpc: request_response::json::Behaviour<A2aRpcWireRequest, A2aRpcWireResponse>,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
}
```

**Features**:

- A2A JSON-RPC over request-response (`/bkg-p2p/a2a-rpc/1.0.0`)
- Kademlia DHT with custom protocol (`/bkg-p2p/kad/1.0.0`)
- mDNS local discovery
- GossipSub with Strict validation, signed messages, 10s heartbeat
- Identify protocol with agent version
- TCP transport + Noise encryption + Yamux
- QUIC transport support (in Cargo.toml)
- Idle connection timeout: 60s

### 2.2 Network Controller (`mod.rs`)

**Struct**: `Network { swarm, event_tx, local_peer_id, connected_peers, config, a2a_state, listen_addresses }`

**Methods**:

- `new(identity, config, a2a_state)` → create swarm
- `start()` → listen on configured addresses, dial bootstrap peers
- `run(shutdown_rx)` → event loop with periodic advertisement
- `process_swarm_event(event)` → handle libp2p events
- `subscribe(topic)` → subscribe to GossipSub topic
- `publish(topic, data)` → publish to GossipSub topic
- `dial(addr)` → dial remote peer
- `event_receiver()` → broadcast receiver for NetworkEvent
- `connected_peers()` → list of connected PeerIds
- `listen_addresses()` → actual bound multiaddresses
- `a2a_state()` → shared A2A state

**Event Handling**:

- `NewListenAddr` → track listen addresses
- `ConnectionEstablished/Closed` → peer tracking
- `Mdns(Discovered/Expired)` → peer discovery + Kademlia address injection
- `Kademlia(RoutingUpdated)` → routing table updates
- `Gossipsub(Message)` → emit `NetworkEvent::GossipMessage`
- `A2aRpc` → JSON-RPC request/response handling
- `Identify(Received)` → peer info + address injection

### 2.3 Network Events (`events.rs`)

```rust
pub enum NetworkEvent {
    PeerDiscovered { peer_id, addresses },
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    GossipMessage { topic, data, source },
    RequestReceived { request_id, from, payload },
    A2aRpcInbound { peer, method },
    ResourceAdvertised { peer_id, manifest },
}
```

### 2.4 Resource Manifest (`resource.rs`)

```rust
pub struct ResourceManifest {
    peer_id: String,
    timestamp: u64,
    signature: Vec<u8>,
    resources: Resources,
    capabilities: Vec<Capability>,
    supported_models: Vec<String>,
    uptime_hours: u64,
}

pub struct Resources {
    cpu_cores: u16,
    cpu_available_mhz: u32,
    gpu: Option<GpuInfo>,
    storage_available_bytes: u64,
    bandwidth_mbps: u32,
    ram_available_mb: u32,
}

pub struct GpuInfo {
    vendor: GpuVendor,  // Nvidia, Amd, Apple, Intel, Other
    vram_mb: u32,
    compute_capability: String,
    model_name: String,
}

pub enum Capability {
    WasmSandbox,
    Inference,
    Storage,
    Relay,
    WebProxy,
    LlmProvider,
}
```

### 2.5 Provider Manifest (`provider.rs`)

```rust
pub const PROVIDER_TOPIC: &str = "bkg-p2p/providers/v1";

pub struct ProviderManifest {
    peer_id: String,
    models: Vec<ModelOffering>,
    rate_limits: ProviderRateLimits,
    timestamp: u64,
    signature: Vec<u8>,
}

pub struct ModelOffering {
    model_name: String,
    context_size: u32,
    price_per_1k_tokens: u64,
    max_tokens_per_request: u32,
    quantization: Option<String>,
    backend: ProviderBackend,  // Ollama, Gguf, OpenAiCompatible
}

pub struct ProviderRateLimits {
    max_requests_per_hour: u32,
    max_tokens_per_day: u64,
    max_concurrent_requests: u32,
}
```

### 2.6 Provider Tracker (`provider_tracker.rs`)

- Tracks discovered LLM providers on the network
- Updates provider manifests
- Filters by model availability

### 2.7 Tool Manifest (`tool_manifest.rs`)

```rust
pub const TOOL_MANIFEST_TOPIC: &str = "bkg-p2p/tools/v1";

pub struct ToolManifest {
    name: String,
    description: String,
    category: String,
    parameters: serde_json::Value,
    local_only: bool,
    tags: Vec<String>,
    peer_id: String,
}

pub struct ToolManifestAdvert {
    peer_id: String,
    tools: Vec<ToolManifest>,
}
```

### 2.8 Other P2P Modules

- `envelope.rs` - A2A JSON-RPC envelope handling
- `invite.rs` - Peer invitation system
- `permissions.rs` - Capability-based access control
- `transfer.rs` - File/data transfer protocol

---

## 3. EXECUTOR SYSTEM (src/executor/)

### 3.1 Task Executor (`mod.rs`)

**Struct**: `TaskExecutor { router, resource_monitor, job_manager, network, config, gguf_engine, loaded_model, inference_engine, remote_executor }`

**Key Methods**:

- `new(resource_monitor, router_config, config)` → create executor
- `with_job_manager()` → attach job manager
- `with_network()` → attach P2P network
- `with_inference_engine()` → attach inference engine
- `with_remote_executor()` → attach remote executor
- `execute(task)` → route and execute (local or remote)
- `execute_local(task_id, task, start)` → force local execution
- `execute_remote(task_id, task, peer_filter, start)` → offload to network
- `execute_inference_local(task)` → local GGUF/Ollama inference
- `execute_inference_streaming(task, delta_tx)` → streaming inference
- `execute_inference_streaming_print(task)` → streaming to stdout
- `execute_web_fetch_local(task)` → HTTP fetch
- `execute_web_search_local(task)` → DuckDuckGo search
- `execute_wasm_local(task)` → WASM sandbox execution
- `resource_state()` → current resource state
- `has_model(model_id)` → check if model loaded

### 3.2 Resource Monitor (`resource.rs`)

**Struct**: `ResourceMonitor { state, config }`

**State**:

```rust
pub struct ResourceState {
    cpu_usage: f64,              // 0.0 - 1.0
    ram_total_mb: u32,
    ram_available_mb: u32,
    gpu_usage: Option<f64>,      // None if no GPU
    vram_total_mb: Option<u32>,
    vram_available_mb: Option<u32>,
    loaded_models: Vec<String>,
    active_inference_tasks: u32,
    active_wasm_tasks: u32,
    active_web_tasks: u32,
}
```

**Methods**:

- `new(config)` / `with_defaults()` → create monitor
- `start_background_updates()` → spawn background task (1s interval)
- `current_state()` → snapshot
- `can_handle(requirements)` → check if resources sufficient
- `available_capacity()` → 0.0-1.0 ratio
- `register_loaded_model(model_id)` / `unregister_model(model_id)`
- `task_started(task_type)` / `task_completed(task_type)`

**GPU Probing**:

- macOS: `ioreg` IOAccelerator
- Linux: `nvidia-smi`
- Windows: None (returns None)

### 3.3 Task Router (`router.rs`)

**Struct**: `TaskRouter { resource_monitor, config }`

**Config**:

```rust
pub struct RouterConfig {
    local_utilization_threshold: f64,  // 0.8 default
    offload_threshold: f64,            // 0.9 default
    allow_network_offload: bool,       // true default
    max_concurrent_inference: u32,     // 2 default
    max_concurrent_wasm: u32,          // 10 default
    local_preference_factor: f64,      // 1.2 default
}
```

**Decisions**:

- `ExecuteLocally` - run on this node
- `OffloadToNetwork { requirements }` - send to P2P
- `InsufficientResources { reason }` - cannot execute

**Peer Filter**:

```rust
pub struct PeerFilter {
    min_ram_mb: Option<u32>,
    min_vram_mb: Option<u32>,
    min_cpu_cores: Option<u16>,
    required_model: Option<String>,
    required_capabilities: Vec<String>,
}
```

### 3.4 Task Types (`task.rs`)

**ExecutionTask enum**:

- `Inference(InferenceTask)` - LLM inference
- `WebFetch(WebFetchTask)` - HTTP fetch
- `WebSearch(WebSearchTask)` - web search
- `WasmExecution(WasmTask)` - WASM tool

**InferenceTask**:

- `model: String`
- `messages: Vec<ChatMessage>`
- `max_tokens: u32`
- `temperature: f32`
- `stop_sequences: Vec<String>`
- `stream: bool`
- `estimate_requirements()` → VRAM/RAM based on model size

**WebFetchTask**:

- `url: String`
- `method: HttpMethod` (GET/POST/PUT/DELETE/PATCH/HEAD)
- `headers: Vec<(String, String)>`
- `body: Option<Vec<u8>>`
- `timeout_secs: u32`

**WebSearchTask**:

- `query: String`
- `max_results: u32`
- `engine: SearchEngine` (Default/Google/Bing/DuckDuckGo)

**WasmTask**:

- `tool_hash: String` (BLAKE3 hash)
- `function: String`
- `params: serde_json::Value`
- `capabilities: WasmCapabilities`
- `timeout_secs: u32`
- `max_fuel: u64`

**WasmCapabilities**:

- `network_access: bool`
- `filesystem_read: bool`
- `filesystem_write: bool`
- `allowed_hosts: Vec<String>`

**ResourceRequirements**:

- `ram_mb: u32`
- `vram_mb: Option<u32>`
- `cpu_cores: u16`
- `estimated_duration: Duration`

### 3.5 Remote Executor (`remote.rs`)

**Struct**: `RemoteExecutor { job_manager, network, local_peer_id, config }`

**Flow**:

1. `execute(task_id, task, peer_filter)` → broadcast request
2. `collect_bids(job_id, peer_filter)` → wait for bids (10s timeout)
3. `select_best_bid(bids)` → score by price + latency + reputation
4. `execute_with_bid(job_id, bid)` → accept bid, wait for result (300s timeout)
5. `settle_job(job_id, true)` → release escrow

**JobProvider**:

- `handle_request(msg)` → evaluate and bid
- `handle_bid_accepted(msg)` → prepare for execution
- `get_pending_request(job_id)` → retrieve for execution
