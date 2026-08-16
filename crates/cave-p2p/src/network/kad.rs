use libp2p::PeerId;
use libp2p::StreamProtocol;

pub use libp2p::kad::store::MemoryStore;
pub use libp2p::kad::{
    Behaviour as KademliaBehaviour, Config as KademliaConfig, Event as KadEvent, PeerRecord,
    QueryId, QueryResult, Record, RecordKey,
};

/// Default Kademlia protocol name.
pub const KAD_PROTOCOL: &str = "/bkg-p2p/kad/1.0.0";

/// Create a Kademlia DHT behaviour.
pub fn create(peer_id: PeerId, enabled: bool) -> KademliaBehaviour<MemoryStore> {
    let store = MemoryStore::new(peer_id);
    if enabled {
        let mut config = KademliaConfig::new(StreamProtocol::new(KAD_PROTOCOL));
        config.set_query_timeout(std::time::Duration::from_secs(60));
        KademliaBehaviour::with_config(peer_id, store, config)
    } else {
        KademliaBehaviour::new(peer_id, store)
    }
}
