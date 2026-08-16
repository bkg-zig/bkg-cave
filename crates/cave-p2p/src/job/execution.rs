use std::sync::Arc;

use crate::compute_fabric::scheduler::Scheduler;
use crate::compute_fabric::worker::WorkerRegistry;
use crate::executor::task_executor::TaskExecutor;
use crate::job::{JobManager};
use crate::Result;

/// Executes jobs using the compute fabric and executor.
pub struct JobExecutionEngine {
    job_manager: Arc<std::sync::Mutex<JobManager>>,
    scheduler: Scheduler,
    executor: TaskExecutor,
}

impl JobExecutionEngine {
    /// Create a new job execution engine.
    pub fn new(
        job_manager: Arc<std::sync::Mutex<JobManager>>,
        worker_registry: Arc<std::sync::RwLock<WorkerRegistry>>,
        executor: TaskExecutor,
    ) -> Self {
        let scheduler = Scheduler::new(worker_registry);
        Self {
            job_manager,
            scheduler,
            executor,
        }
    }

    /// Start executing a job by finding a worker and assigning it.
    pub fn start_job(&self, job_id: &str) -> Result<()> {
        let jm_guard = self.job_manager.lock().map_err(|e| crate::Error::Other(e.to_string()))?;
        let active_jobs: Vec<_> = jm_guard.active_jobs().into_iter().cloned().collect();
        let job = active_jobs
            .iter()
            .find(|j| j.id == job_id)
            .ok_or_else(|| crate::Error::Other(format!("job {job_id} not found or not active")))?;

        let requirements = crate::executor::task::ResourceRequirements {
            ram_mb: job.request.requirements.ram_mb,
            vram_mb: job.request.requirements.vram_mb,
            cpu_cores: job.request.requirements.cpu_cores,
            estimated_duration: job.request.requirements.estimated_duration,
        };

        let worker_type = match job.request.task_type {
            crate::job::ResourceType::Inference
            | crate::job::ResourceType::Embedding
            | crate::job::ResourceType::ImageGeneration => crate::compute_fabric::worker::WorkerType::Ai,
            crate::job::ResourceType::Gpu => crate::compute_fabric::worker::WorkerType::Gpu,
            crate::job::ResourceType::Cpu => crate::compute_fabric::worker::WorkerType::Cpu,
            crate::job::ResourceType::Storage => crate::compute_fabric::worker::WorkerType::Storage,
            crate::job::ResourceType::WebFetch
            | crate::job::ResourceType::VectorSearch
            | crate::job::ResourceType::WasmTool => crate::compute_fabric::worker::WorkerType::Cpu,
        };

        let scheduling_reqs = crate::compute_fabric::scheduler::SchedulingRequirements {
            worker_type,
            min_ram_mb: requirements.ram_mb,
            min_vram_mb: requirements.vram_mb,
            min_cpu_cores: requirements.cpu_cores,
        };

        drop(jm_guard);

        let worker_id = self
            .scheduler
            .select_worker(&scheduling_reqs)
            .ok_or_else(|| crate::Error::Other("no suitable worker found".into()))?;

        self.scheduler
            .assign_task(&worker_id, job_id)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        tracing::info!("Assigned job {job_id} to worker {worker_id}");
        Ok(())
    }

    /// Complete a job execution and record the result.
    pub fn complete_job(
        &self,
        job_id: &str,
        worker_id: &str,
        success: bool,
        data: serde_json::Value,
        error: Option<String>,
    ) -> Result<()> {
        let mut jm = self.job_manager.lock().map_err(|e| crate::Error::Other(e.to_string()))?;
        jm.submit_result(job_id, worker_id.to_string(), data, success, error)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        self.scheduler
            .complete_task(worker_id)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        tracing::info!("Job {job_id} completed on worker {worker_id}");
        Ok(())
    }

    /// Get the current job manager.
    pub fn job_manager(&self) -> &Arc<std::sync::Mutex<JobManager>> {
        &self.job_manager
    }

    /// Get the scheduler.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute_fabric::{WorkerStatus, worker::WorkerType};
    use crate::executor::task::ResourceRequirements;
    use std::time::Duration;

    fn make_job_manager() -> Arc<std::sync::Mutex<JobManager>> {
        Arc::new(std::sync::Mutex::new(JobManager::new()))
    }

