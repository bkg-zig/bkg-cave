# Build Documentation

## bkg-cave — Build Instructions

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building from Source](#building-from-source)
3. [Building with Features](#building-with-features)
4. [Cross-Compilation](#cross-compilation)
5. [Dependencies Explanation](#dependencies-explanation)
6. [Build Profiles](#build-profiles)
7. [Troubleshooting Builds](#troubleshooting-builds)

---

## 1. Prerequisites

### 1.1 Required

| Dependency   | Minimum Version | Purpose                         |
| ------------ | --------------- | ------------------------------- |
| Rust         | 1.75+           | Compiler and Cargo              |
| Cargo        | (with Rust)     | Build system                    |
| Git          | 2.30+           | Source control                  |
| pkg-config   | 0.29+           | Native library detection        |
| libssl-dev   | 3.0+            | TLS support (for remote stages) |
| clang / llvm | 15+             | Bindgen (for some plugins)      |

### 1.2 Optional (for full feature set)

| Dependency       | Purpose                   |
| ---------------- | ------------------------- |
| QEMU             | VM execution testing      |
| nasm             | Kernel module compilation |
| squashfs-tools   | Filesystem creation       |
| xorriso          | ISO image creation        |
| gzip / xz / zstd | Compression support       |
| python3          | Hook script execution     |

### 1.3 Install Prerequisites

**Debian / Ubuntu:**

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    clang \
    llvm-dev \
    libclang-dev \
    git \
    curl
```

**Fedora / RHEL:**

```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    clang \
    llvm-devel \
    libclang-devel \
    git \
    curl
```

**Arch Linux:**

```bash
sudo pacman -S --needed \
    base-devel \
    pkg-config \
    openssl \
    clang \
    llvm \
    git \
    curl
```

**macOS:**

```bash
xcode-select --install
brew install pkg-config openssl llvm
```

---

## 2. Building from Source

### 2.1 Clone the Repository

```bash
git clone https://github.com/example/bkg-cave.git
cd bkg-cave
```

### 2.2 Build All Crates

```bash
# Debug build (fast compile, slower runtime)
cargo build

# Output:
#   Compiling cave-core v0.1.0
#   Compiling cave-surge v0.1.0
#   Compiling cave-kindle v0.1.0
#   Compiling cave-forge v0.1.0
#   Compiling cave v0.1.0
#    Finished dev [unoptimized + debuginfo] target(s) in 45.2s
```

### 2.3 Build Only the Binary

```bash
cargo build --bin cave
```

### 2.4 Run Tests

```bash
cargo test --workspace
```

### 2.5 Install Locally

```bash
cargo install --path crates/cave
```

### 2.6 Build Outputs

| Target          | Path                     |
| --------------- | ------------------------ |
| Debug binary    | `target/debug/cave`      |
| Release binary  | `target/release/cave`    |
| Dynamic plugins | `target/release/lib*.so` |

---

## 3. Building with Features

### 3.1 Feature Flags

| Feature           | Crate       | Default | Description                               |
| ----------------- | ----------- | ------- | ----------------------------------------- |
| `dynamic-plugins` | cave-core   | No      | Enable `dlopen`/`dlclose` for .so plugins |
| `wasm-plugins`    | cave-core   | No      | Enable WASM plugin sandboxing             |
| `gzip`            | cave-core   | Yes     | Gzip compression                          |
| `xz`              | cave-core   | No      | XZ compression                            |
| `zstd`            | cave-core   | No      | Zstandard compression                     |
| `qemu`            | cave-surge  | Yes     | QEMU integration                          |
| `all-archs`       | cave-surge  | No      | Enable all 8 architecture plugins         |
| `hooks`           | cave-kindle | Yes     | Shell hook execution                      |
| `buildroot`       | cave-forge  | Yes     | BuildRoot stage support                   |

### 3.2 Build with Dynamic Plugins

```bash
cargo build --features "dynamic-plugins"
```

### 3.3 Build with All Features

```bash
cargo build --all-features
```

### 3.4 Build with Specific Compression

```bash
# Gzip + XZ
cargo build --features "gzip,xz"

# All compressors
cargo build --features "gzip,xz,zstd"
```

### 3.5 Build All Architectures

```bash
cargo build -p cave-surge --features "all-archs"
```

### 3.6 Feature Combinations

```bash
# Full-featured release build
cargo build --release --all-features

# Minimal build (only gzip, no dynamic plugins)
cargo build --features "gzip"

# Dynamic plugins + all archs
cargo build --features "dynamic-plugins,all-archs"
```

---

## 4. Cross-Compilation

### 4.1 Install Target

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add riscv64gc-unknown-linux-gnu
```

### 4.2 Build for Target

```bash
# x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# aarch64 (ARM 64-bit)
cargo build --release --target aarch64-unknown-linux-gnu

# RISC-V 64-bit
cargo build --release --target riscv64gc-unknown-linux-gnu
```

### 4.3 Cross-Compilation with Docker

```bash
# Use official Rust Docker images
docker run --rm -v $(pwd):/build -w /build \
    rust:1.75-slim-bullseye \
    cargo build --release --target x86_64-unknown-linux-gnu
```

### 4.4 Cross-Compiling Plugins

Plugins must be compiled for the target architecture:

```bash
# Build plugin for x86_64
rustup target add x86_64-unknown-linux-gnu
cargo build --release -p my-plugin --target x86_64-unknown-linux-gnu
```

---

## 5. Dependencies Explanation

### 5.1 Core Dependencies (cave-core)

| Crate            | Purpose                     |
| ---------------- | --------------------------- |
| `clap`           | CLI argument parsing        |
| `serde` / `toml` | Configuration serialization |
| `thiserror`      | Typed error definitions     |
| `anyhow`         | Error propagation           |
| `libc`           | FFI for `dlopen`/`dlclose`  |
| `flate2`         | Gzip compression            |
| `xz2`            | XZ compression              |
| `zstd`           | Zstandard compression       |
| `tracing`        | Structured logging          |
| `walkdir`        | Directory traversal         |

### 5.2 Surge Dependencies (cave-surge)

| Crate       | Purpose                   |
| ----------- | ------------------------- |
| `cave-core` | Core library              |
| `nix`       | Unix system calls         |
| `tempfile`  | Temporary file management |
| `which`     | Finding QEMU binary       |

### 5.3 Kindle Dependencies (cave-kindle)

| Crate       | Purpose                  |
| ----------- | ------------------------ |
| `cave-core` | Core library             |
| `cpio`      | CPIO archive handling    |
| `sha2`      | File integrity checksums |

### 5.4 Forge Dependencies (cave-forge)

| Crate        | Purpose                               |
| ------------ | ------------------------------------- |
| `cave-core`  | Core library                          |
| `serde_json` | JSON pipeline stage I/O               |
| `blake3`     | Fast content hashing for object store |

---

## 6. Build Profiles

### 6.1 Debug Profile (default)

```toml
[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"
```

```bash
cargo build
# Fast compile, slow runtime, full debug info
```

### 6.2 Release Profile

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"
```

```bash
cargo build --release
# Slow compile, fast runtime, minimal binary size
```

### 6.3 Profile Comparison

| Profile       | Compile Time | Runtime | Binary Size | Debug Info |
| ------------- | ------------ | ------- | ----------- | ---------- |
| `dev`         | Fast         | Slow    | Large       | Full       |
| `release`     | Slow         | Fast    | Small       | None       |
| `release-lto` | Very Slow    | Fastest | Smallest    | None       |

### 6.4 Custom Profile

Create `.cargo/config.toml`:

```toml
[profile.fast-release]
inherits = "release"
opt-level = 2
lto = "false"
codegen-units = 16
```

```bash
cargo build --profile fast-release
```

---

## 7. Troubleshooting Builds

### 7.1 Linker Errors

**Symptom:** `linker 'cc' not found`

```bash
# Install build tools
sudo apt install build-essential  # Debian/Ubuntu
sudo dnf groupinstall "Development Tools"  # Fedora
```

### 7.2 OpenSSL Errors

**Symptom:** `openssl-sys` compilation fails

```bash
# Debian/Ubuntu
sudo apt install pkg-config libssl-dev

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### 7.3 Slow Compilation

```bash
# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache

# Or use mold linker (Linux)
sudo apt install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### 7.4 Out of Memory During Build

```bash
# Reduce parallel compilation
cargo build -j 2

# Or use swap
sudo fallocate -l 8G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### 7.5 Bindgen Errors

**Symptom:** `bindgen` fails to find clang headers

```bash
# Install libclang
sudo apt install libclang-dev  # Debian/Ubuntu
brew install llvm  # macOS

# Set LIBCLANG_PATH
export LIBCLANG_PATH=/usr/lib/llvm-15/lib
```

---

_End of Build Documentation_
