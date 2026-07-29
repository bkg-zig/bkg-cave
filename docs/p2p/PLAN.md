# cave-p2p Implementation Plan — Milestone-Based

> **Goal**: Port bkg-p2p v0.3.0 as a cave-p2p plugin with 100% feature preservation.
> **Architecture**: Thin Runtime container over independent subsystems.
> **Defer**: Inference engines, vector DB, WASM sandbox, tools, skills, MCP, agent runtime, crew/flow, safety.

---

## Architecture

```text
cave-p2p
├── identity
│   ├── Ed25519 Identity
│   └── PeerId
├── network
│   ├── libp2p swarm
│   ├── Kademlia
│   ├── GossipSub
│   ├── mDNS
│   └── Identify
├── resources
│   ├── CPU detection
│   ├── RAM detection
│   ├── GPU detection
│   ├── VRAM detection
│   ├── Storage detection
│   └── Resource advertisement
├── compute_fabric
│   ├── CPU sharing
│   ├── GPU sharing
│   ├── Remote workers
│   ├── Capability scheduling
│   └── Resource leasing
├── provider
│   ├── LLM provider discovery
│   ├── Model advertisement
│   └── Inference capacity
├── executor
│   ├── Local execution
│   ├── Remote execution
│   ├── Task routing
│   └── Worker selection
├── marketplace
│   ├── Job request
│   ├── Bid
│   ├── Accept
│   ├── Execute
│   └── Settlement
├── wallet
│   ├── Credits
│   ├── Escrow
│   ├── Staking
│   └── Spending limits
├── runtime
│   └── Thin container (EventBus + subsystem wiring)
├── a2a
│   ├── Agent cards
│   ├── JSON-RPC
│   └── Gossip discovery
├── web
│   ├── Dashboard
│   ├── API
│   └── Monitoring
├── cli
│   └── Clap commands
├── config
│   └── TOML configuration
└── storage
    ├── redb database
    └── File system
```

**Communication Rule**: No subsystem knows another directly. All events flow through `EventBus`.

**Initialization Order**:

1. `Identity` (Ed25519 keypair, PeerId)
2. `Swarm` (libp2p behaviour + transport)
3. `Network` (swarm wrapper, event bus, GossipSub)
4. `A2AState` (agent cards, JSON-RPC — needs Network)
5. `ProviderTracker` (network LLM discovery — needs Network + A2A)
6. `Runtime` (wires all subsystems together)

---

## Milestone Overview

| Milestone | Content                              |
| --------- | ------------------------------------ |
| M0        | Workspace + Plugin Bootstrap         |
| M1        | Identity                             |
| M2        | P2P Network Core                     |
| M3        | A2A Protocol                         |
| M4        | Resource Discovery + Compute Sharing |
| M5        | Job Marketplace + Remote Execution   |
| M6        | Web Dashboard Core                   |
| M7        | Full API                             |
| M8        | CLI + Plugin Integration             |
| M9        | QA + Documentation                   |
| M10       | Deferred AI Features                 |

---

## Golden Rules

1. **Port before refactor**: Existing code is ported 1:1, then modernized after feature parity
2. **One task = one commit**: Small, reviewable, rollback-friendly commits
3. **QA after every task**: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
4. **Reference tracking**: Every task traces back to its bkg-p2p source
5. **Thin containers**: Runtime is a container, not a god object
6. **Protocol preservation**: Network protocols are never renamed
7. **Plugin-first architecture**: Plugin API before domain implementation

---

## M0 — Workspace Bootstrap & Dependency Audit

**Exit Criteria**:

- `crates/cave-p2p/Cargo.toml` exists with all dependencies pinned
- `crates/cave-p2p/src/lib.rs` compiles (empty crate)
- `cave-p2p` added to workspace `Cargo.toml` members
- `cargo build -p cave-p2p` succeeds
- Dependency compatibility verified against cave-core and workspace

**Tasks**:

- [ ] 0.0 Dependency compatibility audit
  - **Source**: `bkg-p2p/Cargo.toml`, `cave-core/Cargo.toml`, workspace `Cargo.toml`
  - **Check**: libp2p, tokio, serde, tracing, axum, thiserror, async-trait, uuid, ed25519-dalek versions
  - **Deliverable**: Compatibility matrix (no version conflicts)
  - **Commit**: `chore(p2p): audit dependencies against workspace`
  - **QA**: build, test, clippy, fmt

- [ ] 0.1 Create crate directory structure
  - **Commit**: `feat(p2p): create crate directory structure`
  - **QA**: build, test, clippy, fmt

- [ ] 0.2 Write `Cargo.toml` with exact dependency versions from bkg-p2p
  - **Source**: `bkg-p2p/Cargo.toml` (all dependencies)
  - **Commit**: `feat(p2p): add Cargo.toml with pinned dependencies`
  - **QA**: build, test, clippy, fmt

- [ ] 0.3 Write `src/lib.rs` with module declarations
  - **Source**: `bkg-p2p/src/lib.rs` (module list)
  - **Commit**: `feat(p2p): add lib.rs with module declarations`
  - **QA**: build, test, clippy, fmt

