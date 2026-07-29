# cave-p2p Implementation Plan

Version: 1.0
Status: Port Phase
Source: bkg-p2p v0.3.0

## Primary Goal

Port bkg-p2p v0.3.0 into the bkg-cave plugin architecture.

Priority order:

1. Feature preservation
2. Protocol preservation
3. Test preservation
4. Plugin integration
5. Refactoring

No redesign before parity.

---

# Non Negotiable Rules

## Rule 1: Port before refactor

During port phase:

Allowed:

- move modules
- rename namespaces
- adapt plugin interfaces

Forbidden:

- changing protocols
- changing serialization
- removing features
- replacing working implementations

---

## Rule 2: Runtime is only composition layer

Runtime responsibilities:

- create components
- connect EventBus
- start services
- shutdown handling

Runtime must NOT:

- contain business logic
- schedule jobs
- manage resources
- control workers

Communication:

- EventBus
- Typed Events
- Protocol Messages

---

# Feature Scope

## Existing bkg-p2p Features

Must preserve:

- Identity
- libp2p networking
- Discovery
- A2A
- Providers
- Jobs
- Wallet
- REST API
- CLI
- Metrics

## Added Core Feature

Distributed Compute Fabric

Supported resources:

- CPU
- GPU
- RAM
- Storage

Capabilities:

- Worker discovery
- Resource advertisement
- Leasing
- Scheduling
- Remote execution

---

# Crate Layout

cave-p2p
src/
identity/
network/
a2a/
resources/
compute_fabric/
providers/
jobs/
wallet/
executor/
web/
metrics/
plugin.rs
runtime.rs

---

# Protocol Compatibility

Protocols MUST stay identical.

---

# Port Mapping

Every bkg-p2p module gets an explicit target in cave-p2p.

