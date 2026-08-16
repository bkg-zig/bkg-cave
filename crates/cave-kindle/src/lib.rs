//! `cave-kindle` — Initramfs generator plugin for bkg-cave.
//!
//! This crate provides the `kindle` command, which generates Linux initramfs
//! images from TOML configuration files.  It is a port / adaptation of the
//! `ugrd` initramfs generator, built as a plugin for the bkg-cave system.
//!
//! # Architecture
//!
//! * **`config`** — [`InitramfsConfig`](config::InitramfsConfig) manages user and module parameters.
//! * **`generator`** — [`Generator`](generator::Generator) orchestrates the build pipeline.
//! * **`generator_helpers`** — [`GeneratorProtocol`](generator_helpers::GeneratorProtocol) bridges the generator
//!   with the [`InitramfsProtocol`](protocol::InitramfsProtocol) trait used by hooks.
//! * **`hooks`** — [`HookRegistry`](hooks::HookRegistry) and [`HookFn`](hooks::HookFn) for extensibility.
//! * **`protocol`** — [`InitramfsProtocol`](protocol::InitramfsProtocol) trait for hook interaction.
//! * **`modules`** — Module discovery, TOML parsing, parameter resolution.
//! * **`base`** — Shell code generators for the init script.
//! * **`error`** — [`KindleError`](error::KindleError) enum with conversions to [`BkgCaveError`](cave_core::BkgCaveError).
//! * **`cli`** — [`KindleArgs`](cli::KindleArgs) via `clap`.
//! * **`plugin`** — [`KindlePlugin`] implementing [`CavePlugin`](cave_core::CavePlugin).

pub mod base;
pub mod cli;
pub mod config;
pub mod error;
pub mod generator;
pub mod generator_helpers;
pub mod hooks;
pub mod modules;
pub mod plugin;
pub mod protocol;

/// Re-export the main plugin entry point.
pub use plugin::KindlePlugin;