- [ ] 0.4 Write `src/error.rs` with `thiserror` error types
  - **Commit**: `feat(p2p): add error.rs with thiserror types`
  - **QA**: build, test, clippy, fmt

- [ ] 0.5 Add `cave-p2p` to workspace `Cargo.toml` members
  - **Commit**: `feat(p2p): add cave-p2p to workspace members`
  - **QA**: build, test, clippy, fmt

- [ ] 0.6 Create plugin skeleton (empty trait impl)
  - **File**: `src/plugin.rs`
  - **Struct**: `CaveP2PPlugin` implementing `CavePlugin` (empty methods)
  - **Purpose**: Detect API breaks early between cave-core and cave-p2p
  - **Commit**: `feat(p2p): add CavePlugin skeleton`
  - **QA**: build, test, clippy, fmt

---

## M1 — Identity

**Exit Criteria**:

- `NodeIdentity` generates, saves, loads, signs, verifies
- Ed25519 keypair maps 1:1 to libp2p PeerId
- All unit tests pass

**Tasks**:

- [ ] 1.1 Create `identity/mod.rs`
  - **Source**: `bkg-p2p/src/identity/mod.rs` (full file)
  - **Structs**: `NodeIdentity`, `IdentityInfo`
  - **Functions**: `generate()`, `from_bytes()`, `load()`, `save()`, `peer_id()`, `public_key()`, `public_key_bytes()`, `sign()`, `verify()`, `to_libp2p_keypair()`
  - **Commit**: `feat(p2p): port NodeIdentity from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 1.2 Add identity tests
  - **Source**: `bkg-p2p/src/identity/mod.rs` (tests module)
  - **Tests**: `test_generate_and_sign`, `test_save_and_load`, `test_peer_id_derivation`
  - **Commit**: `test(p2p): add identity tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

---

## M2 — P2P Network Core

**Exit Criteria**:

- `BkgPeerdBehaviour` compiles with all 5 behaviours
- Swarm builds with TCP + Noise + Yamux
- `Network` starts, listens, dials bootstrap peers
- GossipSub subscribe/publish works
- `NetworkEvent` enum fully typed
- **GATE: Two-node integration test passes**

**Tasks**:

- [ ] 2.1 Create `p2p/behaviour.rs`
  - **Source**: `bkg-p2p/src/p2p/behaviour.rs` (full file)
  - **Structs**: `BkgPeerdBehaviour`
  - **Functions**: `build_swarm()`
  - **Protocols**: `/bkg-p2p/a2a-rpc/1.0.0`, `/bkg-p2p/kad/1.0.0`, `/bkg-p2p/1.0.0`
  - **Commit**: `feat(p2p): port BkgPeerdBehaviour from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 2.2 Add behaviour test
  - **Source**: `bkg-p2p/src/p2p/behaviour.rs` (tests module)
  - **Tests**: `test_build_swarm`
  - **Commit**: `test(p2p): add behaviour test from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 2.3 Create `p2p/mod.rs` (Network controller)
  - **Source**: `bkg-p2p/src/p2p/mod.rs` (full file)
  - **Structs**: `Network`
  - **Functions**: `new()`, `start()`, `run()`, `process_swarm_event()`, `subscribe()`, `publish()`, `dial()`, `event_receiver()`, `connected_peers()`, `listen_addresses()`, `a2a_state()`
  - **Commit**: `feat(p2p): port Network controller from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 2.4 Create `p2p/events.rs`
  - **Source**: `bkg-p2p/src/p2p/events.rs` (full file)
  - **Structs**: `NetworkEvent`
  - **Commit**: `feat(p2p): port NetworkEvent from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 2.5 Create `p2p/resource.rs`
  - **Source**: `bkg-p2p/src/p2p/resource.rs` (full file)
  - **Structs**: `ResourceManifest`, `Resources`, `GpuInfo`, `GpuVendor`, `Capability`
  - **Functions**: `ResourceManifest::new()`, `signing_bytes()`, `sign()`, `verify()`
  - **Commit**: `feat(p2p): port ResourceManifest from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 2.6 Add resource manifest tests
  - **Source**: `bkg-p2p/src/p2p/resource.rs` (tests module)
  - **Tests**: `test_resource_manifest_creation`, `test_manifest_signing`
  - **Commit**: `test(p2p): add resource manifest tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 2.7 Integration test: two-node GossipSub
  - **File**: `tests/p2p_two_nodes.rs`
  - **Scenario**: Node A listens on `/ip4/127.0.0.1/tcp/0`, Node B dials A, A publishes GossipSub message, B receives it, both exchange A2A cards and resource manifests
  - **Commit**: `test(p2p): add two-node GossipSub integration test`
  - **QA**: build, test, clippy, fmt

---

## M3 — A2A Protocol

**Exit Criteria**:

- Agent Card structure ported
- A2A State manages peer cards and tasks
- JSON-RPC envelope handling works
- A2A GossipSub discovery functional
- A2A integrates with Network via EventBus

