//! Network core for cave-p2p.
//!
//! Provides the P2P networking layer based on libp2p.
//! All addresses/ports come from `bkg.conf` via `Config`.

pub mod behaviour;
pub mod events;
pub mod gossip;
pub mod identify;
pub mod kad;
pub mod provider;
pub mod provider_tracker;
pub mod resource;
pub mod swarm;
pub mod tool_manifest;
pub mod transport;

pub use behaviour::{BehaviourConfig, CaveBehaviour};
pub use events::NetworkEvent;
pub use provider::{
    ModelOffering, ProviderBackend, ProviderManifest, ProviderRateLimits, ProviderSharingConfig,
};
pub use provider_tracker::ProviderTracker;
pub use resource::{Capability, GpuInfo, GpuVendor, ResourceManifest, Resources};
pub use swarm::P2pSwarm;
pub use tool_manifest::{ToolManifest, ToolManifestAdvert};
pub use transport::{
    build_bootstrap_peers, build_listen_addresses, resolve_web_listen_addr,
    validate_transport_config, validate_web_config, TransportError,
};

use libp2p::multiaddr::Multiaddr;
use tokio::sync::broadcast;

use crate::config::{P2pConfig, WebConfig};
use crate::identity::NodeIdentity;

/// Errors that can occur in the network layer.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("network error: {0}")]
    General(String),

    #[error("swarm error: {0}")]
    Swarm(String),

    #[error("transport error: {0}")]
    Transport(String),

    #[error("behaviour error: {0}")]
    Behaviour(String),
}

impl From<NetworkError> for crate::Error {
    fn from(err: NetworkError) -> Self {
        crate::Error::Network(err.to_string())
    }
}

impl From<TransportError> for NetworkError {
    fn from(err: TransportError) -> Self {
        NetworkError::Transport(err.to_string())
    }
}

/// Network service wrapping the libp2p Swarm.
///
/// Provides:
/// - Start/stop lifecycle
/// - Event emission via event channel
/// - Peer management
/// - GossipSub topic subscription/publication
///
/// All addresses/ports come from `bkg.conf`, never hardcoded.
pub struct NetworkService {
    identity: NodeIdentity,
    p2p_config: P2pConfig,
    web_config: WebConfig,
    #[allow(dead_code)]
    behaviour_config: BehaviourConfig,
    swarm: Option<P2pSwarm>,
    event_rx: Option<broadcast::Receiver<self::events::NetworkEvent>>,
}

impl NetworkService {
    /// Create a new network service from identity and config.
    pub fn new(
        identity: NodeIdentity,
        p2p_config: P2pConfig,
        web_config: WebConfig,
        #[allow(dead_code)] behaviour_config: BehaviourConfig,
    ) -> Result<Self, NetworkError> {
        validate_transport_config(&p2p_config)?;
        if web_config.enabled {
            validate_web_config(&web_config)?;
        }
        Ok(Self {
            identity,
            p2p_config,
            web_config,
            behaviour_config,
            swarm: None,
            event_rx: None,
        })
    }

    /// Create from a full Config (convenience constructor).
    pub fn from_config(
        identity: NodeIdentity,
        config: &crate::Config,
    ) -> Result<Self, NetworkError> {
        let behaviour_config = BehaviourConfig::new(identity.clone(), config.p2p.clone());
        Self::new(
            identity,
            config.p2p.clone(),
            config.web.clone(),
            behaviour_config,
        )
    }

    /// Get the node's identity.
    pub fn identity(&self) -> &NodeIdentity {
        &self.identity
    }

    /// Get the P2P configuration (from bkg.conf).
    pub fn p2p_config(&self) -> &P2pConfig {
        &self.p2p_config
    }

    /// Get the web configuration (from bkg.conf).
    pub fn web_config(&self) -> &WebConfig {
        &self.web_config
    }

    /// Resolved listen addresses (from config, never hardcoded).
    /// Returns exactly what is in bkg.conf — no fallback, no defaults.
    pub fn listen_addresses(&self) -> Vec<String> {
        build_listen_addresses(&self.p2p_config)
    }

    /// Bootstrap peers (from config).
    pub fn bootstrap_peers(&self) -> Vec<String> {
        build_bootstrap_peers(&self.p2p_config)
    }

    /// Resolved web listen address (from config, never hardcoded).
    /// Returns exactly what is in bkg.conf — no fallback, no defaults.
    pub fn web_listen_addr(&self) -> String {
        resolve_web_listen_addr(&self.web_config)
    }

    /// Whether the network service is running.
    pub fn is_running(&self) -> bool {
        self.swarm.is_some()
    }

    /// Start the network service.
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        if self.swarm.is_some() {
            return Err(NetworkError::Swarm("already running".to_string()));
        }

