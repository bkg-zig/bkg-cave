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
| `src/p2p/events.rs`              | `cave-p2p/src/p2p/events.rs`           | MUST PORT                 |
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