**Tasks**:

- [ ] 3.1 Create `a2a/mod.rs`
  - **Source**: `bkg-p2p/src/a2a/mod.rs` (full file)
  - **Structs**: `AgentCard`, `AgentCapabilities`, `A2aState`, `AgentCardAnnouncement`
  - **Functions**: `new()`, `upsert_peer_card()`, `list_peer_cards()`, `handle_jsonrpc()`, `bkg_p2p_default()`
  - **Constants**: `A2A_GOSSIP_TOPIC`
  - **Commit**: `feat(p2p): port A2A protocol core from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 3.2 Create `a2a/jsonrpc.rs`
  - **Source**: `bkg-p2p/src/a2a/jsonrpc.rs` (full file)
  - **Structs**: `A2aRpcWireRequest`, `A2aRpcWireResponse`, `JsonRpcEnvelope`
  - **Functions**: `handle_jsonrpc_body()`, `dispatch()`
  - **Commit**: `feat(p2p): port A2A JSON-RPC from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 3.3 Create `a2a/gossip.rs`
  - **Source**: `bkg-p2p/src/a2a/gossip.rs` (full file)
  - **Functions**: `gossip_agent_card()`, `handle_agent_card_gossip()`
  - **Commit**: `feat(p2p): port A2A gossip from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 3.4 Add A2A tests
  - **Source**: `bkg-p2p/src/a2a/` (tests modules)
  - **Tests**: Agent card creation, JSON-RPC handling, gossip routing
  - **Commit**: `test(p2p): add A2A tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

---

## M4 — Resource Discovery & Compute Sharing

**Goal**: Not just display resources. The network must offer and use resources:

- CPU Worker
- GPU Worker
- RAM Pools
- Storage Capacity
- LLM Provider
- Build Worker

**Exit Criteria**:

- `ProviderManifest` advertises models with pricing
- `ProviderTracker` discovers and tracks remote LLM providers
- `ToolManifest` advertises local tools via GossipSub
- `ResourceMonitor` tracks CPU/RAM/GPU in real-time
- `Runtime` initializes all subsystems and subscribes to all GossipSub topics

**Tasks**:

- [ ] 4.1 Create `p2p/provider.rs`
  - **Source**: `bkg-p2p/src/p2p/provider.rs` (full file)
  - **Structs**: `ProviderManifest`, `ModelOffering`, `ProviderBackend`, `ProviderRateLimits`, `ProviderSharingConfig`
  - **Functions**: `ProviderManifest::new()`, `is_expired()`, `signing_bytes()`, `sign()`, `has_model()`, `find_model()`
  - **Constants**: `PROVIDER_TOPIC = "bkg-p2p/providers/v1"`
  - **Commit**: `feat(p2p): port ProviderManifest from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.2 Add provider manifest tests
  - **Source**: `bkg-p2p/src/p2p/provider.rs` (tests module)
  - **Tests**: `test_provider_manifest_creation`, `test_provider_manifest_serialization`
  - **Commit**: `test(p2p): add provider manifest tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.3 Create `p2p/provider_tracker.rs`
  - **Source**: `bkg-p2p/src/p2p/provider_tracker.rs` (full file)
  - **Structs**: `ProviderTracker`
  - **Functions**: `new()`, `update_provider()`, `get_providers()`, `find_model()`
  - **Commit**: `feat(p2p): port ProviderTracker from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.4 Create `p2p/tool_manifest.rs`
  - **Source**: `bkg-p2p/src/p2p/tool_manifest.rs` (full file)
  - **Structs**: `ToolManifest`, `ToolManifestAdvert`
  - **Constants**: `TOOL_MANIFEST_TOPIC = "bkg-p2p/tools/v1"`
  - **Commit**: `feat(p2p): port ToolManifest from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.5 Create `executor/resource.rs`
  - **Source**: `bkg-p2p/src/executor/resource.rs` (full file)
  - **Structs**: `ResourceMonitor`, `ResourceState`, `MonitorConfig`, `TaskType`
  - **Functions**: `new()`, `with_defaults()`, `start_background_updates()`, `current_state()`, `can_handle()`, `available_capacity()`, `register_loaded_model()`, `unregister_model()`, `task_started()`, `task_completed()`
  - **GPU Probing**: Linux (`nvidia-smi`), macOS (`ioreg`), Windows (None)
  - **Commit**: `feat(p2p): port ResourceMonitor from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.6 Add resource monitor tests
  - **Source**: `bkg-p2p/src/executor/resource.rs` (tests module)
  - **Tests**: `test_can_handle_sufficient_resources`, `test_can_handle_insufficient_ram`, `test_can_handle_no_gpu_required`, `test_available_capacity`
  - **Commit**: `test(p2p): add resource monitor tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.7 Create `runtime.rs` (thin container)
  - **Source**: `bkg-p2p/src/runtime.rs` (full file, refactored to thin container)
  - **Structs**: `Runtime`, `RuntimeStats`
  - **Functions**: `new()`, `attach_swarm_manager()`, `subscribe_to_job_topics()`, `advertise_provider()`, `advertise_tools()`, `advertise_skills()`, `gossip_resource_manifest()`, `gossip_agent_card()`, `handle_gossip_message()`, `execute_task()`, `inference()`, `inference_batched()`, `inference_streaming_print()`, `web_fetch()`, `resource_state()`, `balance()`, `connected_peers_count()`, `set_pricing()`, `stats()`
  - **Note**: Runtime is a thin container. It wires subsystems but does not own their implementation.
  - **Commit**: `feat(p2p): port Runtime as thin container from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 4.8 Add runtime tests
  - **Source**: `bkg-p2p/src/runtime.rs` (tests module)
  - **Tests**: `test_runtime_creation`, `test_runtime_stats`
  - **Commit**: `test(p2p): add runtime tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

