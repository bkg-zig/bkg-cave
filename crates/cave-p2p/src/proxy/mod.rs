#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub listen_addr: String,
    pub upstream_addr: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_addr: "127.0.0.1:8081".to_string(),
            upstream_addr: "127.0.0.1:8080".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_default() {
        let config = ProxyConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.listen_addr, "127.0.0.1:8081");
        assert_eq!(config.upstream_addr, "127.0.0.1:8080");
    }

    #[test]
    fn test_proxy_config_custom() {
        let config = ProxyConfig {
            enabled: true,
            listen_addr: "0.0.0.0:9090".to_string(),
            upstream_addr: "10.0.0.1:8080".to_string(),
        };
        assert!(config.enabled);
        assert_eq!(config.listen_addr, "0.0.0.0:9090");
        assert_eq!(config.upstream_addr, "10.0.0.1:8080");
    }
}
