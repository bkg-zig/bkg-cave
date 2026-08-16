//! `cave-core` — foundational crate for the bkg-cave project.
//!
//! This crate provides the core types and utilities used across all
//! bkg-cave components: error handling, configuration management,
//! an event bus, a plugin system, a CPIO archive writer, shell code
//! generation, and general-purpose utilities.

pub mod config;
pub mod cpio;
pub mod error;
pub mod event;
pub mod plugin;
pub mod shell;
pub mod util;

// ── Re-exports ──────────────────────────────────────────────────────────────

pub use config::ConfigRegistry;
pub use cpio::{CpioEntryType, CpioWriter};
pub use error::BkgCaveError;
pub use event::{CaveEvent, EventBus};
pub use plugin::{
    CavePlugin, PluginContext, PluginKind, PluginManager, PluginState, RegisteredCommand,
};
pub use shell::ShellCodeGenerator;
pub use util::{copy_recursively, find_binary, path_to_karg, quote_karg, walk_files};
