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
│ └── internal event routing
├── Identity
│ ├── Ed25519 keys
│ └── PeerId
├── Network
│ ├── libp2p swarm
│ ├── Kademlia
│ ├── GossipSub
│ ├── mDNS
│ └── Identify
├── A2AState
│ ├── Agent Cards
│ ├── JSON-RPC
│ └── Discovery
├── ResourceMonitor
│ ├── CPU
│ ├── RAM
│ ├── GPU
│ └── Storage
├── ComputeFabric
│ ├── Worker Registry
│ ├── Scheduler
│ ├── Resource Leasing
│ ├── GPU Sharing
│ ├── CPU Sharing
│ └── Remote Workers
├── ProviderTracker
│ ├── LLM Providers
│ ├── Model Discovery
│ └── Pricing
├── JobManager
│ ├── Requests
│ ├── Bids
│ ├── Acceptance
│ ├── Execution
│ └── Settlement
├── Wallet
│ ├── Credits
│ ├── Escrow
│ └── Staking
├── TaskExecutor
│ ├── Local execution
│ └── Remote execution
├── Web
│ ├── REST API
│ ├── WebSocket
│ └── Dashboard
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
