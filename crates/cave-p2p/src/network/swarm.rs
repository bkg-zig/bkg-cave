//! libp2p Swarm management for cave-p2p.
//!
//! This module provides the Swarm type that wraps libp2p's Swarm
//! and manages the P2P network lifecycle.

use std::time::Duration;

use crate::identity::NodeIdentity;
use crate::network::events::NetworkEvent;
use crate::network::{BehaviourConfig, CaveBehaviour, NetworkError};

/// Configuration for the P2P swarm.
#[derive(Debug, Clone)]
pub struct SwarmConfig {
    /// Node identity.
    pub identity: NodeIdentity,
    /// Listen addresses.
    pub listen_addresses: Vec<String>,
    /// Bootstrap peers.
    pub bootstrap_peers: Vec<String>,
    /// Enable Kademlia DHT.
    pub enable_kademlia: bool,
    /// Enable GossipSub.
    pub enable_gossipsub: bool,
    /// Enable mDNS.
    pub enable_mdns: bool,
    /// Enable Identify protocol.
    pub enable_identify: bool,
    /// Connection idle timeout.
    pub connection_idle_timeout: Duration,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            identity: NodeIdentity::generate().expect("failed to generate identity"),
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
            bootstrap_peers: Vec::new(),
            enable_kademlia: true,
            enable_gossipsub: true,
            enable_mdns: true,
            enable_identify: true,
            connection_idle_timeout: Duration::from_secs(60),
        }
    }
}

impl SwarmConfig {
    /// Create a new swarm configuration.
    pub fn new(identity: NodeIdentity) -> Self {
        Self {
            identity,
            ..Default::default()
        }
    }

    /// Set listen addresses.
    pub fn with_listen_addresses(mut self, addresses: Vec<String>) -> Self {
        self.listen_addresses = addresses;
        self
    }

    /// Set bootstrap peers.
    pub fn with_bootstrap_peers(mut self, peers: Vec<String>) -> Self {
        self.bootstrap_peers = peers;
        self
    }

    /// Disable Kademlia.
    pub fn without_kademlia(mut self) -> Self {
        self.enable_kademlia = false;
        self
    }

    /// Disable GossipSub.
    pub fn without_gossipsub(mut self) -> Self {
        self.enable_gossipsub = false;
        self
    }

    /// Disable mDNS.
    pub fn without_mdns(mut self) -> Self {
        self.enable_mdns = false;
        self
    }

    /// Disable Identify.
    pub fn without_identify(mut self) -> Self {
        self.enable_identify = false;
        self
    }
}

/// Placeholder for the P2P Swarm.
///
/// In M2.2+, this will wrap `libp2p::Swarm<CaveBehaviour>` and provide:
/// - Event loop
/// - Peer management
/// - GossipSub topic management
/// - Kademlia operations
/// - mDNS discovery
///
/// For now, it's a minimal stub to allow the plugin to compile.
pub struct Swarm {
    #[allow(dead_code)]
    config: SwarmConfig,
    #[allow(dead_code)]
    behaviour: CaveBehaviour,
}

impl Swarm {
    /// Create a new swarm from configuration.
    pub fn new(config: SwarmConfig) -> Result<Self, NetworkError> {
        let behaviour_config = BehaviourConfig::new(config.identity.clone())
            .with_gossipsub_validation(libp2p::gossipsub::ValidationMode::Strict);

        let behaviour = CaveBehaviour::new(behaviour_config)
            .map_err(|e| NetworkError::Behaviour(e.to_string()))?;

        Ok(Self {
            config,
            behaviour,
        })
    }

    /// Start the swarm.
    ///
    /// This is a placeholder for M2.2+. Full implementation will:
    /// 1. Create libp2p Swarm with transport and behaviour
    /// 2. Start listening on configured addresses
    /// 3. Dial bootstrap peers
    /// 4. Spawn event loop that emits NetworkEvents
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Placeholder: in M2.2+ this will start the actual swarm
        Ok(())
    }

    /// Stop the swarm.
    pub async fn stop(&mut self) -> Result<(), NetworkError> {
        // Placeholder: in M2.2+ this will stop the swarm
        Ok(())
    }

    /// Get the local peer ID.
    pub fn local_peer_id(&self) -> libp2p::PeerId {
        libp2p::PeerId::from(
            self.config
                .identity
                .to_libp2p_keypair()
                .expect("failed to convert identity to libp2p keypair")
                .public(),
        )
    }

    /// Subscribe to a GossipSub topic.
    pub fn subscribe_topic(&mut self, _topic: &str) -> Result<(), NetworkError> {
        // Placeholder: in M2.2+ this will subscribe to the topic
        Ok(())
    }

    /// Publish a message to a GossipSub topic.
    pub fn publish(&mut self, _topic: &str, _data: Vec<u8>) -> Result<(), NetworkError> {
        // Placeholder: in M2.2+ this will publish to the topic
        Ok(())
    }

    /// Get connected peers.
    pub fn connected_peers(&self) -> Vec<libp2p::PeerId> {
        // Placeholder: in M2.2+ this will return actual connected peers
        Vec::new()
    }

    /// Dial a peer.
    pub fn dial(&mut self, _addr: &str) -> Result<(), NetworkError> {
        // Placeholder: in M2.2+ this will dial the peer
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_swarm() {
        let config = SwarmConfig::default();
        let swarm = Swarm::new(config);
        assert!(swarm.is_ok());
    }

    #[test]
    fn swarm_has_peer_id() {
        let config = SwarmConfig::default();
        let swarm = Swarm::new(config).unwrap();
        let peer_id = swarm.local_peer_id();
        assert!(!peer_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn swarm_start_stop() {
        let config = SwarmConfig::default();
        let mut swarm = Swarm::new(config).unwrap();
        assert!(swarm.start().await.is_ok());
        assert!(swarm.stop().await.is_ok());
    }

    #[test]
    fn custom_config() {
        let identity = NodeIdentity::generate().unwrap();
        let config = SwarmConfig::new(identity)
            .with_listen_addresses(vec!["/ip4/0.0.0.0/tcp/8080".to_string()])
            .with_bootstrap_peers(vec!["/ip4/1.2.3.4/tcp/8080/p2p/12D3KooW".to_string()])
            .without_kademlia()
            .without_mdns();

        assert_eq!(config.listen_addresses.len(), 1);
        assert_eq!(config.bootstrap_peers.len(), 1);
        assert!(!config.enable_kademlia);
        assert!(!config.enable_mdns);
    }
}
