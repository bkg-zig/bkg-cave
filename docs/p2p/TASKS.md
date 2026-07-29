# cave-p2p Implementation Tasks

> **Granular task breakdown** for porting bkg-p2p v0.3.0 as a cave-p2p plugin.
> **Status**: Not started — awaiting approval of PLAN.md and INVENTORY_PART1/2/3.md.

---

## Legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Completed
- `[!]` Blocker

---

## PHASE 1: Foundation (Week 1-2)

### Task 1.1: Crate Structure

- [ ] 1.1.1 Create `crates/cave-p2p/Cargo.toml` with workspace dependencies
- [ ] 1.1.2 Create `crates/cave-p2p/src/lib.rs` with module declarations
- [ ] 1.1.3 Create `crates/cave-p2p/src/error.rs` with `thiserror` error types
- [ ] 1.1.4 Add `cave-p2p` to workspace `Cargo.toml` members
- [ ] 1.1.5 Verify `cargo build -p cave-p2p` compiles (empty crate)

### Task 1.2: Identity Module

- [ ] 1.2.1 Create `crates/cave-p2p/src/identity/mod.rs`
- [ ] 1.2.2 Port `NodeIdentity` struct (Ed25519 signing key, PeerId)
- [ ] 1.2.3 Port `generate()` method (OsRng)
- [ ] 1.2.4 Port `from_bytes(secret: &[u8; 32])` method
- [ ] 1.2.5 Port `load(path: &Path)` / `save(path: &Path)` methods
- [ ] 1.2.6 Port `peer_id()` / `public_key()` / `public_key_bytes()` getters
- [ ] 1.2.7 Port `sign(message: &[u8]) -> Signature` method
- [ ] 1.2.8 Port `verify(message: &[u8], signature: &Signature) -> bool` method
- [ ] 1.2.9 Port `to_libp2p_keypair() -> Libp2pKeypair` method
- [ ] 1.2.10 Port `derive_peer_id(signing_key) -> PeerId` helper
- [ ] 1.2.11 Port `IdentityInfo` struct (peer_id, public_key)
- [ ] 1.2.12 Add tests: `test_generate_and_sign`, `test_save_and_load`, `test_peer_id_derivation`

### Task 1.3: P2P Behaviour

- [ ] 1.3.1 Create `crates/cave-p2p/src/p2p/behaviour.rs`
- [ ] 1.3.2 Port `BkgPeerdBehaviour` struct with `#[derive(NetworkBehaviour)]`
- [ ] 1.3.3 Port `a2a_rpc` field (request_response::json::Behaviour)
- [ ] 1.3.4 Port `kademlia` field (kad::Behaviour with MemoryStore)
- [ ] 1.3.5 Port `mdns` field (mdns::tokio::Behaviour)
- [ ] 1.3.6 Port `gossipsub` field (gossipsub::Behaviour with Strict validation)
- [ ] 1.3.7 Port `identify` field (identify::Behaviour)
- [ ] 1.3.8 Port `build_swarm()` function
- [ ] 1.3.9 Configure A2A protocol: `/bkg-p2p/a2a-rpc/1.0.0`
- [ ] 1.3.10 Configure Kademlia protocol: `/bkg-p2p/kad/1.0.0`
- [ ] 1.3.11 Configure Identify protocol: `/bkg-p2p/1.0.0`
- [ ] 1.3.12 Configure TCP transport + Noise + Yamux
- [ ] 1.3.13 Configure QUIC transport (optional, in Cargo.toml)
- [ ] 1.3.14 Set idle connection timeout: 60s
- [ ] 1.3.15 Add test: `test_build_swarm`

### Task 1.4: Network Controller

- [ ] 1.4.1 Create `crates/cave-p2p/src/p2p/mod.rs`
- [ ] 1.4.2 Port `Network` struct (swarm, event_tx, local_peer_id, connected_peers, config, a2a_state, listen_addresses)
- [ ] 1.4.3 Port `Network::new(identity, config, a2a_state)` constructor
- [ ] 1.4.4 Port `start()` method (listen + dial bootstrap)
- [ ] 1.4.5 Port `run(shutdown_rx)` event loop
- [ ] 1.4.6 Port `process_swarm_event()` method
- [ ] 1.4.7 Port `subscribe(topic)` / `publish(topic, data)` methods
- [ ] 1.4.8 Port `dial(addr)` method
- [ ] 1.4.9 Port `event_receiver()` / `connected_peers()` / `listen_addresses()` / `a2a_state()` getters
- [ ] 1.4.10 Port event handlers: NewListenAddr, ConnectionEstablished/Closed, Mdns, Kademlia, Gossipsub, A2aRpc, Identify

### Task 1.5: Network Events

