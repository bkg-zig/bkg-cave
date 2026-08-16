use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

pub mod gossip;
pub mod jsonrpc;

pub use jsonrpc::{A2aRpcWireRequest, A2aRpcWireResponse, JsonRpcEnvelope};

pub const A2A_GOSSIP_TOPIC: &str = "bkg-p2p/a2a/v1";

/// Wire type for JSON-RPC over request_response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: serde_json::Value,
}

/// Wire type for JSON-RPC over request_response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: serde_json::Value,
}

impl A2aRpcRequest {
    pub fn new(
        method: impl Into<String>,
        params: Option<serde_json::Value>,
        id: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

impl A2aRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: serde_json::Value, error: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub peer_id: String,
    pub models: Vec<String>,
    pub capabilities: AgentCapabilities,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub tools: bool,
    pub crew: bool,
    pub vector_memory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardAnnouncement {
    pub peer_id: String,
    pub card: AgentCard,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aTask {
    pub id: String,
    pub peer_id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub created_at: u64,
}

pub struct A2aState {
    peer_cards: RwLock<HashMap<String, AgentCard>>,
    task_store: RwLock<HashMap<String, A2aTask>>,
}

impl A2aState {
    pub fn new() -> Self {
        Self {
            peer_cards: RwLock::new(HashMap::new()),
            task_store: RwLock::new(HashMap::new()),
        }
    }

    pub fn upsert_peer_card(&self, peer_id: String, card: AgentCard) {
        if let Ok(mut cards) = self.peer_cards.write() {
            cards.insert(peer_id, card);
        }
    }

    pub fn list_peer_cards(&self) -> Vec<(String, AgentCard)> {
        self.peer_cards
            .read()
            .map(|cards| cards.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    pub fn get_peer_card(&self, peer_id: &str) -> Option<AgentCard> {
        self.peer_cards
            .read()
            .ok()
            .and_then(|cards| cards.get(peer_id).cloned())
    }

    pub fn remove_peer_card(&self, peer_id: &str) {
        if let Ok(mut cards) = self.peer_cards.write() {
            cards.remove(peer_id);
        }
    }

    pub fn peer_count(&self) -> usize {
        self.peer_cards.read().map(|c| c.len()).unwrap_or(0)
    }

    pub fn add_task(&self, task: A2aTask) {
        if let Ok(mut tasks) = self.task_store.write() {
            tasks.insert(task.id.clone(), task);
        }
    }

    pub fn get_task(&self, id: &str) -> Option<A2aTask> {
        self.task_store.read().ok().and_then(|t| t.get(id).cloned())
    }

    pub fn handle_jsonrpc(&self, body: &str) -> serde_json::Value {
        match serde_json::from_str::<JsonRpcEnvelope>(body) {
            Ok(envelope) => {
                let response = jsonrpc::dispatch(envelope);
                serde_json::to_value(response).unwrap_or_default()
            }
            Err(e) => serde_json::json!({
                "jsonrpc": "2.0",
                "error": { "code": -32700, "message": format!("Parse error: {e}") },
                "id": null,
            }),
        }
    }
}

impl Default for A2aState {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentCard {
    pub fn bkg_p2p_default(peer_id: &str) -> Self {
        Self {
            name: "bkg-p2p node".into(),
            description: "bkg-p2p autonomous agent node".into(),
            url: format!("/p2p/{peer_id}"),
            peer_id: peer_id.to_string(),
            models: vec![],
            capabilities: AgentCapabilities {
                streaming: true,
                tools: false,
                crew: false,
                vector_memory: false,
            },
            channels: vec!["json-rpc".into()],
        }
    }
}

impl AgentCapabilities {
    pub fn all() -> Self {
        Self {
            streaming: true,
            tools: true,
            crew: true,
            vector_memory: true,
        }
    }

    pub fn none() -> Self {
        Self {
            streaming: false,
            tools: false,
            crew: false,
            vector_memory: false,
        }
    }
}

impl AgentCapabilities {
    pub fn supports(&self, capability: &str) -> bool {
        match capability {
            "streaming" => self.streaming,
            "tools" => self.tools,
            "crew" => self.crew,
            "vector_memory" => self.vector_memory,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_default() {
        let card = AgentCard::bkg_p2p_default("12D3KooWABC");
        assert_eq!(card.name, "bkg-p2p node");
        assert_eq!(card.peer_id, "12D3KooWABC");
        assert!(card.capabilities.streaming);
        assert!(!card.capabilities.tools);
    }

    #[test]
    fn test_capability_supports() {
        let caps = AgentCapabilities::all();
        assert!(caps.supports("streaming"));
        assert!(caps.supports("tools"));
        assert!(caps.supports("crew"));
        assert!(caps.supports("vector_memory"));

        let none = AgentCapabilities::none();
        assert!(!none.supports("streaming"));
        assert!(!none.supports("tools"));
    }

    #[test]
    fn test_a2a_state_upsert_and_list() {
        let state = A2aState::new();
        assert_eq!(state.peer_count(), 0);

        let card = AgentCard::bkg_p2p_default("peer1");
        state.upsert_peer_card("peer1".into(), card.clone());

        assert_eq!(state.peer_count(), 1);

        let cards = state.list_peer_cards();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].0, "peer1");

        let fetched = state.get_peer_card("peer1");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "bkg-p2p node");

        state.remove_peer_card("peer1");
        assert_eq!(state.peer_count(), 0);
    }

    #[test]
    fn test_a2a_state_add_and_get_task() {
        let state = A2aState::new();
        let task = A2aTask {
            id: "task-1".into(),
            peer_id: "peer1".into(),
            method: "test".into(),
            params: serde_json::json!({"key": "value"}),
            created_at: 1000,
        };
        state.add_task(task.clone());
        let fetched = state.get_task("task-1");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().method, "test");

        let missing = state.get_task("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_jsonrpc_dispatch_invalid_json() {
        let state = A2aState::new();
        let result = state.handle_jsonrpc("not json");
        assert!(result.get("error").is_some());
        assert_eq!(result["error"]["code"], -32700);
    }
}