| bkg-p2p Source                   | cave-p2p Target                        | Status       |
| -------------------------------- | -------------------------------------- | ------------ |
| `src/lib.rs` (37 public modules) | `cave-p2p/src/lib.rs`                  | MUST PORT    |
| `src/node.rs`                    | `cave-p2p/src/runtime.rs`              | MUST PORT    |
| `src/runtime.rs`                 | `cave-p2p/src/runtime.rs`              | MUST PORT    |
| `src/identity/mod.rs`            | `cave-p2p/src/identity/mod.rs`         | MUST PORT    |
| `src/p2p/mod.rs`                 | `cave-p2p/src/network/mod.rs`          | MUST PORT    |
| `src/p2p/behaviour.rs`           | `cave-p2p/src/network/behaviour.rs`    | MUST PORT    |
| `src/p2p/events.rs`              | `cave-p2p/src/network/events.rs`       | MUST PORT    |
| `src/p2p/resource.rs`            | `cave-p2p/src/resources/mod.rs`        | MUST PORT    |
| `src/p2p/provider.rs`            | `cave-p2p/src/providers/mod.rs`        | MUST PORT    |
| `src/p2p/provider_tracker.rs`    | `cave-p2p/src/providers/tracker.rs`    | MUST PORT    |
| `src/p2p/tool_manifest.rs`       | `cave-p2p/src/providers/tools.rs`      | MUST PORT    |
| `src/p2p/envelope.rs`            | `cave-p2p/src/a2a/envelope.rs`         | MUST PORT    |
| `src/p2p/invite.rs`              | `cave-p2p/src/network/invite.rs`       | MUST PORT    |
| `src/p2p/permissions.rs`         | `cave-p2p/src/identity/permissions.rs` | MUST PORT    |
| `src/p2p/transfer.rs`            | `cave-p2p/src/executor/transfer.rs`    | MUST PORT    |
| `src/executor/mod.rs`            | `cave-p2p/src/executor/mod.rs`         | MUST PORT    |
| `src/executor/resource.rs`       | `cave-p2p/src/executor/resource.rs`    | MUST PORT    |
| `src/executor/remote.rs`         | `cave-p2p/src/executor/remote.rs`      | MUST PORT    |
| `src/executor/router.rs`         | `cave-p2p/src/executor/router.rs`      | MUST PORT    |
| `src/executor/task.rs`           | `cave-p2p/src/executor/task.rs`        | MUST PORT    |
| `src/web/mod.rs`                 | `cave-p2p/src/web/mod.rs`              | MUST PORT    |
| `src/wallet/mod.rs`              | `cave-p2p/src/wallet/mod.rs`           | MUST PORT    |
| `src/job/mod.rs`                 | `cave-p2p/src/jobs/mod.rs`             | MUST PORT    |
| `src/config.rs`                  | `cave-p2p/src/config.rs`               | MUST PORT    |
| `src/bootstrap.rs`               | `cave-p2p/src/bootstrap.rs`            | MUST PORT    |
| `src/db.rs`                      | `cave-p2p/src/db.rs`                   | MUST PORT    |
| `src/observability.rs`           | `cave-p2p/src/metrics/mod.rs`          | MUST PORT    |
| `src/safety.rs`                  | `cave-p2p/src/safety/mod.rs`           | DEFER (v1.0) |
| `src/inference/`                 | `cave-p2p/src/inference/`              | DEFER (v1.0) |
| `src/vector/`                    | `cave-p2p/src/vector/`                 | DEFER (v1.0) |
| `src/tools/`                     | `cave-p2p/src/tools/`                  | DEFER (v1.0) |
| `src/skills/`                    | `cave-p2p/src/skills/`                 | DEFER (v1.0) |
| `src/mcp/`                       | `cave-p2p/src/mcp/`                    | DEFER (v1.0) |
| `src/agent/`                     | `cave-p2p/src/agent/`                  | DEFER (v1.0) |
| `src/crew/`                      | `cave-p2p/src/crew/`                   | DEFER (v1.0) |
| `src/flow/`                      | `cave-p2p/src/flow/`                   | DEFER (v1.0) |
| `src/a2a/`                       | `cave-p2p/src/a2a/mod.rs`              | MUST PORT    |
| `src/swarm/`                     | `cave-p2p/src/metrics/swarm.rs`        | MUST PORT    |
| `src/hitl/`                      | `cave-p2p/src/hitl/`                   | DEFER (v1.0) |
| `src/wasm/`                      | `cave-p2p/src/wasm/`                   | DEFER (v1.0) |
| `src/messaging/`                 | `cave-p2p/src/messaging/`              | DEFER (v1.0) |
| `src/channels/`                  | `cave-p2p/src/channels/`               | DEFER (v1.0) |
| `src/reputation.rs`              | `cave-p2p/src/metrics/reputation.rs`   | MUST PORT    |
| `src/routines.rs`                | `cave-p2p/src/metrics/routines.rs`     | MUST PORT    |
| `src/run/`                       | `cave-p2p/src/run/`                    | MUST PORT    |
| `src/workspace.rs`               | `cave-p2p/src/workspace.rs`            | MUST PORT    |
| `src/prompts.rs`                 | `cave-p2p/src/prompts.rs`              | MUST PORT    |
| `src/proxy/`                     | `cave-p2p/src/proxy/`                  | MUST PORT    |
| `src/documentation/`             | `cave-p2p/src/documentation/`          | NICE-TO-HAVE |
| `src/models_hf/`                 | `cave-p2p/src/models_hf/`              | DEFER (v1.0) |
| `src/cli.rs`                     | `cave-p2p/src/cli.rs`                  | MUST PORT    |
| `src/agent_library.rs`           | `cave-p2p/src/agent_library.rs`        | MUST PORT    |

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

## Rule 4: Exit Gates are Integration Tests

No milestone is complete without:

- Two-node network test (where applicable)
- Protocol compliance test
- Feature parity checklist

"Compiles" is not enough.

---

## Rule 5: Deferred Features are Protected

During port phase, NO agent may implement:

- MCP
- Skills system
- Inference engines (GGUF, Ollama, Candle, HF-Hub)
- Vector database
- WASM sandbox
- Agent runtime (ReAct loop)
- Crew/Flow orchestration
- Safety layer (leak detection, injection defense)
- Messaging channels (Telegram, Webhook)

These are v1.0 features. Port phase ends when all MUST PORT modules are integrated and tested.

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

- NodeIdentity
- IdentityInfo

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

- network/
  - behaviour.rs
  - events.rs
  - mod.rs
  - resource.rs

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

- a2a/
  - mod.rs
  - jsonrpc.rs
  - gossip.rs

---

# Milestone 4

# Resource Discovery