- [ ] 1.5.1 Create `crates/cave-p2p/src/p2p/events.rs`
- [ ] 1.5.2 Port `NetworkEvent` enum (PeerDiscovered, PeerConnected, PeerDisconnected, GossipMessage, RequestReceived, A2aRpcInbound, ResourceAdvertised)
- [ ] 1.5.3 Port `Display` implementation for NetworkEvent

### Task 1.6: Resource Manifest

- [ ] 1.6.1 Create `crates/cave-p2p/src/p2p/resource.rs`
- [ ] 1.6.2 Port `ResourceManifest` struct (peer_id, timestamp, signature, resources, capabilities, supported_models, uptime_hours)
- [ ] 1.6.3 Port `new()` / `signing_bytes()` / `sign()` / `verify()` methods
- [ ] 1.6.4 Port `Resources` struct (cpu_cores, cpu_available_mhz, gpu, storage_available_bytes, bandwidth_mbps, ram_available_mb)
- [ ] 1.6.5 Port `GpuInfo` struct (vendor, vram_mb, compute_capability, model_name)
- [ ] 1.6.6 Port `GpuVendor` enum (Nvidia, Amd, Apple, Intel, Other)
- [ ] 1.6.7 Port `Capability` enum (WasmSandbox, Inference, Storage, Relay, WebProxy, LlmProvider)
- [ ] 1.6.8 Add tests: `test_resource_manifest_creation`, `test_manifest_signing`

### Task 1.7: Integration

- [ ] 1.7.1 Create `crates/cave-p2p/src/plugin.rs` with `CavePlugin` trait implementation
- [ ] 1.7.2 Register cave-p2p plugin in `cave` binary
- [ ] 1.7.3 Add `cave p2p serve` CLI command
- [ ] 1.7.4 Verify plugin loads and network starts
- [ ] 1.7.5 Run `cargo build --workspace` and `cargo test --workspace`

**Phase 1 Deliverable**: Basic P2P node that can discover peers and exchange resource manifests.

---

## PHASE 2: Resource Sharing & Provider Discovery (Week 3-4)

### Task 2.1: Provider Manifest

- [ ] 2.1.1 Create `crates/cave-p2p/src/p2p/provider.rs`
- [ ] 2.1.2 Port `PROVIDER_TOPIC` constant
- [ ] 2.1.3 Port `ProviderManifest` struct (peer_id, models, rate_limits, timestamp, signature)
- [ ] 2.1.4 Port `new()` / `is_expired()` / `signing_bytes()` / `sign()` / `has_model()` / `find_model()` methods
- [ ] 2.1.5 Port `ModelOffering` struct (model_name, context_size, price_per_1k_tokens, max_tokens_per_request, quantization, backend)
- [ ] 2.1.6 Port `ProviderBackend` enum (Ollama, Gguf, OpenAiCompatible)
- [ ] 2.1.7 Port `ProviderRateLimits` struct (max_requests_per_hour, max_tokens_per_day, max_concurrent_requests)
- [ ] 2.1.8 Port `ProviderSharingConfig` struct
- [ ] 2.1.9 Add tests: `test_provider_manifest_creation`, `test_provider_manifest_serialization`

### Task 2.2: Provider Tracker

- [ ] 2.2.1 Create `crates/cave-p2p/src/p2p/provider_tracker.rs`
- [ ] 2.2.2 Port `ProviderTracker` struct
- [ ] 2.2.3 Port `update_provider()` method
- [ ] 2.2.4 Port `get_providers()` / `find_model()` methods

### Task 2.3: Tool Manifest

- [ ] 2.3.1 Create `crates/cave-p2p/src/p2p/tool_manifest.rs`
- [ ] 2.3.2 Port `TOOL_MANIFEST_TOPIC` constant
- [ ] 2.3.3 Port `ToolManifest` struct (name, description, category, parameters, local_only, tags, peer_id)
- [ ] 2.3.4 Port `ToolManifestAdvert` struct (peer_id, tools)
- [ ] 2.3.5 Port `ToolManifestAdvert::new()` / `add_tool()` methods

### Task 2.4: Resource Monitor

- [ ] 2.4.1 Create `crates/cave-p2p/src/executor/resource.rs`
- [ ] 2.4.2 Port `ResourceMonitor` struct (state, config)
- [ ] 2.4.3 Port `new(config)` / `with_defaults()` constructors
- [ ] 2.4.4 Port `start_background_updates()` (1s interval, sysinfo refresh)
- [ ] 2.4.5 Port `current_state()` / `can_handle()` / `available_capacity()` methods
- [ ] 2.4.6 Port `register_loaded_model()` / `unregister_model()` methods
- [ ] 2.4.7 Port `task_started()` / `task_completed()` methods
- [ ] 2.4.8 Port `ResourceState` struct (all fields)
- [ ] 2.4.9 Port `MonitorConfig` struct
- [ ] 2.4.10 Port `TaskType` enum (Inference, Wasm, Web)
- [ ] 2.4.11 Port GPU probing (Linux: nvidia-smi, macOS: ioreg)
- [ ] 2.4.12 Add tests: `test_can_handle_sufficient_resources`, `test_can_handle_insufficient_ram`, `test_can_handle_no_gpu_required`, `test_available_capacity`

