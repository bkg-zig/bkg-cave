//! Kindle-specific error types.
//!
//! These errors are used throughout the `cave-kindle` crate and can be
//! converted into the crate-wide [`BkgCaveError`] for unified error handling.

use cave_core::BkgCaveError;

/// Errors that can occur during initramfs generation.
#[derive(thiserror::Error, Debug)]
pub enum KindleError {
    /// A configuration value failed validation.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Automatic detection of a required resource failed.
    #[error("Autodetect failed: {0}")]
    Autodetect(String),

    /// A configuration file or value was malformed or missing.
    #[error("Config error: {0}")]
    Config(String),

    /// A hook function returned an error.
    #[error("Hook error: {0}")]
    Hook(String),

    /// A module definition could not be loaded or processed.
    #[error("Module error: {0}")]
    Module(String),

    /// An I/O operation specific to initramfs building failed.
    #[error("Build error: {0}")]
    Build(String),
}

impl From<KindleError> for BkgCaveError {
    fn from(e: KindleError) -> Self {
        BkgCaveError::Plugin(e.to_string())
    }
}

impl From<std::io::Error> for KindleError {
    fn from(e: std::io::Error) -> Self {
        KindleError::Build(e.to_string())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_validation() {
        let e = KindleError::Validation("root device missing".into());
        assert_eq!(e.to_string(), "Validation error: root device missing");
    }

    #[test]
    fn display_autodetect() {
        let e = KindleError::Autodetect("no kernel found".into());
        assert_eq!(e.to_string(), "Autodetect failed: no kernel found");
    }

    #[test]
    fn display_config() {
        let e = KindleError::Config("invalid compression".into());
        assert_eq!(e.to_string(), "Config error: invalid compression");
    }

    #[test]
    fn display_hook() {
        let e = KindleError::Hook("hook panicked".into());
        assert_eq!(e.to_string(), "Hook error: hook panicked");
    }

    #[test]
    fn display_module() {
        let e = KindleError::Module("module not found".into());
        assert_eq!(e.to_string(), "Module error: module not found");
    }

    #[test]
    fn into_bkg_cave_error() {
        let ke = KindleError::Validation("test".into());
        let bce: BkgCaveError = ke.into();
        assert!(bce.to_string().contains("Validation error: test"));
    }
}
