//! Error types for cave-p2p.

use thiserror::Error;

/// Result alias for cave-p2p operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in cave-p2p.
#[derive(Debug, Error)]
pub enum Error {
    /// P2P network error.
    #[error("network error: {0}")]
    Network(String),

    /// Identity/crypto error.
    #[error("identity error: {0}")]
    Identity(String),

    /// Wallet/transaction error.
    #[error("wallet error: {0}")]
    Wallet(String),

    /// Job marketplace error.
    #[error("job error: {0}")]
    Job(String),

    /// Executor error.
    #[error("executor error: {0}")]
    Executor(String),

    /// Web server error.
    #[error("web error: {0}")]
    Web(String),

    /// Database error.
    #[error("database error: {0}")]
    Database(String),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Plugin integration error.
    #[error("plugin error: {0}")]
    Plugin(String),

    /// Resource monitoring error.
    #[error("resource error: {0}")]
    Resource(String),

    /// Inference engine error.
    #[error("inference error: {0}")]
    Inference(String),

    /// WASM execution error.
    #[error("wasm error: {0}")]
    Wasm(String),

    /// A2A protocol error.
    #[error("a2a error: {0}")]
    A2a(String),

    /// Generic/fallback error.
    #[error("p2p error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Other(err.to_string())
    }
}