### Task 2.5: Runtime Integration

- [ ] 2.5.1 Create `crates/cave-p2p/src/runtime.rs`
- [ ] 2.5.2 Port `Runtime` struct (all 20+ fields)
- [ ] 2.5.3 Port `Runtime::new()` initialization sequence
- [ ] 2.5.4 Port `attach_swarm_manager()` method
- [ ] 2.5.5 Port `subscribe_to_job_topics()` (all GossipSub topics)
- [ ] 2.5.6 Port `advertise_provider()` / `advertise_tools()` / `advertise_skills()` methods
- [ ] 2.5.7 Port `gossip_resource_manifest()` / `gossip_agent_card()` methods
- [ ] 2.5.8 Port `handle_gossip_message()` (all topic routing)
- [ ] 2.5.9 Port `execute_task()` / `inference()` / `inference_batched()` / `inference_streaming_print()` methods
- [ ] 2.5.10 Port `web_fetch()` / `resource_state()` / `balance()` / `connected_peers_count()` methods
- [ ] 2.5.11 Port `set_pricing()` / `stats()` methods
- [ ] 2.5.12 Add tests: `test_runtime_creation`, `test_runtime_stats`

**Phase 2 Deliverable**: Node that advertises resources, discovers LLM providers, and tracks tools.

---

## PHASE 3: Job Marketplace (Week 5-6)

### Task 3.1: Wallet

- [ ] 3.1.1 Create `crates/cave-p2p/src/wallet/mod.rs`
- [ ] 3.1.2 Port `TOKEN_DECIMALS` / `MICRO_BKG` constants
- [ ] 3.1.3 Port `to_micro()` / `from_micro()` helpers
- [ ] 3.1.4 Port `WalletError` enum
- [ ] 3.1.5 Port `WalletConfig` struct (max_spend_per_tx, per_hour, per_day, reserve_balance)
- [ ] 3.1.6 Port `Wallet` struct (identity, state, config, database)
- [ ] 3.1.7 Port `WalletState` struct (available, in_escrow, staked, escrows, transactions, hourly_spend, daily_spend)
- [ ] 3.1.8 Port `new()` / `balance()` / `credit()` / `debit()` methods
- [ ] 3.1.9 Port `check_spending_limits()` method
- [ ] 3.1.10 Port `create_escrow()` / `release_escrow()` / `refund_escrow()` methods
- [ ] 3.1.11 Port `stake()` / `unstake()` methods
- [ ] 3.1.12 Port `transactions()` / `active_escrows()` / `sweep_expired_escrows()` methods
- [ ] 3.1.13 Port `sign()` / `verify()` methods
- [ ] 3.1.14 Port `Escrow` struct (id, amount, recipient, job_id, created_at, expires_at, status)
- [ ] 3.1.15 Port `Transaction` struct (id, tx_type, amount, timestamp, reason, counterparty, job_id)
- [ ] 3.1.16 Add tests: `test_credit_and_balance`, `test_debit_insufficient_balance`, `test_escrow_lifecycle`, `test_stake_and_unstake`

### Task 3.2: Job Manager

- [ ] 3.2.1 Create `crates/cave-p2p/src/job/mod.rs`
- [ ] 3.2.2 Port `JobError` enum
- [ ] 3.2.3 Port `JobBroadcast` struct / `JobBroadcastTx` type alias
- [ ] 3.2.4 Port `JobManager` struct (all fields)
- [ ] 3.2.5 Port `new()` / `with_resource_monitor()` / `set_broadcast_tx()` methods
- [ ] 3.2.6 Port `create_request()` / `receive_bid()` / `get_bids()` methods
- [ ] 3.2.7 Port `accept_bid()` method (create escrow, broadcast acceptance)
- [ ] 3.2.8 Port `evaluate_request()` / `can_fulfill()` / `submit_bid()` methods
- [ ] 3.2.9 Port `start_job()` / `submit_result()` / `settle_job()` methods
- [ ] 3.2.10 Port `active_jobs()` / `completed_jobs()` / `pending_requests()` / `bids_count()` methods
- [ ] 3.2.11 Port `describe_job_for_tools()` method
- [ ] 3.2.12 Port `ResourceType` enum (Inference, Gpu, Cpu, Storage, Embedding, ImageGeneration, WebFetch, VectorSearch, WasmTool)
- [ ] 3.2.13 Port `PricingStrategy` enum (Fixed, Dynamic, Auction)
- [ ] 3.2.14 Port `ResourcePricing` struct
- [ ] 3.2.15 Add tests: `test_create_request`, `test_bid_flow`

