use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifest {
    peer_id: String,
    timestamp: u64,
    signature: Vec<u8>,
    resources: Resources,
    capabilities: Vec<Capability>,
    supported_models: Vec<String>,
    uptime_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    cpu_cores: u16,
    cpu_available_mhz: u32,
    gpu: Option<GpuInfo>,
    storage_available_bytes: u64,
    bandwidth_mbps: u32,
    ram_available_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    vendor: GpuVendor,
    vram_mb: u32,
    compute_capability: String,
    model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Apple,
    Intel,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Capability {
    WasmSandbox,
    Inference,
    Storage,
    Relay,
    WebProxy,
    LlmProvider,
}

impl ResourceManifest {
    pub fn new(
        peer_id: String,
        resources: Resources,
        capabilities: Vec<Capability>,
        supported_models: Vec<String>,
        uptime_hours: u64,
    ) -> Self {
        Self {
            peer_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: Vec::new(),
            resources,
            capabilities,
            supported_models,
            uptime_hours,
        }
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    pub fn supported_models(&self) -> &[String] {
        &self.supported_models
    }

    pub fn uptime_hours(&self) -> u64 {
        self.uptime_hours
    }

    pub fn signing_bytes(&self) -> Vec<u8> {
        let signing = SigningFields {
            peer_id: &self.peer_id,
            timestamp: self.timestamp,
            resources: &self.resources,
            capabilities: &self.capabilities,
            supported_models: &self.supported_models,
            uptime_hours: self.uptime_hours,
        };
        serde_json::to_vec(&signing).unwrap_or_default()
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

#[derive(Serialize)]
struct SigningFields<'a> {
    peer_id: &'a str,
    timestamp: u64,
    resources: &'a Resources,
    capabilities: &'a [Capability],
    supported_models: &'a [String],
    uptime_hours: u64,
}

impl Resources {
    pub fn new(
        cpu_cores: u16,
        cpu_available_mhz: u32,
        gpu: Option<GpuInfo>,
        storage_available_bytes: u64,
        bandwidth_mbps: u32,
        ram_available_mb: u32,
    ) -> Self {
        Self {
            cpu_cores,
            cpu_available_mhz,
            gpu,
            storage_available_bytes,
            bandwidth_mbps,
            ram_available_mb,
        }
    }

    pub fn cpu_cores(&self) -> u16 {
        self.cpu_cores
    }

    pub fn cpu_available_mhz(&self) -> u32 {
        self.cpu_available_mhz
    }

    pub fn gpu(&self) -> Option<&GpuInfo> {
        self.gpu.as_ref()
    }

    pub fn storage_available_bytes(&self) -> u64 {
        self.storage_available_bytes
    }

    pub fn bandwidth_mbps(&self) -> u32 {
        self.bandwidth_mbps
    }

    pub fn ram_available_mb(&self) -> u32 {
        self.ram_available_mb
    }
}

impl GpuInfo {
    pub fn new(
        vendor: GpuVendor,
        vram_mb: u32,
        compute_capability: String,
        model_name: String,
    ) -> Self {
        Self {
            vendor,
            vram_mb,
            compute_capability,
            model_name,
        }
    }

    pub fn vendor(&self) -> &GpuVendor {
        &self.vendor
    }

    pub fn vram_mb(&self) -> u32 {
        self.vram_mb
    }

    pub fn compute_capability(&self) -> &str {
        &self.compute_capability
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manifest_creation() {
        let resources = Resources::new(8, 3200, None, 500_000_000_000, 1000, 16_000);
        let manifest = ResourceManifest::new(
            "12D3KooWABC".into(),
            resources,
            vec![Capability::Inference, Capability::Storage],
            vec!["llama3.1:8b".into()],
            24,
        );

        assert_eq!(manifest.peer_id(), "12D3KooWABC");
        assert_eq!(manifest.uptime_hours(), 24);
        assert_eq!(manifest.capabilities().len(), 2);
        assert_eq!(manifest.supported_models().len(), 1);
        assert!(manifest.timestamp() > 0);
        assert!(manifest.signature().is_empty());
    }

    #[test]
    fn test_manifest_signing_roundtrip() {
        let keypair = crate::identity::NodeKeypair::generate().unwrap();
        let peer_id = crate::identity::NodeId::from_public_key(&keypair.public_key()).to_base58();

        let resources = Resources::new(4, 2400, None, 100_000_000_000, 500, 8_000);
        let mut manifest =
            ResourceManifest::new(peer_id, resources, vec![Capability::WasmSandbox], vec![], 0);

        manifest.sign(&keypair);
        assert!(!manifest.signature().is_empty());

        let public_key = keypair.public_bytes();
        assert!(manifest.verify(&public_key).is_ok());
    }

    #[test]
    fn test_manifest_verify_rejects_wrong_key() {
        let keypair = crate::identity::NodeKeypair::generate().unwrap();
        let peer_id = crate::identity::NodeId::from_public_key(&keypair.public_key()).to_base58();

        let resources = Resources::new(4, 2400, None, 100_000_000_000, 500, 8_000);
        let mut manifest = ResourceManifest::new(peer_id, resources, vec![], vec![], 0);
        manifest.sign(&keypair);

        let wrong = crate::identity::NodeKeypair::generate().unwrap();
        assert!(manifest.verify(&wrong.public_bytes()).is_err());
    }

    #[test]
    fn test_resources_with_gpu() {
        let gpu = GpuInfo::new(
            GpuVendor::Nvidia,
            24_000,
            "8.9".into(),
            "NVIDIA GeForce RTX 4090".into(),
        );

        let resources = Resources::new(16, 4800, Some(gpu), 2_000_000_000_000, 2500, 64_000);
        assert_eq!(resources.cpu_cores(), 16);
        let gpu_ref = resources.gpu().unwrap();
        assert_eq!(gpu_ref.vram_mb(), 24_000);
        assert_eq!(gpu_ref.compute_capability(), "8.9");
        assert_eq!(gpu_ref.model_name(), "NVIDIA GeForce RTX 4090");
    }
}
