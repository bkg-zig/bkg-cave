use crate::executor::resource::ResourceMonitor;
use crate::executor::task::{ExecutionTask, ResourceRequirements};

pub struct TaskRouter {
    resource_monitor: Option<ResourceMonitor>,
    config: RouterConfig,
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub max_local_ram_mb: u32,
    pub max_local_vram_mb: u32,
    pub offload_threshold_ram_mb: u32,
    pub offload_threshold_vram_mb: u32,
    pub prefer_local: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_local_ram_mb: 32_000,
            max_local_vram_mb: 8_000,
            offload_threshold_ram_mb: 16_000,
            offload_threshold_vram_mb: 4_000,
            prefer_local: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingDecision {
    ExecuteLocally,
    OffloadToNetwork,
    InsufficientResources(String),
}

impl TaskRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            resource_monitor: None,
            config,
        }
    }

    pub fn with_resource_monitor(mut self, monitor: ResourceMonitor) -> Self {
        self.resource_monitor = Some(monitor);
        self
    }

    pub fn route(&self, task: &ExecutionTask) -> RoutingDecision {
        let requirements = match task {
            ExecutionTask::Inference(t) => t.estimate_requirements(),
            ExecutionTask::WebFetch(_) => ResourceRequirements::minimal(),
            ExecutionTask::WebSearch(_) => ResourceRequirements::minimal(),
            ExecutionTask::WasmExecution(t) => ResourceRequirements {
                ram_mb: 256,
                vram_mb: None,
                cpu_cores: 1,
                estimated_duration: std::time::Duration::from_secs(t.timeout_secs as u64),
            },
        };

        self.route_with_requirements(&requirements)
    }

    pub fn route_with_requirements(&self, req: &ResourceRequirements) -> RoutingDecision {
        if let Some(monitor) = &self.resource_monitor {
            if !monitor.can_handle(req.ram_mb, req.vram_mb) {
                return RoutingDecision::InsufficientResources(self.build_insufficient_reason(req));
            }
        }

        if self.should_offload(req) {
            RoutingDecision::OffloadToNetwork
        } else {
            RoutingDecision::ExecuteLocally
        }
    }

    pub fn should_execute_locally(&self, req: &ResourceRequirements) -> bool {
        if req.ram_mb > self.config.max_local_ram_mb {
            return false;
        }
        if let Some(vram) = req.vram_mb {
            if vram > self.config.max_local_vram_mb {
                return false;
            }
        }
        if let Some(monitor) = &self.resource_monitor {
            return monitor.can_handle(req.ram_mb, req.vram_mb);
        }
        true
    }

    pub fn should_offload(&self, req: &ResourceRequirements) -> bool {
        if !self.config.prefer_local {
            return true;
        }
        if req.ram_mb > self.config.offload_threshold_ram_mb {
            return true;
        }
        if let Some(vram) = req.vram_mb {
            if vram > self.config.offload_threshold_vram_mb {
                return true;
            }
        }
        false
    }

    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    pub fn build_insufficient_reason(&self, req: &ResourceRequirements) -> String {
        let mut reasons = Vec::new();
        if let Some(monitor) = &self.resource_monitor {
            let state = monitor.current_state();
            if state.ram_available_mb < req.ram_mb {
                reasons.push(format!(
                    "need {} MB RAM, have {} MB",
                    req.ram_mb, state.ram_available_mb
                ));
            }
        }
        if reasons.is_empty() {
            reasons.push("insufficient local resources".into());
        }
        reasons.join("; ")
    }

    pub fn to_job_requirements(&self, task: &ExecutionTask) -> ResourceRequirements {
        match task {
            ExecutionTask::Inference(t) => t.estimate_requirements(),
            ExecutionTask::WebFetch(_) => ResourceRequirements {
                ram_mb: 256,
                vram_mb: None,
                cpu_cores: 1,
                estimated_duration: std::time::Duration::from_secs(30),
            },
            ExecutionTask::WebSearch(_) => ResourceRequirements {
                ram_mb: 256,
                vram_mb: None,
                cpu_cores: 1,
                estimated_duration: std::time::Duration::from_secs(15),
            },
            ExecutionTask::WasmExecution(t) => ResourceRequirements {
                ram_mb: 256,
                vram_mb: None,
                cpu_cores: 1,
                estimated_duration: std::time::Duration::from_secs(t.timeout_secs as u64),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerFilter {
    pub min_ram_mb: u32,
    pub min_vram_mb: u32,
    pub min_cpu_cores: u16,
    pub required_model: Option<String>,
    pub required_capabilities: Vec<String>,
}

impl PeerFilter {
    pub fn satisfied_by(&self, _resource_state: &crate::executor::resource::ResourceState) -> bool {
        if let Some(ref _model) = self.required_model {
            if _resource_state.loaded_models.is_empty() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_lightweight_task_locally() {
        let config = RouterConfig::default();
        let monitor = ResourceMonitor::with_defaults();
        let router = TaskRouter::new(config).with_resource_monitor(monitor);

        let task = ExecutionTask::WebFetch(crate::executor::task::WebFetchTask::get(
            "https://example.com".into(),
        ));

        match router.route(&task) {
            RoutingDecision::ExecuteLocally => {}
            _ => panic!("expected local execution"),
        }
    }

    #[test]
    fn test_job_requirements_satisfied() {
        let filter = PeerFilter {
            min_ram_mb: 1000,
            min_vram_mb: 0,
            min_cpu_cores: 1,
            required_model: None,
            required_capabilities: vec![],
        };
        let state = crate::executor::resource::ResourceState {
            cpu_usage: 0.5,
            ram_total_mb: 16000,
            ram_available_mb: 8000,
            gpu_usage: None,
            vram_total_mb: None,
            vram_available_mb: None,
            loaded_models: vec![],
            active_inference_tasks: 0,
            active_wasm_tasks: 0,
            active_web_tasks: 0,
        };
        assert!(filter.satisfied_by(&state));
    }

    #[test]
    fn test_job_requirements_not_satisfied_missing_model() {
        let filter = PeerFilter {
            min_ram_mb: 1000,
            min_vram_mb: 0,
            min_cpu_cores: 1,
            required_model: Some("llama3.1:8b".into()),
            required_capabilities: vec![],
        };
        let state = crate::executor::resource::ResourceState {
            cpu_usage: 0.5,
            ram_total_mb: 16000,
            ram_available_mb: 8000,
            gpu_usage: None,
            vram_total_mb: None,
            vram_available_mb: None,
            loaded_models: vec![],
            active_inference_tasks: 0,
            active_wasm_tasks: 0,
            active_web_tasks: 0,
        };
        assert!(!filter.satisfied_by(&state));
    }
}