### Task 3.3: Remote Executor

- [ ] 3.3.1 Create `crates/cave-p2p/src/executor/remote.rs`
- [ ] 3.3.2 Port `RemoteExecutor` struct (job_manager, network, local_peer_id, config)
- [ ] 3.3.3 Port `new()` constructor
- [ ] 3.3.4 Port `execute()` method (broadcast → collect bids → select best → execute → settle)
- [ ] 3.3.5 Port `task_to_request()` / `broadcast_request()` / `collect_bids()` / `select_best_bid()` helpers
- [ ] 3.3.6 Port `execute_with_bid()` / `wait_for_result()` / `result_to_task_data()` helpers
- [ ] 3.3.7 Port `RemoteExecutorConfig` struct
- [ ] 3.3.8 Port `JobProvider` struct (job_manager, network, local_peer_id, pending_requests)
- [ ] 3.3.9 Port `handle_request()` / `handle_bid_accepted()` / `get_pending_request()` / `remove_pending_request()` methods
- [ ] 3.3.10 Add test: `test_remote_executor_config`

### Task 3.4: Task Executor

- [ ] 3.4.1 Create `crates/cave-p2p/src/executor/mod.rs`
- [ ] 3.4.2 Port `TaskExecutor` struct (all fields)
- [ ] 3.4.3 Port `new()` constructor
- [ ] 3.4.4 Port builder methods: `with_job_manager()` / `with_network()` / `with_inference_engine()` / `with_remote_executor()`
- [ ] 3.4.5 Port `execute()` method (route local vs remote)
- [ ] 3.4.6 Port `execute_local()` / `execute_remote()` methods
- [ ] 3.4.7 Port `execute_inference_local()` / `execute_inference_streaming()` / `execute_inference_streaming_print()` methods
- [ ] 3.4.8 Port `execute_web_fetch_local()` / `execute_web_search_local()` / `execute_wasm_local()` methods
- [ ] 3.4.9 Port `find_gguf_model()` helper (fuzzy matching)
- [ ] 3.4.10 Port `resource_state()` / `has_model()` getters
- [ ] 3.4.11 Port `ExecutorConfig` / `ExecutorError` types
- [ ] 3.4.12 Add tests: `test_execute_web_fetch_local`, `test_execute_inference_placeholder`, `test_resource_state`

### Task 3.5: Task Router

- [ ] 3.5.1 Create `crates/cave-p2p/src/executor/router.rs`
- [ ] 3.5.2 Port `TaskRouter` struct (resource_monitor, config)
- [ ] 3.5.3 Port `new()` constructor
- [ ] 3.5.4 Port `route()` / `route_with_requirements()` methods
- [ ] 3.5.5 Port `should_execute_locally()` / `should_offload()` decision logic
- [ ] 3.5.6 Port `build_insufficient_reason()` / `to_job_requirements()` helpers
- [ ] 3.5.7 Port `RouterConfig` struct (all defaults)
- [ ] 3.5.8 Port `RoutingDecision` enum (ExecuteLocally, OffloadToNetwork, InsufficientResources)
- [ ] 3.5.9 Port `PeerFilter` struct (min_ram_mb, min_vram_mb, min_cpu_cores, required_model, required_capabilities)
- [ ] 3.5.10 Port `PeerFilter::satisfied_by()` method
- [ ] 3.5.11 Add tests: `test_route_lightweight_task_locally`, `test_job_requirements_satisfied`, `test_job_requirements_not_satisfied_missing_model`

### Task 3.6: Task Types

