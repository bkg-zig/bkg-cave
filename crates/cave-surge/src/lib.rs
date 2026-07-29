//! `cave-surge` — kernel & image builder plugin for the bkg-cave project.
//!
//! Surge is a port of `virtme` that lets you boot a Linux VM sharing the
//! host's root filesystem via 9p virtfs.  It is implemented as a
//! [`CavePlugin`](cave_core::plugin::CavePlugin) so it can be loaded
//! dynamically by the cave-core plugin manager.
//!
//! # Sub-modules
//!
//! | Module        | Description                                      |
//! |---------------|--------------------------------------------------|
//! | [`arch`]      | Architecture abstraction (8 targets).             |
//! | [`qemu`]      | QEMU binary discovery, version probing, virtfs.  |
//! | [`modules`]   | Kernel module resolution with `modprobe`.        |
//! | [`modaliases`]| Static modalias table for auto-detection.        |
//! | [`initramfs`] | CPIO initramfs builder (busybox, dev, init).     |
//! | [`cli`]       | CLI argument parser (clap derive).               |
//! | [`plugin`]    | [`SurgePlugin`] + main `execute()` function.     |
//!
//! # Quick start
//!
//! ```ignore
//! use cave_surge::SurgePlugin;
//! use cave_core::plugin::{PluginManager, PluginContext};
//!
//! let mut mgr = PluginManager::new();
//! let ctx = PluginContext {
//!     config_dir: "/etc/cave".into(),
//!     data_dir:   "/var/lib/cave".into(),
//! };
//! mgr.load_static(Box::new(SurgePlugin::new()), &ctx).unwrap();
//! mgr.execute("surge", &["--dry-run".to_string()]).unwrap();
//! ```

pub mod arch;
pub mod cli;
pub mod initramfs;
pub mod modaliases;
pub mod modules;
pub mod plugin;
pub mod qemu;

/// Re-export the main plugin type for convenience.
pub use plugin::SurgePlugin;
