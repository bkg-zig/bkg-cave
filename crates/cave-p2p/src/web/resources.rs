use axum::extract::State;
use axum::Json;
use serde_json::json;

use crate::executor::resource::ResourceMonitor;
use crate::network::provider_tracker::ProviderTracker;
use crate::web::WebState;

/// GET /api/resources/local
/// Returns the local node's resource state (or null if not monitored).
pub async fn handle_resources_local(State(state): State<WebState>) -> Json<serde_json::Value> {
    let local = state.resource_monitor.as_ref().map(|m| m.current_state());
    Json(json!(local))
}

/// GET /api/resources/peers
/// Returns a list of peer resource summaries (empty if not implemented).
pub async fn handle_resources_peers(State(_state): State<WebState>) -> Json<serde_json::Value> {
    // We do not have a resource tracker; peers' resource info is not stored.
    // Return empty list.
    Json(json!([]))
}

/// GET /api/resources
/// Returns an object with `local` and `peers` keys.
pub async fn handle_resources(State(state): State<WebState>) -> Json<serde_json::Value> {
    let local = state.resource_monitor.as_ref().map(|m| m.current_state());
    let peers = state.provider_tracker.as_ref()
        .map(|pt| {
            let _guard = pt.lock().unwrap();
            // No resource data in ProviderManifest; return empty vec.
            Vec::<serde_json::Value>::new()
        })
        .unwrap_or_default();
    Json(json!({
        "local": local,
        "peers": peers
    }))
}