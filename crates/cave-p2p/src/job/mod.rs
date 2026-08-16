use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod execution;
pub mod settlement;

pub use execution::JobExecutionEngine;

pub type JobId = String;
pub type BidId = String;

pub type JobBroadcastTx = tokio::sync::broadcast::Sender<JobBroadcast>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Inference,
    Gpu,
    Cpu,
    Storage,
    Embedding,
    ImageGeneration,
    WebFetch,
    VectorSearch,
    WasmTool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingStrategy {
    Fixed,
    Dynamic,
    Auction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePricing {
    pub gpu_per_hour: f64,
    pub cpu_per_hour: f64,
    pub storage_per_gb_per_hour: f64,
}

impl Default for ResourcePricing {
    fn default() -> Self {
        Self {
            gpu_per_hour: 0.50,
            cpu_per_hour: 0.10,
            storage_per_gb_per_hour: 0.01,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    pub id: JobId,
    pub requester: String,
    pub task_type: ResourceType,
    pub requirements: crate::executor::task::ResourceRequirements,
    pub max_price: u64,
    pub pricing_strategy: PricingStrategy,
    pub description: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobBid {
    pub id: BidId,
    pub job_id: JobId,
    pub worker: String,
    pub price: u64,
    pub estimated_duration_secs: u64,
    pub worker_rating: f32,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: JobId,
    pub request: JobRequest,
    pub bids: Vec<JobBid>,
    pub accepted_bid: Option<JobBid>,
    pub status: JobStatus,
    pub result: Option<JobResult>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: JobId,
    pub worker: String,
    pub data: serde_json::Value,
    pub completed_at: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum JobBroadcast {
    NewRequest(JobRequest),
    BidSubmitted(JobBid),
    BidAccepted { job_id: JobId, bid: JobBid },
    JobCompleted(JobResult),
    JobCancelled(JobId),
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("job not found: {0}")]
    NotFound(String),
    #[error("job already completed")]
    AlreadyCompleted,
    #[error("no bids available")]
    NoBids,
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct JobManager {
    requests: HashMap<JobId, JobRequest>,
    jobs: HashMap<JobId, Job>,
    bids: HashMap<BidId, JobBid>,
    broadcast_tx: Option<JobBroadcastTx>,
    next_id: u64,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            jobs: HashMap::new(),
            bids: HashMap::new(),
            broadcast_tx: None,
            next_id: 0,
        }
    }

    pub fn set_broadcast_tx(&mut self, tx: JobBroadcastTx) {
        self.broadcast_tx = Some(tx);
    }

    fn generate_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{}", self.next_id)
    }

    pub fn create_request(
        &mut self,
        requester: String,
        task_type: ResourceType,
        requirements: crate::executor::task::ResourceRequirements,
        max_price: u64,
        pricing_strategy: PricingStrategy,
        description: String,
    ) -> JobRequest {
        let request = JobRequest {
            id: self.generate_id("req"),
            requester,
            task_type,
            requirements,
            max_price,
            pricing_strategy,
            description,
            created_at: now_secs(),
        };
        self.requests.insert(request.id.clone(), request.clone());

        if let Some(tx) = &self.broadcast_tx {
            let _ = tx.send(JobBroadcast::NewRequest(request.clone()));
        }

        request
    }

    pub fn receive_bid(&mut self, bid: JobBid) -> Result<(), JobError> {
        if !self.requests.contains_key(&bid.job_id) {
            return Err(JobError::NotFound(bid.job_id.clone()));
        }
        self.bids.insert(bid.id.clone(), bid.clone());
        Ok(())
    }

    pub fn get_bids(&self, job_id: &str) -> Vec<&JobBid> {
        self.bids.values().filter(|b| b.job_id == job_id).collect()
    }

    pub fn accept_bid(&mut self, job_id: &str, bid_id: &str) -> Result<&Job, JobError> {
        let request = self
            .requests
            .get(job_id)
            .ok_or_else(|| JobError::NotFound(job_id.to_string()))?;

        let bid = self
            .bids
            .get(bid_id)
            .ok_or_else(|| JobError::NotFound(bid_id.to_string()))?;

        if bid.job_id != job_id {
            return Err(JobError::Internal("bid does not match job".into()));
        }

        let job = Job {
            id: job_id.to_string(),
            request: request.clone(),
            bids: self.get_bids(job_id).into_iter().cloned().collect(),
            accepted_bid: Some(bid.clone()),
            status: JobStatus::Active,
            result: None,
            created_at: now_secs(),
        };

        if let Some(tx) = &self.broadcast_tx {
            let _ = tx.send(JobBroadcast::BidAccepted {
                job_id: job_id.to_string(),
                bid: bid.clone(),
            });
        }

        self.jobs.insert(job_id.to_string(), job);
        self.requests.remove(job_id);

        Ok(self.jobs.get(job_id).unwrap())
    }

    pub fn evaluate_request(&self, request: &JobRequest) -> bool {
        request.max_price > 0
    }

    pub fn can_fulfill(
        &self,
        request: &JobRequest,
        resource_monitor: Option<&crate::executor::resource::ResourceMonitor>,
    ) -> bool {
        if let Some(monitor) = resource_monitor {
            if !monitor.can_handle(request.requirements.ram_mb, request.requirements.vram_mb) {
                return false;
            }
        }
        true
    }

    pub fn submit_bid(
        &mut self,
        job_id: &str,
        worker: String,
        price: u64,
        estimated_duration_secs: u64,
        worker_rating: f32,
    ) -> Result<JobBid, JobError> {
        let request = self
            .requests
            .get(job_id)
            .ok_or_else(|| JobError::NotFound(job_id.to_string()))?;

        if price > request.max_price {
            return Err(JobError::InsufficientBalance);
        }

        let bid = JobBid {
            id: self.generate_id("bid"),
            job_id: job_id.to_string(),
            worker,
            price,
            estimated_duration_secs,
            worker_rating,
            created_at: now_secs(),
        };

        self.bids.insert(bid.id.clone(), bid.clone());

        if let Some(tx) = &self.broadcast_tx {
            let _ = tx.send(JobBroadcast::BidSubmitted(bid.clone()));
        }

        Ok(bid)
    }

    pub fn start_job(&mut self, _job_id: &str) -> Result<(), JobError> {
        Ok(())
    }

    pub fn submit_result(
        &mut self,
        job_id: &str,
        worker: String,
        data: serde_json::Value,
        success: bool,
        error: Option<String>,
    ) -> Result<(), JobError> {
        let job = self
            .jobs
            .get_mut(job_id)
            .ok_or_else(|| JobError::NotFound(job_id.to_string()))?;

        if matches!(job.status, JobStatus::Completed | JobStatus::Failed) {
            return Err(JobError::AlreadyCompleted);
        }

        let result = JobResult {
            job_id: job_id.to_string(),
            worker,
            data,
            completed_at: now_secs(),
            success,
            error,
        };

        job.status = if success {
            JobStatus::Completed
        } else {
            JobStatus::Failed
        };
        job.result = Some(result.clone());

        if let Some(tx) = &self.broadcast_tx {
            let _ = tx.send(JobBroadcast::JobCompleted(result));
        }

        Ok(())
    }

    pub fn active_jobs(&self) -> Vec<&Job> {
        self.jobs
            .values()
            .filter(|j| matches!(j.status, JobStatus::Active))
            .collect()
    }

    pub fn completed_jobs(&self) -> Vec<&Job> {
        self.jobs
            .values()
            .filter(|j| matches!(j.status, JobStatus::Completed))
            .collect()
    }

    pub fn pending_requests(&self) -> Vec<&JobRequest> {
        self.requests.values().collect()
    }

    pub fn bids_count(&self) -> usize {
        self.bids.len()
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
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
    use crate::executor::task::ResourceRequirements;
    use std::time::Duration;

    #[test]
    fn test_create_request() {
        let mut jm = JobManager::new();
        let req = jm.create_request(
            "alice".into(),
            ResourceType::Inference,
            ResourceRequirements {
                ram_mb: 4096,
                vram_mb: Some(2048),
                cpu_cores: 2,
                estimated_duration: Duration::from_secs(30),
            },
            1000,
            PricingStrategy::Fixed,
            "run llama3.1:8b".into(),
        );
        assert!(req.id.starts_with("req-"));
        assert_eq!(req.requester, "alice");
        assert_eq!(jm.pending_requests().len(), 1);
    }

    #[test]
    fn test_bid_flow() {
        let mut jm = JobManager::new();

        let req = jm.create_request(
            "alice".into(),
            ResourceType::Inference,
            ResourceRequirements::minimal(),
            500,
            PricingStrategy::Fixed,
            "test job".into(),
        );

        let bid = jm.submit_bid(&req.id, "bob".into(), 300, 30, 4.5).unwrap();
        assert_eq!(bid.worker, "bob");

        let bids = jm.get_bids(&req.id);
        assert_eq!(bids.len(), 1);

        let job = jm.accept_bid(&req.id, &bid.id).unwrap();
        assert_eq!(job.status, JobStatus::Active);
        assert_eq!(jm.active_jobs().len(), 1);
    }
}
