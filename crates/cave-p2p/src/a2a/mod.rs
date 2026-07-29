//! A2A (Agent-to-Agent) protocol types for cave-p2p.
//!
//! This module provides the JSON-RPC types used for A2A communication
//! over the P2P network.

use serde::{Deserialize, Serialize};

/// A2A JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aRpcRequest {
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Parameters (optional).
    pub params: Option<serde_json::Value>,
    /// Request ID.
    pub id: serde_json::Value,
}

/// A2A JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aRpcResponse {
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Result (if successful).
    pub result: Option<serde_json::Value>,
    /// Error (if failed).
    pub error: Option<serde_json::Value>,
    /// Request ID.
    pub id: serde_json::Value,
}

impl A2aRpcRequest {
    /// Create a new A2A RPC request.
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
    /// Create a successful A2A RPC response.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error A2A RPC response.
    pub fn error(id: serde_json::Value, error: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_creation() {
        let req = A2aRpcRequest::new("test.method", None, serde_json::json!(1));
        assert_eq!(req.method, "test.method");
        assert_eq!(req.jsonrpc, "2.0");
    }

    #[test]
    fn response_success() {
        let resp = A2aRpcResponse::success(serde_json::json!(1), serde_json::json!("ok"));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn response_error() {
        let resp = A2aRpcResponse::error(
            serde_json::json!(1),
            serde_json::json!({"code": -1, "message": "error"}),
        );
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }
}