- [ ] 3.6.1 Create `crates/cave-p2p/src/executor/task.rs`
- [ ] 3.6.2 Port `TaskId` type alias
- [ ] 3.6.3 Port `ExecutionTask` enum (Inference, WebFetch, WebSearch, WasmExecution)
- [ ] 3.6.4 Port `task_type()` / `estimate_requirements()` methods
- [ ] 3.6.5 Port `InferenceTask` struct (model, messages, max_tokens, temperature, stop_sequences, stream)
- [ ] 3.6.6 Port `InferenceTask::new()` / `with_max_tokens()` / `with_temperature()` / `with_streaming()` builders
- [ ] 3.6.7 Port `InferenceTask::estimate_requirements()` (model size parsing, VRAM estimation)
- [ ] 3.6.8 Port `ChatMessage` struct (role, content) and constructors
- [ ] 3.6.9 Port `MessageRole` enum (System, User, Assistant)
- [ ] 3.6.10 Port `WebFetchTask` struct (url, method, headers, body, timeout_secs)
- [ ] 3.6.11 Port `WebFetchTask::get()` / `post()` / `with_header()` constructors
- [ ] 3.6.12 Port `HttpMethod` enum (Get, Post, Put, Delete, Patch, Head)
- [ ] 3.6.13 Port `WebSearchTask` struct (query, max_results, engine)
- [ ] 3.6.14 Port `SearchEngine` enum (Default, Google, Bing, DuckDuckGo)
- [ ] 3.6.15 Port `WasmTask` struct (tool_hash, function, params, capabilities, timeout_secs, max_fuel)
- [ ] 3.6.16 Port `WasmCapabilities` struct (network_access, filesystem_read, filesystem_write, allowed_hosts)
- [ ] 3.6.17 Port `ResourceRequirements` struct (ram_mb, vram_mb, cpu_cores, estimated_duration)
- [ ] 3.6.18 Port `ResourceRequirements::minimal()` helper
- [ ] 3.6.19 Port `ExecutionLocation` enum (Local, Remote)
- [ ] 3.6.20 Port `TaskResult` / `TaskData` / `InferenceResult` / `WebFetchResult` / `WebSearchResult` / `WasmResult` structs
- [ ] 3.6.21 Port `FinishReason` enum (Stop, Length, ContentFilter)
- [ ] 3.6.22 Port `TaskMetrics` struct (ttfb_ms, total_time_ms, queue_time_ms)
- [ ] 3.6.23 Port `parse_model_size()` helper
- [ ] 3.6.24 Add tests: `test_parse_model_size`, `test_inference_cloud_minimal_requirements`, `test_inference_unknown_name_minimal_requirements`, `test_inference_task_requirements`, `test_task_type`

**Phase 3 Deliverable**: Full job marketplace with token payments and distributed task execution.

---

## PHASE 4: Web Dashboard (Week 7-8)

### Task 4.1: WebState

- [ ] 4.1.1 Create `crates/cave-p2p/src/web/mod.rs`
- [ ] 4.1.2 Port `WebState` struct (all 50+ fields)
- [ ] 4.1.3 Port `InferenceRequest` / `InferenceResponse` structs
- [ ] 4.1.4 Port `JobSubmitRequest` / `JobSubmitResponse` structs
- [ ] 4.1.5 Port `WebJobInfo` / `WebTask` / `ChatMessage` structs
- [ ] 4.1.6 Port `CommunityPeerEntry` / `P2pNetworkHints` structs
- [ ] 4.1.7 Port `create_web_state()` / `create_web_state_with_swarm()` constructors
- [ ] 4.1.8 Port `CrewKickoffJob` / `FlowKickoffJob` structs

### Task 4.2: WebSocket Control Plane

- [ ] 4.2.1 Port `ws_handler()` function
- [ ] 4.2.2 Port control plane message types (status, tasks_changed, task_stream_delta, task_tool_call, agents_library_changed, crews_changed, flows_changed, flow_log)
- [ ] 4.2.3 Port `broadcast_tasks_changed()` / `broadcast_crews_changed()` / `broadcast_flows_changed()` helpers
- [ ] 4.2.4 Port `broadcast_flow_log()` helper
- [ ] 4.2.5 Port `AgenticTaskProgressSink` struct (append_log, set_react_pass, record_tool_step, set_tokens)

### Task 4.3: REST API - Core

- [ ] 4.3.1 Port `api_router()` with all 60+ routes
- [ ] 4.3.2 Port `create_router()` (CORS, NormalizePath, state)
- [ ] 4.3.3 Port `api_status()` endpoint
- [ ] 4.3.4 Port `api_onboarding()` endpoint
- [ ] 4.3.5 Port `api_network_multiaddrs()` / `api_peers_dial()` / `api_peers_network()` endpoints

### Task 4.4: REST API - Tasks & Workflows

- [ ] 4.4.1 Port `api_tasks_router()` (create, list, detail, stop)
- [ ] 4.4.2 Port `api_workflows_router()` (validate, kickoff, runs, stream, stop)
- [ ] 4.4.3 Port legacy aliases (`/api/crews/*`, `/api/flows/*`)

### Task 4.5: REST API - Agents & Skills

- [ ] 4.5.1 Port `api_agent_library_list()` / `api_agent_library_post()` / `api_agent_library_delete()` endpoints
- [ ] 4.5.2 Port `api_skills_local()` / `api_skills_network()` / `api_skills_meta()` / `api_skills_scan()` / `api_skills_templates()` endpoints
- [ ] 4.5.3 Port `api_skills_toggle()` / `api_skills_studio_ai()` / `api_skills_studio_list()` / `api_skills_studio_get()` / `api_skills_studio_put()` endpoints

### Task 4.6: REST API - Jobs & Tools

- [ ] 4.6.1 Port `api_jobs()` / `api_submit_job()` endpoints
- [ ] 4.6.2 Port `api_tools()` / `api_tool_detail()` / `api_tool_execute()` endpoints