    fn make_worker_registry() -> Arc<std::sync::RwLock<WorkerRegistry>> {
        let mut registry = WorkerRegistry::new();
        registry.register(crate::compute_fabric::worker::Worker {
            id: "worker-1".into(),
            peer_id: "peer-1".into(),
            worker_type: WorkerType::Cpu,
            status: WorkerStatus::Available,
            resources: crate::compute_fabric::worker::WorkerResources {
                cpu_cores: 4,
                ram_mb: 8192,
                vram_mb: None,
            },
            last_heartbeat: 1000,
        });
        Arc::new(std::sync::RwLock::new(registry))
    }

    #[test]
    fn test_start_job_assigns_worker() {
        let jm = make_job_manager();
        let registry = make_worker_registry();
        let executor = TaskExecutor::new(crate::executor::router::RouterConfig::default());
        let engine = JobExecutionEngine::new(jm.clone(), registry, executor);

        let mut jm_guard = jm.lock().unwrap();
        let req = jm_guard.create_request(
            "alice".into(),
            crate::job::ResourceType::Cpu,
            ResourceRequirements {
                ram_mb: 4096,
                vram_mb: None,
                cpu_cores: 2,
                estimated_duration: Duration::from_secs(30),
            },
            1000,
            crate::job::PricingStrategy::Fixed,
            "test job".into(),
        );
        let job_id = req.id.clone();
        let bid = jm_guard
            .submit_bid(&job_id, "worker-1".into(), 500, 30, 4.5)
            .unwrap();
        jm_guard.accept_bid(&job_id, &bid.id).unwrap();
        drop(jm_guard);

        engine.start_job(&job_id).unwrap();

        let jm_guard = jm.lock().unwrap();
        let active_jobs: Vec<_> = jm_guard.active_jobs().iter().cloned().collect();
        let job = active_jobs.iter().find(|j| j.id == job_id).unwrap();
        assert_eq!(job.status, crate::job::JobStatus::Active);
    }

    #[test]
    fn test_complete_job_updates_status() {
        let jm = make_job_manager();
        let registry = make_worker_registry();
        let executor = TaskExecutor::new(crate::executor::router::RouterConfig::default());
        let engine = JobExecutionEngine::new(jm.clone(), registry, executor);

        let mut jm_guard = jm.lock().unwrap();
        let req = jm_guard.create_request(
            "alice".into(),
            crate::job::ResourceType::Cpu,
            ResourceRequirements {
                ram_mb: 4096,
                vram_mb: None,
                cpu_cores: 2,
                estimated_duration: Duration::from_secs(30),
            },
            1000,
            crate::job::PricingStrategy::Fixed,
            "test job".into(),
        );
        let job_id = req.id.clone();
        let bid = jm_guard
            .submit_bid(&job_id, "worker-1".into(), 500, 30, 4.5)
            .unwrap();
        jm_guard.accept_bid(&job_id, &bid.id).unwrap();
        drop(jm_guard);

        engine.start_job(&job_id).unwrap();
        engine
            .complete_job(&job_id, "worker-1", true, serde_json::json!({"result": "ok"}), None)
            .unwrap();

        let jm_guard = jm.lock().unwrap();
        let completed_jobs: Vec<_> = jm_guard.completed_jobs().iter().cloned().collect();
        let job = completed_jobs.iter().find(|j| j.id == job_id).unwrap();
        assert_eq!(job.status, crate::job::JobStatus::Completed);
        assert!(job.result.as_ref().unwrap().success);
    }

    #[test]
    fn test_start_job_fails_when_no_worker() {
        let jm = make_job_manager();
        let registry = Arc::new(std::sync::RwLock::new(WorkerRegistry::new()));
        let executor = TaskExecutor::new(crate::executor::router::RouterConfig::default());
        let engine = JobExecutionEngine::new(jm, registry, executor);

        let mut jm_guard = engine.job_manager().lock().unwrap();
        let req = jm_guard.create_request(
            "alice".into(),
            crate::job::ResourceType::Gpu,
            ResourceRequirements {
                ram_mb: 4096,
                vram_mb: Some(2048),
                cpu_cores: 2,
                estimated_duration: Duration::from_secs(30),
            },
            1000,
            crate::job::PricingStrategy::Fixed,
            "gpu job".into(),
        );
        let job_id = req.id.clone();
        let bid = jm_guard
            .submit_bid(&job_id, "worker-1".into(), 500, 30, 4.5)
            .unwrap();
        jm_guard.accept_bid(&job_id, &bid.id).unwrap();
        drop(jm_guard);

        let result = engine.start_job(&job_id);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("no suitable worker") || err_msg.contains("not found"),
            "unexpected error: {err_msg}"
        );
    }
}
