use serde::{Deserialize, Serialize};

pub const PROVIDER_TOPIC: &str = "bkg-p2p/providers/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderManifest {
    peer_id: String,
    models: Vec<ModelOffering>,
    rate_limits: ProviderRateLimits,
    timestamp: u64,
    signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOffering {
    pub model_name: String,
    pub context_size: u32,
    pub price_per_1k_tokens: u64,
    pub max_tokens_per_request: u32,
    pub quantization: Option<String>,
    pub backend: ProviderBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderBackend {
    Ollama,
    Gguf,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRateLimits {
    pub max_requests_per_hour: u32,
    pub max_tokens_per_day: u64,
    pub max_concurrent_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSharingConfig {
    pub enabled: bool,
    pub advertise_models: bool,
    pub max_requests_per_hour: u32,
    pub max_tokens_per_day: u64,
    pub max_concurrent_requests: u32,
    pub price_multiplier: f64,
}

impl Default for ProviderSharingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            advertise_models: true,
            max_requests_per_hour: 100,
            max_tokens_per_day: 10_000_000,
            max_concurrent_requests: 2,
            price_multiplier: 1.0,
        }
    }
}

impl Default for ProviderRateLimits {
    fn default() -> Self {
        Self {
            max_requests_per_hour: 100,
            max_tokens_per_day: 10_000_000,
            max_concurrent_requests: 2,
        }
    }
}

impl ProviderManifest {
    pub fn new(
        peer_id: String,
        models: Vec<ModelOffering>,
        rate_limits: ProviderRateLimits,
    ) -> Self {
        Self {
            peer_id,
            models,
            rate_limits,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: Vec::new(),
        }
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn models(&self) -> &[ModelOffering] {
        &self.models
    }

    pub fn rate_limits(&self) -> &ProviderRateLimits {
        &self.rate_limits
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    pub fn is_expired(&self, max_age_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.timestamp) > max_age_secs
    }

    pub fn has_model(&self, model_name: &str) -> bool {
        self.models.iter().any(|m| m.model_name == model_name)
    }

    pub fn find_model(&self, model_name: &str) -> Option<&ModelOffering> {
        self.models.iter().find(|m| m.model_name == model_name)
    }

    pub fn signing_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct SigningFields<'a> {
            peer_id: &'a str,
            model_count: usize,
            timestamp: u64,
        }
        let fields = SigningFields {
            peer_id: &self.peer_id,
            model_count: self.models.len(),
            timestamp: self.timestamp,
        };
        serde_json::to_vec(&fields).unwrap_or_default()
    }

    pub fn sign(&mut self, keypair: &crate::identity::NodeKeypair) {
        let bytes = self.signing_bytes();
        self.signature = keypair.sign(&bytes).to_vec();
    }

    pub fn verify(&self, public_key: &[u8; 32]) -> crate::identity::Result<()> {
        let bytes = self.signing_bytes();
        let sig: [u8; 64] = self
            .signature
            .clone()
            .try_into()
            .map_err(|_| crate::Error::Identity("invalid signature length".into()))?;
        crate::identity::NodeKeypair::verify(&bytes, &sig, public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_manifest_creation() {
        let models = vec![ModelOffering {
            model_name: "llama3.1:8b".into(),
            context_size: 8192,
            price_per_1k_tokens: 5,
            max_tokens_per_request: 4096,
            quantization: Some("Q4_K_M".into()),
            backend: ProviderBackend::Ollama,
        }];

        let limits = ProviderRateLimits {
            max_requests_per_hour: 100,
            max_tokens_per_day: 1_000_000,
            max_concurrent_requests: 2,
        };

        let manifest = ProviderManifest::new("12D3KooWABC".into(), models, limits);
        assert_eq!(manifest.peer_id(), "12D3KooWABC");
        assert_eq!(manifest.models().len(), 1);
        assert!(manifest.has_model("llama3.1:8b"));
        assert!(manifest.find_model("llama3.1:8b").is_some());
    }

    #[test]
    fn test_provider_manifest_serialization() {
        let models = vec![ModelOffering {
            model_name: "llama3.1:8b".into(),
            context_size: 8192,
            price_per_1k_tokens: 5,
            max_tokens_per_request: 4096,
            quantization: None,
            backend: ProviderBackend::Gguf,
        }];

        let limits = ProviderRateLimits::default();
        let manifest = ProviderManifest::new("peer1".into(), models, limits);

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: ProviderManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.peer_id(), "peer1");
        assert_eq!(deserialized.models().len(), 1);
    }

    #[test]
    fn test_provider_manifest_expiry() {
        let manifest = ProviderManifest::new("peer1".into(), vec![], ProviderRateLimits::default());
        assert!(!manifest.is_expired(86400));
    }

    #[test]
    fn test_provider_manifest_sign_and_verify() {
        let keypair = crate::identity::NodeKeypair::generate().unwrap();
        let mut manifest = ProviderManifest::new(
            crate::identity::NodeId::from_public_key(&keypair.public_key()).to_base58(),
            vec![],
            ProviderRateLimits::default(),
        );

        manifest.sign(&keypair);
        assert!(!manifest.signature().is_empty());

        let pub_key = keypair.public_bytes();
        assert!(manifest.verify(&pub_key).is_ok());

        let wrong = crate::identity::NodeKeypair::generate().unwrap();
        assert!(manifest.verify(&wrong.public_bytes()).is_err());
    }

    #[test]
    fn test_provider_sharing_config_defaults() {
        let config = ProviderSharingConfig::default();
        assert!(!config.enabled);
        assert!(config.advertise_models);
        assert_eq!(config.price_multiplier, 1.0);
    }
}