---

## M5 — Job Marketplace + Remote Execution

**Flow**:

```text
Request → Network Broadcast → Workers evaluate → Bid → Accept → Resource Lease → Execution → Result → Settlement
```

**Exit Criteria**:

- `Wallet` credits, debits, escrows, stakes with spending limits
- `JobManager` handles full request→bid→accept→execute→settle lifecycle
- `RemoteExecutor` offloads tasks to network peers
- `TaskExecutor` routes tasks locally or remotely
- `TaskRouter` decides execution location based on resources
- **GATE: Two-node integration test passes** (job marketplace over P2P)

**Tasks**:

- [ ] 5.1 Create `wallet/mod.rs`
  - **Source**: `bkg-p2p/src/wallet/mod.rs` (full file)
  - **Structs**: `Wallet`, `WalletState`, `WalletConfig`, `Escrow`, `Transaction`, `BalanceSnapshot`, `EscrowId`, `TransactionId`, `TransactionType`, `EscrowStatus`
  - **Functions**: `new()`, `balance()`, `credit()`, `debit()`, `check_spending_limits()`, `create_escrow()`, `release_escrow()`, `refund_escrow()`, `stake()`, `unstake()`, `transactions()`, `active_escrows()`, `sweep_expired_escrows()`, `sign()`, `verify()`
  - **Constants**: `TOKEN_DECIMALS = 6`, `MICRO_BKG = 1_000_000`, `to_micro()`, `from_micro()`
  - **Commit**: `feat(p2p): port Wallet from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.2 Add wallet tests
  - **Source**: `bkg-p2p/src/wallet/mod.rs` (tests module)
  - **Tests**: `test_credit_and_balance`, `test_debit_insufficient_balance`, `test_escrow_lifecycle`, `test_stake_and_unstake`
  - **Commit**: `test(p2p): add wallet tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.3 Create `job/mod.rs`
  - **Source**: `bkg-p2p/src/job/mod.rs` (full file)
  - **Structs**: `JobManager`, `JobRequest`, `JobBid`, `Job`, `JobResult`, `JobId`, `BidId`, `JobError`, `JobBroadcast`, `JobBroadcastTx`
  - **Functions**: `new()`, `with_resource_monitor()`, `set_broadcast_tx()`, `create_request()`, `receive_bid()`, `get_bids()`, `accept_bid()`, `evaluate_request()`, `can_fulfill()`, `submit_bid()`, `start_job()`, `submit_result()`, `settle_job()`, `active_jobs()`, `completed_jobs()`, `pending_requests()`, `bids_count()`, `describe_job_for_tools()`
  - **Enums**: `ResourceType` (Inference, Gpu, Cpu, Storage, Embedding, ImageGeneration, WebFetch, VectorSearch, WasmTool), `PricingStrategy` (Fixed, Dynamic, Auction), `JobStatus`
  - **Commit**: `feat(p2p): port JobManager from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.4 Add job manager tests
  - **Source**: `bkg-p2p/src/job/mod.rs` (tests module)
  - **Tests**: `test_create_request`, `test_bid_flow`
  - **Commit**: `test(p2p): add job manager tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.5 Create `executor/task.rs`
  - **Source**: `bkg-p2p/src/executor/task.rs` (full file)
  - **Structs**: `InferenceTask`, `WebFetchTask`, `WebSearchTask`, `WasmTask`, `ChatMessage`, `ResourceRequirements`, `InferenceResult`, `WebFetchResult`, `WebSearchResult`, `WasmResult`, `TaskMetrics`
  - **Enums**: `ExecutionTask`, `MessageRole`, `HttpMethod`, `SearchEngine`, `FinishReason`, `ExecutionLocation`, `TaskData`
  - **Functions**: `parse_model_size()`, `estimate_requirements()`
  - **Commit**: `feat(p2p): port task types from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.6 Add task type tests
  - **Source**: `bkg-p2p/src/executor/task.rs` (tests module)
  - **Tests**: `test_parse_model_size`, `test_inference_cloud_minimal_requirements`, `test_inference_unknown_name_minimal_requirements`, `test_inference_task_requirements`, `test_task_type`
  - **Commit**: `test(p2p): add task type tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.7 Create `executor/router.rs`
  - **Source**: `bkg-p2p/src/executor/router.rs` (full file)
  - **Structs**: `TaskRouter`, `RouterConfig`, `PeerFilter`
  - **Enums**: `RoutingDecision`
  - **Functions**: `new()`, `route()`, `route_with_requirements()`, `should_execute_locally()`, `should_offload()`, `build_insufficient_reason()`, `to_job_requirements()`, `satisfied_by()`
  - **Commit**: `feat(p2p): port TaskRouter from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.8 Add router tests
  - **Source**: `bkg-p2p/src/executor/router.rs` (tests module)
  - **Tests**: `test_route_lightweight_task_locally`, `test_job_requirements_satisfied`, `test_job_requirements_not_satisfied_missing_model`
  - **Commit**: `test(p2p): add router tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.9 Create `executor/remote.rs`
  - **Source**: `bkg-p2p/src/executor/remote.rs` (full file)
  - **Structs**: `RemoteExecutor`, `RemoteExecutorConfig`, `JobProvider`
  - **Functions**: `new()`, `execute()`, `task_to_request()`, `broadcast_request()`, `collect_bids()`, `select_best_bid()`, `execute_with_bid()`, `wait_for_result()`, `result_to_task_data()`, `handle_request()`, `handle_bid_accepted()`, `get_pending_request()`, `remove_pending_request()`
  - **Commit**: `feat(p2p): port RemoteExecutor and JobProvider from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.10 Add remote executor test
  - **Source**: `bkg-p2p/src/executor/remote.rs` (tests module)
  - **Tests**: `test_remote_executor_config`
  - **Commit**: `test(p2p): add remote executor test from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.11 Create `executor/mod.rs`
  - **Source**: `bkg-p2p/src/executor/mod.rs` (full file)
  - **Structs**: `TaskExecutor`, `ExecutorConfig`, `ExecutorError`
  - **Functions**: `new()`, `with_job_manager()`, `with_network()`, `with_inference_engine()`, `with_remote_executor()`, `execute()`, `execute_local()`, `execute_remote()`, `execute_inference_local()`, `execute_inference_streaming()`, `execute_inference_streaming_print()`, `execute_web_fetch_local()`, `execute_web_search_local()`, `execute_wasm_local()`, `find_gguf_model()`, `resource_state()`, `has_model()`
  - **Commit**: `feat(p2p): port TaskExecutor from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.12 Add executor tests
  - **Source**: `bkg-p2p/src/executor/mod.rs` (tests module)
  - **Tests**: `test_execute_web_fetch_local`, `test_execute_inference_placeholder`, `test_resource_state`
  - **Commit**: `test(p2p): add executor tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 5.13 Integration test: two-node job marketplace
  - **File**: `tests/p2p_job_marketplace.rs`
  - **Scenario**: Node A creates job request (inference task), Node B receives request, evaluates, submits bid, Node A accepts bid, creates escrow, Node B executes job (local inference), Node B submits result, Node A receives result, settles job, releases escrow
  - **Commit**: `test(p2p): add two-node job marketplace integration test`
  - **QA**: build, test, clippy, fmt

