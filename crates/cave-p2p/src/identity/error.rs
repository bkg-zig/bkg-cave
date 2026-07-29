//! Identity-specific error types.

use crate::Error;
use thiserror::Error;

/// Result alias for identity operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors specific to identity operations.
#[derive(Debug, Error)]
pub enum IdentityError {
    /// Key generation failed.
    #[error("key generation failed: {0}")]
    KeyGeneration(String),

    /// Keypair is invalid.
    #[error("invalid keypair: {0}")]
    InvalidKeypair(String),

    /// Signature verification failed.
    #[error("signature verification failed")]
    SignatureVerification,

    /// Identity file I/O error.
    #[error("identity I/O error: {0}")]
    Io(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Identity not found.
    #[error("identity not found: {0}")]
    NotFound(String),
}

impl From<IdentityError> for crate::Error {
    fn from(err: IdentityError) -> Self {
        crate::Error::Identity(err.to_string())
    }
}

impl From<std::io::Error> for IdentityError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for IdentityError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