### Task 4.7: REST API - Chat & Inference

- [ ] 4.7.1 Port `api_chat()` / `api_chat_stream()` endpoints
- [ ] 4.7.2 Port `api_inference_settings_get()` / `api_inference_settings_put()` endpoints
- [ ] 4.7.3 Port `api_models_download()` / `api_ollama_models()` / `api_ollama_pull()` endpoints

### Task 4.8: REST API - Wallet & Vector

- [ ] 4.8.1 Port `api_wallet_balance()` / `api_wallet_transactions()` endpoints
- [ ] 4.8.2 Port `api_vector_list_collections()` / `api_vector_create_collection()` / `api_vector_delete_collection()` / `api_vector_search()` / `api_vector_insert()` endpoints

### Task 4.9: REST API - A2A & Metrics

- [ ] 4.9.1 Port `api_agent_card_well_known()` / `api_a2a_jsonrpc()` / `api_a2a_peers()` endpoints
- [ ] 4.9.2 Port `api_metrics()` endpoint (Prometheus-style)
- [ ] 4.9.3 Port `api_approvals_list()` / `api_approvals_approve()` / `api_approvals_reject()` endpoints

### Task 4.10: REST API - Swarm & Channels

- [ ] 4.10.1 Port `api_swarm_agents()` / `api_swarm_topology()` / `api_swarm_timeline()` endpoints
- [ ] 4.10.2 Port `api_list_channels()` / `api_register_channel()` / `api_remove_channel()` / `api_test_channel()` endpoints

### Task 4.11: OpenAI Compatibility

- [ ] 4.11.1 Create `crates/cave-p2p/src/web/openai.rs`
- [ ] 4.11.2 Port `chat_completions()` (SSE streaming)
- [ ] 4.11.3 Port `list_models()` / `embeddings()` endpoints

### Task 4.12: Static UI

- [ ] 4.12.1 Port `assistant_index()` (chat.html embedded)
- [ ] 4.12.2 Port `console_index()` (dashboard.html embedded)
- [ ] 4.12.3 Port SPA fallback (`web/dist/`)
- [ ] 4.12.4 Port `spa_dist_dir()` logic

### Task 4.13: Server Startup

- [ ] 4.13.1 Port `start_server()` function
- [ ] 4.13.2 Port `P2pNetworkHints` / `default_community_peer_directory()`
- [ ] 4.13.3 Verify server starts on `cave p2p serve --web 127.0.0.1:8080`

**Phase 4 Deliverable**: Full web dashboard with real-time updates, REST API, and OpenAI compatibility.

---

## PHASE 5: CLI & Configuration (Week 9)

### Task 5.1: CLI

- [ ] 5.1.1 Create `crates/cave-p2p/src/cli.rs`
- [ ] 5.1.2 Port `clap` CLI structure (run, chat, models, serve, network, peers, vector, skill, tool, wallet, job, doctor)
- [ ] 5.1.3 Port `run` subcommand (interactive chat, single query)
- [ ] 5.1.4 Port `chat` subcommand (slash commands)
- [ ] 5.1.5 Port `models` subcommand (list, download, pull)
- [ ] 5.1.6 Port `serve` subcommand (--web, --ollama, --share-inference, --agent)
- [ ] 5.1.7 Port `network` subcommand (status)
- [ ] 5.1.8 Port `peers` subcommand (list)
- [ ] 5.1.9 Port `vector` subcommand (create, insert, search)
- [ ] 5.1.10 Port `skill` subcommand (list, install, search)
- [ ] 5.1.11 Port `tool` subcommand (list, info, build)
- [ ] 5.1.12 Port `wallet` subcommand (balance, send, history)
- [ ] 5.1.13 Port `job` subcommand (submit, status, list)
- [ ] 5.1.14 Port `doctor` subcommand (diagnostics)

### Task 5.2: Configuration

- [ ] 5.2.1 Create `crates/cave-p2p/src/config.rs`
- [ ] 5.2.2 Port `Config` struct (all sections)
- [ ] 5.2.3 Port `P2pConfig` / `WebConfig` / `ResourceConfig` / `DatabaseConfig`
- [ ] 5.2.4 Port `InferenceConfig` / `VectorConfig` / `SafetyConfig` / `ExecutorConfig`
- [ ] 5.2.5 Port `WasmConfig` / `EconomyConfig` / `ProviderSharingConfig`
- [ ] 5.2.6 Port `SkillsConfig` / `McpConfig` / `OrchestrationConfig` / `PromptsConfig`
- [ ] 5.2.7 Port TOML parsing and defaults
- [ ] 5.2.8 Port environment variable support

### Task 5.3: Bootstrap

- [ ] 5.3.1 Create `crates/cave-p2p/src/bootstrap.rs`
- [ ] 5.3.2 Port `ensure_dirs()` / `identity_path()` / `base_dir()` functions
- [ ] 5.3.3 Port all directory getters

