use std::sync::{Arc, Mutex};

use crate::a2a::A2aState;
use crate::compute_fabric::{LeaseManager, Scheduler, WorkerRegistry};
use crate::events::EventPublisher;
use crate::executor::resource::ResourceMonitor;
use crate::identity::NodeIdentity;
use crate::job::{JobExecutionEngine, JobManager};
use crate::network::provider_tracker::ProviderTracker;
use crate::network::{NetworkEvent, P2pSwarm};
use crate::wallet::Wallet;

pub struct Runtime {
    identity: Arc<NodeIdentity>,
    resource_monitor: Option<Arc<ResourceMonitor>>,
    provider_tracker: Option<Arc<Mutex<ProviderTracker>>>,
    swarm: Option<P2pSwarm>,
    a2a: Arc<A2aState>,
    worker_registry: Option<Arc<std::sync::RwLock<WorkerRegistry>>>,
    scheduler: Option<Scheduler>,
    lease_manager: Option<Arc<std::sync::Mutex<LeaseManager>>>,
    job_manager: Option<Arc<std::sync::Mutex<JobManager>>>,
    job_executor: Option<JobExecutionEngine>,
    wallet: Option<Arc<Mutex<Wallet>>>,
    event_publisher: Option<EventPublisher>,
    started_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub uptime_secs: u64,
    pub peer_count: usize,
    pub provider_count: usize,
    pub resource_available: bool,
}

impl Runtime {
    pub fn new(identity: NodeIdentity) -> Self {
        Self {
            identity: Arc::new(identity),
            resource_monitor: None,
            provider_tracker: None,
            swarm: None,
            a2a: Arc::new(A2aState::new()),
            worker_registry: None,
            scheduler: None,
            lease_manager: None,
            job_manager: None,
            job_executor: None,
            wallet: None,
            event_publisher: None,
            started_at: std::time::Instant::now(),
        }
    }

    pub fn with_resource_monitor(mut self, monitor: ResourceMonitor) -> Self {
        self.resource_monitor = Some(Arc::new(monitor));
        self
    }

    pub fn with_provider_tracker(mut self, tracker: ProviderTracker) -> Self {
        self.provider_tracker = Some(Arc::new(Mutex::new(tracker)));
        self
    }

    pub fn with_worker_registry(mut self, registry: Arc<std::sync::RwLock<WorkerRegistry>>) -> Self {
        self.worker_registry = Some(registry);
        self.scheduler = Some(Scheduler::new(self.worker_registry.clone().unwrap()));
        self
    }

    pub fn with_lease_manager(mut self, manager: Arc<Mutex<LeaseManager>>) -> Self {
        self.lease_manager = Some(manager);
        self
    }

    pub fn with_job_manager(mut self, manager: Arc<Mutex<JobManager>>) -> Self {
        self.job_manager = Some(manager.clone());
        if let Some(registry) = &self.worker_registry {
            let executor = crate::executor::task_executor::TaskExecutor::new(
                crate::executor::router::RouterConfig::default(),
            );
            self.job_executor = Some(JobExecutionEngine::new(manager, registry.clone(), executor));
        }
        self
    }

    pub fn with_wallet(mut self, wallet: Arc<Mutex<Wallet>>) -> Self {
        self.wallet = Some(wallet);
        self
    }

    pub fn with_event_publisher(mut self, publisher: EventPublisher) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    pub fn attach_swarm(&mut self, swarm: P2pSwarm) {
        self.swarm = Some(swarm);
    }

    pub fn a2a(&self) -> &Arc<A2aState> {
        &self.a2a
    }

    pub fn resource_monitor(&self) -> Option<&Arc<ResourceMonitor>> {
        self.resource_monitor.as_ref()
    }

    pub fn provider_tracker(&self) -> Option<&Arc<Mutex<ProviderTracker>>> {
        self.provider_tracker.as_ref()
    }

    pub fn identity(&self) -> &Arc<NodeIdentity> {
        &self.identity
    }

    pub fn swarm(&self) -> Option<&P2pSwarm> {
        self.swarm.as_ref()
    }

    pub fn worker_registry(&self) -> Option<&Arc<std::sync::RwLock<WorkerRegistry>>> {
        self.worker_registry.as_ref()
    }

    pub fn scheduler(&self) -> Option<&Scheduler> {
        self.scheduler.as_ref()
    }

    pub fn lease_manager(&self) -> Option<&Arc<std::sync::Mutex<LeaseManager>>> {
        self.lease_manager.as_ref()
    }

    pub fn job_manager(&self) -> Option<&Arc<std::sync::Mutex<JobManager>>> {
        self.job_manager.as_ref()
    }

    pub fn job_executor(&self) -> Option<&JobExecutionEngine> {
        self.job_executor.as_ref()
    }

    pub fn wallet(&self) -> Option<&Arc<Mutex<Wallet>>> {
        self.wallet.as_ref()
    }

