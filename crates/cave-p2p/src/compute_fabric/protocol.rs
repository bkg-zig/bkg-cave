use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Compute protocol version.
pub const COMPUTE_PROTOCOL_VERSION: &str = "bkg-p2p/compute/v1";

/// Compute protocol message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeMessageType {
    /// Request worker capabilities.
    WorkerQuery,
    /// Response with worker capabilities.
    WorkerAnnounce,
    /// Request task execution.
    TaskSubmit,
    /// Task execution result.
    TaskResult,
    /// Heartbeat from worker.
    Heartbeat,
}

/// Compute protocol message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMessage {
    /// Protocol version.
    pub version: String,
    /// Message type.
    pub msg_type: ComputeMessageType,
    /// Sender peer ID.
    pub sender: String,
    /// Message payload.
    pub payload: serde_json::Value,
    /// Timestamp in seconds since epoch.
    pub timestamp: u64,
}

impl ComputeMessage {
    /// Create a new compute protocol message.
    pub fn new(
        msg_type: ComputeMessageType,
        sender: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            version: COMPUTE_PROTOCOL_VERSION.into(),
            msg_type,
            sender: sender.into(),
            payload,
            timestamp: now_secs(),
        }
    }

    /// Create a worker query message.
    pub fn worker_query(sender: impl Into<String>, worker_type: &str) -> Self {
        Self::new(
            ComputeMessageType::WorkerQuery,
            sender,
            serde_json::json!({ "worker_type": worker_type }),
        )
    }

    /// Create a worker announcement message.
    pub fn worker_announce(
        sender: impl Into<String>,
        worker_id: &str,
        worker_type: &str,
        resources: &serde_json::Value,
    ) -> Self {
        Self::new(
            ComputeMessageType::WorkerAnnounce,
            sender,
            serde_json::json!({
                "worker_id": worker_id,
                "worker_type": worker_type,
                "resources": resources,
            }),
        )
    }

    /// Create a task submission message.
    pub fn task_submit(
        sender: impl Into<String>,
        job_id: &str,
        task: &serde_json::Value,
    ) -> Self {
        Self::new(
            ComputeMessageType::TaskSubmit,
            sender,
            serde_json::json!({
                "job_id": job_id,
                "task": task,
            }),
        )
    }

    /// Create a task result message.
    pub fn task_result(
        sender: impl Into<String>,
        job_id: &str,
        result: &serde_json::Value,
        success: bool,
    ) -> Self {
        Self::new(
            ComputeMessageType::TaskResult,
            sender,
            serde_json::json!({
                "job_id": job_id,
                "result": result,
                "success": success,
            }),
        )
    }

    /// Create a heartbeat message.
    pub fn heartbeat(sender: impl Into<String>, worker_id: &str) -> Self {
        Self::new(
            ComputeMessageType::Heartbeat,
            sender,
            serde_json::json!({ "worker_id": worker_id }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_message() {
        let msg = ComputeMessage::new(
            ComputeMessageType::WorkerQuery,
            "peer-1",
            serde_json::json!({"test": true}),
        );
        assert_eq!(msg.version, COMPUTE_PROTOCOL_VERSION);
        assert_eq!(msg.msg_type, ComputeMessageType::WorkerQuery);
        assert_eq!(msg.sender, "peer-1");
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_worker_query() {
        let msg = ComputeMessage::worker_query("peer-1", "gpu");
        assert_eq!(msg.msg_type, ComputeMessageType::WorkerQuery);
        assert_eq!(msg.payload["worker_type"], "gpu");
    }

    #[test]
    fn test_worker_announce() {
        let resources = serde_json::json!({"cpu_cores": 8, "ram_mb": 16384});
        let msg = ComputeMessage::worker_announce("peer-1", "worker-1", "gpu", &resources);
        assert_eq!(msg.msg_type, ComputeMessageType::WorkerAnnounce);
        assert_eq!(msg.payload["worker_id"], "worker-1");
        assert_eq!(msg.payload["worker_type"], "gpu");
    }

    #[test]
    fn test_task_submit() {
        let task = serde_json::json!({"model": "llama3.1"});
        let msg = ComputeMessage::task_submit("peer-1", "job-1", &task);
        assert_eq!(msg.msg_type, ComputeMessageType::TaskSubmit);
        assert_eq!(msg.payload["job_id"], "job-1");
    }

    #[test]
    fn test_task_result() {
        let result = serde_json::json!({"output": "done"});
        let msg = ComputeMessage::task_result("peer-1", "job-1", &result, true);
        assert_eq!(msg.msg_type, ComputeMessageType::TaskResult);
        assert_eq!(msg.payload["success"], true);
    }

    #[test]
    fn test_heartbeat() {
        let msg = ComputeMessage::heartbeat("peer-1", "worker-1");
        assert_eq!(msg.msg_type, ComputeMessageType::Heartbeat);
        assert_eq!(msg.payload["worker_id"], "worker-1");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let msg = ComputeMessage::worker_query("peer-1", "cpu");
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ComputeMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.msg_type, msg.msg_type);
        assert_eq!(decoded.sender, msg.sender);
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