---

## M6 — Web Dashboard Core

**Exit Criteria**:

- Axum server starts on configured address
- `/ws` WebSocket control plane broadcasts status and task events
- `/api/status` returns resource + peer + job counts
- `/api/peers` lists connected peers
- `/api/jobs` lists active/completed jobs
- Embedded chat UI at `/` and `/chat`
- Console dashboard at `/console`

**Tasks**:

- [ ] 6.1 Create `web/mod.rs` (WebState)
  - **Source**: `bkg-p2p/src/web/mod.rs` (full file)
  - **Structs**: `WebState`, `InferenceRequest`, `InferenceResponse`, `JobSubmitRequest`, `JobSubmitResponse`, `WebJobInfo`, `WebTask`, `ChatMessage`, `CommunityPeerEntry`, `P2pNetworkHints`
  - **Functions**: `create_web_state()`, `create_web_state_with_swarm()`, `create_web_state_with_channels()`, `create_web_state_with_inference()`
  - **Commit**: `feat(p2p): port WebState from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 6.2 Create WebSocket handler
  - **Source**: `bkg-p2p/src/web/mod.rs` (ws_handler function)
  - **Functions**: `ws_handler()`, `broadcast_tasks_changed()`, `broadcast_crews_changed()`, `broadcast_flows_changed()`, `broadcast_flow_log()`
  - **Structs**: `AgenticTaskProgressSink`
  - **Commit**: `feat(p2p): port WebSocket control plane from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 6.3 Create core REST endpoints
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_status, api_onboarding, api_network_multiaddrs, api_peers_dial, api_peers_network, api_peers)
  - **Functions**: `api_status()`, `api_onboarding()`, `api_network_multiaddrs()`, `api_peers_dial()`, `api_peers_network()`, `api_peers()`
  - **Commit**: `feat(p2p): port core REST endpoints from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 6.4 Create static UI
  - **Source**: `bkg-p2p/src/web/mod.rs` (assistant_index, console_index, spa_dist_dir)
  - **Files**: `web/chat.html`, `web/dashboard.html` (embedded)
  - **Functions**: `assistant_index()`, `console_index()`, `spa_dist_dir()`
  - **Commit**: `feat(p2p): port static UI from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 6.5 Create server startup
  - **Source**: `bkg-p2p/src/web/mod.rs` (create_router, start_server)
  - **Functions**: `create_router()`, `start_server()`
  - **Commit**: `feat(p2p): port server startup from bkg-p2p`
  - **QA**: build, test, clippy, fmt

