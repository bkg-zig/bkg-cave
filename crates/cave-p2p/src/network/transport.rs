//! Transport layer for cave-p2p networking.
//!
//! All addresses and ports come from `bkg.conf` via `Config`.
//! No values are hardcoded. If config is empty, validation fails.

use crate::config::{P2pConfig, WebConfig};

/// Errors that can occur during transport setup.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("transport configuration error: {0}")]
    Config(String),

    #[error("transport creation failed: {0}")]
    Creation(String),
}

impl From<TransportError> for crate::Error {
    fn from(err: TransportError) -> Self {
        crate::Error::Network(err.to_string())
    }
}

/// Build listen addresses from config.
///
/// Returns exactly what is configured. No fallback, no defaults.
/// If `p2p_config.listen_addresses` is empty, returns empty vector
/// and validation will fail.
pub fn build_listen_addresses(p2p_config: &P2pConfig) -> Vec<String> {
    p2p_config.listen_addresses.clone()
}

/// Build bootstrap peer list from config.
pub fn build_bootstrap_peers(p2p_config: &P2pConfig) -> Vec<String> {
    p2p_config.bootstrap_peers.clone()
}

/// Validate that transport configuration is consistent.
///
/// Requires:
/// - At least one listen address
/// - At least one transport enabled (TCP or QUIC)
pub fn validate_transport_config(p2p_config: &P2pConfig) -> Result<(), TransportError> {
    if p2p_config.listen_addresses.is_empty() {
        return Err(TransportError::Config(
            "no listen_addresses configured in bkg.conf [p2p] section".to_string(),
        ));
    }
    if !p2p_config.tcp_enabled && !p2p_config.quic_enabled {
        return Err(TransportError::Config(
            "at least one transport (TCP or QUIC) must be enabled in bkg.conf [p2p] section"
                .to_string(),
        ));
    }
    Ok(())
}

/// Resolve the web listen address from config.
///
/// Returns exactly what is configured. No fallback, no defaults.
/// If `web_config.listen_addr` is empty, returns empty string
/// and the web server should refuse to start.
pub fn resolve_web_listen_addr(web_config: &WebConfig) -> String {
    web_config.listen_addr.clone()
}

/// Validate web configuration.
///
/// Requires:
/// - listen_addr must be set if web is enabled
pub fn validate_web_config(web_config: &WebConfig) -> Result<(), TransportError> {
    if web_config.enabled && web_config.listen_addr.is_empty() {
        return Err(TransportError::Config(
            "web.enabled = true but listen_addr is empty in bkg.conf [web] section".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_listen_addresses_fails_validation() {
        let p2p = P2pConfig::default();
        let addrs = build_listen_addresses(&p2p);
        assert!(addrs.is_empty());
        assert!(validate_transport_config(&p2p).is_err());
    }

    #[test]
    fn configured_listen_addresses_are_used() {
        let mut p2p = P2pConfig::default();
        p2p.listen_addresses = vec!["/ip4/0.0.0.0/tcp/9999".to_string()];
        p2p.tcp_enabled = true;
        let addrs = build_listen_addresses(&p2p);
        assert_eq!(addrs[0], "/ip4/0.0.0.0/tcp/9999");
        assert!(validate_transport_config(&p2p).is_ok());
    }

    #[test]
    fn validate_requires_tcp_or_quic() {
        let mut p2p = P2pConfig::default();
        p2p.listen_addresses = vec!["/ip4/0.0.0.0/tcp/4001".to_string()];
        p2p.tcp_enabled = false;
        p2p.quic_enabled = false;
        assert!(validate_transport_config(&p2p).is_err());
    }

    #[test]
    fn validate_passes_with_tcp() {
        let mut p2p = P2pConfig::default();
        p2p.listen_addresses = vec!["/ip4/0.0.0.0/tcp/4001".to_string()];
        p2p.tcp_enabled = true;
        assert!(validate_transport_config(&p2p).is_ok());
    }

    #[test]
    fn web_addr_from_config() {
        let web = WebConfig {
            listen_addr: "0.0.0.0:3000".to_string(),
            enabled: true,
        };
        assert_eq!(resolve_web_listen_addr(&web), "0.0.0.0:3000");
    }

    #[test]
    fn web_validation_requires_addr_when_enabled() {
        let web = WebConfig {
            listen_addr: String::new(),
            enabled: true,
        };
        assert!(validate_web_config(&web).is_err());
    }

    #[test]
    fn web_validation_passes_when_disabled() {
        let web = WebConfig {
            listen_addr: String::new(),
            enabled: false,
        };
        assert!(validate_web_config(&web).is_ok());
    }
}
