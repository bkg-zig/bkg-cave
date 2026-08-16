//! Events and EventBus integration for cave-p2p.
//!
//! This module provides an EventBus for cave-p2p to communicate with other
//! plugins and the host system through the bkg-cave event system.

use std::sync::Arc;

use cave_core::event::{CaveEvent, EventBus};
use libp2p::PeerId;

/// Events specific to cave-p2p that can be emitted to the EventBus.
#[derive(Debug, Clone)]
pub enum P2pEvent {
    /// Network layer event wrapper.
    Network(Box<crate::network::events::NetworkEvent>),
    /// Plugin state change event.
    StateChange(String),
    /// Peer count updated.
    PeerCountUpdated { connected: usize, discovered: usize },
    /// Resource manifest published.
    ManifestPublished { peer_id: PeerId, topic: String },
}

/// Event publisher wrapper that integrates with cave-core's EventBus.
pub struct EventPublisher {
    bus: Arc<EventBus>,
}

impl EventPublisher {
    /// Create a new event publisher with the given EventBus.
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    /// Publish a cave-p2p specific event.
    pub fn publish(&self, event: P2pEvent) {
        match event {
            P2pEvent::Network(net_event) => {
                if let Some(peer_id) = extract_peer_id(&net_event) {
                    let cave_event = CaveEvent::ConfigChanged(format!("p2p-peer:{}", peer_id));
                    self.bus.emit(cave_event);
                }
            }
            P2pEvent::StateChange(state) => {
                let cave_event = CaveEvent::ConfigChanged(format!("p2p-state:{}", state));
                self.bus.emit(cave_event);
            }
            P2pEvent::PeerCountUpdated {
                connected,
                discovered,
            } => {
                let cave_event =
                    CaveEvent::ConfigChanged(format!("p2p-stats:{}:{}", connected, discovered));
                self.bus.emit(cave_event);
            }
            P2pEvent::ManifestPublished { peer_id, topic } => {
                let cave_event =
                    CaveEvent::ConfigChanged(format!("p2p-manifest:{}:{}", peer_id, topic));
                self.bus.emit(cave_event);
            }
        }
    }

    /// Attach a handler for cave-p2p events.
    pub fn subscribe(&self, name: &str, handler: Arc<dyn Fn(&CaveEvent) + Send + Sync>) {
        self.bus.subscribe(name, handler);
    }
}

fn extract_peer_id(event: &crate::network::events::NetworkEvent) -> Option<PeerId> {
    match event {
        crate::network::events::NetworkEvent::PeerDiscovered { peer_id, .. } => Some(*peer_id),
        crate::network::events::NetworkEvent::PeerConnected(peer_id) => Some(*peer_id),
        crate::network::events::NetworkEvent::PeerDisconnected(peer_id) => Some(*peer_id),
        crate::network::events::NetworkEvent::GossipMessage { source, .. } => Some(*source),
        crate::network::events::NetworkEvent::RequestReceived { from, .. } => Some(*from),
        crate::network::events::NetworkEvent::ResponseReceived { from, .. } => Some(*from),
        crate::network::events::NetworkEvent::ResourceAdvertised { peer_id, .. } => Some(*peer_id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn event_publisher_creates() {
        let bus = Arc::new(EventBus::new());
        let publisher = EventPublisher::new(bus);
        assert_eq!(publisher.bus.handler_count(), 0);
    }

    #[test]
    fn publishes_state_change_event() {
        let bus = Arc::new(EventBus::new());
        let publisher = EventPublisher::new(bus.clone());

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        publisher.subscribe(
            "test",
            Arc::new(move |event| {
                if matches!(event, CaveEvent::ConfigChanged(_)) {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            }),
        );

        publisher.publish(P2pEvent::StateChange("ready".to_string()));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn extracts_peer_from_network_event() {
        let peer_id: PeerId = libp2p::PeerId::random();
        let event = crate::network::events::NetworkEvent::PeerConnected(peer_id);

        let extracted = extract_peer_id(&event);
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), peer_id);
    }
}