    pub fn event_publisher(&self) -> Option<&EventPublisher> {
        self.event_publisher.as_ref()
    }

    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            uptime_secs: self.started_at.elapsed().as_secs(),
            peer_count: self.a2a.peer_count(),
            provider_count: self
                .provider_tracker
                .as_ref()
                .map(|t| t.lock().unwrap().provider_count())
                .unwrap_or(0),
            resource_available: self.resource_monitor.is_some(),
        }
    }

    pub fn gossip_agent_card(
        &self,
        public_base_url: &str,
    ) -> Option<crate::a2a::AgentCardAnnouncement> {
        let peer_id = self.identity.node_id_string();
        let card = crate::a2a::AgentCard::bkg_p2p_default(&peer_id);
        let card = crate::a2a::AgentCard {
            url: format!("{public_base_url}/.well-known/agent-card.json"),
            ..card
        };
        let announcement =
            crate::a2a::gossip::create_announcement(&peer_id, card, self.identity.keypair());

        if let Some(swarm) = &self.swarm {
            if let Ok(data) = crate::a2a::gossip::encode_announcement(&announcement) {
                let _ = swarm.publish(crate::a2a::A2A_GOSSIP_TOPIC, data);
            }
        }

        Some(announcement)
    }

    pub fn gossip_resource_manifest(&self) {
        let resources = self.resource_monitor.as_ref().map(|m| m.current_state());
        let Some(state) = resources else { return };

        let manifest = crate::network::ResourceManifest::new(
            self.identity.node_id_string(),
            crate::network::Resources::new(
                0,
                0,
                None,
                state.ram_total_mb as u64 * 1024 * 1024,
                0,
                state.ram_available_mb,
            ),
            vec![],
            vec![],
            0,
        );

        if let Some(swarm) = &self.swarm {
            if let Ok(data) = serde_json::to_vec(&manifest) {
                let _ = swarm.publish("bkg-p2p/resources/v1", data);
            }
        }
    }

    pub fn handle_gossip_message(&self, event: &NetworkEvent) {
        if let NetworkEvent::GossipMessage { topic, data, .. } = event {
            match topic.as_str() {
                crate::a2a::A2A_GOSSIP_TOPIC => {
                    let _ = crate::a2a::gossip::handle_agent_card_gossip(&self.a2a, data);
                }
                "bkg-p2p/providers/v1" => {
                    if let Ok(manifest) =
                        serde_json::from_slice::<crate::network::ProviderManifest>(data)
                    {
                        if let Some(tracker) = &self.provider_tracker {
                            if let Ok(mut tracker) = tracker.lock() {
                                tracker.update_provider(manifest);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn subscribe_to_job_topics(&self) {
        let topics = [
            crate::a2a::A2A_GOSSIP_TOPIC,
            "bkg-p2p/resources/v1",
            "bkg-p2p/providers/v1",
            "bkg-p2p/tools/v1",
            "bkg-p2p/skills/v1",
        ];

        if let Some(swarm) = &self.swarm {
            for topic in &topics {
                let _ = swarm.subscribe(topic);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let identity = NodeIdentity::generate().unwrap();
        let runtime = Runtime::new(identity);
        let stats = runtime.stats();
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.provider_count, 0);
        assert!(!stats.resource_available);
    }

    #[test]
    fn test_runtime_stats() {
        let identity = NodeIdentity::generate().unwrap();
        let runtime = Runtime::new(identity);
        let stats = runtime.stats();
        assert!(stats.uptime_secs < 5);
    }

    #[test]
    fn test_runtime_with_resource_monitor() {
        let identity = NodeIdentity::generate().unwrap();
        let monitor = ResourceMonitor::with_defaults();
        let runtime = Runtime::new(identity).with_resource_monitor(monitor);
        assert!(runtime.resource_monitor().is_some());
        assert!(runtime.stats().resource_available);
    }

    #[test]
    fn test_runtime_gossip_agent_card() {
        let identity = NodeIdentity::generate().unwrap();
        let runtime = Runtime::new(identity);
        let result = runtime.gossip_agent_card("http://localhost:8080");
        assert!(result.is_some());
        let announcement = result.unwrap();
        assert!(!announcement.signature.is_empty());
    }

    #[test]
    fn test_runtime_with_provider_tracker() {
        let identity = NodeIdentity::generate().unwrap();
        let runtime = Runtime::new(identity).with_provider_tracker(ProviderTracker::new());
        assert!(runtime.provider_tracker().is_some());
        assert_eq!(runtime.stats().provider_count, 0);
    }

    #[test]
    fn test_runtime_with_worker_registry() {
        let identity = NodeIdentity::generate().unwrap();
        let registry = Arc::new(std::sync::RwLock::new(WorkerRegistry::new()));
        let runtime = Runtime::new(identity).with_worker_registry(registry);
        assert!(runtime.worker_registry().is_some());
        assert!(runtime.scheduler().is_some());
    }

    #[test]
    fn test_runtime_with_lease_manager() {
        let identity = NodeIdentity::generate().unwrap();
        let manager = Arc::new(Mutex::new(LeaseManager::new()));
        let runtime = Runtime::new(identity).with_lease_manager(manager);
        assert!(runtime.lease_manager().is_some());
    }

    #[test]
    fn test_runtime_with_job_manager() {
        let identity = NodeIdentity::generate().unwrap();
        let registry = Arc::new(std::sync::RwLock::new(WorkerRegistry::new()));
        let manager = Arc::new(Mutex::new(JobManager::new()));
        let runtime = Runtime::new(identity)
            .with_worker_registry(registry)
            .with_job_manager(manager);
        assert!(runtime.job_manager().is_some());
        assert!(runtime.job_executor().is_some());
    }

    #[test]
    fn test_runtime_with_wallet() {
        let identity = NodeIdentity::generate().unwrap();
        let config = crate::wallet::WalletConfig::default();
        let wallet = Arc::new(Mutex::new(Wallet::new(config, 1000)));
        let runtime = Runtime::new(identity).with_wallet(wallet);
        assert!(runtime.wallet().is_some());
    }

    #[test]
    fn test_runtime_with_event_publisher() {
        let identity = NodeIdentity::generate().unwrap();
        let bus = std::sync::Arc::new(cave_core::EventBus::new());
        let publisher = EventPublisher::new(bus);
        let runtime = Runtime::new(identity).with_event_publisher(publisher);
        assert!(runtime.event_publisher().is_some());
    }
}
