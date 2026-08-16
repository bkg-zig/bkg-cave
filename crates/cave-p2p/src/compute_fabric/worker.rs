use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkerType {
    Cpu,
    Gpu,
    Build,
    Ai,
    Storage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    Available,
    Busy,
    Reserved,
    Offline,
}

#[derive(Debug, Clone)]
pub struct WorkerResources {
    pub cpu_cores: u16,
    pub ram_mb: u32,
    pub vram_mb: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Worker {
    pub id: String,
    pub peer_id: String,
    pub worker_type: WorkerType,
    pub status: WorkerStatus,
    pub resources: WorkerResources,
    pub last_heartbeat: u64,
}

#[derive(Debug, Default)]
pub struct WorkerRegistry {
    pub(crate) workers: HashMap<String, Worker>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub fn register(&mut self, worker: Worker) {
        self.workers.insert(worker.id.clone(), worker);
    }

    pub fn unregister(&mut self, id: &str) {
        self.workers.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&Worker> {
        self.workers.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Worker> {
        self.workers.get_mut(id)
    }

    pub fn available(&self, worker_type: WorkerType) -> Vec<&Worker> {
        self.workers
            .values()
            .filter(|w| w.worker_type == worker_type && w.status == WorkerStatus::Available)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.workers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_worker(id: &str, worker_type: WorkerType) -> Worker {
        Worker {
            id: id.to_string(),
            peer_id: format!("peer_{}", id),
            worker_type,
            status: WorkerStatus::Available,
            resources: WorkerResources {
                cpu_cores: 4,
                ram_mb: 8192,
                vram_mb: None,
            },
            last_heartbeat: 1000,
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = WorkerRegistry::new();
        assert_eq!(registry.count(), 0);

        let worker = make_worker("w1", WorkerType::Cpu);
        registry.register(worker);
        assert_eq!(registry.count(), 1);

        let fetched = registry.get("w1");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, "w1");
    }

    #[test]
    fn test_unregister() {
        let mut registry = WorkerRegistry::new();
        registry.register(make_worker("w1", WorkerType::Cpu));
        assert_eq!(registry.count(), 1);
        registry.unregister("w1");
        assert_eq!(registry.count(), 0);
        assert!(registry.get("w1").is_none());
    }

    #[test]
    fn test_available_filtering() {
        let mut registry = WorkerRegistry::new();
        registry.register(make_worker("w1", WorkerType::Cpu));
        registry.register(make_worker("w2", WorkerType::Gpu));

        let mut gpu_worker = make_worker("w3", WorkerType::Gpu);
        gpu_worker.status = WorkerStatus::Busy;
        registry.register(gpu_worker);

        let available_cpu = registry.available(WorkerType::Cpu);
        assert_eq!(available_cpu.len(), 1);

        let available_gpu = registry.available(WorkerType::Gpu);
        assert_eq!(available_gpu.len(), 1);

        let available_ai = registry.available(WorkerType::Ai);
        assert_eq!(available_ai.len(), 0);
    }

    #[test]
    fn test_get_mut_updates_status() {
        let mut registry = WorkerRegistry::new();
        registry.register(make_worker("w1", WorkerType::Cpu));

        let worker = registry.get_mut("w1").unwrap();
        worker.status = WorkerStatus::Busy;

        let fetched = registry.get("w1").unwrap();
        assert_eq!(fetched.status, WorkerStatus::Busy);
    }
}
