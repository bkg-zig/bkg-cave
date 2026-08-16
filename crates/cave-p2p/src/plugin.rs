use std::path::PathBuf;

use cave_core::event::EventBus;
use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::BkgCaveError;

use crate::config::Config;
use crate::events::EventPublisher;
use crate::identity::NodeIdentity;
use crate::network::{BehaviourConfig, NetworkService};

/// The cave-p2p plugin identifier.
pub const PLUGIN_ID: &str = "cave-p2p";

/// P2P plugin for bkg-cave.
///
/// Provides decentralized P2P networking for AI agent collaboration.
pub struct P2pPlugin {
    initialized: bool,
    identity: Option<NodeIdentity>,
    network: Option<NetworkService>,
    data_dir: PathBuf,
    event_bus: Option<EventPublisher>,
}

impl P2pPlugin {
    /// Create a new P2P plugin instance.
    pub fn new() -> Self {
        Self {
            initialized: false,
            identity: None,
            network: None,
            data_dir: PathBuf::new(),
            event_bus: None,
        }
    }

    fn identity_path(&self) -> PathBuf {
        self.data_dir.join("identity.json")
    }

    fn config_path(&self) -> PathBuf {
        self.data_dir.join("bkg.conf")
    }

    fn load_or_create_identity(&self) -> Result<NodeIdentity, BkgCaveError> {
        let path = self.identity_path();
        if path.exists() {
            NodeIdentity::load(&path).map_err(|e| BkgCaveError::Plugin(e.to_string()))
        } else {
            let identity =
                NodeIdentity::generate().map_err(|e| BkgCaveError::Plugin(e.to_string()))?;
            identity
                .save(&path)
                .map_err(|e| BkgCaveError::Plugin(e.to_string()))?;
            Ok(identity)
        }
    }
}

impl Default for P2pPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl P2pPlugin {
    /// Attach an EventBus for publishing events.
    ///
    /// This allows the plugin to emit events that other plugins
    /// and the host can subscribe to.
    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(crate::events::EventPublisher::new(std::sync::Arc::new(bus)));
        tracing::info!("EventBus attached to {}", PLUGIN_ID);
        self
    }

    /// Get the event publisher if one is attached.
    pub fn event_bus(&self) -> Option<&EventPublisher> {
        self.event_bus.as_ref()
    }
}

impl CavePlugin for P2pPlugin {
    fn id(&self) -> &'static str {
        PLUGIN_ID
    }

    fn name(&self) -> &str {
        "Cave P2P Network"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn kind(&self) -> PluginKind {
        PluginKind::Utility
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), BkgCaveError> {
        tracing::info!("{} initializing...", PLUGIN_ID);

        self.data_dir = ctx.data_dir.clone();

        let identity = self.load_or_create_identity()?;
        tracing::info!("node identity: {}", identity.node_id_string());

        let config = Config::load(self.config_path())
            .or_else(|_| Config::load_default())
            .unwrap_or_else(|e| {
                tracing::warn!("no config found, using defaults: {e}");
                Config::default()
            });

        let behaviour_config = BehaviourConfig::new(identity.clone(), config.p2p.clone());
        let network = NetworkService::new(
            identity.clone(),
            config.p2p.clone(),
            config.web.clone(),
            behaviour_config,
        )
        .map_err(|e| BkgCaveError::Plugin(e.to_string()))?;

        self.identity = Some(identity);
        self.network = Some(network);
        self.initialized = true;

        tracing::info!("{} initialized (M2: identity + network)", PLUGIN_ID);
        Ok(())
    }

    fn commands(&self) -> Vec<RegisteredCommand> {
        vec![
            RegisteredCommand {
                name: "p2p-status",
                description: "Show P2P plugin status",
                run: |_args| {
                    println!("cave-p2p: plugin loaded (M2: identity + network)");
                    Ok(0)
                },
            },
            RegisteredCommand {
                name: "p2p-peer-id",
                description: "Show the local node's peer ID",
                run: |_args| {
                    println!("Not available outside plugin context");
                    Ok(0)
                },
            },
        ]
    }

    fn hot_reloadable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_metadata() {
        let plugin = P2pPlugin::new();
        assert_eq!(plugin.id(), "cave-p2p");
        assert_eq!(plugin.name(), "Cave P2P Network");
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn plugin_commands() {
        let plugin = P2pPlugin::new();
        let cmds = plugin.commands();
        assert!(!cmds.is_empty());
        assert_eq!(cmds[0].name, "p2p-status");
    }
}
