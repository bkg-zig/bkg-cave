//! Ed25519 keypair handling for node identity.

use crate::identity::error::{IdentityError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Ed25519 keypair wrapper with serialization support.
#[derive(Debug, Clone)]
pub struct NodeKeypair {
    inner: SigningKey,
}

impl NodeKeypair {
    /// Generate a new random keypair.
    pub fn generate() -> Result<Self> {
        let mut csprng = OsRng;
        let inner = SigningKey::generate(&mut csprng);
        Ok(Self { inner })
    }

    /// Load a keypair from raw bytes (32-byte secret key).
    pub fn from_bytes(secret_bytes: &[u8]) -> Result<Self> {
        if secret_bytes.len() != 32 {
            return Err(IdentityError::InvalidKeypair(format!(
                "expected 32 bytes, got {}",
                secret_bytes.len()
            ))
            .into());
        }
        let inner = SigningKey::from_bytes(secret_bytes.try_into().unwrap());
        Ok(Self { inner })
    }

    /// Create from an existing ed25519_dalek SigningKey.
    pub fn from_dalek(inner: SigningKey) -> Self {
        Self { inner }
    }

    /// Export the secret key bytes (32 bytes).
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }

    /// Export the public key bytes (32 bytes).
    pub fn public_bytes(&self) -> [u8; 32] {
        self.inner.verifying_key().to_bytes()
    }

    /// Sign a message with the secret key.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.inner.sign(message).to_bytes()
    }

    /// Verify a signature against a message and public key.
    pub fn verify(message: &[u8], signature: &[u8; 64], public_key: &[u8; 32]) -> Result<()> {
        let verifying_key = VerifyingKey::from_bytes(public_key)
            .map_err(|e| IdentityError::InvalidKeypair(e.to_string()))?;
        let sig = Signature::from_bytes(signature.into());
        verifying_key
            .verify(message, &sig)
            .map_err(|_| IdentityError::SignatureVerification)?;
        Ok(())
    }

    /// Get the public key.
    pub fn public_key(&self) -> VerifyingKey {
        self.inner.verifying_key()
    }

    /// Get the inner dalek signing key.
    pub fn inner(&self) -> &SigningKey {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_keypair() {
        let kp = NodeKeypair::generate().unwrap();
        let pub_bytes = kp.public_bytes();
        let sec_bytes = kp.secret_bytes();
        assert_eq!(pub_bytes.len(), 32);
        assert_eq!(sec_bytes.len(), 32);
    }

    #[test]
    fn roundtrip_serialization() {
        let kp1 = NodeKeypair::generate().unwrap();
        let bytes = kp1.secret_bytes();
        let kp2 = NodeKeypair::from_bytes(&bytes).unwrap();
        assert_eq!(kp1.public_bytes(), kp2.public_bytes());
    }

    #[test]
    fn sign_and_verify() {
        let kp = NodeKeypair::generate().unwrap();
        let message = b"hello p2p";
        let sig = kp.sign(message);
        let pub_bytes = kp.public_bytes();
        assert!(NodeKeypair::verify(message, &sig, &pub_bytes).is_ok());
    }

    #[test]
    fn verify_rejects_tampered_message() {
        let kp = NodeKeypair::generate().unwrap();
        let message = b"hello p2p";
        let sig = kp.sign(message);
        let pub_bytes = kp.public_bytes();
        let tampered = b"hello p2p!";
        assert!(NodeKeypair::verify(tampered, &sig, &pub_bytes).is_err());
    }

    #[test]
    fn from_bytes_rejects_invalid_length() {
        let err = NodeKeypair::from_bytes(&[0u8; 16]).unwrap_err();
        assert!(matches!(err, crate::Error::Identity(_)));
    }
}
