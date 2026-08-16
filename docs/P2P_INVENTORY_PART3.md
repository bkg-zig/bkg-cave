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
src/prompts/        →        cave-p2p/src/prompts/
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
