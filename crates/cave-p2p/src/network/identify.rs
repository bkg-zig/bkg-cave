pub use libp2p::identify::{
    Behaviour as IdentifyBehaviour, Config as IdentifyConfig, Event as IdentifyEvent,
    Info as IdentifyInfo,
};

/// Create an Identify protocol behaviour.
pub fn create(
    protocol_version: String,
    agent_version: String,
    public_key: libp2p_identity::PublicKey,
) -> IdentifyBehaviour {
    let config =
        IdentifyConfig::new(protocol_version, public_key).with_agent_version(agent_version);
    IdentifyBehaviour::new(config)
}