        let listen_multiaddrs: Vec<Multiaddr> = self
            .p2p_config
            .listen_addresses
            .iter()
            .filter_map(|s| s.parse::<Multiaddr>().ok())
            .collect();

        if listen_multiaddrs.is_empty() {
            return Err(NetworkError::Transport(
                "no valid listen addresses in config".to_string(),
            ));
        }

        let (swarm, event_rx) = P2pSwarm::new(self.identity.clone(), listen_multiaddrs)?;

        for peer_str in &self.p2p_config.bootstrap_peers {
            if let Ok(addr) = peer_str.parse::<Multiaddr>() {
                let _ = swarm.dial(addr);
            }
        }

        self.swarm = Some(swarm);
        self.event_rx = Some(event_rx);
        Ok(())
    }

    /// Stop the network service.
    pub async fn stop(&mut self) -> Result<(), NetworkError> {
        self.swarm.take();
        self.event_rx.take();
        Ok(())
    }

    /// Subscribe to a GossipSub topic.
    pub fn subscribe(&self, topic: &str) -> Result<(), NetworkError> {
        self.swarm
            .as_ref()
            .ok_or_else(|| NetworkError::Swarm("not running".to_string()))?
            .subscribe(topic)
    }

    /// Unsubscribe from a GossipSub topic.
    pub fn unsubscribe(&self, topic: &str) -> Result<(), NetworkError> {
        self.swarm
            .as_ref()
            .ok_or_else(|| NetworkError::Swarm("not running".to_string()))?
            .unsubscribe(topic)
    }

    /// Publish a message to a GossipSub topic.
    pub fn publish(&self, topic: &str, data: Vec<u8>) -> Result<(), NetworkError> {
        self.swarm
            .as_ref()
            .ok_or_else(|| NetworkError::Swarm("not running".to_string()))?
            .publish(topic, data)
    }

    /// Dial a peer at the given multiaddress.
    pub fn dial(&self, addr: Multiaddr) -> Result<(), NetworkError> {
        self.swarm
            .as_ref()
            .ok_or_else(|| NetworkError::Swarm("not running".to_string()))?
            .dial(addr)
    }

    /// Get a receiver for network events.
    pub fn event_receiver(
        &self,
    ) -> Result<broadcast::Receiver<self::events::NetworkEvent>, NetworkError> {
        self.event_rx
            .as_ref()
            .map(|rx| rx.resubscribe())
            .ok_or_else(|| NetworkError::Swarm("not running".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_network_service() {
        let identity = NodeIdentity::generate().unwrap();
        let mut p2p_config = P2pConfig::default();
        p2p_config.listen_addresses = vec!["/ip4/0.0.0.0/tcp/7777".to_string()];
        p2p_config.tcp_enabled = true;
        let web_config = WebConfig::default();
        let behaviour_config = BehaviourConfig::new(identity.clone(), p2p_config.clone());
        let service = NetworkService::new(identity, p2p_config, web_config, behaviour_config);
        assert!(service.is_ok());
    }

    #[test]
    fn listen_addresses_from_config() {
        let identity = NodeIdentity::generate().unwrap();
        let mut p2p_config = P2pConfig::default();
        p2p_config.listen_addresses = vec!["/ip4/0.0.0.0/tcp/7777".to_string()];
        p2p_config.tcp_enabled = true;
        let web_config = WebConfig::default();
        let behaviour_config = BehaviourConfig::new(identity.clone(), p2p_config.clone());
        let service =
            NetworkService::new(identity, p2p_config, web_config, behaviour_config).unwrap();
        let addrs = service.listen_addresses();
        assert_eq!(addrs[0], "/ip4/0.0.0.0/tcp/7777");
    }

    #[test]
    fn network_service_validates_transport() {
        let identity = NodeIdentity::generate().unwrap();
        let p2p_config = P2pConfig::default();
        let web_config = WebConfig::default();
        let behaviour_config = BehaviourConfig::new(identity.clone(), p2p_config.clone());
        let service = NetworkService::new(identity, p2p_config, web_config, behaviour_config);
        assert!(service.is_err());
    }

    #[test]
    fn web_addr_from_config() {
        let identity = NodeIdentity::generate().unwrap();
        let mut p2p_config = P2pConfig::default();
        p2p_config.listen_addresses = vec!["/ip4/0.0.0.0/tcp/4001".to_string()];
        p2p_config.tcp_enabled = true;
        let web_config = WebConfig {
            listen_addr: "0.0.0.0:3000".to_string(),
            enabled: true,
        };
        let behaviour_config = BehaviourConfig::new(identity.clone(), p2p_config.clone());
        let service =
            NetworkService::new(identity, p2p_config, web_config, behaviour_config).unwrap();
        assert_eq!(service.web_listen_addr(), "0.0.0.0:3000");
    }
}
