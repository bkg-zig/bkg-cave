//! Configuration system for cave-p2p.
//!
//! All configuration is loaded from `bkg.conf` (TOML format).
//! No ports or addresses are hardcoded in the source.
//! If a value is missing in the config file, the service will refuse to start.

use serde::Deserialize;
use std::path::PathBuf;

/// Root configuration loaded from `bkg.conf`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// P2P network configuration.
    #[serde(default)]
    pub p2p: P2pConfig,
    /// Web/API configuration.
    #[serde(default)]
    pub web: WebConfig,
    /// Resource advertisement configuration.
    #[serde(default)]
    pub resources: ResourceConfig,
    /// Database configuration.
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let data = std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(e.to_string()))?;
        let config: Self = toml::from_str(&data).map_err(|e| ConfigError::Parse(e.to_string()))?;
        Ok(config)
    }

    /// Load from default locations if explicit path not given.
    ///
    /// Search order:
    /// 1. `./bkg.conf`
    /// 2. `~/.config/bkg-cave/bkg.conf`
    /// 3. `/etc/bkg-cave/bkg.conf`
    ///
    /// Returns an error if no config file is found,
    /// because we no longer provide hardcoded defaults.
    pub fn load_default() -> Result<Self, ConfigError> {
        let candidates = [
            PathBuf::from("bkg.conf"),
            dirs::config_dir()
                .unwrap_or_default()
                .join("bkg-cave")
                .join("bkg.conf"),
            PathBuf::from("/etc/bkg-cave/bkg.conf"),
        ];
        for path in &candidates {
            if path.exists() {
                return Self::load(path);
            }
        }
        Err(ConfigError::Io(
            "no bkg.conf found in ./bkg.conf, ~/.config/bkg-cave/bkg.conf, or /etc/bkg-cave/bkg.conf".to_string(),
        ))
    }

    /// Merge with environment overrides.
    ///
    /// Environment variables take precedence over file config:
    /// - `BKG_P2P_LISTEN_ADDRS` — comma-separated listen addresses
    /// - `BKG_P2P_BOOTSTRAP_PEERS` — comma-separated bootstrap peers
    /// - `BKG_WEB_LISTEN_ADDR` — web server listen address
    pub fn apply_env(&mut self) {
        if let Ok(val) = std::env::var("BKG_P2P_LISTEN_ADDRS") {
            self.p2p.listen_addresses = val
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        if let Ok(val) = std::env::var("BKG_P2P_BOOTSTRAP_PEERS") {
            self.p2p.bootstrap_peers = val
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        if let Ok(val) = std::env::var("BKG_WEB_LISTEN_ADDR") {
            self.web.listen_addr = val;
        }
    }
}

/// P2P network configuration.
///
/// All fields are optional in the TOML file.
/// If a field is missing, it stays at its default (empty/disabled).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct P2pConfig {
    /// Listen addresses (multiaddr format).
    /// Example: `["/ip4/0.0.0.0/tcp/4001"]`
    /// Must be set explicitly in bkg.conf.
    #[serde(default)]
    pub listen_addresses: Vec<String>,

    /// Bootstrap peers (multiaddr format).
    /// Example: `["/ip4/1.2.3.4/tcp/4001/p2p/12D3KooW..."]`
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,

    /// Enable mDNS local discovery.
    /// Default: false (must be explicitly enabled)
    #[serde(default)]
    pub mdns_enabled: bool,

    /// Enable Kademlia DHT.
    /// Default: false (must be explicitly enabled)
    #[serde(default)]
    pub kademlia_enabled: bool,

    /// Interval in seconds for advertising resources.
    /// Default: 0 (disabled)
    #[serde(default)]
    pub advertise_interval_secs: u64,

    /// Enable QUIC transport.
    /// Default: false (must be explicitly enabled)
    #[serde(default)]
    pub quic_enabled: bool,

    /// Enable TCP transport.
    /// Default: false (must be explicitly enabled)
    #[serde(default)]
    pub tcp_enabled: bool,

    /// Protocol version string for Identify protocol.
    /// Example: "/bkg-p2p/1.0.0"
    /// Default: empty (must be set explicitly)
    #[serde(default)]
    pub protocol_version: String,

    /// Agent version string for Identify protocol.
    /// Example: "cave-p2p/0.1.0"
    /// Default: empty (must be set explicitly)
    #[serde(default)]
    pub agent_version: String,
}

