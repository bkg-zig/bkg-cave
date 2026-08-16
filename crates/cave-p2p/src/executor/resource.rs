use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    pub cpu_usage: f64,
    pub ram_total_mb: u32,
    pub ram_available_mb: u32,
    pub gpu_usage: Option<f64>,
    pub vram_total_mb: Option<u32>,
    pub vram_available_mb: Option<u32>,
    pub loaded_models: Vec<String>,
    pub active_inference_tasks: u32,
    pub active_wasm_tasks: u32,
    pub active_web_tasks: u32,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub update_interval: std::time::Duration,
    pub reserve_ram_mb: u32,
    pub max_concurrent_inference: u32,
    pub max_concurrent_wasm: u32,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            update_interval: std::time::Duration::from_secs(1),
            reserve_ram_mb: 500,
            max_concurrent_inference: 2,
            max_concurrent_wasm: 10,
        }
    }
}

#[derive(Debug, Clone)]
struct InternalState {
    cpu_usage: f64,
    ram_total_mb: u32,
    ram_available_mb: u32,
    gpu_usage: Option<f64>,
    vram_total_mb: Option<u32>,
    vram_available_mb: Option<u32>,
    loaded_models: Vec<String>,
    active_inference_tasks: u32,
    active_wasm_tasks: u32,
    active_web_tasks: u32,
    #[allow(dead_code)]
    last_update: Instant,
}

pub struct ResourceMonitor {
    state: Mutex<InternalState>,
    config: MonitorConfig,
}

impl ResourceMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        let ram_total_mb = total_system_ram();
        Self {
            state: Mutex::new(InternalState {
                cpu_usage: 0.0,
                ram_total_mb,
                ram_available_mb: ram_total_mb.saturating_sub(config.reserve_ram_mb),
                gpu_usage: None,
                vram_total_mb: None,
                vram_available_mb: None,
                loaded_models: Vec::new(),
                active_inference_tasks: 0,
                active_wasm_tasks: 0,
                active_web_tasks: 0,
                last_update: Instant::now(),
            }),
            config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(MonitorConfig::default())
    }

    pub fn start_background_updates(&self) {
        let config = self.config.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(config.update_interval);
        });
    }

    pub fn current_state(&self) -> ResourceState {
        let state = self.state.lock().unwrap();
        ResourceState {
            cpu_usage: state.cpu_usage,
            ram_total_mb: state.ram_total_mb,
            ram_available_mb: state.ram_available_mb,
            gpu_usage: state.gpu_usage,
            vram_total_mb: state.vram_total_mb,
            vram_available_mb: state.vram_available_mb,
            loaded_models: state.loaded_models.clone(),
            active_inference_tasks: state.active_inference_tasks,
            active_wasm_tasks: state.active_wasm_tasks,
            active_web_tasks: state.active_web_tasks,
        }
    }

    pub fn can_handle(&self, ram_mb: u32, vram_mb: Option<u32>) -> bool {
        let state = self.state.lock().unwrap();
        if state.ram_available_mb < ram_mb {
            return false;
        }
        if let Some(vram) = vram_mb {
            if let Some(avail) = state.vram_available_mb {
                if avail < vram {
                    return false;
                }
            }
        }
        true
    }

    pub fn available_capacity(&self) -> f64 {
        let state = self.state.lock().unwrap();
        if state.ram_total_mb == 0 {
            return 0.0;
        }
        state.ram_available_mb as f64 / state.ram_total_mb as f64
    }

    pub fn register_loaded_model(&self, model_id: String) {
        if let Ok(mut state) = self.state.lock() {
            if !state.loaded_models.contains(&model_id) {
                state.loaded_models.push(model_id);
            }
        }
    }

    pub fn unregister_model(&self, model_id: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.loaded_models.retain(|m| m != model_id);
        }
    }

    pub fn task_started(&self, task_type: &TaskType) {
        if let Ok(mut state) = self.state.lock() {
            match task_type {
                TaskType::Inference => state.active_inference_tasks += 1,
                TaskType::Wasm => state.active_wasm_tasks += 1,
                TaskType::Web => state.active_web_tasks += 1,
            }
        }
    }

    pub fn task_completed(&self, task_type: &TaskType) {
        if let Ok(mut state) = self.state.lock() {
            match task_type {
                TaskType::Inference => {
                    state.active_inference_tasks = state.active_inference_tasks.saturating_sub(1)
                }
                TaskType::Wasm => {
                    state.active_wasm_tasks = state.active_wasm_tasks.saturating_sub(1)
                }
                TaskType::Web => state.active_web_tasks = state.active_web_tasks.saturating_sub(1),
            }
        }
    }

    pub fn max_concurrent(&self, task_type: &TaskType) -> u32 {
        match task_type {
            TaskType::Inference => self.config.max_concurrent_inference,
            TaskType::Wasm => self.config.max_concurrent_wasm,
            TaskType::Web => u32::MAX,
        }
    }
}

pub enum TaskType {
    Inference,
    Wasm,
    Web,
}

fn total_system_ram() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(info) = std::fs::read_to_string("/proc/meminfo") {
            for line in info.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = val.parse::<u64>() {
                            return (kb / 1024) as u32;
                        }
                    }
                }
            }
        }
    }
    16_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::with_defaults();
        let state = monitor.current_state();
        assert!(state.ram_total_mb > 0);
        assert!(state.ram_available_mb <= state.ram_total_mb);
    }

    #[test]
    fn test_can_handle_sufficient_resources() {
        let config = MonitorConfig {
            reserve_ram_mb: 0,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);
        assert!(monitor.can_handle(100, None));
        assert!(monitor.can_handle(0, None));
    }

    #[test]
    fn test_can_handle_insufficient_ram() {
        let config = MonitorConfig {
            reserve_ram_mb: 0,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);
        assert!(!monitor.can_handle(1_000_000, None));
    }

    #[test]
    fn test_can_handle_no_gpu_required() {
        let monitor = ResourceMonitor::with_defaults();
        assert!(monitor.can_handle(1, None));
    }

    #[test]
    fn test_available_capacity() {
        let monitor = ResourceMonitor::with_defaults();
        let capacity = monitor.available_capacity();
        assert!((0.0..=1.0).contains(&capacity));
    }

    #[test]
    fn test_model_registration() {
        let monitor = ResourceMonitor::with_defaults();
        monitor.register_loaded_model("llama3.1:8b".into());
        let state = monitor.current_state();
        assert_eq!(state.loaded_models.len(), 1);
        assert_eq!(state.loaded_models[0], "llama3.1:8b");

        monitor.unregister_model("llama3.1:8b");
        let state = monitor.current_state();
        assert!(state.loaded_models.is_empty());
    }

    #[test]
    fn test_task_tracking() {
        let monitor = ResourceMonitor::with_defaults();
        assert_eq!(monitor.current_state().active_inference_tasks, 0);

        monitor.task_started(&TaskType::Inference);
        assert_eq!(monitor.current_state().active_inference_tasks, 1);

        monitor.task_completed(&TaskType::Inference);
        assert_eq!(monitor.current_state().active_inference_tasks, 0);
    }
}