---

## M7 — Full API

**Exit Criteria**:

- All 60+ REST endpoints implemented
- OpenAI-compatible API (`/v1/chat/completions`, `/v1/models`, `/v1/embeddings`)
- Agent library CRUD
- Workflow/crew/flow validation and kickoff
- Task management (create, list, detail, stop)
- Skills management (local, network, meta, scan, studio)
- MCP configuration
- Chat with session history
- Provider settings
- Model download/pull
- Channel management
- Wallet balance/transactions
- Vector memory CRUD
- A2A peer discovery
- Metrics endpoint
- HITL approvals
- Swarm visualization (agents, topology, timeline)

**Tasks**:

- [ ] 7.1 Port Tasks & Workflows API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_tasks_router, api_workflows_router, api_crew__, api_flow__)
  - **Commit**: `feat(p2p): port Tasks & Workflows API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.2 Port Agents & Skills API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_agent_library__, api_skills__)
  - **Commit**: `feat(p2p): port Agents & Skills API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.3 Port Jobs & Tools API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_jobs, api_submit_job, api_tools, api_tool_detail, api_tool_execute)
  - **Commit**: `feat(p2p): port Jobs & Tools API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.4 Port Chat & Inference API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_chat, api_chat_stream, api_inference__, api_models__, api_ollama_*)
  - **Commit**: `feat(p2p): port Chat & Inference API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.5 Port Wallet & Vector API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_wallet__, api_vector__)
  - **Commit**: `feat(p2p): port Wallet & Vector API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.6 Port A2A & Metrics API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_agent_card_well_known, api_a2a__, api_metrics, api_approvals__)
  - **Commit**: `feat(p2p): port A2A & Metrics API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.7 Port Swarm & Channels API
  - **Source**: `bkg-p2p/src/web/mod.rs` (api_swarm_*, api_list_channels, api_register_channel, api_remove_channel, api_test_channel)
  - **Commit**: `feat(p2p): port Swarm & Channels API from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 7.8 Port OpenAI compatibility
  - **Source**: `bkg-p2p/src/web/openai.rs` (full file)
  - **Functions**: `chat_completions()`, `list_models()`, `embeddings()`
  - **Commit**: `feat(p2p): port OpenAI compatibility from bkg-p2p`
  - **QA**: build, test, clippy, fmt

---

## M8 — CLI, Config & Plugin Integration

**Exit Criteria**:

- `cave p2p serve [--web] [--ollama] [--share-inference] [--agent]` starts node
- `cave p2p peers list` shows connected peers
- `cave p2p wallet balance` shows token balance
- `cave p2p job submit|status|list` manages jobs
- `cave p2p models list|download|pull` manages models
- `cave p2p skill list|install|search` manages skills
- `cave p2p tool list|info|build` manages tools
- `cave p2p vector create|insert|search` manages vector memory
- `cave p2p network status` shows network health
- `cave p2p doctor` runs diagnostics
- `cave-p2p` loads as static plugin in `cave` binary
- Plugin load integration test passes
- All config sections parsed from TOML

**Tasks**:

- [ ] 8.1 Create `cli.rs`
  - **Source**: `bkg-p2p/src/cli.rs` (full file)
  - **Commands**: `run`, `chat`, `models`, `serve`, `network`, `peers`, `vector`, `skill`, `tool`, `wallet`, `job`, `doctor`
  - **Commit**: `feat(p2p): port CLI from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.2 Create `config.rs`
  - **Source**: `bkg-p2p/src/config.rs` (full file)
  - **Structs**: `Config`, `P2pConfig`, `WebConfig`, `ResourceConfig`, `DatabaseConfig`, `AgentConfig`, `InferenceConfig`, `VectorConfig`, `SafetyConfig`, `ExecutorConfig`, `WasmConfig`, `EconomyConfig`, `ProviderSharingConfig`, `SkillsConfig`, `McpConfig`, `OrchestrationConfig`, `PromptsConfig`
  - **Commit**: `feat(p2p): port Config from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.3 Create `bootstrap.rs`
  - **Source**: `bkg-p2p/src/bootstrap.rs` (full file)
  - **Functions**: `ensure_dirs()`, `identity_path()`, `base_dir()`, `data_dir()`, `models_dir()`, `skills_dir()`, `tools_dir()`, `prompts_dir()`
  - **Commit**: `feat(p2p): port bootstrap from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.4 Create `db.rs`
  - **Source**: `bkg-p2p/src/db.rs` (full file)
  - **Structs**: `Database`
  - **Functions**: `open()`, `store_setting()`, `get_setting()`, `list_run_ids()`, `get_run()`, `list_run_events()`
  - **Commit**: `feat(p2p): port Database from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.5 Create `observability.rs`
  - **Source**: `bkg-p2p/src/observability.rs` (full file)
  - **Structs**: `Metrics`, `MetricsSnapshot`
  - **Functions**: `new()`, `snapshot()`
  - **Tracing spans**: `react_pass`, `tool_execute`, `crew_task`, `job_phase`
  - **Commit**: `feat(p2p): port observability from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.6 Create `swarm/mod.rs`
  - **Source**: `bkg-p2p/src/swarm/mod.rs` (full file)
  - **Structs**: `SwarmManager`, `SwarmAgent`, `AgentProfile`, `AgentStatus`
  - **Enums**: `SwarmEvent`, `ActionType`
  - **Functions**: `new()`, `register_local_agent()`, `handle_network_event()`, `record_job_action()`, `any_local_agent_id()`
  - **Commit**: `feat(p2p): port SwarmManager from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.7 Create `reputation.rs`
  - **Source**: `bkg-p2p/src/reputation.rs` (full file)
  - **Structs**: `PeerReputation`, `TrustLevel`, `ReputationStore`, `ReputationConfig`
  - **Commit**: `feat(p2p): port reputation from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.8 Create `routines.rs`
  - **Source**: `bkg-p2p/src/routines.rs` (full file)
  - **Structs**: `RoutineEngine`, `Routine`, `RoutineConfig`
  - **Commit**: `feat(p2p): port routines from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.9 Create `prompts.rs`
  - **Source**: `bkg-p2p/src/prompts.rs` (full file)
  - **Structs**: `PromptBundle`
  - **Functions**: `load_prompt_bundle()`, `agentic_system_intro()`, `crew_worker_prompt()`
  - **Commit**: `feat(p2p): port prompts from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.10 Create `workspace.rs`
  - **Source**: `bkg-p2p/src/workspace.rs` (full file)
  - **Structs**: `Workspace`, `WorkspaceConfig`
  - **Commit**: `feat(p2p): port workspace from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.11 Create `agent_library.rs`
  - **Source**: `bkg-p2p/src/agent_library.rs` (full file)
  - **Structs**: `AgentLibraryEntry`
  - **Functions**: `builtin_entries()`, `save_user_entries()`
  - **Built-ins**: `deep-researcher`, `code-reviewer`, `creative-writer`, `data-analyst`
  - **Commit**: `feat(p2p): port agent library from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 8.12 Register plugin in `cave` binary
  - **Source**: N/A (integration point)
  - **File**: `crates/cave/src/main.rs`
  - **Commit**: `feat(p2p): register cave-p2p plugin in cave binary`
  - **QA**: build, test, clippy, fmt

