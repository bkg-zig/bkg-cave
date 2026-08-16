use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionTask {
    Inference(InferenceTask),
    WebFetch(WebFetchTask),
    WebSearch(WebSearchTask),
    WasmExecution(WasmTask),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceTask {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTask {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTask {
    pub query: String,
    pub max_results: u32,
    pub engine: SearchEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchEngine {
    Default,
    Google,
    Bing,
    DuckDuckGo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTask {
    pub tool_hash: String,
    pub function: String,
    pub params: serde_json::Value,
    pub capabilities: WasmCapabilities,
    pub timeout_secs: u32,
    pub max_fuel: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCapabilities {
    pub network_access: bool,
    pub filesystem_read: bool,
    pub filesystem_write: bool,
    pub allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub ram_mb: u32,
    pub vram_mb: Option<u32>,
    pub cpu_cores: u16,
    pub estimated_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionLocation {
    Local,
    Remote,
}

#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub text: String,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct WebFetchResult {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct WebSearchResult {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct WasmResult {
    pub output: Vec<u8>,
    pub fuel_used: u64,
}

#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub ttfb_ms: u64,
    pub total_time_ms: u64,
    pub queue_time_ms: u64,
}

#[derive(Debug, Clone)]
pub enum TaskData {
    Inference(InferenceResult),
    WebFetch(WebFetchResult),
    WebSearch(WebSearchResult),
    Wasm(WasmResult),
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub data: TaskData,
    pub metrics: TaskMetrics,
    pub success: bool,
    pub error: Option<String>,
}

impl InferenceTask {
    pub fn new(model: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            model,
            messages,
            max_tokens: 2048,
            temperature: 0.7,
            stop_sequences: Vec::new(),
            stream: false,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn estimate_requirements(&self) -> ResourceRequirements {
        let params = self.model.to_lowercase();
        let (ram_mb, vram_mb) =
            if params.contains("0.5b") || params.contains("1b") || params.contains("1.5b") {
                (2048, Some(1024))
            } else if params.contains("3b") || params.contains("7b") || params.contains("8b") {
                (8192, Some(4096))
            } else if params.contains("13b") || params.contains("34b") || params.contains("70b") {
                (16384, Some(12288))
            } else {
                (4096, None)
            };

        ResourceRequirements {
            ram_mb,
            vram_mb,
            cpu_cores: 2,
            estimated_duration: Duration::from_secs(30),
        }
    }
}

impl WebFetchTask {
    pub fn get(url: String) -> Self {
        Self {
            url,
            method: HttpMethod::Get,
            headers: Vec::new(),
            body: None,
            timeout_secs: 30,
        }
    }

    pub fn post(url: String, body: Vec<u8>) -> Self {
        Self {
            url,
            method: HttpMethod::Post,
            headers: Vec::new(),
            body: Some(body),
            timeout_secs: 30,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
}

impl ChatMessage {
    pub fn system(content: String) -> Self {
        Self {
            role: MessageRole::System,
            content,
        }
    }

    pub fn user(content: String) -> Self {
        Self {
            role: MessageRole::User,
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: MessageRole::Assistant,
            content,
        }
    }
}

pub fn parse_model_size(model: &str) -> u64 {
    let model = model.to_lowercase();
    if let Some(end) = model.rfind(|c: char| c.is_ascii_digit() || c == '.') {
        let start = model[..end]
            .rfind(|c: char| !c.is_ascii_digit() && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        let num_str = &model[start..=end];
        if let Ok(size) = num_str.parse::<f64>() {
            if model.contains('b') {
                return (size * 1_000_000_000.0) as u64;
            } else if model.contains('m') {
                return (size * 1_000_000.0) as u64;
            }
            return size as u64;
        }
    }
    0
}

impl ResourceRequirements {
    pub fn minimal() -> Self {
        Self {
            ram_mb: 512,
            vram_mb: None,
            cpu_cores: 1,
            estimated_duration: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_size() {
        assert_eq!(parse_model_size("llama3.1:8b"), 8_000_000_000);
        assert_eq!(parse_model_size("llama3.2:1b"), 1_000_000_000);
        assert_eq!(parse_model_size("phi-2:2.7b"), 2_700_000_000);
    }

    #[test]
    fn test_inference_cloud_minimal_requirements() {
        let task = InferenceTask::new("gpt-4".into(), vec![ChatMessage::user("hello".into())]);
        let req = task.estimate_requirements();
        assert_eq!(req.ram_mb, 4096);
        assert_eq!(req.cpu_cores, 2);
        assert!(req.vram_mb.is_none());
    }

    #[test]
    fn test_inference_unknown_name_minimal_requirements() {
        let task = InferenceTask::new(
            "unknown-model".into(),
            vec![ChatMessage::user("hello".into())],
        );
        let req = task.estimate_requirements();
        assert_eq!(req.ram_mb, 4096);
    }

    #[test]
    fn test_inference_task_requirements() {
        let task = InferenceTask::new(
            "llama3.1:8b".into(),
            vec![ChatMessage::user("hello".into())],
        )
        .with_max_tokens(4096)
        .with_temperature(0.3);

        let req = task.estimate_requirements();
        assert_eq!(req.ram_mb, 8192);
        assert_eq!(req.vram_mb, Some(4096));
    }

    #[test]
    fn test_task_type() {
        let task = ExecutionTask::Inference(InferenceTask::new(
            "llama3.1:8b".into(),
            vec![ChatMessage::user("hello".into())],
        ));
        match task {
            ExecutionTask::Inference(t) => assert_eq!(t.model, "llama3.1:8b"),
            _ => panic!("wrong task type"),
        }
    }

    #[test]
    fn test_web_fetch_task() {
        let task = WebFetchTask::get("https://example.com".into())
            .with_header("Accept", "application/json");
        assert_eq!(task.url, "https://example.com");
        assert_eq!(task.headers.len(), 1);
    }

    #[test]
    fn test_resource_requirements_minimal() {
        let req = ResourceRequirements::minimal();
        assert_eq!(req.ram_mb, 512);
        assert_eq!(req.cpu_cores, 1);
    }
}
