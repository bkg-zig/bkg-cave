use crate::a2a::{A2aState, AgentCard, AgentCardAnnouncement, A2A_GOSSIP_TOPIC};
use crate::identity::NodeKeypair;

pub fn create_announcement(
    peer_id: &str,
    card: AgentCard,
    keypair: &NodeKeypair,
) -> AgentCardAnnouncement {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut announcement = AgentCardAnnouncement {
        peer_id: peer_id.to_string(),
        card,
        timestamp,
        signature: Vec::new(),
    };

    let signing_bytes = signing_bytes(&announcement);
    announcement.signature = keypair.sign(&signing_bytes).to_vec();

    announcement
}

pub fn verify_announcement(
    announcement: &AgentCardAnnouncement,
    public_key: &[u8; 32],
) -> Result<(), crate::Error> {
    let bytes = signing_bytes(announcement);
    let sig: [u8; 64] = announcement
        .signature
        .clone()
        .try_into()
        .map_err(|_| crate::Error::Identity("invalid signature length".into()))?;
    NodeKeypair::verify(&bytes, &sig, public_key)
}

fn signing_bytes(announcement: &AgentCardAnnouncement) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct SigningFields<'a> {
        peer_id: &'a str,
        name: &'a str,
        url: &'a str,
        capabilities_streaming: bool,
        capabilities_tools: bool,
        capabilities_crew: bool,
        capabilities_vector_memory: bool,
        timestamp: u64,
    }

    let fields = SigningFields {
        peer_id: &announcement.peer_id,
        name: &announcement.card.name,
        url: &announcement.card.url,
        capabilities_streaming: announcement.card.capabilities.streaming,
        capabilities_tools: announcement.card.capabilities.tools,
        capabilities_crew: announcement.card.capabilities.crew,
        capabilities_vector_memory: announcement.card.capabilities.vector_memory,
        timestamp: announcement.timestamp,
    };

    serde_json::to_vec(&fields).unwrap_or_default()
}

pub fn handle_agent_card_gossip(
    state: &A2aState,
    data: &[u8],
) -> Result<Option<AgentCard>, crate::Error> {
    let announcement: AgentCardAnnouncement =
        serde_json::from_slice(data).map_err(|e| crate::Error::A2a(e.to_string()))?;

    state.upsert_peer_card(announcement.peer_id.clone(), announcement.card.clone());

    Ok(Some(announcement.card.clone()))
}

pub fn encode_announcement(announcement: &AgentCardAnnouncement) -> Result<Vec<u8>, crate::Error> {
    serde_json::to_vec(announcement).map_err(|e| crate::Error::A2a(e.to_string()))
}

pub fn gossip_topic() -> &'static str {
    A2A_GOSSIP_TOPIC
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a2a::{AgentCapabilities, AgentCard};

    #[test]
    fn test_create_and_verify_announcement() {
        let keypair = NodeKeypair::generate().unwrap();
        let peer_id = crate::identity::NodeId::from_public_key(&keypair.public_key()).to_base58();

        let card = AgentCard {
            name: "test-node".into(),
            description: "test".into(),
            url: "/p2p/test".into(),
            peer_id: peer_id.clone(),
            models: vec!["llama3.1:8b".into()],
            capabilities: AgentCapabilities {
                streaming: true,
                tools: false,
                crew: false,
                vector_memory: false,
            },
            channels: vec!["json-rpc".into()],
        };

        let announcement = create_announcement(&peer_id, card, &keypair);
        assert!(!announcement.signature.is_empty());

        let pub_key = keypair.public_bytes();
        assert!(verify_announcement(&announcement, &pub_key).is_ok());
    }

    #[test]
    fn test_verify_rejects_wrong_key() {
        let keypair = NodeKeypair::generate().unwrap();
        let peer_id = crate::identity::NodeId::from_public_key(&keypair.public_key()).to_base58();

        let card = AgentCard::bkg_p2p_default(&peer_id);
        let announcement = create_announcement(&peer_id, card, &keypair);

        let wrong = NodeKeypair::generate().unwrap();
        assert!(verify_announcement(&announcement, &wrong.public_bytes()).is_err());
    }

    #[test]
    fn test_handle_gossip_updates_state() {
        let state = A2aState::new();
        let keypair = NodeKeypair::generate().unwrap();
        let peer_id = crate::identity::NodeId::from_public_key(&keypair.public_key()).to_base58();

        let card = AgentCard::bkg_p2p_default(&peer_id);
        let announcement = create_announcement(&peer_id, card, &keypair);
        let data = encode_announcement(&announcement).unwrap();

        let result = handle_agent_card_gossip(&state, &data);
        assert!(result.is_ok());

        let cards = state.list_peer_cards();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].1.name, "bkg-p2p node");
    }
}