- [ ] 8.13 Wire EventBus and ConfigRegistry integration
  - **Commit**: `feat(p2p): wire EventBus and ConfigRegistry integration`
  - **QA**: build, test, clippy, fmt

- [ ] 8.14 Integration test: plugin load
  - **File**: `tests/plugin_load.rs`
  - **Scenario**: Plugin registers itself, config namespace available, EventBus works, CLI command appears
  - **Commit**: `test(p2p): add plugin load integration test`
  - **QA**: build, test, clippy, fmt

---

## M9 — Testing, QA & Documentation

**Exit Criteria**:

- `cargo build --workspace` clean
- `cargo test --workspace` all pass (348+ tests from cave + ~30 from cave-p2p)
- `cargo clippy --all -- -D warnings` clean
- `cargo fmt --all --check` clean
- `cargo doc --workspace --no-deps` 0 warnings
- All features from INVENTORY_PART1/2/3.md verified implemented
- Documentation complete

**Tasks**:

- [ ] 9.1 Port all unit tests from bkg-p2p
  - **Source**: `bkg-p2p/src/*/tests` (all test modules)
  - **Commit**: `test(p2p): port all unit tests from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 9.2 Add integration tests
  - **Tests**: plugin loading, P2P discovery (loopback), job marketplace (local), wallet (escrow lifecycle)
  - **Commit**: `test(p2p): add integration tests`
  - **QA**: build, test, clippy, fmt

- [ ] 9.3 Run full QA gate suite
  - **Commands**:
    ```bash
    cargo build --workspace
    cargo test --workspace
    cargo clippy --all -- -D warnings
    cargo fmt --all --check
    cargo doc --workspace --no-deps
    ```
  - **Commit**: `chore(p2p): run full QA gate suite`
  - **QA**: All commands must pass

- [ ] 9.4 Write documentation
  - **Files**: `docs/p2p/README.md`, `ARCHITECTURE.md`, `PLUGIN_API.md`, `CONFIGURATION.md`, `CLI_REFERENCE.md`, `WEB_API.md`, `P2P_PROTOCOL.md`
  - **Commit**: `docs(p2p): add complete documentation`
  - **QA**: build, test, clippy, fmt

- [ ] 9.5 Cross-check against bkg-p2p source
  - **Source**: `bkg-p2p/src/**/*.rs`
  - **Commit**: `chore(p2p): verify feature parity against bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 9.6 Cross-check against references
  - **Source**: `/home/workspace/work/alpha/bkg-cave/references/` (virtme, ugrd, osbuild)
  - **Commit**: `chore(p2p): verify against references`
  - **QA**: build, test, clippy, fmt

