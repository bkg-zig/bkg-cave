# bkg-p2p → cave-p2p Feature Inventory — Part 2

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
