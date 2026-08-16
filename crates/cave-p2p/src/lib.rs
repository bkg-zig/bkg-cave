//! cave-p2p — Decentralized P2P AI agent network plugin for bkg-cave.
//!
//! This crate ports bkg-p2p v0.3.0 into the bkg-cave plugin architecture.
//!
//! ## Milestones
//!
//! - **M0**: Workspace bootstrap & plugin skeleton (completed)
//! - **M1**: Identity system (completed)
//! - **M2**: Network core (libp2p) (completed)
//! - **M3-A**: Events & EventBus integration (completed)
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

pub mod a2a;
pub mod agent_library;
pub mod bootstrap;
pub mod cli;
pub mod compute_fabric;
pub mod config;
pub mod db;
pub mod error;
pub mod events;
pub mod executor;
pub mod identity;
pub mod job;
pub mod network;
pub mod node;
pub mod observability;
pub mod p2p;
pub mod plugin;
pub mod prompts;
pub mod proxy;
pub mod reputation;
pub mod routines;
pub mod run;
pub mod runtime;
pub mod swarm;
pub mod wallet;
pub mod web;
pub mod workspace;

pub use config::Config;
pub use error::{Error, Result};
pub use events::{EventPublisher, P2pEvent};
pub use identity::{NodeId, NodeIdentity};
pub use network::NetworkService;
pub use plugin::P2pPlugin;
