//! Extensible plugin system for bkg-cave.
//!
//! The [`PluginManager`] loads [`CavePlugin`] implementations and exposes
//! their [`RegisteredCommand`]s so they can be dispatched at runtime.
//!
//! Two kinds of plugins are supported:
//!
//! * **Static plugins** — compiled in and registered via
//!   [`PluginManager::load_static`].
//! * **Dynamic plugins** (behind `cfg(feature = "dynamic")`) — loaded at
//!   runtime from shared libraries via `libloading`.

use crate::error::BkgCaveError;
use crate::event::EventBus;

/// The category of a plugin.
///
/// Used for grouping and filtering in CLI output and UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginKind {
    /// Surge — kernel & image builder plugin.
    Surge,
    /// Kindle — device / platform plugin.
    Kindle,
    /// Forge — filesystem & initramfs plugin.
    Forge,
    /// Utility — helper / standalone command plugin.
    Utility,
}

/// A command registered by a plugin.
///
/// `run` receives a slice of string arguments (excluding the command name)
/// and returns an exit code (0 for success, non-zero for failure).
#[derive(Debug, Clone, Copy)]
pub struct RegisteredCommand {
    /// Short, unique command name (e.g. `"build-kernel"`).
    pub name: &'static str,
    /// Human-readable description shown in `--help`.
    pub description: &'static str,
    /// The function to invoke when this command is executed.
    pub run: fn(&[String]) -> Result<i32, BkgCaveError>,
}

/// Context provided to plugins during initialisation.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Directory for persistent configuration files.
    pub config_dir: std::path::PathBuf,
    /// Directory for persistent data files.
    pub data_dir: std::path::PathBuf,
}

/// Trait every cave plugin must implement.
///
/// All methods **except** `id` have default implementations, but most plugins
/// will want to override `name`, `version`, `kind`, and `commands`.
pub trait CavePlugin: Send + Sync {
    /// Unique, stable identifier for this plugin (e.g. `"cave-surge"`).
    ///
    /// This is used for logging, event emission, and deduplication.
    fn id(&self) -> &'static str;

    /// Human-readable display name (e.g. `"Cave Surge Builder"`).
    fn name(&self) -> &str {
        self.id()
    }

    /// Semantic version string.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// The kind / category of this plugin.
    fn kind(&self) -> PluginKind {
        PluginKind::Utility
    }

    /// Initialise the plugin with the given context.
    ///
    /// Return `Err` to signal that the plugin should not be registered.
    fn init(&mut self, _ctx: &PluginContext) -> Result<(), BkgCaveError> {
        Ok(())
    }

    /// Return the list of commands this plugin provides.
    fn commands(&self) -> Vec<RegisteredCommand> {
        Vec::new()
    }

    /// Whether this plugin supports hot-reloading (re-initialisation without
    /// restarting the host process).
    fn hot_reloadable(&self) -> bool {
        true
    }
}

/// Storage for a loaded plugin.
pub enum PluginState {
    /// A statically-compiled plugin.
    Static(Box<dyn CavePlugin>),
    /// A dynamically-loaded plugin (requires `dynamic` feature).
    #[cfg(feature = "dynamic")]
    Dynamic {
        /// The plugin instance.
        plugin: Box<dyn CavePlugin>,
        /// The loaded shared library handle.
        library: Box<libloading::Library>,
        /// Filesystem path of the `.so` / `.dll`.
        path: std::path::PathBuf,
    },
}

