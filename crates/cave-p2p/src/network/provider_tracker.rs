use std::collections::HashMap;

use crate::network::provider::{ModelOffering, ProviderManifest};

pub struct ProviderTracker {
    providers: HashMap<String, ProviderManifest>,
}

impl ProviderTracker {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn update_provider(&mut self, manifest: ProviderManifest) {
        self.providers
            .insert(manifest.peer_id().to_string(), manifest);
    }

    pub fn get_providers(&self) -> Vec<&ProviderManifest> {
        self.providers.values().collect()
    }

    pub fn find_model(&self, model_name: &str) -> Vec<(&str, &ModelOffering)> {
        self.providers
            .iter()
            .filter_map(|(peer_id, manifest)| {
                manifest
                    .find_model(model_name)
                    .map(|model| (peer_id.as_str(), model))
            })
            .collect()
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn remove_provider(&mut self, peer_id: &str) {
        self.providers.remove(peer_id);
    }

    pub fn cleanup_expired(&mut self, max_age_secs: u64) {
        self.providers
            .retain(|_, manifest| !manifest.is_expired(max_age_secs));
    }
}

impl Default for ProviderTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::provider::{ProviderBackend, ProviderRateLimits};

    #[test]
    fn test_provider_tracker_update_and_get() {
        let mut tracker = ProviderTracker::new();
        assert_eq!(tracker.provider_count(), 0);

        let manifest = ProviderManifest::new("peer1".into(), vec![], ProviderRateLimits::default());
        tracker.update_provider(manifest);

        assert_eq!(tracker.provider_count(), 1);
        assert_eq!(tracker.get_providers().len(), 1);
    }

    #[test]
    fn test_provider_tracker_find_model() {
        let mut tracker = ProviderTracker::new();

        let models = vec![ModelOffering {
            model_name: "llama3.1:8b".into(),
            context_size: 8192,
            price_per_1k_tokens: 5,
            max_tokens_per_request: 4096,
            quantization: None,
            backend: ProviderBackend::Ollama,
        }];

        let manifest = ProviderManifest::new("peer1".into(), models, ProviderRateLimits::default());
        tracker.update_provider(manifest);

        let results = tracker.find_model("llama3.1:8b");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "peer1");

        let results = tracker.find_model("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_provider_tracker_remove() {
        let mut tracker = ProviderTracker::new();
        tracker.update_provider(ProviderManifest::new(
            "peer1".into(),
            vec![],
            ProviderRateLimits::default(),
        ));
        assert_eq!(tracker.provider_count(), 1);

        tracker.remove_provider("peer1");
        assert_eq!(tracker.provider_count(), 0);
    }
}
