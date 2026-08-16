use crate::Result;

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub interactive: bool,
    pub timeout_secs: u64,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            model: None,
            prompt: None,
            interactive: false,
            timeout_secs: 300,
        }
    }
}

pub fn run_inference(config: RunConfig) -> Result<String> {
    let model = config.model.unwrap_or_else(|| "default".to_string());
    let prompt = config.prompt.unwrap_or_default();
    Ok(format!(
        "Inference on model '{}' with prompt '{}'",
        model, prompt
    ))
}

pub fn interactive_chat(model: &str) -> Result<()> {
    tracing::info!("Starting interactive chat with model: {}", model);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_config_default() {
        let config = RunConfig::default();
        assert!(!config.interactive);
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn test_run_inference() {
        let config = RunConfig {
            model: Some("gpt-4".to_string()),
            prompt: Some("Hello".to_string()),
            interactive: false,
            timeout_secs: 60,
        };
        let result = run_inference(config).unwrap();
        assert!(result.contains("gpt-4"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_interactive_chat() {
        let result = interactive_chat("gpt-4");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_inference_defaults() {
        let config = RunConfig::default();
        let result = run_inference(config).unwrap();
        assert!(result.contains("default"));
    }
}
