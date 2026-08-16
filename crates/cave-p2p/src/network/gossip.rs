use libp2p::gossipsub::{ConfigBuilder, IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p_identity::Keypair;

pub use libp2p::gossipsub::Behaviour as GossipsubBehaviour;
pub use libp2p::gossipsub::{
    ConfigBuilderError as GossipConfigError, Event as GossipEvent, Message as GossipMessage,
    MessageId, TopicScoreParams,
};

/// Default topic prefix for cave-p2p GossipSub messages.
pub const DEFAULT_TOPIC_PREFIX: &str = "bkg-p2p";

/// Create a GossipSub behaviour.
pub fn create(
    keypair: &Keypair,
    validation: ValidationMode,
) -> Result<GossipsubBehaviour, &'static str> {
    let config = ConfigBuilder::default()
        .validation_mode(validation)
        .build()
        .map_err(|e| {
            tracing::warn!("gossipsub config error: {e}");
            "gossipsub configuration failed"
        })?;
    GossipsubBehaviour::new(MessageAuthenticity::Signed(keypair.clone()), config)
}

/// Create a topic with the default prefix.
pub fn topic(name: &str) -> IdentTopic {
    IdentTopic::new(format!("{DEFAULT_TOPIC_PREFIX}/{name}"))
}
