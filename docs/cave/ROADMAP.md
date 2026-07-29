# Roadmap

## bkg-cave — Project Roadmap

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [Completed Milestones](#completed-milestones)
2. [Current Status](#current-status)
3. [Future Plans](#future-plans)
4. [Feature Requests](#feature-requests)
5. [Release Cadence](#release-cadence)

---

## 1. Completed Milestones

### M1: Project Foundation ✅

- Cargo workspace setup with 5 crates
- `cave-core` foundation library
- Unified `Error` type with `thiserror`
- Basic `Config` and `Shell` utilities

### M2: CPIO Format Support ✅

- `CpioWriter` with newc (SVR4) format
- `CpioReader` for archive parsing
- File, directory, symlink, device node support
- Gzip compression integration

### M3: Plugin System Core ✅

- `CavePlugin` trait definition
- `PluginManager` with static plugin registration
- `PluginContext` and `RegisteredCommand`
- `PluginKind` enum

### M4: Event System ✅

- `EventBus` with typed events
- Subscribe/unsubscribe mechanism
- Async event emission
- Domain-specific events (Surge, Kindle, Forge)

### M5: cave-surge (VM Runner) ✅

- QEMU integration
- 8 architecture plugins (x86_64, i386, aarch64, arm, riscv64, ppc64le, s390x, mips64)
- Kernel module loading
- Custom kernel arguments
- Initramfs integration

### M6: cave-kindle (Initramfs Generator) ✅

- TOML configuration parsing
- Hook system (prep, modules, files, post)
- Binary collection with `ldd` resolution
- Library dependency tracking
- Module inclusion/exclusion
- Compression: gzip, xz, zstd

### M7: cave-forge (OS Image Builder) ✅

- Pipeline stage execution engine
- BuildRoot directory management
- Object store with content-addressed storage
- Stage types: mount, packages, grub2
- Output formats: qcow2, raw, iso
- Parallel stage execution

### M8: CLI and Plugin Management ✅

- `cave surge` subcommand group
- `cave kindle` subcommand group
- `cave forge` subcommand group
- `cave plugin` management (list, load, unload, reload, info)
- `cave init` and `cave doctor`
- Clap-based argument parsing

### M9: Hot-Reload and Dynamic Plugins ✅

- Dynamic plugin loading via `dlopen`
- Hot-reload via CLI and `SIGHUP`
- Plugin ABI version checking
- Event emission on plugin lifecycle
- WASM plugin support (optional, via wasmtime)

---

## 2. Current Status

### Version 0.1.0 — Initial Release

**Status:** Stable  
**Release Date:** 2026-07-29

| Component     | Status      | Notes                              |
| ------------- | ----------- | ---------------------------------- |
| cave-core     | ✅ Stable   | Core library, all features working |
| cave-surge    | ✅ Stable   | All 8 architectures supported      |
| cave-kindle   | ✅ Stable   | Full initramfs generation          |
| cave-forge    | ✅ Stable   | Pipeline-based image building      |
| cave (CLI)    | ✅ Stable   | All subcommands functional         |
| Plugin system | ✅ Stable   | Static + dynamic + hot-reload      |
| Documentation | ✅ Complete | All 10 docs written                |

### Known Limitations

| Limitation                   | Severity | Planned Fix |
| ---------------------------- | -------- | ----------- |
| No WASM sandbox yet          | Medium   | M10         |
| No GUI/TUI                   | Low      | M10         |
| Limited stage types in forge | Medium   | M11         |
| No remote object store       | Low      | M12         |
| No incremental builds        | Medium   | M11         |

---

## 3. Future Plans

### M10: Web UI and WASM (v0.2.0)

**Target:** Q4 2026

| Feature             | Description                     | Priority |
| ------------------- | ------------------------------- | -------- |
| Dioxus TUI          | Terminal UI for interactive use | High     |
| Dioxus Web UI       | Browser-based UI                | Medium   |
| WASM plugin sandbox | Full wasmtime integration       | High     |
| Plugin marketplace  | Browse/install plugins          | Medium   |
| Remote object store | S3/GCS backend for forge        | Low      |

**Dioxus UI Mockup:**

```
┌─────────────────────────────────────────────┐
│  🏔️ bkg-cave                                │
├─────────────────────────────────────────────┤
│  [Surge] [Kindle] [Forge] [Plugins]        │
├─────────────────────────────────────────────┤
│                                             │
│  Surge Runner                               │
│  ┌─────────────────────────────────────┐    │
│  │ Kernel: [/path/to/vmlinuz]          │    │
│  │ Memory: [2G]                        │    │
│  │ CPU: [host]                         │    │
│  │ SMP: [4]                            │    │
│  │                                     │    │
│  │ [▶ Run]  [■ Stop]                   │    │
│  └─────────────────────────────────────┘    │
│                                             │
│  Console Output:                            │
│  ┌─────────────────────────────────────┐    │
│  │ Booting kernel...                    │    │
│  │ [    0.000000] Linux version 6.1... │    │
│  │ [    0.001234] Booting...           │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

### M11: Advanced Build Features (v0.3.0)

**Target:** Q1 2027

| Feature            | Description                            |
| ------------------ | -------------------------------------- |
| Incremental builds | Only rebuild changed stages            |
| Build caching      | Persistent cache across sessions       |
| More stage types   | LVM, RAID, encryption, cloud-init      |
| Build dependencies | Stage dependency graph                 |
| Parallel pipelines | Run independent pipelines concurrently |
| Build signing      | Sign output images                     |

### M12: Cloud and Distribution (v0.4.0)

**Target:** Q2 2027

| Feature                | Description                 |
| ---------------------- | --------------------------- |
| Remote object store    | S3, GCS, Azure Blob         |
| Cloud-init integration | Generate cloud-init configs |
| Container images       | Build OCI/Docker images     |
| Package formats        | DEB, RPM, Flatpak           |
| Update system          | A/B update support          |

### M13: Ecosystem and Polish (v0.5.0)

**Target:** Q3 2027

| Feature               | Description                       |
| --------------------- | --------------------------------- |
| Plugin registry       | Central plugin repository         |
| Plugin signing        | Cryptographic plugin verification |
| IDE integration       | LSP, rust-analyzer support        |
| Comprehensive testing | 90%+ code coverage                |
| Performance profiling | Built-in profiling tools          |
| Internationalization  | i18n support                      |

---

## 4. Feature Requests

### Under Consideration

| Feature                         | Requested By  | Status    | Notes                     |
| ------------------------------- | ------------- | --------- | ------------------------- |
| Docker/Podman backend for forge | Community     | Planned   | Alternative to BuildRoot  |
| Nix integration                 | Community     | Exploring | Use Nix as package source |
| Disk image signing              | Security team | Planned   | GPG/sigstore integration  |
| SBOM generation                 | Security team | Planned   | SPDX/CycloneDX output     |
| Live migration support          | VM team       | Exploring | QEMU migration protocol   |
| GPU passthrough                 | VM team       | Planned   | VFIO support              |
| TPM emulation                   | Security team | Planned   | QEMU TPM device           |
| Secure boot                     | Security team | Planned   | Shim/sbctl integration    |

### How to Request a Feature

1. Open a [GitHub Issue](https://github.com/example/bkg-cave/issues) with the `enhancement` label
2. Describe the feature and use case
3. Discuss with maintainers in the issue
4. If approved, it will be added to the roadmap

---

## 5. Release Cadence

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **0.x.y** — Initial development, API may change
- **1.0.0** — First stable release, API frozen
- **x.y.z** — Minor/patch releases are backwards-compatible

### Release Schedule

| Version | Target Date | Type              |
| ------- | ----------- | ----------------- |
| 0.1.0   | 2026-07-29  | Initial release   |
| 0.2.0   | Q4 2026     | Web UI, WASM      |
| 0.3.0   | Q1 2027     | Advanced builds   |
| 0.4.0   | Q2 2027     | Cloud integration |
| 0.5.0   | Q3 2027     | Ecosystem         |
| 1.0.0   | TBD         | Stable API        |

### Release Process

1. Feature freeze — no new features, only bug fixes
2. Beta release — `0.x.0-beta.1` for testing
3. Release candidate — `0.x.0-rc.1`
4. Final release — `0.x.0`
5. Backport critical fixes to previous release

---

## 6. Long-term Vision

### Year 1: Foundation

- Complete core functionality (M1-M9) ✅
- Stable plugin system ✅
- Comprehensive documentation ✅

### Year 2: Expansion

- Web UI and WASM plugins (M10)
- Advanced build features (M11)
- Cloud integration (M12)

### Year 3: Ecosystem

- Plugin marketplace
- IDE integrations
- Enterprise features (signing, SBOM, compliance)

### Ultimate Goal

bkg-cave aims to become the **standard toolchain for OS artifact management** — a single, extensible, safe platform that replaces the current patchwork of virtme, ugrd, osbuild, and other specialized tools.

---

_End of Roadmap_
