use libp2p::{
    gossipsub, identify, kad, multiaddr::Multiaddr, request_response, swarm::SwarmEvent,
    SwarmBuilder,
};
use tokio::sync::{broadcast, mpsc};

use crate::config::P2pConfig;
use crate::identity::NodeIdentity;
use crate::network::behaviour::{BehaviourConfig, CaveBehaviour, CaveBehaviourEvent};
use crate::network::events::NetworkEvent;
use crate::network::NetworkError;

enum SwarmCmd {
    Dial { addr: Multiaddr },
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Publish { topic: String, data: Vec<u8> },
}

/// A running libp2p swarm with command and event channels.
pub struct P2pSwarm {
    cmd_tx: mpsc::UnboundedSender<SwarmCmd>,
    event_tx: broadcast::Sender<NetworkEvent>,
}

impl P2pSwarm {
    /// Create and start a new P2P swarm.
    pub fn new(
        identity: NodeIdentity,
        listen_addresses: Vec<Multiaddr>,
    ) -> Result<(Self, broadcast::Receiver<NetworkEvent>), NetworkError> {
        let keypair = identity
            .to_libp2p_keypair()
            .map_err(|e| NetworkError::Swarm(e.to_string()))?;

        let behaviour_config = BehaviourConfig::new(identity.clone(), P2pConfig::default())
            .with_gossipsub_validation(gossipsub::ValidationMode::Strict);
        let behaviour = CaveBehaviour::new(behaviour_config)
            .map_err(|e| NetworkError::Behaviour(e.to_string()))?;

        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                |kp: &libp2p_identity::Keypair| libp2p::noise::Config::new(kp),
                libp2p::yamux::Config::default,
            )
            .map_err(|e: libp2p::noise::Error| NetworkError::Transport(e.to_string()))?
            .with_quic()
            .with_behaviour(|_| Ok(behaviour))
            .map_err(|e| NetworkError::Behaviour(e.to_string()))?
            .build();

        for addr in &listen_addresses {
            match swarm.listen_on(addr.clone()) {
                Ok(_) => tracing::info!("listening on {addr}"),
                Err(e) => tracing::warn!("failed to listen on {addr}: {e}"),
            }
        }

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = broadcast::channel(256);

        tokio::spawn(Self::run_event_loop(swarm, cmd_rx, event_tx.clone()));