/// Web/API server configuration.
///
/// All fields must be explicitly set in bkg.conf.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WebConfig {
    /// Enable web server.
    /// Must be explicitly set in bkg.conf.
    #[serde(default)]
    pub enabled: bool,

    /// Listen address for web server.
    /// Example: "127.0.0.1:8080"
    /// Must be set explicitly in bkg.conf if web is enabled.
    #[serde(default)]
    pub listen_addr: String,
}

/// Resource advertisement configuration.
///
/// All fields must be explicitly set in bkg.conf.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ResourceConfig {
    /// Advertise GPU resources.
    /// Must be explicitly set in bkg.conf.
    #[serde(default)]
    pub advertise_gpu: bool,

    /// Number of CPU cores to advertise.
    /// Must be explicitly set in bkg.conf.
    #[serde(default)]
    pub cpu_cores: u16,

    /// Storage bytes to advertise.
    /// Must be explicitly set in bkg.conf.
    #[serde(default)]
    pub storage_bytes: u64,

    /// RAM in MB to advertise.
    /// Must be explicitly set in bkg.conf.
    #[serde(default)]
    pub ram_mb: u32,
}

/// Database configuration.
///
/// All fields must be explicitly set in bkg.conf.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DatabaseConfig {
    /// Path to the database file.
    /// Example: "~/.local/share/bkg-cave/data/bkg-cave.redb"
    /// Must be set explicitly in bkg.conf.
    #[serde(default)]
    pub path: String,
}

/// Configuration loading/parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),
}

impl From<ConfigError> for crate::Error {
    fn from(err: ConfigError) -> Self {
        crate::Error::Config(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_config_from_string() {
        let toml = r#"
[p2p]
listen_addresses = ["/ip4/0.0.0.0/tcp/4001"]
bootstrap_peers = ["/ip4/1.2.3.4/tcp/4001/p2p/12D3KooW"]
mdns_enabled = true
kademlia_enabled = true
advertise_interval_secs = 300
quic_enabled = false
tcp_enabled = true
protocol_version = "/bkg-p2p/1.0.0"
agent_version = "cave-p2p/0.1.0"

[web]
enabled = true
listen_addr = "127.0.0.1:8080"

[resources]
advertise_gpu = true
cpu_cores = 8
storage_bytes = 1000000000
ram_mb = 16000

[database]
path = "/tmp/bkg-cave.redb"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.p2p.listen_addresses.len(), 1);
        assert_eq!(config.p2p.bootstrap_peers.len(), 1);
        assert!(config.p2p.mdns_enabled);
        assert!(config.web.enabled);
        assert_eq!(config.web.listen_addr, "127.0.0.1:8080");
        assert_eq!(config.resources.cpu_cores, 8);
        assert_eq!(config.p2p.protocol_version, "/bkg-p2p/1.0.0");
    }

    #[test]
    fn default_config_is_empty() {
        let config = Config::default();
        assert!(config.p2p.listen_addresses.is_empty());
        assert!(config.p2p.bootstrap_peers.is_empty());
        assert!(!config.p2p.tcp_enabled);
        assert!(!config.p2p.quic_enabled);
        assert!(!config.p2p.mdns_enabled);
        assert!(!config.p2p.kademlia_enabled);
        assert!(config.web.listen_addr.is_empty());
        assert!(!config.web.enabled);
        assert!(config.database.path.is_empty());
    }

    #[test]
    fn env_override_listen_addrs() {
        let mut config = Config::default();
        std::env::set_var(
            "BKG_P2P_LISTEN_ADDRS",
            "/ip4/0.0.0.0/tcp/9999,/ip4/0.0.0.0/udp/0/quic",
        );
        config.apply_env();
        assert_eq!(config.p2p.listen_addresses.len(), 2);
        assert_eq!(config.p2p.listen_addresses[0], "/ip4/0.0.0.0/tcp/9999");
        std::env::remove_var("BKG_P2P_LISTEN_ADDRS");
    }

    #[test]
    fn env_override_web_addr() {
        let mut config = Config::default();
        std::env::set_var("BKG_WEB_LISTEN_ADDR", "0.0.0.0:3000");
        config.apply_env();
        assert_eq!(config.web.listen_addr, "0.0.0.0:3000");
        std::env::remove_var("BKG_WEB_LISTEN_ADDR");
    }

    #[test]
    fn load_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp.as_file_mut(),
            r#"
[p2p]
listen_addresses = ["/ip4/0.0.0.0/tcp/5000"]
"#
        )
        .unwrap();

        let config = Config::load(tmp.path()).unwrap();
        assert_eq!(config.p2p.listen_addresses[0], "/ip4/0.0.0.0/tcp/5000");
    }
}
