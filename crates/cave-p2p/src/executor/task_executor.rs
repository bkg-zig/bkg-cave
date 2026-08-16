use crate::executor::remote::RemoteExecutor;
use crate::executor::resource::ResourceMonitor;
use crate::executor::router::{RouterConfig, TaskRouter};
use crate::executor::task::{ExecutionTask, WebFetchTask, WebSearchTask};
use crate::Error;

#[derive(Debug, Clone)]
pub struct TaskOutput {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum ExecutionLocation {
    Local,
    Remote(String),
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(f),
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("no tokio runtime available");
            rt.block_on(f)
        }
    }
}

pub struct TaskExecutor {
    router: TaskRouter,
    remote: Option<RemoteExecutor>,
    http_client: Option<reqwest::Client>,
}

impl TaskExecutor {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            router: TaskRouter::new(config),
            remote: None,
            http_client: None,
        }
    }

    pub fn with_router(router: TaskRouter) -> Self {
        Self {
            router,
            remote: None,
            http_client: None,
        }
    }

    pub fn with_resource_monitor(mut self, monitor: ResourceMonitor) -> Self {
        self.router = TaskRouter::new(self.router_config().clone()).with_resource_monitor(monitor);
        self
    }

    pub fn with_remote_executor(mut self, remote: RemoteExecutor) -> Self {
        self.remote = Some(remote);
        self
    }

    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    fn client(&self) -> reqwest::Client {
        self.http_client.clone().unwrap_or_default()
    }

    pub fn router_config(&self) -> &RouterConfig {
        self.router.config()
    }

    pub fn router(&self) -> &TaskRouter {
        &self.router
    }

    pub fn decide_location(&self, task: &ExecutionTask) -> ExecutionLocation {
        match self.router.route(task) {
            crate::executor::router::RoutingDecision::ExecuteLocally => ExecutionLocation::Local,
            crate::executor::router::RoutingDecision::OffloadToNetwork => {
                ExecutionLocation::Remote("auto".into())
            }
            crate::executor::router::RoutingDecision::InsufficientResources(_) => {
                ExecutionLocation::Remote("fallback".into())
            }
        }
    }

    pub fn execute_local(&self, task: &ExecutionTask) -> Result<TaskOutput, Error> {
        match task {
            ExecutionTask::Inference(_) => Err(Error::Inference(
                "no inference engine loaded – deferred to v1.0".into(),
            )),
            ExecutionTask::WebFetch(t) => self.execute_web_fetch_local(t),
            ExecutionTask::WebSearch(t) => self.execute_web_search_local(t),
            ExecutionTask::WasmExecution(t) => Err(Error::Executor(format!(
                "wasm execution not yet implemented locally: {}",
                t.timeout_secs
            ))),
        }
    }

    fn execute_web_fetch_local(&self, task: &WebFetchTask) -> Result<TaskOutput, Error> {
        let client = self.client();
        let url = task.url.clone();
        let method = match task.method {
            crate::executor::task::HttpMethod::Get => reqwest::Method::GET,
            crate::executor::task::HttpMethod::Post => reqwest::Method::POST,
            crate::executor::task::HttpMethod::Put => reqwest::Method::PUT,
            crate::executor::task::HttpMethod::Delete => reqwest::Method::DELETE,
            crate::executor::task::HttpMethod::Patch => reqwest::Method::PATCH,
            crate::executor::task::HttpMethod::Head => reqwest::Method::HEAD,
        };

        let headers = task.headers.clone();
        let req_body = task.body.clone();

        let (status, content) = block_on(async move {
            let mut req = client.request(method, &url);
            for (k, v) in &headers {
                req = req.header(k, v);
            }
            if let Some(b) = &req_body {
                req = req.body(b.clone());
            }
            let resp = req.send().await.map_err(|e| format!("{e}"))?;
            let status = resp.status().as_u16();
            let text = resp.text().await.map_err(|e| format!("{e}"))?;
            Ok::<(u16, String), String>((status, text))
        })
        .map_err(|e| Error::Other(format!("web fetch failed: {e}")))?;

        Ok(TaskOutput {
            success: status < 500,
            data: serde_json::json!({
                "url": task.url,
                "status": status,
                "body": content,
            }),
            error: None,
        })
    }

    fn execute_web_search_local(&self, task: &WebSearchTask) -> Result<TaskOutput, Error> {
        let client = self.client();
        let query = task.query.clone();

        let json = block_on(async move {
            let resp = client
                .get("https://api.duckduckgo.com/")
                .query(&[
                    ("q", query.as_str()),
                    ("format", "json"),
                    ("no_html", "1"),
                    ("skip_disambig", "1"),
                ])
                .send()
                .await
                .map_err(|e| format!("{e}"))?;
            let json: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
            Ok::<serde_json::Value, String>(json)
        })
        .map_err(|e| Error::Other(format!("web search failed: {e}")))?;

        let mut results = Vec::new();
        if let Some(abstract_text) = json["AbstractText"].as_str() {
            if !abstract_text.is_empty() {
                results.push(serde_json::json!({
                    "title": json["Heading"].as_str().unwrap_or(""),
                    "abstract": abstract_text,
                    "url": json["AbstractURL"].as_str().unwrap_or(""),
                    "source": "DuckDuckGo"
                }));
            }
        }
        if let Some(related) = json["RelatedTopics"].as_array() {
            for topic in related {
                if let Some(text) = topic["Text"].as_str() {
                    results.push(serde_json::json!({
                        "title": text.lines().next().unwrap_or(""),
                        "abstract": text,
                        "url": topic["FirstURL"].as_str().unwrap_or(""),
                        "source": "DuckDuckGo"
                    }));
                }
            }
        }

        Ok(TaskOutput {
            success: true,
            data: serde_json::json!({ "results": results }),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::task::{ChatMessage, InferenceTask, MessageRole};

    #[test]
    fn test_decide_local_for_small_inference() {
        let config = RouterConfig {
            max_local_ram_mb: 32_000,
            max_local_vram_mb: 8_000,
            offload_threshold_ram_mb: 32_000,
            offload_threshold_vram_mb: 8_000,
            prefer_local: true,
        };
        let executor = TaskExecutor::new(config);

        let task = InferenceTask::new(
            "llama3.1:8b".into(),
            vec![ChatMessage {
                role: MessageRole::User,
                content: "hello".into(),
            }],
        );

        match executor.decide_location(&ExecutionTask::Inference(task)) {
            ExecutionLocation::Local => {}
            other => panic!("expected local, got {:?}", other),
        }
    }

    #[test]
    fn test_execute_inference_local_returns_error() {
        let executor = TaskExecutor::new(RouterConfig::default());
        let task = InferenceTask::new(
            "llama3.1:8b".into(),
            vec![ChatMessage {
                role: MessageRole::User,
                content: "hello".into(),
            }],
        );

        let result = executor.execute_local(&ExecutionTask::Inference(task));
        assert!(result.is_err(), "inference without engine should error");
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("inference engine"),
            "error should mention missing engine, got: {msg}"
        );
    }

    #[test]
    fn test_execute_web_fetch_local_makes_request() {
        let executor =
            TaskExecutor::new(RouterConfig::default()).with_http_client(reqwest::Client::new());
        let task = WebFetchTask::get("https://example.com".into());
        let output = executor
            .execute_local(&ExecutionTask::WebFetch(task))
            .unwrap();
        assert!(output.success);
        assert_eq!(output.data["status"], 200);
        let body = output.data["body"].as_str().unwrap();
        assert!(
            body.contains("Example Domain") || body.contains("example"),
            "body should contain expected content, got: {body:.100}"
        );
    }
}