/// Central plugin registry and command dispatcher.
///
/// # Example
///
/// ```ignore
/// use cave_core::plugin::*;
/// use cave_core::BkgCaveError;
///
/// struct MyPlugin;
/// impl CavePlugin for MyPlugin {
///     fn id(&self) -> &'static str { "my-plugin" }
/// }
///
/// let mut mgr = PluginManager::new();
/// let ctx = PluginContext {
///     config_dir: "/etc/cave".into(),
///     data_dir: "/var/lib/cave".into(),
/// };
/// mgr.load_static(Box::new(MyPlugin), &ctx).unwrap();
/// assert_eq!(mgr.list_plugins().len(), 1);
/// ```
pub struct PluginManager {
    plugins: Vec<(&'static str, PluginState)>,
    event_bus: EventBus,
}

impl PluginManager {
    /// Create a new, empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            event_bus: EventBus::new(),
        }
    }

    /// Create a new plugin manager with a shared event bus.
    pub fn with_event_bus(event_bus: EventBus) -> Self {
        Self {
            plugins: Vec::new(),
            event_bus,
        }
    }

    /// Load a static plugin, calling its `init` method.
    ///
    /// If the plugin's `init` returns an error the plugin is **not**
    /// registered and the error is propagated.
    pub fn load_static(
        &mut self,
        mut plugin: Box<dyn CavePlugin>,
        ctx: &PluginContext,
    ) -> Result<(), BkgCaveError> {
        plugin.init(ctx)?;
        let id = plugin.id();
        self.plugins.push((id, PluginState::Static(plugin)));
        self.event_bus
            .emit(crate::event::CaveEvent::PluginLoaded(id.to_owned()));
        Ok(())
    }

    /// Find a registered command by name across all loaded plugins.
    ///
    /// Returns `None` if no plugin exposes a command with that name.
    pub fn find_command(&self, name: &str) -> Option<RegisteredCommand> {
        for (_, state) in &self.plugins {
            let plugin: &dyn CavePlugin = match state {
                PluginState::Static(p) => p.as_ref(),
                #[cfg(feature = "dynamic")]
                PluginState::Dynamic { plugin, .. } => plugin.as_ref(),
            };
            for cmd in plugin.commands() {
                if cmd.name == name {
                    return Some(cmd);
                }
            }
        }
        None
    }

    /// Execute a command by name with the given arguments.
    ///
    /// Returns `Err(BkgCaveError::NotFound(_))` if no such command exists,
    /// otherwise propagates the command's result.
    pub fn execute(&self, name: &str, args: &[String]) -> Result<i32, BkgCaveError> {
        let cmd = self
            .find_command(name)
            .ok_or_else(|| BkgCaveError::NotFound(format!("command '{name}' not found")))?;
        (cmd.run)(args)
    }

    /// List all loaded plugins as `(id, name, kind)` tuples.
    pub fn list_plugins(&self) -> Vec<(&str, &str, PluginKind)> {
        self.plugins
            .iter()
            .map(|(_, state)| match state {
                PluginState::Static(p) => (p.id(), p.name(), p.kind()),
                #[cfg(feature = "dynamic")]
                PluginState::Dynamic { plugin, .. } => (plugin.id(), plugin.name(), plugin.kind()),
            })
            .collect()
    }

    /// Iterate over all loaded plugins.
    pub fn iter_plugins(&self) -> impl Iterator<Item = &dyn CavePlugin> {
        self.plugins.iter().map(|(_, state)| match state {
            PluginState::Static(p) => p.as_ref(),
            #[cfg(feature = "dynamic")]
            PluginState::Dynamic { plugin, .. } => plugin.as_ref(),
        })
    }

    /// Return a reference to the internal event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Minimal test plugin.
    struct TestPlugin {
        id: &'static str,
        init_flag: std::sync::Arc<AtomicBool>,
    }

    impl TestPlugin {
        fn new(id: &'static str) -> Self {
            Self {
                id,
                init_flag: std::sync::Arc::new(AtomicBool::new(false)),
            }
        }
        fn with_flag(id: &'static str, flag: std::sync::Arc<AtomicBool>) -> Self {
            Self {
                id,
                init_flag: flag,
            }
        }
    }

    impl CavePlugin for TestPlugin {
        fn id(&self) -> &'static str {
            self.id
        }
        fn init(&mut self, _ctx: &PluginContext) -> Result<(), BkgCaveError> {
            self.init_flag.store(true, Ordering::SeqCst);
            Ok(())
        }
        fn commands(&self) -> Vec<RegisteredCommand> {
            vec![RegisteredCommand {
                name: "hello",
                description: "say hello",
                run: |args| {
                    println!("hello {args:?}");
                    Ok(0)
                },
            }]
        }
    }

    #[test]
    fn load_static_plugin() {
        let mut mgr = PluginManager::new();
        let ctx = PluginContext {
            config_dir: "/tmp/cfg".into(),
            data_dir: "/tmp/data".into(),
        };

        let plugin = TestPlugin::new("test-load");
        mgr.load_static(Box::new(plugin), &ctx).unwrap();

        let plugins = mgr.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].0, "test-load");
    }

    #[test]
    fn find_and_execute_command() {
        let mut mgr = PluginManager::new();
        let ctx = PluginContext {
            config_dir: "/tmp/cfg".into(),
            data_dir: "/tmp/data".into(),
        };

        mgr.load_static(Box::new(TestPlugin::new("test-cmd")), &ctx)
            .unwrap();

        let cmd = mgr.find_command("hello");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "hello");

        let exit = mgr.execute("hello", &[]).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn execute_unknown_command() {
        let mgr = PluginManager::new();
        let result = mgr.execute("nope", &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BkgCaveError::NotFound(_)));
    }

    #[test]
    fn init_called_on_load() {
        let mut mgr = PluginManager::new();
        let ctx = PluginContext {
            config_dir: "/tmp/cfg".into(),
            data_dir: "/tmp/data".into(),
        };

        let flag = std::sync::Arc::new(AtomicBool::new(false));
        let plugin = TestPlugin::with_flag("test-init", flag.clone());
        mgr.load_static(Box::new(plugin), &ctx).unwrap();

        // init() was called during load_static — verify via the shared flag.
        assert!(flag.load(Ordering::SeqCst));
        assert_eq!(mgr.list_plugins().len(), 1);
    }

    #[test]
    fn empty_manager() {
        let mgr = PluginManager::new();
        assert!(mgr.list_plugins().is_empty());
        assert!(mgr.find_command("anything").is_none());
    }

    #[test]
    fn event_bus_integration() {
        let bus = EventBus::new();
        let loaded = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = loaded.clone();

        bus.subscribe(
            "test-counter",
            std::sync::Arc::new(move |event| {
                if matches!(event, crate::event::CaveEvent::PluginLoaded(_)) {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }),
        );

        let mut mgr = PluginManager::with_event_bus(bus);
        let ctx = PluginContext {
            config_dir: "/tmp/cfg".into(),
            data_dir: "/tmp/data".into(),
        };

        mgr.load_static(Box::new(TestPlugin::new("p1")), &ctx)
            .unwrap();
        mgr.load_static(Box::new(TestPlugin::new("p2")), &ctx)
            .unwrap();

        assert_eq!(loaded.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
