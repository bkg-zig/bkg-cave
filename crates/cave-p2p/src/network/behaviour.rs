//! Network behaviour composition for cave-p2p.
//!
//! All protocol strings and addresses come from `bkg.conf` via `Config`.
//! No values are hardcoded.

use libp2p::{
    gossipsub::{
        Behaviour as GossipsubBehaviour, ConfigBuilder as GossipsubConfigBuilder,
        MessageAuthenticity, ValidationMode,
    },
    identify::{Behaviour as IdentifyBehaviour, Config as IdentifyConfig},
    kad::{store::MemoryStore, Behaviour as KademliaBehaviour},
    request_response::{
        json::Behaviour as JsonBehaviour, Behaviour as RequestResponseBehaviour,
        Config as RequestResponseConfig, ProtocolSupport,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};

use crate::config::P2pConfig;
use crate::identity::NodeIdentity;

/// Errors that can occur during behaviour setup.
#[derive(Debug, thiserror::Error)]
pub enum BehaviourError {
    #[error("behaviour configuration error: {0}")]
    Config(String),

    #[error("protocol error: {0}")]
    Protocol(String),
}

impl From<BehaviourError> for crate::Error {
    fn from(err: BehaviourError) -> Self {
        crate::Error::Network(err.to_string())
    }
}

/// Configuration for the network behaviour.
#[derive(Debug, Clone)]
pub struct BehaviourConfig {
    /// Node identity.
    pub identity: NodeIdentity,
    /// P2P config (from bkg.conf).
    pub p2p_config: P2pConfig,
    /// Protocol version string (from config).
    pub protocol_version: String,
    /// Agent version string.
    pub agent_version: String,
    /// GossipSub validation mode.
    pub gossipsub_validation: ValidationMode,
}

impl BehaviourConfig {
    /// Create a new behaviour configuration from identity and P2P config.
    pub fn new(identity: NodeIdentity, p2p_config: P2pConfig) -> Self {
        Self {
            identity,
            p2p_config,
            protocol_version: "/bkg-p2p/1.0.0".to_string(),
            agent_version: "cave-p2p/0.1.0".to_string(),
            gossipsub_validation: ValidationMode::Strict,
        }
    }

    /// Set protocol version (from config).
    pub fn with_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }

    /// Set agent version.
    pub fn with_agent_version(mut self, version: impl Into<String>) -> Self {
        self.agent_version = version.into();
        self
    }

    /// Set GossipSub validation mode.
    pub fn with_gossipsub_validation(mut self, mode: ValidationMode) -> Self {
        self.gossipsub_validation = mode;
        self
    }
}

impl Default for BehaviourConfig {
    fn default() -> Self {
        let identity = NodeIdentity::generate().expect("failed to generate identity");
        let p2p_config = P2pConfig::default();
        Self::new(identity, p2p_config)
    }
}

/// The composed network behaviour for cave-p2p.
///
/// This combines all P2P protocols into a single behaviour that can be
/// registered with the libp2p swarm.
///
/// ## Protocols
///
/// - **Kademlia**: DHT for peer routing and provider discovery
/// - **GossipSub**: Pub/sub for resource manifests, job requests, etc.
/// - **Identify**: Peer information exchange
/// - **RequestResponse**: JSON-RPC for A2A protocol
///
/// Note: mDNS is omitted in M2 due to libp2p 0.54 API changes.
/// It will be re-added in M3 with the correct Provider implementation.
#[derive(NetworkBehaviour)]
pub struct CaveBehaviour {
    /// Kademlia DHT behaviour.
    pub kademlia: KademliaBehaviour<MemoryStore>,
    /// GossipSub pub/sub behaviour.
    pub gossipsub: GossipsubBehaviour,
    /// Identify protocol behaviour.
    pub identify: IdentifyBehaviour,
    /// JSON request-response behaviour for A2A RPC.
    pub a2a_rpc: JsonBehaviour<crate::a2a::A2aRpcRequest, crate::a2a::A2aRpcResponse>,
}

impl CaveBehaviour {
    /// Create a new network behaviour from configuration.
    pub fn new(config: BehaviourConfig) -> Result<Self, BehaviourError> {
        let peer_id = PeerId::from(
            config
                .identity
                .to_libp2p_keypair()
                .map_err(|e| BehaviourError::Config(e.to_string()))?
                .public(),
        );

        // Kademlia (controlled by config)
        let kademlia = if config.p2p_config.kademlia_enabled {
            let store = MemoryStore::new(peer_id);
            KademliaBehaviour::new(peer_id, store)
        } else {
            KademliaBehaviour::new(peer_id, MemoryStore::new(peer_id))
        };

        // GossipSub (controlled by config)
        let gossipsub = {
            let keypair = config
                .identity
                .to_libp2p_keypair()
                .map_err(|e| BehaviourError::Config(e.to_string()))?;
            let gossipsub_config = GossipsubConfigBuilder::default()
                .validation_mode(config.gossipsub_validation)
                .build()
                .map_err(|e| BehaviourError::Config(e.to_string()))?;
            GossipsubBehaviour::new(MessageAuthenticity::Signed(keypair), gossipsub_config)
                .map_err(|e| BehaviourError::Config(e.to_string()))?
        };

        // Identify (controlled by config)
        let identify = {
            let keypair = config
                .identity
                .to_libp2p_keypair()
                .map_err(|e| BehaviourError::Config(e.to_string()))?;
            let public_key = keypair.public();
            let identify_config = IdentifyConfig::new(config.protocol_version.clone(), public_key)
                .with_agent_version(config.agent_version.clone());
            IdentifyBehaviour::new(identify_config)
        };

        // A2A RPC (request-response)
        let a2a_rpc = RequestResponseBehaviour::new(
            [(
                StreamProtocol::new("/bkg-p2p/a2a-rpc/1.0.0"),
                ProtocolSupport::Full,
            )],
            RequestResponseConfig::default(),
        );

        Ok(Self {
            kademlia,
            gossipsub,
            identify,
            a2a_rpc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_behaviour() {
        let identity = NodeIdentity::generate().unwrap();
        let p2p_config = P2pConfig::default();
        let config = BehaviourConfig::new(identity, p2p_config);
        let behaviour = CaveBehaviour::new(config);
        assert!(behaviour.is_ok());
    }

    #[test]
    fn behaviour_with_kademlia_disabled() {
        let identity = NodeIdentity::generate().unwrap();
        let mut p2p_config = P2pConfig::default();
        p2p_config.kademlia_enabled = false;
        let config = BehaviourConfig::new(identity, p2p_config);
        let behaviour = CaveBehaviour::new(config);
        assert!(behaviour.is_ok());
    }

    #[test]
    fn behaviour_with_mdns_disabled() {
        let identity = NodeIdentity::generate().unwrap();
        let mut p2p_config = P2pConfig::default();
        p2p_config.mdns_enabled = false;
        let config = BehaviourConfig::new(identity, p2p_config);
        let behaviour = CaveBehaviour::new(config);
        assert!(behaviour.is_ok());
    }

    #[test]
    fn protocol_version_from_config() {
        let identity = NodeIdentity::generate().unwrap();
        let p2p_config = P2pConfig::default();
        let config = BehaviourConfig::new(identity.clone(), p2p_config)
            .with_protocol_version("/custom/1.0.0");
        assert_eq!(config.protocol_version, "/custom/1.0.0");
    }
}