---

## M10 — Deferred Features (v1.0)

**Exit Criteria**:

- Inference engines (GGUF, Ollama, Candle, HF-Hub) ported
- Vector database (vectX) integrated
- WASM sandbox (wasmtime) ported
- 20+ builtin tools ported
- Skills system ported
- MCP client ported
- Agent runtime (ReAct loop) ported
- Crew/Flow orchestration ported
- Safety layer (leak detection, injection defense, HITL) ported
- Messaging channels (Telegram, Webhook, REPL) ported

**Tasks**:

- [ ] 10.1 Port `inference/` module
  - **Commit**: `feat(p2p): port inference engines from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.2 Port `vector/` module
  - **Commit**: `feat(p2p): port vector database from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.3 Port `wasm/` module
  - **Commit**: `feat(p2p): port WASM sandbox from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.4 Port `tools/` module
  - **Commit**: `feat(p2p): port tools from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.5 Port `skills/` module
  - **Commit**: `feat(p2p): port skills from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.6 Port `mcp/` module
  - **Commit**: `feat(p2p): port MCP from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.7 Port `agent/` module
  - **Commit**: `feat(p2p): port agent runtime from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.8 Port `crew/` module
  - **Commit**: `feat(p2p): port crew orchestration from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.9 Port `flow/` module
  - **Commit**: `feat(p2p): port flow orchestration from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.10 Port `safety/` module
  - **Commit**: `feat(p2p): port safety layer from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.11 Port `messaging/` module
  - **Commit**: `feat(p2p): port messaging channels from bkg-p2p`
  - **QA**: build, test, clippy, fmt

- [ ] 10.12 Port `channels/` module
  - **Commit**: `feat(p2p): port channel registry from bkg-p2p`
  - **QA**: build, test, clippy, fmt

---

## Integration Tests

### After M2: Two-Node GossipSub

**File**: `tests/p2p_two_nodes.rs`

**Scenario**:

1. Node A listens on `/ip4/127.0.0.1/tcp/0`
2. Node B dials Node A
3. Both subscribe to `bkg-p2p/test/v1`
4. Node A publishes message
5. Node B receives message
6. Both exchange A2A agent cards
7. Both exchange resource manifests

**Purpose**: Verify that packets actually leave the machine.

### After M5: Two-Node Job Marketplace

**File**: `tests/p2p_job_marketplace.rs`

**Scenario**:

1. Node A creates job request (inference task)
2. Node B receives request, evaluates, submits bid
3. Node A accepts bid, creates escrow
4. Node B executes job (local inference)
5. Node B submits result
6. Node A receives result, settles job, releases escrow

**Purpose**: Verify end-to-end job lifecycle over P2P.

### After M8: Plugin Load

**File**: `tests/plugin_load.rs`

**Scenario**:

1. `cave` binary starts
2. `cave-p2p` plugin registers via `CavePlugin` trait
3. Config namespace `p2p` available
4. EventBus subscription works
5. CLI command `cave p2p` appears

**Purpose**: Verify plugin integration with cave-core.

---

## Success Criteria

1. **Feature Parity**: 100% of MUST PORT features from INVENTORY_PART1/2/3.md implemented
2. **Test Coverage**: All unit tests pass; integration tests for plugin loading, P2P discovery, job marketplace
3. **QA Gates**: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, `cargo doc` all clean after EVERY task
4. **Documentation**: Complete docs for architecture, plugin API, configuration, CLI, web API, P2P protocol
5. **Plugin Integration**: cave-p2p loads as a static plugin in the `cave` binary
6. **Zero Regressions**: All existing cave-core, cave-surge, cave-kindle, cave-forge tests still pass
7. **Architecture**: Runtime is a thin container; subsystems communicate via EventBus
8. **Traceability**: Every task references its bkg-p2p source file and functions
9. **Commit Discipline**: One task = one commit; small, reviewable, rollback-friendly
10. **Two-Node Tests**: Real P2P communication verified (not just compilation)
11. **Plugin Load Test**: cave-p2p integrates cleanly with cave-core plugin system

---

## Commit Convention

```
feat(p2p): port <module> from bkg-p2p
test(p2p): add <module> tests from bkg-p2p
chore(p2p): <description>
docs(p2p): <description>
```

**Examples**:

- `feat(p2p): port NodeIdentity from bkg-p2p`
- `test(p2p): add identity tests from bkg-p2p`
- `feat(p2p): port Network controller from bkg-p2p`
- `chore(p2p): run full QA gate suite`