Goal: Nodes can advertise and discover resources.

Resources:

- CPU: cores, threads, architecture, frequency, load
- RAM: total memory, free memory
- GPU: vendor, model, VRAM, driver, compute capability
- Storage: capacity, free space

Modules:

- resources/
  - mod.rs
  - manifest.rs
  - tracker.rs

Exit gate: Two nodes exchange resource manifests via GossipSub.

---

# Milestone 4.5

# Worker Protocol Stabilization

Goal: Freeze compute worker communication before economy layer.

Tasks:

- worker handshake
- capability exchange
- heartbeat
- lease protocol
- resource reservation protocol

Exit: Two workers can discover and reserve resources without jobs.

---

# Milestone 5

# Compute Fabric

Goal: Distributed resources become usable network resources.

New module:

- compute_fabric/
  - mod.rs
  - worker.rs
  - scheduler.rs
  - capability.rs
  - lease.rs
  - resource_pool.rs
  - heartbeat.rs
  - protocol.rs

## Worker System

A node can become:

- CPU Worker
- GPU Worker
- Build Worker
- AI Worker
- Storage Worker

Worker states:

- Available
- Busy
- Reserved
- Offline

## GPU Sharing

Technical boundary:

- GPU Worker does NOT expose raw GPU access
- GPU sharing happens through:
  - worker capability advertisement
  - job requirements
  - reservation
  - execution backend
- Direct PCI passthrough is NOT part of cave-p2p

Supported:

- NVIDIA (via nvidia-smi probing)
- Future AMD support

## CPU Sharing

Features:

- CPU worker registration
- core reservation
- load monitoring
- compile jobs
- CPU inference

## RAM Sharing

Features:

- memory advertisement
- memory limits
- job requirements

## Storage Sharing

Features:

- artifact storage
- distributed cache preparation
- capacity advertisement

---

# Milestone 6

# Economy and Distributed Jobs

Job lifecycle:

- Request
- Discovery
- Bid
- Accept
- Resource Lease
- Execution
- Result
- Settlement

Job types:

- Inference
- GPU Compute
- CPU Compute
- Build
- Storage Artifact
- Cache
- WASM Execution

Components:

- jobs/
  - mod.rs
  - manager.rs
  - bid.rs
  - settlement.rs
- executor/
  - task.rs
  - router.rs
  - remote.rs
  - worker.rs
  - scheduler.rs

---

# Milestone 7

# Wallet

Features:

- balance
- credit
- debit
- escrow
- staking
- settlement

Prerequisite: Jobs must be running (Milestone 6).

---

# Milestone 8

# Web Dashboard Core

Features:

- Axum server
- WebSocket control plane
- peer view
- resource view
- job view

---

# Milestone 9

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

- /v1/chat/completions
- /v1/models
- /v1/embeddings

---

# Milestone 10

# CLI and Plugin Integration

Commands:

- cave p2p serve
- cave p2p peers
- cave p2p wallet
- cave p2p jobs
- cave p2p resources
- cave p2p workers
- cave p2p gpu
- cave p2p network
- cave p2p doctor

---

# Milestone 11

# QA and Documentation

Required:

- cargo build --workspace
- cargo test --workspace
- cargo clippy --all -- -D warnings
- cargo fmt --all --check
- cargo doc --workspace --no-deps

Documentation:

- docs/p2p/
  - README.md
  - ARCHITECTURE.md
  - PROTOCOL.md
  - COMPUTE_FABRIC.md
  - WEB_API.md
  - CLI_REFERENCE.md

---

# Milestone 12

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

- GPU Worker
- CPU Worker
- Resource Provider

Node B:

- Submit Compute Job

Verify:

- discovery
- scheduling
- lease
- execution
- result delivery

---

## Worker Protocol Test

Node A: Register as GPU Worker
Node B: Register as CPU Worker
Node C: Request GPU resources

Verify:

- handshake
- capability exchange
- heartbeat
- reservation
- release

---

# Success Criteria

- 100% bkg-p2p core feature parity
- Plugin loads in cave
- CPU sharing works
- GPU sharing works (via worker protocol, NOT raw access)
- Remote execution works
- Job marketplace works
- Wallet settlement works
- Existing cave tests remain green
- No protocol breaking changes
- All deferred features remain deferred
