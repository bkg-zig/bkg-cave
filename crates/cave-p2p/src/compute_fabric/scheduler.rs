use std::sync::{Arc, RwLock};

use crate::compute_fabric::worker::{WorkerRegistry, WorkerStatus, WorkerType};
use crate::Result;

#[derive(Debug, Clone)]
pub struct SchedulingRequirements {
    pub worker_type: WorkerType,
    pub min_ram_mb: u32,
    pub min_vram_mb: Option<u32>,
    pub min_cpu_cores: u16,
}

pub struct Scheduler {
    registry: Arc<RwLock<WorkerRegistry>>,
}

impl Scheduler {
    pub fn new(registry: Arc<RwLock<WorkerRegistry>>) -> Self {
        Self { registry }
    }

    pub fn select_worker(&self, requirements: &SchedulingRequirements) -> Option<String> {
        let registry = self.registry.read().ok()?;
        let workers = registry.available(requirements.worker_type);
        workers
            .into_iter()
            .find(|w| {
                w.resources.ram_mb >= requirements.min_ram_mb
                    && w.resources.cpu_cores >= requirements.min_cpu_cores
                    && match requirements.min_vram_mb {
                        Some(min_vram) => w.resources.vram_mb.is_some_and(|v| v >= min_vram),
                        None => true,
                    }
            })
            .map(|w| w.id.clone())
    }

    pub fn assign_task(&self, worker_id: &str, task_id: &str) -> Result<()> {
        let mut registry = self
            .registry
            .write()
            .map_err(|e| crate::Error::Other(e.to_string()))?;
        let worker = registry
            .get_mut(worker_id)
            .ok_or_else(|| crate::Error::Other(format!("worker {worker_id} not found")))?;
        if worker.status != WorkerStatus::Available {
            return Err(crate::Error::Other(format!(
                "worker {worker_id} is not available"
            )));
        }
        worker.status = WorkerStatus::Busy;
        tracing::info!("Assigned task {task_id} to worker {worker_id}");
        Ok(())
    }

    pub fn complete_task(&self, worker_id: &str) -> Result<()> {
        let mut registry = self
            .registry
            .write()
            .map_err(|e| crate::Error::Other(e.to_string()))?;
        if let Some(worker) = registry.get_mut(worker_id) {
            worker.status = WorkerStatus::Available;
        }
        tracing::info!("Task completed on worker {worker_id}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute_fabric::worker::*;

    fn make_worker(id: &str, worker_type: WorkerType) -> Worker {
        Worker {
            id: id.to_string(),
            peer_id: format!("peer_{}", id),
            worker_type,
            status: WorkerStatus::Available,
            resources: WorkerResources {
                cpu_cores: 8,
                ram_mb: 16384,
                vram_mb: None,
            },
            last_heartbeat: 1000,
        }
    }

    #[test]
    fn test_select_worker_finds_suitable() {
        let mut registry = WorkerRegistry::new();
        registry.register(make_worker("w1", WorkerType::Cpu));

        let registry = Arc::new(RwLock::new(registry));
        let scheduler = Scheduler::new(registry);

        let requirements = SchedulingRequirements {
            worker_type: WorkerType::Cpu,
            min_ram_mb: 8192,
            min_vram_mb: None,
            min_cpu_cores: 4,
        };

        let selected = scheduler.select_worker(&requirements);
        assert_eq!(selected, Some("w1".to_string()));
    }

    #[test]
    fn test_select_worker_no_match() {
        let registry = Arc::new(RwLock::new(WorkerRegistry::new()));
        let scheduler = Scheduler::new(registry);

        let requirements = SchedulingRequirements {
            worker_type: WorkerType::Gpu,
            min_ram_mb: 8192,
            min_vram_mb: None,
            min_cpu_cores: 4,
        };

        let selected = scheduler.select_worker(&requirements);
        assert!(selected.is_none());
    }

    #[test]
    fn test_assign_task() {
        let mut registry = WorkerRegistry::new();
        registry.register(make_worker("w1", WorkerType::Cpu));

        let registry = Arc::new(RwLock::new(registry));
        let scheduler = Scheduler::new(registry.clone());

        assert!(scheduler.assign_task("w1", "task1").is_ok());

        let reg = registry.read().unwrap();
        let worker = reg.get("w1").unwrap();
        assert_eq!(worker.status, WorkerStatus::Busy);
    }

    #[test]
    fn test_assign_nonexistent_worker_fails() {
        let registry = Arc::new(RwLock::new(WorkerRegistry::new()));
        let scheduler = Scheduler::new(registry);

        let result = scheduler.assign_task("nonexistent", "task1");
        assert!(result.is_err());
    }

    #[test]
    fn test_complete_task() {
        let mut registry = WorkerRegistry::new();
        registry.register(make_worker("w1", WorkerType::Cpu));

        let registry = Arc::new(RwLock::new(registry));
        let scheduler = Scheduler::new(registry.clone());

        scheduler.assign_task("w1", "task1").unwrap();
        scheduler.complete_task("w1").unwrap();

        let reg = registry.read().unwrap();
        let worker = reg.get("w1").unwrap();
        assert_eq!(worker.status, WorkerStatus::Available);
    }
}
