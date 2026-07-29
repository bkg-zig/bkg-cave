//! Network events for the P2P subsystem.

use libp2p::PeerId;

/// Events emitted by the network layer.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A new peer was discovered (via mDNS or DHT).
    PeerDiscovered {
        peer_id: PeerId,
        addresses: Vec<String>,
    },
    /// A peer connected.
    PeerConnected(PeerId),
    /// A peer disconnected.
    PeerDisconnected(PeerId),
    /// A GossipSub message was received.
    GossipMessage {
        topic: String,
        data: Vec<u8>,
        source: PeerId,
    },
    /// An A2A JSON-RPC request was received.
    RequestReceived {
        request_id: String,
        from: PeerId,
        payload: Vec<u8>,
    },
    /// An A2A JSON-RPC response was received.
    ResponseReceived {
        request_id: String,
        from: PeerId,
        payload: Vec<u8>,
    },
    /// A resource manifest was received.
    ResourceAdvertised { peer_id: PeerId, manifest: Vec<u8> },
    /// The local node started listening on new addresses.
    NewListenAddr(String),
    /// The local node's external address was discovered.
    ExternalAddrDiscovered(String),
    /// A Kademlia routing table update occurred.
    KademliaRoutingUpdated { peer_id: PeerId, is_new_peer: bool },
    /// An identify response was received.
    IdentifyReceived { peer_id: PeerId, info: Vec<u8> },
}
