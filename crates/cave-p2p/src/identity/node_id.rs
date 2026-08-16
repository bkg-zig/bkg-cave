//! Stable node identifier derived from Ed25519 public key.

use crate::identity::error::{IdentityError, Result};
use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

/// Stable node identifier derived from Ed25519 public key.
///
/// This is a deterministic identifier derived from the node's Ed25519 public key.
/// The same key always produces the same `NodeId`.
///
/// Format: Base58-encoded SHA256 hash of the public key (compatible with libp2p PeerId concept).
///
/// Note: In M2 (Network Core), this will be mapped to libp2p's PeerId.
/// For M1, we use a self-contained stable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    inner: [u8; 32],
}

impl NodeId {
    /// Create a NodeId from an Ed25519 public key.
    pub fn from_public_key(public_key: &VerifyingKey) -> Self {
        let pub_bytes = public_key.to_bytes();
        let hash = Sha256::digest(pub_bytes);
        let mut inner = [0u8; 32];
        inner.copy_from_slice(hash.as_slice());
        Self { inner }
    }

    /// Create a NodeId from raw 32-byte hash.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { inner: bytes }
    }

    /// Get the raw bytes of the node ID.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.inner
    }

    /// Get the string representation (Base58-encoded).
    pub fn to_base58(&self) -> String {
        bs58::encode(self.inner).into_string()
    }

    /// Parse a NodeId from its string representation.
    pub fn from_string(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| IdentityError::InvalidKeypair(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(IdentityError::InvalidKeypair(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            ))
            .into());
        }
        let mut inner = [0u8; 32];
        inner.copy_from_slice(&bytes);
        Ok(Self { inner })
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl PartialEq<str> for NodeId {
    fn eq(&self, other: &str) -> bool {
        self.to_base58() == other
    }
}

impl PartialEq<NodeId> for str {
    fn eq(&self, other: &NodeId) -> bool {
        self == other.to_base58()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    #[test]
    fn node_id_from_public_key() {
        let kp = SigningKey::generate(&mut rand::rngs::OsRng);
        let node_id = NodeId::from_public_key(&kp.verifying_key());
        let s = node_id.to_base58();
        assert!(!s.is_empty());
    }

    #[test]
    fn node_id_roundtrip_string() {
        let kp = SigningKey::generate(&mut rand::rngs::OsRng);
        let node_id = NodeId::from_public_key(&kp.verifying_key());
        let s = node_id.to_base58();
        let parsed = NodeId::from_string(&s).unwrap();
        assert_eq!(node_id, parsed);
    }

    #[test]
    fn same_key_same_node_id() {
        let kp = SigningKey::generate(&mut rand::rngs::OsRng);
        let id1 = NodeId::from_public_key(&kp.verifying_key());
        let id2 = NodeId::from_public_key(&kp.verifying_key());
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_keys_different_node_ids() {
        let kp1 = SigningKey::generate(&mut rand::rngs::OsRng);
        let kp2 = SigningKey::generate(&mut rand::rngs::OsRng);
        let id1 = NodeId::from_public_key(&kp1.verifying_key());
        let id2 = NodeId::from_public_key(&kp2.verifying_key());
        assert_ne!(id1, id2);
    }

    #[test]
    fn display_trait() {
        let kp = SigningKey::generate(&mut rand::rngs::OsRng);
        let node_id = NodeId::from_public_key(&kp.verifying_key());
        let s = format!("{}", node_id);
        assert_eq!(s, node_id.to_base58());
    }

    #[test]
    fn deterministic_node_id() {
        let mut seed = [0u8; 32];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = i as u8;
        }
        let kp = SigningKey::from_bytes(&seed);
        let id1 = NodeId::from_public_key(&kp.verifying_key());
        let id2 = NodeId::from_public_key(&kp.verifying_key());
        assert_eq!(id1, id2);
    }
}
