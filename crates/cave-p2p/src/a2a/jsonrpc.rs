use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcEnvelope {
    pub jsonrpc: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcEnvelope {
    pub fn request(method: &str, params: Option<serde_json::Value>, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: Some(method.into()),
            params,
            id: Some(id),
            result: None,
            error: None,
        }
    }

    pub fn response(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: None,
            params: None,
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: None,
            params: None,
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn is_request(&self) -> bool {
        self.method.is_some()
    }

    pub fn is_response(&self) -> bool {
        self.result.is_some() || self.error.is_some()
    }

    pub fn method_name(&self) -> Option<&str> {
        self.method.as_deref()
    }

    pub fn request_id(&self) -> Option<&serde_json::Value> {
        self.id.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aRpcWireRequest {
    pub envelope: JsonRpcEnvelope,
    pub peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aRpcWireResponse {
    pub envelope: JsonRpcEnvelope,
    pub peer_id: String,
}

pub fn dispatch(envelope: JsonRpcEnvelope) -> JsonRpcEnvelope {
    if !envelope.is_request() {
        return JsonRpcEnvelope::error(envelope.id, -32600, "only requests can be dispatched");
    }

    let method = match envelope.method_name() {
        Some(m) => m.to_string(),
        None => {
            return JsonRpcEnvelope::error(envelope.id, -32600, "method is required");
        }
    };

    let _params = envelope.params.clone().unwrap_or(serde_json::Value::Null);

    match method.as_str() {
        "ping" => JsonRpcEnvelope::response(
            envelope.id.unwrap_or(serde_json::Value::Null),
            serde_json::json!({ "status": "pong" }),
        ),
        "rpc.discover" => JsonRpcEnvelope::response(
            envelope.id.unwrap_or(serde_json::Value::Null),
            serde_json::json!({
                "methods": ["ping", "rpc.discover", "agent.card.get", "task.create"]
            }),
        ),
        _ => JsonRpcEnvelope::error(envelope.id, -32601, &format!("method not found: {method}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request() {
        let req = JsonRpcEnvelope::request("ping", None, serde_json::json!(1));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method_name(), Some("ping"));
        assert!(req.is_request());
        assert!(!req.is_response());
    }

    #[test]
    fn test_jsonrpc_response() {
        let resp = JsonRpcEnvelope::response(serde_json::json!(1), serde_json::json!("ok"));
        assert!(!resp.is_request());
        assert!(resp.is_response());
        assert_eq!(resp.request_id(), Some(&serde_json::json!(1)));
    }

    #[test]
    fn test_jsonrpc_error() {
        let err = JsonRpcEnvelope::error(Some(serde_json::json!(1)), -32601, "method not found");
        assert_eq!(err.error.as_ref().unwrap().code, -32601);
        assert!(!err.is_request());
        assert!(err.is_response());
    }

    #[test]
    fn test_dispatch_ping() {
        let req = JsonRpcEnvelope::request("ping", None, serde_json::json!(1));
        let resp = dispatch(req);
        assert_eq!(resp.result.as_ref().unwrap()["status"], "pong");
    }

    #[test]
    fn test_dispatch_unknown() {
        let req = JsonRpcEnvelope::request("unknown", None, serde_json::json!(1));
        let resp = dispatch(req);
        assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn test_dispatch_response_rejected() {
        let resp = JsonRpcEnvelope::response(serde_json::json!(1), serde_json::json!("ok"));
        let result = dispatch(resp);
        assert_eq!(result.error.as_ref().unwrap().code, -32600);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let req = JsonRpcEnvelope::request("ping", None, serde_json::json!(1));
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: JsonRpcEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.method_name(), Some("ping"));
        assert_eq!(deserialized.request_id(), Some(&serde_json::json!(1)));
    }
}
