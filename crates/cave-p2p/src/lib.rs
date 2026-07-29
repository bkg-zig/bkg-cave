//! cave-p2p — Decentralized P2P AI agent network plugin for bkg-cave.
//!
//! This crate ports bkg-p2p v0.3.0 into the bkg-cave plugin architecture.
//!
//! ## Milestones
//!
//! - **M0**: Workspace bootstrap & plugin skeleton (completed)
//! - **M1**: Identity system (completed)
//! - **M2**: Network core (libp2p) (in progress)
//! - **M3**: A2A protocol
//! - **M4**: Discovery & compute fabric
//! - **M5**: Economy & distributed jobs
//! - **M6**: Web dashboard core
//! - **M7**: Full web API
//! - **M8**: CLI & plugin integration
//! - **M9**: QA & documentation
//!
//! ## Architecture
//!
//! Runtime is a container, not a god object. All subsystems communicate
//! through the `EventBus` from `cave-core`.
//!
//! ## Configuration
//!
//! All ports and addresses come from `bkg.conf` (TOML).
//! No values are hardcoded in the source.

pub mod config;
pub mod error;
pub mod identity;
pub mod network;
pub mod a2a;
pub mod plugin;

pub use config::Config;
pub use error::{Error, Result};
pub use identity::{NodeId, NodeIdentity};
pub use network::NetworkService;
pub use plugin::P2pPlugin;
