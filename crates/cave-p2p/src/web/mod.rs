use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::a2a::A2aState;
use crate::executor::resource::ResourceMonitor;
use crate::job::{JobBroadcastTx, JobManager, PricingStrategy, ResourceType};
use crate::network::provider_tracker::ProviderTracker;
use crate::wallet::Wallet;

pub mod openai;

#[derive(Clone)]
pub struct WebState {
    pub node_id: String,
    pub started_at: Arc<std::time::Instant>,
    pub resource_monitor: Option<Arc<ResourceMonitor>>,
    pub a2a: Arc<A2aState>,
    pub provider_tracker: Option<Arc<Mutex<ProviderTracker>>>,
    pub job_broadcast_tx: Option<JobBroadcastTx>,
    pub wallet: Arc<Mutex<Wallet>>,
    pub job_manager: Arc<Mutex<JobManager>>,
}

impl WebState {
    pub fn new(node_id: String, wallet: Wallet) -> Self {
        let (tx, _) = broadcast::channel(256);
        let mut jm = JobManager::new();
        jm.set_broadcast_tx(tx.clone());
        Self {
            node_id,
            started_at: Arc::new(std::time::Instant::now()),
            resource_monitor: None,
            a2a: Arc::new(A2aState::new()),
            provider_tracker: None,
            job_broadcast_tx: Some(tx),
            wallet: Arc::new(Mutex::new(wallet)),
            job_manager: Arc::new(Mutex::new(jm)),
        }
    }

    pub fn with_resource_monitor(mut self, monitor: ResourceMonitor) -> Self {
        self.resource_monitor = Some(Arc::new(monitor));
        self
    }

    pub fn with_a2a(mut self, a2a: Arc<A2aState>) -> Self {
        self.a2a = a2a;
        self
    }

    pub fn with_provider_tracker(mut self, tracker: ProviderTracker) -> Self {
        self.provider_tracker = Some(Arc::new(Mutex::new(tracker)));
        self
    }
}

const ROOT_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>bkg-cave Dashboard</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f172a;color:#e2e8f0;padding:2rem;max-width:900px;margin:0 auto}
h1{font-size:2rem;margin-bottom:.5rem;color:#38bdf8}h2{font-size:1.25rem;margin:1.5rem 0 .75rem;color:#94a3b8}
.stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;margin:1rem 0}
.card{background:#1e293b;border-radius:8px;padding:1.25rem;border:1px solid #334155}
.card .label{font-size:.75rem;text-transform:uppercase;color:#64748b;letter-spacing:.05em}
.card .value{font-size:1.75rem;font-weight:700;color:#38bdf8;margin-top:.25rem}
ul{list-style:none;margin-top:.5rem}li{padding:.35rem 0;border-bottom:1px solid #1e293b;font-size:.875rem}
a{color:#38bdf8;text-decoration:none}a:hover{text-decoration:underline}
.footer{margin-top:3rem;padding-top:1rem;border-top:1px solid #334155;font-size:.75rem;color:#64748b}
</style></head><body>
<h1>bkg-cave</h1>
<p>Decentralized P2P AI Agent Network</p>
<div class="stats" id="stats">
<div class="card"><div class="label">Peers</div><div class="value" id="peer-count">-</div></div>
<div class="card"><div class="label">Providers</div><div class="value" id="provider-count">-</div></div>
<div class="card"><div class="label">Active Jobs</div><div class="value" id="active-jobs">-</div></div>
<div class="card"><div class="label">Completed Jobs</div><div class="value" id="completed-jobs">-</div></div>
<div class="card"><div class="label">Uptime</div><div class="value" id="uptime">-</div></div>
</div>
<h2>API Endpoints</h2>
<ul>
<li><a href="/api/status">/api/status</a></li>
<li><a href="/api/peers">/api/peers</a></li>
<li><a href="/api/jobs">/api/jobs</a></li>
<li><a href="/v1/models">/v1/models</a></li>
</ul>
<div class="footer">bkg-cave &middot; cave-p2p</div>
<script>
async function refresh(){try{
const r=await fetch('/api/status');const d=await r.json();
document.getElementById('peer-count').textContent=d.peers;
document.getElementById('provider-count').textContent=d.providers;
document.getElementById('active-jobs').textContent=d.active_jobs;
document.getElementById('completed-jobs').textContent=d.completed_jobs;
document.getElementById('uptime').textContent=Math.floor(d.uptime_secs)+'s';
}catch(e){}}
setInterval(refresh,3000);refresh();
</script></body></html>"#;

async fn root() -> Html<&'static str> {
    Html(ROOT_HTML)
}

async fn handle_ws(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<WebState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, state))
}

async fn handle_ws_socket(mut socket: axum::extract::ws::WebSocket, state: WebState) {
    let rx = state.job_broadcast_tx.as_ref().map(|tx| tx.subscribe());
    let Some(mut rx) = rx else { return };

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        if let Ok(text) = serde_json::to_string(&event) {
                            if socket
                                .send(axum::extract::ws::Message::Text(text))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) => break,
                    Some(Ok(_)) => {},
                    _ => break,
                }
            }
        }
    }
}

