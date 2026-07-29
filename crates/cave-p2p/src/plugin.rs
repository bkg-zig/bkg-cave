//! cave-p2p plugin integration with cave-core.

use cave_core::plugin::{CavePlugin, PluginContext, PluginKind, RegisteredCommand};
use cave_core::BkgCaveError;

/// The cave-p2p plugin identifier.
pub const PLUGIN_ID: &str = "cave-p2p";

/// P2P plugin for bkg-cave.
///
/// Provides decentralized P2P networking for AI agent collaboration.
pub struct P2pPlugin {
    initialized: bool,
}

impl P2pPlugin {
    /// Create a new P2P plugin instance.
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for P2pPlugin {
    fn default() -> Self {
        Self::new()
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

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), BkgCaveError> {
        tracing::info!("{} initializing...", PLUGIN_ID);
        self.initialized = true;
        tracing::info!(
            "{} initialized (M0-M2: plugin, identity, network)",
            PLUGIN_ID
        );
        Ok(())
    }

    fn commands(&self) -> Vec<RegisteredCommand> {
        vec![RegisteredCommand {
            name: "p2p-status",
            description: "Show P2P plugin status",
            run: |_args| {
                println!("cave-p2p: plugin loaded (M0-M2 complete)");
                Ok(0)
            },
        }]
    }

    fn hot_reloadable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cave_core::plugin::PluginContext;

    #[test]
    fn plugin_metadata() {
        let plugin = P2pPlugin::new();
        assert_eq!(plugin.id(), "cave-p2p");
        assert_eq!(plugin.name(), "Cave P2P Network");
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn plugin_init() {
        let mut plugin = P2pPlugin::new();
        let ctx = PluginContext {
            config_dir: "/tmp/cave-p2p-cfg".into(),
            data_dir: "/tmp/cave-p2p-data".into(),
        };
        assert!(plugin.init(&ctx).is_ok());
    }

    #[test]
    fn plugin_commands() {
        let plugin = P2pPlugin::new();
        let cmds = plugin.commands();
        assert!(!cmds.is_empty());
        assert_eq!(cmds[0].name, "p2p-status");
    }
}
