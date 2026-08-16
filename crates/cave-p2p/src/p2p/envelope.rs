use crate::a2a::A2aRpcRequest;
use serde::{Deserialize, Serialize};

pub const A2A_PROTOCOL: &str = "bkg-p2p/a2a-rpc";
pub const A2A_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum A2aMessageType {
    Request,
    Response,
    Notification,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2aEnvelope {
    pub protocol: String,
    pub version: String,
    pub peer_id: String,
    pub timestamp: u64,
    pub message_type: A2aMessageType,
    pub payload: serde_json::Value,
}

impl A2aEnvelope {
    pub fn new(peer_id: String, message_type: A2aMessageType, payload: serde_json::Value) -> Self {
        Self {
            protocol: A2A_PROTOCOL.to_string(),
            version: A2A_VERSION.to_string(),
            peer_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            message_type,
            payload,
        }
    }

    pub fn wrap(request: &A2aRpcRequest, peer_id: String) -> Self {
        let payload = serde_json::to_value(request).unwrap_or_default();
        Self::new(peer_id, A2aMessageType::Request, payload)
    }

    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| crate::Error::A2a(e.to_string()))
    }

    pub fn decode(data: &[u8]) -> crate::Result<Self> {
        serde_json::from_slice(data).map_err(|e| crate::Error::A2a(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_new() {
        let envelope = A2aEnvelope::new(
            "peer1".to_string(),
            A2aMessageType::Request,
            serde_json::json!({"method": "ping"}),
        );
        assert_eq!(envelope.protocol, A2A_PROTOCOL);
        assert_eq!(envelope.version, A2A_VERSION);
        assert_eq!(envelope.peer_id, "peer1");
        assert_eq!(envelope.message_type, A2aMessageType::Request);
    }

    #[test]
    fn test_envelope_wrap() {
        let request = A2aRpcRequest::new("ping", None, serde_json::json!(1));
        let envelope = A2aEnvelope::wrap(&request, "peer1".to_string());
        assert_eq!(envelope.message_type, A2aMessageType::Request);
        assert_eq!(envelope.peer_id, "peer1");
    }

    #[test]
    fn test_envelope_encode_decode_roundtrip() {
        let envelope = A2aEnvelope::new(
            "peer1".to_string(),
            A2aMessageType::Response,
            serde_json::json!({"result": "ok"}),
        );
        let encoded = envelope.encode().unwrap();
        let decoded = A2aEnvelope::decode(&encoded).unwrap();
        assert_eq!(decoded.peer_id, "peer1");
        assert_eq!(decoded.message_type, A2aMessageType::Response);
        assert_eq!(decoded.payload["result"], "ok");
    }

    #[test]
    fn test_envelope_error_type() {
        let envelope = A2aEnvelope::new(
            "peer1".to_string(),
            A2aMessageType::Error,
            serde_json::json!({"code": -32601, "message": "method not found"}),
        );
        assert_eq!(envelope.message_type, A2aMessageType::Error);
    }
}