        Ok((Self { cmd_tx, event_tx }, event_rx))
    }

    async fn run_event_loop(
        mut swarm: libp2p::Swarm<CaveBehaviour>,
        mut cmd_rx: mpsc::UnboundedReceiver<SwarmCmd>,
        event_tx: broadcast::Sender<NetworkEvent>,
    ) {
        use futures::StreamExt;

        loop {
            tokio::select! {
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        SwarmCmd::Dial { addr } => {
                            match swarm.dial(addr.clone()) {
                                Ok(()) => tracing::info!("dialling {addr}"),
                                Err(e) => tracing::warn!("dial failed to {addr}: {e:?}"),
                            }
                        }
                        SwarmCmd::Subscribe { topic } => {
                            let topic_id = gossipsub::IdentTopic::new(&topic);
                            match swarm.behaviour_mut().gossipsub.subscribe(&topic_id) {
                                Ok(_) => {
                                    tracing::info!("subscribed to {topic}");
                                }
                                Err(e) => tracing::warn!("subscribe to {topic} failed: {e}"),
                            }
                        }
                        SwarmCmd::Unsubscribe { topic } => {
                            let _ = swarm.behaviour_mut().gossipsub.unsubscribe(
                                &gossipsub::IdentTopic::new(&topic),
                            );
                        }
                        SwarmCmd::Publish { topic, data } => {
                            let topic_id = gossipsub::IdentTopic::new(&topic);
                            match swarm.behaviour_mut().gossipsub.publish(topic_id, data) {
                                Ok(msg_id) => tracing::trace!("published to {topic}: {msg_id}"),
                                Err(e) => tracing::warn!("publish to {topic} failed: {e}"),
                            }
                        }
                    }
                }
                event = swarm.next() => {
                    let Some(event) = event else {
                        tracing::warn!("swarm stream ended");
                        break;
                    };
                    Self::handle_swarm_event(&mut swarm, event, &event_tx);
                }
            }
        }
    }

    fn handle_swarm_event(
        _swarm: &mut libp2p::Swarm<CaveBehaviour>,
        event: SwarmEvent<CaveBehaviourEvent>,
        event_tx: &broadcast::Sender<NetworkEvent>,
    ) {
        let network_event = match event {
            SwarmEvent::Behaviour(be_event) => match be_event {
                CaveBehaviourEvent::Gossipsub(gossip_event) => match gossip_event {
                    gossipsub::Event::Message {
                        propagation_source,
                        message_id: _,
                        message,
                    } => NetworkEvent::GossipMessage {
                        topic: message.topic.to_string(),
                        data: message.data,
                        source: propagation_source,
                    },
                    _ => return,
                },
                CaveBehaviourEvent::Identify(identify_event) => match identify_event {
                    identify::Event::Received { peer_id, .. } => NetworkEvent::IdentifyReceived {
                        peer_id,
                        info: vec![],
                    },
                    identify::Event::Error {
                        peer_id,
                        error,
                        connection_id: _,
                    } => {
                        tracing::debug!("identify error from {peer_id}: {error}");
                        return;
                    }
                    _ => return,
                },
                CaveBehaviourEvent::Kademlia(kad_event) => match kad_event {
                    kad::Event::RoutingUpdated {
                        peer, is_new_peer, ..
                    } => NetworkEvent::KademliaRoutingUpdated {
                        peer_id: peer,
                        is_new_peer,
                    },
                    _ => return,
                },
                CaveBehaviourEvent::A2aRpc(rr_event) => match rr_event {
                    request_response::Event::Message { peer, message } => match message {
                        request_response::Message::Request {
                            request_id,
                            request,
                            channel: _,
                        } => {
                            let payload = serde_json::to_vec(&request).unwrap_or_default();
                            NetworkEvent::RequestReceived {
                                request_id: request_id.to_string(),
                                from: peer,
                                payload,
                            }
                        }
                        request_response::Message::Response {
                            request_id,
                            response,
                        } => {
                            let payload = serde_json::to_vec(&response).unwrap_or_default();
                            NetworkEvent::ResponseReceived {
                                request_id: request_id.to_string(),
                                from: peer,
                                payload,
                            }
                        }
                    },
                    _ => return,
                },
            },
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                NetworkEvent::PeerConnected(peer_id)
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => NetworkEvent::PeerDisconnected(peer_id),
            SwarmEvent::NewListenAddr { address, .. } => {
                NetworkEvent::NewListenAddr(address.to_string())
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                NetworkEvent::ExternalAddrDiscovered(address.to_string())
            }
            _ => return,
        };

        let _ = event_tx.send(network_event);
    }

    pub fn dial(&self, addr: Multiaddr) -> Result<(), NetworkError> {
        self.cmd_tx
            .send(SwarmCmd::Dial { addr })
            .map_err(|_| NetworkError::Swarm("command channel closed".to_string()))
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), NetworkError> {
        self.cmd_tx
            .send(SwarmCmd::Subscribe {
                topic: topic.to_string(),
            })
            .map_err(|_| NetworkError::Swarm("command channel closed".to_string()))
    }

    pub fn unsubscribe(&self, topic: &str) -> Result<(), NetworkError> {
        self.cmd_tx
            .send(SwarmCmd::Unsubscribe {
                topic: topic.to_string(),
            })
            .map_err(|_| NetworkError::Swarm("command channel closed".to_string()))
    }

    pub fn publish(&self, topic: &str, data: Vec<u8>) -> Result<(), NetworkError> {
        self.cmd_tx
            .send(SwarmCmd::Publish {
                topic: topic.to_string(),
                data,
            })
            .map_err(|_| NetworkError::Swarm("command channel closed".to_string()))
    }

    pub fn event_receiver(&self) -> broadcast::Receiver<NetworkEvent> {
        self.event_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_p2p_swarm() {
        let identity = NodeIdentity::generate().unwrap();
        let addrs: Vec<Multiaddr> = vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()];
        let result = P2pSwarm::new(identity, addrs);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn swarm_events_channel() {
        let identity = NodeIdentity::generate().unwrap();
        let addrs: Vec<Multiaddr> = vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()];
        let (swarm, mut rx) = P2pSwarm::new(identity, addrs).unwrap();

        let event = rx.try_recv();
        assert!(event.is_err());

        drop(swarm);
    }

    #[tokio::test]
    async fn subscribe_and_publish() {
        let identity = NodeIdentity::generate().unwrap();
        let addrs: Vec<Multiaddr> = vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()];
        let (swarm, _rx) = P2pSwarm::new(identity, addrs).unwrap();

        assert!(swarm.subscribe("bkg-p2p/test/v1").is_ok());
        assert!(swarm.publish("bkg-p2p/test/v1", b"hello".to_vec()).is_ok());
    }

    #[tokio::test]
    async fn two_node_gossipsub() {
        const TOPIC: &str = "bkg-p2p/test/v1";
        const PAYLOAD: &[u8] = b"hello from node A";

        let identity_a = NodeIdentity::generate().unwrap();
        let identity_b = NodeIdentity::generate().unwrap();

        let (swarm_a, mut rx_a) = P2pSwarm::new(
            identity_a.clone(),
            vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()],
        )
        .unwrap();

        let addr_a = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match rx_a.recv().await {
                    Ok(NetworkEvent::NewListenAddr(addr)) => break addr,
                    _ => continue,
                }
            }
        })
        .await
        .expect("node A did not emit NewListenAddr in time");

        let peer_id_a = identity_a
            .to_libp2p_keypair()
            .unwrap()
            .public()
            .to_peer_id();

        swarm_a.subscribe(TOPIC).unwrap();

        let (swarm_b, mut rx_b) = P2pSwarm::new(
            identity_b.clone(),
            vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()],
        )
        .unwrap();

        swarm_b.subscribe(TOPIC).unwrap();

        let dial_addr: Multiaddr = format!("{addr_a}/p2p/{peer_id_a}")
            .parse()
            .expect("valid multiaddr");
        swarm_b.dial(dial_addr).unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        swarm_a.publish(TOPIC, PAYLOAD.to_vec()).unwrap();

        let received = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                match rx_b.recv().await {
                    Ok(NetworkEvent::GossipMessage {
                        topic,
                        data,
                        source: _,
                    }) if topic == TOPIC && data == PAYLOAD => return true,
                    _ => continue,
                }
            }
        })
        .await;

        assert!(
            received.unwrap_or(false),
            "node B did not receive gossip message from node A"
        );

        drop(swarm_a);
        drop(swarm_b);
    }
}