### Task 5.4: Database

- [ ] 5.4.1 Create `crates/cave-p2p/src/db.rs`
- [ ] 5.4.2 Port `Database` struct (redb-backed)
- [ ] 5.4.3 Port `open()` / `store_setting()` / `get_setting()` / `list_run_ids()` / `get_run()` / `list_run_events()` methods

### Task 5.5: Observability

- [ ] 5.5.1 Create `crates/cave-p2p/src/observability.rs`
- [ ] 5.5.2 Port `Metrics` struct (counters, gauges, histograms)
- [ ] 5.5.3 Port `MetricsSnapshot` struct
- [ ] 5.5.4 Port tracing spans (react_pass, tool_execute, crew_task, job_phase)
- [ ] 5.5.5 Port `tracing` + `tracing-subscriber` setup

### Task 5.6: Swarm Manager

- [ ] 5.6.1 Create `crates/cave-p2p/src/swarm/mod.rs`
- [ ] 5.6.2 Port `SwarmManager` struct (event_buffer, action_history, agents)
- [ ] 5.6.3 Port `SwarmAgent` / `AgentProfile` / `AgentStatus` structs
- [ ] 5.6.4 Port `SwarmEvent` / `ActionType` enums
- [ ] 5.6.5 Port `register_local_agent()` / `handle_network_event()` / `record_job_action()` methods

### Task 5.7: A2A Protocol

- [ ] 5.7.1 Create `crates/cave-p2p/src/a2a/mod.rs`
- [ ] 5.7.2 Port `AgentCard` / `AgentCapabilities` structs
- [ ] 5.7.3 Port `A2aState` struct (peer_cards, task_store)
- [ ] 5.7.4 Port `upsert_peer_card()` / `list_peer_cards()` / `handle_jsonrpc()` methods
- [ ] 5.7.5 Port `AgentCardAnnouncement` / `A2A_GOSSIP_TOPIC`
- [ ] 5.7.6 Port JSON-RPC envelope handling

### Task 5.8: Reputation

- [ ] 5.8.1 Create `crates/cave-p2p/src/reputation.rs`
- [ ] 5.8.2 Port `PeerReputation` / `TrustLevel` / `ReputationStore` / `ReputationConfig`

### Task 5.9: Routines

- [ ] 5.9.1 Create `crates/cave-p2p/src/routines.rs`
- [ ] 5.9.2 Port `RoutineEngine` / `Routine` / `RoutineConfig`
- [ ] 5.9.3 Port cron-based heartbeat

### Task 5.10: Prompts & Workspace

- [ ] 5.10.1 Create `crates/cave-p2p/src/prompts.rs`
- [ ] 5.10.2 Port `PromptBundle` (embedded defaults + overlay)
- [ ] 5.10.3 Create `crates/cave-p2p/src/workspace.rs`
- [ ] 5.10.4 Port `Workspace` / `WorkspaceConfig`

### Task 5.11: Agent Library

- [ ] 5.11.1 Create `crates/cave-p2p/src/agent_library.rs`
- [ ] 5.11.2 Port `AgentLibraryEntry` struct
- [ ] 5.11.3 Port built-in entries (deep-researcher, code-reviewer, creative-writer, data-analyst)
- [ ] 5.11.4 Port `save_user_entries()` / `builtin_entries()` functions

### Task 5.12: Plugin Integration

- [ ] 5.12.1 Implement `CavePlugin` trait for cave-p2p
- [ ] 5.12.2 Wire `PluginManager::register_static(cave_p2p_plugin)`
- [ ] 5.12.3 Add `cave p2p` CLI group
- [ ] 5.12.4 Integrate with `EventBus` for P2P events
- [ ] 5.12.5 Integrate with `ConfigRegistry` for P2P config

**Phase 5 Deliverable**: Complete CLI, configuration, and plugin integration.

---

## PHASE 6: Testing & QA (Week 10)

### Task 6.1: Unit Tests

- [ ] 6.1.1 Port all ~30 unit tests from bkg-p2p
- [ ] 6.1.2 Add integration tests for plugin loading
- [ ] 6.1.3 Add integration tests for P2P discovery (loopback)
- [ ] 6.1.4 Add integration tests for job marketplace (local)
- [ ] 6.1.5 Add integration tests for wallet (escrow lifecycle)

### Task 6.2: QA Gates

- [ ] 6.2.1 Run `cargo build --workspace` (must be clean)
- [ ] 6.2.2 Run `cargo test --workspace` (all must pass)
- [ ] 6.2.3 Run `cargo clippy --all -- -D warnings` (must be clean)
- [ ] 6.2.4 Run `cargo fmt --all --check` (must be clean)
- [ ] 6.2.5 Run `cargo doc --workspace --no-deps` (0 warnings)

