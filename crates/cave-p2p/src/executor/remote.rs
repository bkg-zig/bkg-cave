use crate::a2a::A2aRpcRequest;
use crate::executor::task::{ExecutionTask, ResourceRequirements};
use crate::job::{JobBroadcast, JobBroadcastTx, JobId, JobResult};
use crate::Error;

pub struct RemoteExecutor {
    broadcast_tx: Option<JobBroadcastTx>,
    pending: std::collections::HashMap<String, PendingRemoteTask>,
}

#[allow(dead_code)]
struct PendingRemoteTask {
    job_id: JobId,
    started_at: u64,
}

impl RemoteExecutor {
    pub fn new() -> Self {
        Self {
            broadcast_tx: None,
            pending: std::collections::HashMap::new(),
        }
    }

    pub fn set_broadcast_tx(&mut self, tx: JobBroadcastTx) {
        self.broadcast_tx = Some(tx);
    }

    pub fn execute(&mut self, task: ExecutionTask, _peer_id: String) -> Result<JobId, Error> {
        let job_id = format!("remote-{}", uuid());

        match &task {
            ExecutionTask::Inference(t) => {
                let _rpc = A2aRpcRequest {
                    jsonrpc: "2.0".into(),
                    method: "execute.inference".into(),
                    params: Some(serde_json::json!({
                        "model": t.model,
                        "messages": t.messages,
                    })),
                    id: serde_json::json!(job_id),
                };

                self.pending.insert(
                    job_id.clone(),
                    PendingRemoteTask {
                        job_id: job_id.clone(),
                        started_at: now_secs(),
                    },
                );

                if let Some(tx) = &self.broadcast_tx {
                    let _ = tx.send(JobBroadcast::NewRequest(crate::job::JobRequest {
                        id: job_id.clone(),
                        requester: "local".into(),
                        task_type: crate::job::ResourceType::Inference,
                        requirements: t.estimate_requirements(),
                        max_price: 100,
                        pricing_strategy: crate::job::PricingStrategy::Fixed,
                        description: format!("remote inference: {}", t.model),
                        created_at: now_secs(),
                    }));
                }

                Ok(job_id)
            }
            ExecutionTask::WebFetch(t) => {
                let request = crate::job::JobRequest {
                    id: job_id.clone(),
                    requester: "local".into(),
                    task_type: crate::job::ResourceType::WebFetch,
                    requirements: ResourceRequirements::minimal(),
                    max_price: 50,
                    pricing_strategy: crate::job::PricingStrategy::Fixed,
                    description: format!("remote web fetch: {}", t.url),
                    created_at: now_secs(),
                };
                self.pending.insert(
                    job_id.clone(),
                    PendingRemoteTask {
                        job_id: job_id.clone(),
                        started_at: now_secs(),
                    },
                );
                if let Some(tx) = &self.broadcast_tx {
                    let _ = tx.send(JobBroadcast::NewRequest(request));
                }
                Ok(job_id)
            }
            ExecutionTask::WebSearch(t) => {
                let request = crate::job::JobRequest {
                    id: job_id.clone(),
                    requester: "local".into(),
                    task_type: crate::job::ResourceType::WebFetch,
                    requirements: ResourceRequirements::minimal(),
                    max_price: 30,
                    pricing_strategy: crate::job::PricingStrategy::Fixed,
                    description: format!("remote web search: {}", t.query),
                    created_at: now_secs(),
                };
                self.pending.insert(
                    job_id.clone(),
                    PendingRemoteTask {
                        job_id: job_id.clone(),
                        started_at: now_secs(),
                    },
                );
                if let Some(tx) = &self.broadcast_tx {
                    let _ = tx.send(JobBroadcast::NewRequest(request));
                }
                Ok(job_id)
            }
            ExecutionTask::WasmExecution(t) => {
                let request = crate::job::JobRequest {
                    id: job_id.clone(),
                    requester: "local".into(),
                    task_type: crate::job::ResourceType::WasmTool,
                    requirements: ResourceRequirements::minimal(),
                    max_price: 80,
                    pricing_strategy: crate::job::PricingStrategy::Fixed,
                    description: format!("remote wasm execution: {}s timeout", t.timeout_secs),
                    created_at: now_secs(),
                };
                self.pending.insert(
                    job_id.clone(),
                    PendingRemoteTask {
                        job_id: job_id.clone(),
                        started_at: now_secs(),
                    },
                );
                if let Some(tx) = &self.broadcast_tx {
                    let _ = tx.send(JobBroadcast::NewRequest(request));
                }
                Ok(job_id)
            }
        }
    }

    pub fn submit_result(&mut self, result: JobResult) {
        self.pending.remove(&result.job_id);
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for RemoteExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::task::{ChatMessage, InferenceTask, MessageRole};

    fn make_inference() -> InferenceTask {
        InferenceTask::new(
            "llama3.1:8b".into(),
            vec![ChatMessage {
                role: MessageRole::User,
                content: "hello".into(),
            }],
        )
    }

    #[test]
    fn test_execute_remote_inference() {
        let mut executor = RemoteExecutor::new();
        let task = make_inference();
        let result = executor.execute(ExecutionTask::Inference(task), "QmPeer123".into());
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("remote-"));
        assert_eq!(executor.pending_count(), 1);
    }

    #[test]
    fn test_submit_result_clears_pending() {
        let mut executor = RemoteExecutor::new();
        let task = make_inference();
        let job_id = executor
            .execute(ExecutionTask::Inference(task), "peer".into())
            .unwrap();

        let result = JobResult {
            job_id: job_id.clone(),
            worker: "peer".into(),
            data: serde_json::json!({"output": "done"}),
            completed_at: now_secs(),
            success: true,
            error: None,
        };
        executor.submit_result(result);
        assert_eq!(executor.pending_count(), 0);
    }
}
