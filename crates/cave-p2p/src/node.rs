use tokio::sync::broadcast;

use crate::config::Config;
use crate::identity::NodeIdentity;

pub struct Node {
    pub config: Config,
    pub identity: NodeIdentity,
    pub peer_id: libp2p::PeerId,
    pub shutdown_tx: broadcast::Sender<()>,
}

impl Node {
    pub fn new(config: Config) -> crate::Result<Self> {
        let identity_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("bkg-cave");
        std::fs::create_dir_all(&identity_dir)?;

        let identity_path = identity_dir.join("identity.json");
        let identity = if identity_path.exists() {
            NodeIdentity::load(&identity_path)?
        } else {
            let id = NodeIdentity::generate()?;
            id.save(&identity_path)?;
            id
        };

        let libp2p_kp = identity.to_libp2p_keypair()?;
        let peer_id = libp2p::PeerId::from(libp2p_kp.public());

        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Self {
            config,
            identity,
            peer_id,
            shutdown_tx,
        })
    }

    pub fn peer_id(&self) -> &libp2p::PeerId {
        &self.peer_id
    }

    pub fn shutdown_handle(&self) -> broadcast::Sender<()> {
        self.shutdown_tx.clone()
    }

    pub async fn run(&self) -> crate::Result<()> {
        tracing::info!("Node {} starting up", self.peer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_node_create() {
        let identity = NodeIdentity::generate().unwrap();
        let libp2p_kp = identity.to_libp2p_keypair().unwrap();
        let peer_id = libp2p::PeerId::from(libp2p_kp.public());
        let config = Config::default();
        let (shutdown_tx, _) = broadcast::channel(1);
        let node = Node {
            config,
            identity,
            peer_id,
            shutdown_tx,
        };
        assert!(!node.peer_id().to_string().is_empty());
    }

    #[test]
    fn test_shutdown_handle() {
        let identity = NodeIdentity::generate().unwrap();
        let libp2p_kp = identity.to_libp2p_keypair().unwrap();
        let peer_id = libp2p::PeerId::from(libp2p_kp.public());
        let config = Config::default();
        let (shutdown_tx, _rx) = broadcast::channel(1);
        let node = Node {
            config,
            identity,
            peer_id,
            shutdown_tx,
        };
        let handle = node.shutdown_handle();
        assert!(handle.send(()).is_ok());
    }

    #[tokio::test]
    async fn test_node_run() {
        let identity = NodeIdentity::generate().unwrap();
        let libp2p_kp = identity.to_libp2p_keypair().unwrap();
        let peer_id = libp2p::PeerId::from(libp2p_kp.public());
        let config = Config::default();
        let (shutdown_tx, _) = broadcast::channel(1);
        let node = Node {
            config,
            identity,
            peer_id,
            shutdown_tx,
        };
        let result = node.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_peer_id_string() {
        let identity = NodeIdentity::generate().unwrap();
        let libp2p_kp = identity.to_libp2p_keypair().unwrap();
        let peer_id = libp2p::PeerId::from(libp2p_kp.public());
        let peer_str = peer_id.to_string();
        assert!(!peer_str.is_empty());
    }
}