async fn handle_status(State(state): State<WebState>) -> Json<serde_json::Value> {
    let peer_count = state.a2a.peer_count();
    let provider_count = state
        .provider_tracker
        .as_ref()
        .map(|t| t.lock().unwrap().provider_count())
        .unwrap_or(0);
    let resource_state = state.resource_monitor.as_ref().map(|m| m.current_state());
    let (active_jobs, completed_jobs) = {
        let jm = state.job_manager.lock().unwrap();
        (jm.active_jobs().len(), jm.completed_jobs().len())
    };
    let uptime = state.started_at.elapsed().as_secs();

    Json(serde_json::json!({
        "node_id": state.node_id,
        "uptime_secs": uptime,
        "peers": peer_count,
        "providers": provider_count,
        "active_jobs": active_jobs,
        "completed_jobs": completed_jobs,
        "resources": resource_state,
    }))
}

async fn handle_peers(State(state): State<WebState>) -> Json<serde_json::Value> {
    let peers = state.a2a.list_peer_cards();
    Json(serde_json::json!({
        "peers": peers.iter().map(|(id, card)| {
            serde_json::json!({
                "peer_id": id,
                "name": card.name,
                "description": card.description,
                "models": card.models,
                "capabilities": card.capabilities,
            })
        }).collect::<Vec<_>>(),
        "count": peers.len(),
    }))
}

async fn handle_jobs(State(state): State<WebState>) -> Json<serde_json::Value> {
    let jm = state.job_manager.lock().unwrap();
    Json(serde_json::json!({
        "active": jm.active_jobs().len(),
        "completed": jm.completed_jobs().len(),
        "pending": jm.pending_requests().len(),
        "total_bids": jm.bids_count(),
    }))
}

async fn handle_list_jobs(State(state): State<WebState>) -> Json<serde_json::Value> {
    let jm = state.job_manager.lock().unwrap();
    Json(serde_json::json!({
        "active": jm.active_jobs(),
        "completed": jm.completed_jobs(),
        "pending": jm.pending_requests(),
    }))
}

#[derive(Debug, Deserialize)]
struct CreateJobBody {
    requester: String,
    task_type: String,
    max_price: u64,
    description: String,
}

async fn handle_create_job(
    State(state): State<WebState>,
    Json(body): Json<CreateJobBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let task_type = match body.task_type.to_lowercase().as_str() {
        "inference" => ResourceType::Inference,
        "gpu" => ResourceType::Gpu,
        "cpu" => ResourceType::Cpu,
        "storage" => ResourceType::Storage,
        "embedding" => ResourceType::Embedding,
        "image_generation" | "image" => ResourceType::ImageGeneration,
        "web_fetch" | "web" => ResourceType::WebFetch,
        "vector_search" => ResourceType::VectorSearch,
        "wasm_tool" | "wasm" => ResourceType::WasmTool,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::json!({"error": format!("unknown task type: {}", body.task_type)}),
                ),
            ));
        }
    };
    let requester = if body.requester.is_empty() {
        state.node_id.clone()
    } else {
        body.requester
    };
    let mut jm = state.job_manager.lock().unwrap();
    let req = jm.create_request(
        requester,
        task_type,
        crate::executor::task::ResourceRequirements::minimal(),
        body.max_price,
        PricingStrategy::Fixed,
        body.description,
    );
    Ok(Json(serde_json::json!(req)))
}

async fn handle_tasks() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Tasks API is not yet implemented.",
        })),
    )
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    model: Option<String>,
    messages: Vec<ChatMessageInput>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ChatMessageInput {
    role: String,
    content: String,
}

async fn handle_chat(
    Json(body): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if body.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "no messages provided",
            })),
        ));
    }

    let model = body.model.as_deref().unwrap_or("default");

    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": format!("model '{model}' not available – no inference engine loaded (deferred to v1.0)"),
            "model": model,
            "message_count": body.messages.len(),
        })),
    ))
}

async fn handle_wallet_balance(State(state): State<WebState>) -> Json<serde_json::Value> {
    let wallet = state.wallet.lock().unwrap();
    let bal = wallet.balance();
    Json(serde_json::json!({
        "available": bal.available,
        "in_escrow": bal.in_escrow,
        "staked": bal.staked,
        "total": bal.total,
    }))
}

async fn handle_a2a_peers(State(state): State<WebState>) -> Json<serde_json::Value> {
    let peers = state.a2a.list_peer_cards();
    let cards: Vec<serde_json::Value> = peers
        .into_iter()
        .map(|(id, card)| {
            serde_json::json!({
                "peer_id": id,
                "name": card.name,
                "url": card.url,
                "models": card.models,
                "capabilities": card.capabilities,
                "channels": card.channels,
            })
        })
        .collect();
    Json(serde_json::json!({
        "agents": cards,
        "count": cards.len(),
    }))
}

async fn handle_agent_library() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Agent library is not yet implemented.",
        })),
    )
}

async fn handle_skills() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Skills API is not yet implemented.",
        })),
    )
}

pub fn create_router(state: WebState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/ws", get(handle_ws))
        .route("/api/status", get(handle_status))
        .route("/api/peers", get(handle_peers))
        .route("/api/jobs", get(handle_jobs))
        .route("/api/jobs/list", get(handle_list_jobs))
        .route("/api/jobs/create", post(handle_create_job))
        .route("/api/tasks", get(handle_tasks))
        .route("/api/chat", post(handle_chat))
        .route("/api/wallet/balance", get(handle_wallet_balance))
        .route("/api/a2a/peers", get(handle_a2a_peers))
        .route("/api/agents", get(handle_agent_library))
        .route("/api/skills", get(handle_skills))
        .route("/v1/models", get(openai::handle_models))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn start_server(state: WebState, addr: &str) -> Result<(), crate::Error> {
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::Error::Web(format!("Failed to bind to {addr}: {e}")))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::Error::Web(format!("Server error: {e}")))?;
    Ok(())
}