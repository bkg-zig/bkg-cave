# Contributing to bkg-cave

## Contribution Guidelines

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Code Style](#code-style)
4. [Commit Conventions](#commit-conventions)
5. [Pull Request Process](#pull-request-process)
6. [Adding New Plugins](#adding-new-plugins)
7. [Adding New Architectures](#adding-new-architectures)
8. [Reporting Bugs](#reporting-bugs)
9. [Development Setup](#development-setup)

---

## 1. Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Respect differing viewpoints and experiences

---

## 2. Getting Started

### 2.1 Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/bkg-cave.git
cd bkg-cave
git remote add upstream https://github.com/example/bkg-cave.git
```

### 2.2 Create a Branch

```bash
# Each feature/fix gets its own branch
git checkout -b feature/my-new-feature
# or
git checkout -b fix/issue-123
```

### 2.3 Keep Up to Date

```bash
git fetch upstream
git rebase upstream/main
```

---

## 3. Code Style

### 3.1 Formatting

All code must be formatted with `rustfmt`:

```bash
# Format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt -- --check
```

**rustfmt.toml** (project standard):

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
imports_granularity = "Crate"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
```

### 3.2 Linting

All code must pass `clippy` with no warnings:

```bash
# Run clippy on all targets
cargo clippy --all-targets --all-features -- -D warnings

# Run clippy on a specific crate
cargo clippy -p cave-core -- -D warnings
```

**Common clippy rules to follow:**

| Rule                  | Description              |
| --------------------- | ------------------------ |
| `clippy::all`         | All standard lints       |
| `clippy::pedantic`    | Strict pedantic lints    |
| `clippy::cargo`       | Cargo-related lints      |
| `clippy::restriction` | Optional, stricter rules |

### 3.3 Naming Conventions

| Item         | Convention      | Example             |
| ------------ | --------------- | ------------------- |
| Crates       | snake_case      | `cave-core`         |
| Functions    | snake_case      | `build_initramfs()` |
| Structs      | PascalCase      | `CpioWriter`        |
| Traits       | PascalCase      | `CavePlugin`        |
| Enums        | PascalCase      | `PluginKind`        |
| Constants    | SCREAMING_SNAKE | `MAX_SIZE`          |
| Modules      | snake_case      | `cpio_writer.rs`    |
| Type aliases | PascalCase      | `Result<T>`         |

### 3.4 Documentation

All public items must have doc comments:

````rust
/// Builds an initramfs from the given configuration.
///
/// # Arguments
///
/// * `config` - The initramfs configuration
/// * `output` - Path to write the generated initramfs
///
/// # Errors
///
/// Returns an error if:
/// - The configuration is invalid
/// - A required file is missing
/// - The output cannot be written
///
/// # Examples
///
/// ```
/// use cave_kindle::Generator;
///
/// let gen = Generator::from_toml(r#"
///     [general]
///     output = "initramfs.cpio.gz"
/// "#)?;
/// gen.generate()?;
/// ```
pub fn build(config: &Config, output: &Path) -> Result<(), Error> {
    // ...
}
````

### 3.5 Error Handling

```rust
// Good: Use thiserror for typed errors
#[derive(Debug, thiserror::Error)]
pub enum CpioError {
    #[error("invalid magic: expected 070701, got {0}")]
    InvalidMagic(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// Bad: Generic error types
fn build() -> Result<(), String> { ... }
fn build() -> Result<(), Box<dyn std::error::Error>> { ... }
```

---

## 4. Commit Conventions

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### 4.1 Types

| Type       | Description                                             |
| ---------- | ------------------------------------------------------- |
| `feat`     | New feature                                             |
| `fix`      | Bug fix                                                 |
| `docs`     | Documentation only                                      |
| `style`    | Formatting, missing semicolons, etc.                    |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf`     | Performance improvement                                 |
| `test`     | Adding or updating tests                                |
| `chore`    | Maintenance tasks                                       |
| `ci`       | CI/CD changes                                           |

### 4.2 Scopes

| Scope     | Description   |
| --------- | ------------- |
| `core`    | cave-core     |
| `surge`   | cave-surge    |
| `kindle`  | cave-kindle   |
| `forge`   | cave-forge    |
| `cli`     | cave binary   |
| `plugins` | Plugin system |
| `docs`    | Documentation |
| `ci`      | CI/CD         |

### 4.3 Examples

```
feat(surge): add RISC-V 64-bit architecture support

Implements the riscv64 architecture plugin with virt machine type.
Supports -cpu rv64, -machine virt, and OpenSBI firmware loading.

Closes #42
```

```
fix(kindle): handle empty hook directories gracefully

Previously, an empty hooks directory would cause a build error.
Now it's treated as a no-op.

Fixes #67
```

```
docs(architecture): add CPIO format specification

Adds detailed documentation of the newc format header structure
and file type bits.
```

```
chore(deps): update clap to 4.5

Bumps clap from 4.4 to 4.5 for improved derive macros.
```

---

## 5. Pull Request Process

### 5.1 Before Submitting

```bash
# 1. Format code
cargo fmt --all

# 2. Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo test --workspace --all-features

# 4. Update documentation
cargo doc --workspace --no-deps

# 5. Commit changes
git add .
git commit -m "feat(scope): description"
```

### 5.2 PR Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Clippy passes with no warnings
- [ ] Code is formatted with `cargo fmt`
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for user-facing changes)
- [ ] Tests are added for new functionality
- [ ] Commit messages follow conventions

### 5.3 PR Description Template

```markdown
## Summary

Brief description of the change.

## Motivation

Why is this change needed? What problem does it solve?

## Changes

- Change 1
- Change 2
- Change 3

## Testing

How was this tested?

## Breaking Changes

List any breaking changes, or state "None".

## Related Issues

Closes #123
Relates to #456
```

### 5.4 Review Process

1. **Automated checks** — CI runs tests, clippy, fmt
2. **Maintainer review** — At least one maintainer approval required
3. **Address feedback** — Update PR based on review comments
4. **Squash and merge** — PR is squashed into a single commit on main

---

## 6. Adding New Plugins

### 6.1 Plugin Structure

```
plugins/
└── my-plugin/
    ├── Cargo.toml
    ├── README.md
    └── src/
        └── lib.rs
```

### 6.2 Cargo.toml Template

```toml
[package]
name = "cave-surge-myplugin"
version = "0.1.0"
edition = "2021"
description = "My custom surge plugin"
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/bkg-cave"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cave-core = { path = "../../crates/cave-core" }

[dev-dependencies]
cave-core = { path = "../../crates/cave-core", features = ["test-utils"] }
```

### 6.3 Registration

Add the plugin to `cave/src/plugins.rs`:

```rust
#[cfg(feature = "myplugin")]
mod my_plugin;

fn register_plugins(pm: &mut PluginManager) {
    #[cfg(feature = "myplugin")]
    pm.register_static(Box::new(my_plugin::MyPlugin::new()));
}
```

### 6.4 Feature Gate

Add to `cave/Cargo.toml`:

```toml
[features]
default = ["qemu", "hooks"]
myplugin = ["cave-surge/myplugin"]
```

---

## 7. Adding New Architectures

### 7.1 Architecture Plugin Structure

```
crates/cave-surge/src/arch/
├── mod.rs
├── x86_64.rs
├── aarch64.rs
├── riscv64.rs   # New architecture
└── ...
```

### 7.2 Implementation Steps

1. **Create architecture module:**

```rust
// src/arch/riscv64.rs

use cave_core::error::Error;
use std::path::PathBuf;

pub struct RiscV64Arch;

impl RiscV64Arch {
    pub fn qemu_binary() -> &'static str {
        "qemu-system-riscv64"
    }

    pub fn default_machine() -> &'static str {
        "virt"
    }

    pub fn default_cpu() -> &'static str {
        "rv64"
    }

    pub fn firmware_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/usr/share/opensbi/generic/fw_jump.elf"),
            PathBuf::from("/usr/lib/opensbi/generic/fw_jump.elf"),
        ]
    }

    pub fn kernel_args(arch_config: &ArchConfig) -> Vec<String> {
        vec![
            "-machine".to_string(),
            format!("{},gic=3".format(Self::default_machine())),
            "-cpu".to_string(),
            arch_config.cpu.clone().unwrap_or_else(|| Self::default_cpu().to_string()),
            "-smp".to_string(),
            arch_config.smp.unwrap_or(4).to_string(),
            "-m".to_string(),
            arch_config.memory.clone().unwrap_or_else(|| "2G".to_string()),
        ]
    }
}
```

2. **Register in `mod.rs`:**

```rust
mod riscv64;
pub use riscv64::RiscV64Arch;

// In ArchPlugin enum:
pub enum ArchPlugin {
    X86_64(x86_64::X8664Arch),
    AArch64(aarch64::AArch64Arch),
    RiscV64(riscv64::RiscV64Arch),
    // ...
}
```

3. **Add to feature gate:**

```toml
# cave-surge/Cargo.toml
[features]
default = ["x86_64", "aarch64"]
all-archs = ["x86_64", "aarch64", "riscv64", "ppc64le", "s390x", "mips64", "arm", "i386"]
```

4. **Add tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riscv64_defaults() {
        assert_eq!(RiscV64Arch::qemu_binary(), "qemu-system-riscv64");
        assert_eq!(RiscV64Arch::default_machine(), "virt");
    }

    #[test]
    fn test_riscv64_kernel_args() {
        let args = RiscV64Arch::kernel_args(&ArchConfig::default());
        assert!(args.contains(&"-machine".to_string()));
        assert!(args.contains(&"virt".to_string()));
    }
}
```

---

## 8. Reporting Bugs

### 8.1 Before Reporting

1. Check [existing issues](https://github.com/example/bkg-cave/issues)
2. Update to the latest version
3. Try to reproduce with minimal configuration

### 8.2 Bug Report Template

```markdown
**Description**
Brief description of the bug.

**Steps to Reproduce**

1. Run `cave surge run --kernel /path/to/vmlinuz`
2. Observe error: "..."

**Expected Behavior**
What should happen.

**Actual Behavior**
What actually happens.

**Environment**

- OS: (e.g., Arch Linux, Ubuntu 24.04)
- bkg-cave version: (output of `cave --version`)
- Rust version: (output of `rustc --version`)
- QEMU version: (if applicable)

**Logs**
```

Paste relevant log output here

````

**Configuration**
```toml
Paste relevant config.toml sections
````

**Additional Context**
Any other information that might help.

````

### 8.3 Feature Requests

```markdown
**Feature Description**
What feature would you like to see?

**Use Case**
Why is this feature needed? What problem does it solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
What other approaches did you consider?

**Additional Context**
Screenshots, mockups, references, etc.
````

---

## 9. Development Setup

### 9.1 Full Development Environment

```bash
# 1. Clone repository
git clone https://github.com/example/bkg-cave.git
cd bkg-cave

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy

# 3. Install pre-commit hooks
pip install pre-commit
pre-commit install

# 4. Build everything
cargo build --all-features

# 5. Run tests
cargo test --workspace --all-features

# 6. Generate docs
cargo doc --workspace --open
```

### 9.2 IDE Setup

**VS Code** (`.vscode/settings.json`):

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "files.insertFinalNewline": true
}
```

**Vim/Neovim** (with rust-analyzer):

```lua
require('lspconfig').rust_analyzer.setup{
    settings = {
        ["rust-analyzer"] = {
            cargo = { allFeatures = true },
            checkOnSave = { command = "clippy" },
        },
    },
}
```

### 9.3 Useful Commands

```bash
# Watch mode for development
cargo watch -x "check --workspace" -x "test --workspace"

# Run specific test repeatedly
cargo watch -x "test test_name -- --exact"

# Generate coverage
cargo tarpaulin --workspace --out Html

# Check for security vulnerabilities
cargo audit

# Update dependencies
cargo update
```

---

_End of Contributing Guide_