### Task 6.3: Documentation

- [ ] 6.3.1 Write `docs/p2p/README.md` (plugin overview)
- [ ] 6.3.2 Write `docs/p2p/ARCHITECTURE.md` (system design)
- [ ] 6.3.3 Write `docs/p2p/PLUGIN_API.md` (plugin interface)
- [ ] 6.3.4 Write `docs/p2p/CONFIGURATION.md` (all config keys)
- [ ] 6.3.5 Write `docs/p2p/CLI_REFERENCE.md` (all commands)
- [ ] 6.3.6 Write `docs/p2p/WEB_API.md` (all endpoints)
- [ ] 6.3.7 Write `docs/p2p/P2P_PROTOCOL.md` (GossipSub topics, A2A protocol)

### Task 6.4: Verification

- [ ] 6.4.1 Verify all features from INVENTORY_PART1.md are implemented
- [ ] 6.4.2 Verify all features from INVENTORY_PART2.md are implemented
- [ ] 6.4.3 Verify all features from INVENTORY_PART3.md are implemented
- [ ] 6.4.4 Cross-check with bkg-p2p source code
- [ ] 6.4.5 Verify no features from references/ are missed

**Phase 6 Deliverable**: Production-ready cave-p2p plugin with full test coverage and documentation.

---

## PHASE 7: Deferred Features (v1.0)

### Task 7.1: Inference Engines

- [ ] 7.1.1 GGUF engine (llama-cpp-2)
- [ ] 7.1.2 Ollama integration
- [ ] 7.1.3 Candle backend
- [ ] 7.1.4 HF-Hub model downloads
- [ ] 7.1.5 Remote API (OpenAI-compatible)

### Task 7.2: Vector Database

- [ ] 7.2.1 vectX integration (HNSW + BM25)
- [ ] 7.2.2 Vector store CRUD
- [ ] 7.2.3 Semantic search

### Task 7.3: WASM Sandbox

- [ ] 7.3.1 wasmtime component model
- [ ] 7.3.2 WASM tool execution
- [ ] 7.3.3 Capability grants (network, filesystem)

### Task 7.4: Tools

- [ ] 7.4.1 20+ builtin tools (web_search, web_fetch, file_read, file_write, shell, code_exec, etc.)
- [ ] 7.4.2 WASM tools
- [ ] 7.4.3 MCP tools

### Task 7.5: Skills

- [ ] 7.5.1 SKILL.md system
- [ ] 7.5.2 Skill registry
- [ ] 7.5.3 Skill scanning
- [ ] 7.5.4 Skill studio (AI generation)

### Task 7.6: MCP

- [ ] 7.6.1 Model Context Protocol client
- [ ] 7.6.2 MCP server management
- [ ] 7.6.3 Tool catalog

### Task 7.7: Agent Runtime

- [ ] 7.7.1 ReAct loop
- [ ] 7.7.2 Tool calling
- [ ] 7.7.3 Budget enforcement
- [ ] 7.7.4 TOML agent specs

### Task 7.8: Crew/Flow Orchestration

- [ ] 7.8.1 Multi-agent pipelines
- [ ] 7.8.2 Visual workflow builder
- [ ] 7.8.3 P2P crew market
- [ ] 7.8.4 4 multi-step templates

### Task 7.9: Safety Layer

- [ ] 7.9.1 Leak detection
- [ ] 7.9.2 Prompt injection defense
- [ ] 7.9.3 SSRF guard
- [ ] 7.9.4 Egress filtering
- [ ] 7.9.5 HITL approval gate

### Task 7.10: Messaging Channels

- [ ] 7.10.1 Telegram bot integration
- [ ] 7.10.2 Webhook endpoint
- [ ] 7.10.3 REPL (rustyline)

**Phase 7 Deliverable**: Full feature parity with bkg-p2p v0.5+.

---

## TOTAL TASK COUNT

| Phase                     | Tasks                             | Status          |
| ------------------------- | --------------------------------- | --------------- |
| Phase 1: Foundation       | 7 major tasks, ~50 subtasks       | Not started     |
| Phase 2: Resource Sharing | 5 major tasks, ~30 subtasks       | Not started     |
| Phase 3: Job Marketplace  | 6 major tasks, ~60 subtasks       | Not started     |
| Phase 4: Web Dashboard    | 13 major tasks, ~80 subtasks      | Not started     |
| Phase 5: CLI & Config     | 12 major tasks, ~60 subtasks      | Not started     |
| Phase 6: Testing & QA     | 4 major tasks, ~20 subtasks       | Not started     |
| Phase 7: Deferred         | 10 major tasks, ~30 subtasks      | Not started     |
| **Total**                 | **57 major tasks, ~330 subtasks** | **0% complete** |
