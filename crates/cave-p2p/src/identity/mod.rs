//! Node identity system for cave-p2p.
//!
//! Provides Ed25519 keypair management, stable node identifiers (PeerId),
//! signing, and verification. This is the foundation for all P2P operations.
//!
//! ## Structure
//!
//! - `NodeKeypair` — Ed25519 keypair wrapper with serialization
//! - `NodeId` — stable node identifier derived from public key
//! - `NodeIdentity` — high-level identity handle combining keypair + node ID

pub mod error;
pub mod keypair;
pub mod node_id;

pub use error::{IdentityError, Result};
pub use keypair::NodeKeypair;
pub use node_id::NodeId;

use std::path::PathBuf;

/// High-level node identity handle.
///
/// This is the primary type used by other cave-p2p modules. It combines:
/// - Ed25519 keypair for signing/verification
/// - Stable NodeId (libp2p PeerId) for network identity
///
/// ## Lifecycle
///
/// 1. Generate new identity: `NodeIdentity::generate()`
/// 2. Save to disk: `identity.save(path)?`
/// 3. Load from disk: `NodeIdentity::load(path)?`
/// 4. Use for signing: `identity.sign(message)`
/// 5. Derive libp2p keys: `identity.to_libp2p_keypair()`
#[derive(Debug, Clone)]
pub struct NodeIdentity {
    keypair: NodeKeypair,
    node_id: NodeId,
}

impl NodeIdentity {
    /// Generate a new random identity.
    pub fn generate() -> Result<Self> {
        let keypair = NodeKeypair::generate()?;
        let node_id = NodeId::from_public_key(&keypair.public_key());
        Ok(Self { keypair, node_id })
    }

    /// Load an identity from a file.
    ///
    /// The file should contain the 32-byte secret key in JSON or raw format.
    /// Currently expects JSON format: `{"secret_key": "<hex>"}`.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let data = std::fs::read(&path).map_err(IdentityError::from)?;
        let secret_bytes = serde_json::from_slice::<[u8; 32]>(&data)
            .map_err(|e| IdentityError::Serialization(e.to_string()))?;
        let keypair = NodeKeypair::from_bytes(&secret_bytes)?;
        let node_id = NodeId::from_public_key(&keypair.public_key());
        Ok(Self { keypair, node_id })
    }

    /// Save the identity to a file.
    ///
    /// Writes the 32-byte secret key as JSON.
    pub fn save(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(IdentityError::from)?;
        }
        let secret_bytes = self.keypair.secret_bytes();
        let json = serde_json::to_vec(&secret_bytes)
            .map_err(|e| IdentityError::Serialization(e.to_string()))?;
        std::fs::write(&path, json).map_err(IdentityError::from)?;
        Ok(())
    }

    /// Get the stable node identifier (libp2p PeerId).
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the node ID as a string (Base58-encoded).
    pub fn node_id_string(&self) -> String {
        self.node_id.to_base58()
    }

    /// Get the public key bytes (32 bytes).
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public_bytes()
    }

    /// Sign a message with the node's secret key.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.keypair.sign(message)
    }

    /// Verify a signature against a message and the node's public key.
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> Result<()> {
        NodeKeypair::verify(message, signature, &self.public_key_bytes())
    }

    /// Verify a signature against a message and an arbitrary public key.
    pub fn verify_with_key(
        message: &[u8],
        signature: &[u8; 64],
        public_key: &[u8; 32],
    ) -> Result<()> {
        NodeKeypair::verify(message, signature, public_key)
    }

    /// Convert to a libp2p identity keypair for use with libp2p swarm.
    pub fn to_libp2p_keypair(&self) -> Result<libp2p_identity::Keypair> {
        let secret_bytes = self.keypair.secret_bytes();
        libp2p_identity::Keypair::ed25519_from_bytes(secret_bytes)
            .map_err(|e| IdentityError::KeyGeneration(e.to_string()).into())
    }

    /// Get a reference to the inner keypair.
    pub fn keypair(&self) -> &NodeKeypair {
        &self.keypair
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_identity() {
        let identity = NodeIdentity::generate().unwrap();
        let id_str = identity.node_id_string();
        assert!(!id_str.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path();
        let identity1 = NodeIdentity::generate().unwrap();
        identity1.save(path).unwrap();

        let identity2 = NodeIdentity::load(path).unwrap();
        assert_eq!(identity1.node_id(), identity2.node_id());
        assert_eq!(identity1.public_key_bytes(), identity2.public_key_bytes());
    }

    #[test]
    fn sign_and_verify() {
        let identity = NodeIdentity::generate().unwrap();
        let message = b"test message";
        let sig = identity.sign(message);
        assert!(identity.verify(message, &sig).is_ok());
    }

    #[test]
    fn verify_rejects_wrong_message() {
        let identity = NodeIdentity::generate().unwrap();
        let message = b"original";
        let sig = identity.sign(message);
        assert!(identity.verify(b"tampered", &sig).is_err());
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let identity1 = NodeIdentity::generate().unwrap();
        let identity2 = NodeIdentity::generate().unwrap();
        let message = b"test";
        let sig = identity1.sign(message);
        let wrong_key = identity2.public_key_bytes();
        assert!(NodeIdentity::verify_with_key(message, &sig, &wrong_key).is_err());
    }

    #[test]
    fn stable_node_id_same_key() {
        let kp_bytes = [42u8; 32];
        let kp = NodeKeypair::from_bytes(&kp_bytes).unwrap();
        let id1 = NodeId::from_public_key(&kp.public_key());
        let id2 = NodeId::from_public_key(&kp.public_key());
        assert_eq!(id1, id2);
    }

    #[test]
    fn libp2p_keypair_conversion() {
        let identity = NodeIdentity::generate().unwrap();
        let libp2p_kp = identity.to_libp2p_keypair().unwrap();
        let peer_id = libp2p_identity::PeerId::from(libp2p_kp.public());
        // Both are derived from the same Ed25519 key, but use different encodings:
        // - NodeId: SHA256(public_key) -> Base58
        // - libp2p PeerId: multihash(public_key) -> Base58
        // They will differ in representation but both are valid identifiers.
        assert!(!peer_id.to_string().is_empty());
        assert!(!identity.node_id_string().is_empty());
    }

    #[test]
    fn load_missing_file_errors() {
        let err = NodeIdentity::load("/nonexistent/path/identity.json").unwrap_err();
        assert!(matches!(err, crate::Error::Identity(_)));
    }

    #[test]
    fn deterministic_peer_id() {
        // Fixed seed bytes for deterministic test
        let mut seed = [0u8; 32];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = i as u8;
        }
        let kp = NodeKeypair::from_bytes(&seed).unwrap();
        let id1 = NodeId::from_public_key(&kp.public_key());
        let id2 = NodeId::from_public_key(&kp.public_key());
        assert_eq!(id1, id2);
        assert_eq!(id1.to_base58(), id2.to_base58());
    }
}
