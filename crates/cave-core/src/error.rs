//! Core error type for the bkg-cave project.
//!
//! All fallible operations across `cave-core` and its sibling crates
//! return [`BkgCaveError`], a `thiserror`-derived enum covering I/O,
//! parsing, serialisation, plugin, configuration and subprocess errors.

use std::fmt;

/// Unified error type for the bkg-cave ecosystem.
#[derive(Debug)]
pub enum BkgCaveError {
    /// Wraps [`std::io::Error`].
    Io(std::io::Error),

    /// A file or binary was not found.
    NotFound(String),

    /// Invalid input / bad argument.
    InvalidInput(String),

    /// Wraps [`std::num::ParseIntError`].
    ParseInt(std::num::ParseIntError),

    /// Wraps [`std::string::FromUtf8Error`].
    ParseUtf8(std::string::FromUtf8Error),

    /// Serialisation / deserialisation failure (serde, JSON, TOML, …).
    Serialization(String),

    /// Plugin-system error.
    Plugin(String),

    /// Configuration error.
    Config(String),

    /// CPIO archive error.
    Cpio(String),

    /// External subprocess failed.
    Execution {
        /// The command that was run.
        command: String,
        /// The exit code returned by the process.
        exit_code: i32,
        /// Anything written to stderr.
        stderr: String,
    },

    /// Feature / operation is not supported.
    Unsupported(String),
}

// ── std::error::Error impl ──────────────────────────────────────────────────

impl std::error::Error for BkgCaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BkgCaveError::Io(e) => Some(e),
            BkgCaveError::ParseInt(e) => Some(e),
            BkgCaveError::ParseUtf8(e) => Some(e),
            _ => None,
        }
    }
}

// ── Display ─────────────────────────────────────────────────────────────────

impl fmt::Display for BkgCaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BkgCaveError::Io(e) => write!(f, "I/O error: {e}"),
            BkgCaveError::NotFound(msg) => write!(f, "not found: {msg}"),
            BkgCaveError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            BkgCaveError::ParseInt(e) => write!(f, "integer parse error: {e}"),
            BkgCaveError::ParseUtf8(e) => write!(f, "UTF-8 parse error: {e}"),
            BkgCaveError::Serialization(msg) => write!(f, "serialisation error: {msg}"),
            BkgCaveError::Plugin(msg) => write!(f, "plugin error: {msg}"),
            BkgCaveError::Config(msg) => write!(f, "configuration error: {msg}"),
            BkgCaveError::Cpio(msg) => write!(f, "CPIO error: {msg}"),
            BkgCaveError::Execution {
                command,
                exit_code,
                stderr,
            } => {
                write!(
                    f,
                    "command `{command}` failed with exit code {exit_code}: {stderr}"
                )
            }
            BkgCaveError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

// ── From impls ──────────────────────────────────────────────────────────────

impl From<std::io::Error> for BkgCaveError {
    fn from(e: std::io::Error) -> Self {
        BkgCaveError::Io(e)
    }
}

impl From<std::num::ParseIntError> for BkgCaveError {
    fn from(e: std::num::ParseIntError) -> Self {
        BkgCaveError::ParseInt(e)
    }
}

impl From<std::string::FromUtf8Error> for BkgCaveError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        BkgCaveError::ParseUtf8(e)
    }
}

impl From<serde_json::Error> for BkgCaveError {
    fn from(e: serde_json::Error) -> Self {
        BkgCaveError::Serialization(e.to_string())
    }
}

impl From<toml::de::Error> for BkgCaveError {
    fn from(e: toml::de::Error) -> Self {
        BkgCaveError::Config(format!("TOML deserialisation error: {e}"))
    }
}

impl From<toml::ser::Error> for BkgCaveError {
    fn from(e: toml::ser::Error) -> Self {
        BkgCaveError::Config(format!("TOML serialisation error: {e}"))
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn display_io_error() {
        let e = BkgCaveError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
        let msg = e.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("missing"));
    }

    #[test]
    fn display_not_found() {
        let e = BkgCaveError::NotFound("foobar".into());
        assert_eq!(e.to_string(), "not found: foobar");
    }

    #[test]
    fn display_execution() {
        let e = BkgCaveError::Execution {
            command: "false".into(),
            exit_code: 1,
            stderr: "oh no".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("false"));
        assert!(msg.contains("exit code 1"));
        assert!(msg.contains("oh no"));
    }

    #[test]
    fn from_serde_json() {
        let invalid = r#"{"broken"#;
        let err: BkgCaveError = serde_json::from_str::<serde_json::Value>(invalid)
            .unwrap_err()
            .into();
        assert!(matches!(err, BkgCaveError::Serialization(_)));
    }

    #[test]
    fn from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: BkgCaveError = io_err.into();
        assert!(matches!(err, BkgCaveError::Io(_)));
    }

    #[test]
    fn error_impl_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let e = BkgCaveError::Io(io_err);
        assert!(e.source().is_some());
    }
}
