//! `cave-forge` — pipeline-based OS image builder (port of osbuild).
//!
//! This crate provides a full pipeline engine for building operating-system
//! images from composable stage manifests.  It is implemented as a
//! [`CavePlugin`](cave_core::CavePlugin) that can be loaded into any
//! bkg-cave host process.
//!
//! # Architecture
//!
//! * **pipeline** — manifest data model and topological resolver
//! * **buildroot** — bubblewrap-based sandbox for running stages
//! * **objectstore** — content-addressed tree store (SHA256-keyed)
//! * **meta** — module index that validates stage/assembler availability
//! * **sources** — source resolution and download
//! * **cli** — command-line argument parsing
//! * **plugin** — [`CavePlugin`](cave_core::CavePlugin) trait implementation and command dispatch

pub mod buildroot;
pub mod cli;
pub mod meta;
pub mod objectstore;
pub mod pipeline;
pub mod plugin;
pub mod sources;

// ── Re-exports ───────────────────────────────────────────────────────────────

pub use plugin::ForgePlugin;
