# Changelog

## bkg-cave — Version History

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-07-29

### Added

#### Core Infrastructure (cave-core)

- `CpioWriter` with newc (SVR4) format support
- `CpioReader` for archive parsing
- `CpioEntry` with file, directory, symlink, device types
- `PluginManager` with static and dynamic plugin support
- `PluginContext` for plugin initialization
- `RegisteredCommand` for CLI command registration
- `PluginKind` enum (Surge, Kindle, Forge, Core, Utility)
- `EventBus` with typed event subscription
- `CaveEvent` enum covering all domains
- `Config` with TOML deserialization
- `Shell` utility for command execution
- Unified `Error` type with `thiserror`
- Compression support: gzip, xz, zstd

#### VM Runner (cave-surge)

- QEMU integration with process spawning
- 8 architecture plugins:
  - x86_64 (KVM support)
  - i386
  - aarch64
  - arm
  - riscv64
  - ppc64le
  - s390x
  - mips64
- Kernel module loading from `/lib/modules`
- Custom kernel argument support
- Initramfs integration with cave-kindle
- QEMU argument passthrough

#### Initramfs Generator (cave-kindle)

- TOML-based configuration
- Hook system (prep, modules, files, post)
- Binary collection with `ldd` resolution
- Library dependency tracking
- Kernel module inclusion/exclusion filtering
- Device node creation
- Compression with gzip/xz/zstd
- CPIO archive generation via cave-core

#### OS Image Builder (cave-forge)

- Pipeline stage execution engine
- BuildRoot directory management
- Object store with content-addressed storage
- Stage types: mount, packages, grub2, and more
- Output format support: qcow2, raw, iso
- Parallel stage execution
- Build artifact caching

#### CLI (cave)

- `cave surge` subcommand group
- `cave kindle` subcommand group
- `cave forge` subcommand group
- `cave plugin` management commands:
  - `list` — show loaded plugins
  - `load` — load dynamic plugin
  - `unload` — unload plugin
  - `reload` — hot-reload plugin
  - `info` — show plugin details
- `cave init` — initialize configuration
- `cave doctor` — system dependency check
- Clap-based argument parsing
- Colored terminal output

#### Plugin System

- Static plugin registration at compile time
- Dynamic plugin loading via `dlopen`
- Hot-reload via `SIGHUP` and CLI command
- Plugin ABI version checking
- Event emission on plugin lifecycle
- WASM plugin sandboxing (optional, via wasmtime)

### Documentation

- Complete architecture documentation
- Plugin API guide with examples
- End-user guide for all domains
- Build instructions with cross-compilation
- Testing guide with golden tests
- Contributing guidelines
- This changelog

---

## [Unreleased]

### Planned for 0.2.0

#### Added

- WASM plugin sandboxing with wasmtime
- Dioxus-based TUI/Web UI
- Plugin marketplace / registry
- Remote object store support (S3, GCS)
- Incremental build support for cave-forge
- Initramfs compression benchmarking
- QEMU snapshot support
- Parallel QEMU execution

#### Changed

- Improved error messages with suggestions
- Faster CPIO generation with parallel file hashing

---

## Versioning Policy

- **MAJOR** — Incompatible API changes
- **MINOR** — New functionality, backwards-compatible
- **PATCH** — Bug fixes, backwards-compatible

---

_End of Changelog_
