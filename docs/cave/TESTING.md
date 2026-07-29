# Testing Guide

## bkg-cave — Testing Documentation

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [Running Tests](#running-tests)
2. [Test Structure per Crate](#test-structure-per-crate)
3. [Writing New Tests](#writing-new-tests)
4. [Golden Tests for CPIO](#golden-tests-for-cpio)
5. [Integration Tests](#integration-tests)
6. [CI/CD Pipeline](#cicd-pipeline)

---

## 1. Running Tests

### 1.1 Run All Tests

```bash
cargo test --workspace
```

### 1.2 Run Tests for a Specific Crate

```bash
cargo test -p cave-core
cargo test -p cave-surge
cargo test -p cave-kindle
cargo test -p cave-forge
```

### 1.3 Run Tests with Output

```bash
cargo test -- --nocapture
```

### 1.4 Run a Specific Test

```bash
cargo test test_name
cargo test test_name -- --exact
```

### 1.5 Run Tests by Filter

```bash
cargo test cpio    # Run all tests containing "cpio"
cargo test surge   # Run all surge tests
```

### 1.6 Run Tests with Logging

```bash
RUST_LOG=debug cargo test
```

### 1.7 Run Benchmarks

```bash
cargo bench --workspace
```

---

## 2. Test Structure per Crate

### 2.1 cave-core

```
crates/cave-core/
├── src/
│   ├── lib.rs
│   ├── cpio.rs
│   ├── plugin.rs
│   ├── event.rs
│   ├── config.rs
│   └── ...
└── tests/
    ├── cpio_tests.rs
    ├── plugin_tests.rs
    ├── event_tests.rs
    └── config_tests.rs
```

**Test categories:**

| Category          | Location                                 | Description                |
| ----------------- | ---------------------------------------- | -------------------------- |
| Unit tests        | `#[cfg(test)] mod tests` in source files | Test individual functions  |
| Integration tests | `tests/*.rs`                             | Test public API            |
| Doc tests         | In `///` comments                        | Test code examples in docs |

### 2.2 cave-surge

```
crates/cave-surge/
├── src/
│   ├── lib.rs
│   ├── qemu.rs
│   ├── arch.rs
│   └── modules.rs
└── tests/
    ├── qemu_tests.rs
    ├── arch_tests.rs
    └── integration_tests.rs
```

### 2.3 cave-kindle

```
crates/cave-kindle/
├── src/
│   ├── lib.rs
│   ├── generator.rs
│   ├── hooks.rs
│   └── compressor.rs
└── tests/
    ├── generator_tests.rs
    ├── hooks_tests.rs
    └── golden_tests.rs
```

### 2.4 cave-forge

```
crates/cave-forge/
├── src/
│   ├── lib.rs
│   ├── pipeline.rs
│   ├── stage.rs
│   └── objectstore.rs
└── tests/
    ├── pipeline_tests.rs
    ├── stage_tests.rs
    └── objectstore_tests.rs
```

---

## 3. Writing New Tests

### 3.1 Unit Tests

Place unit tests in the same file as the code:

```rust
// src/cpio.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_cpio_newc_header_size() {
        let entry = CpioEntry::new_file("test.txt", b"hello");
        let mut buf = Vec::new();
        entry.write_header(&mut buf).unwrap();
        assert_eq!(buf.len(), 110); // newc header is 110 bytes
    }

    #[test]
    fn test_cpio_roundtrip() {
        let mut writer = CpioWriter::new(CpioFormat::Newc);
        writer.add_file("hello.txt", b"Hello, world!").unwrap();
        writer.add_dir("etc").unwrap();

        let mut buf = Vec::new();
        writer.write(&mut buf).unwrap();

        // Read back
        let mut cursor = Cursor::new(buf);
        let reader = CpioReader::new(&mut cursor);
        let entries: Vec<_> = reader.collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    #[should_panic(expected = "invalid path")]
    fn test_cpio_rejects_absolute_paths() {
        let mut writer = CpioWriter::new(CpioFormat::Newc);
        writer.add_file("/etc/passwd", b"root:x:0:0").unwrap();
    }
}
```

### 3.2 Integration Tests

Place integration tests in `tests/`:

```rust
// tests/cpio_integration.rs

use cave_core::cpio::{CpioWriter, CpioFormat};
use std::io::Cursor;

#[test]
fn test_full_cpio_archive() {
    let mut writer = CpioWriter::new(CpioFormat::Newc);

    writer.add_file("init", b"#!/bin/sh\necho hello").unwrap();
    writer.add_dir("dev").unwrap();
    writer.add_dir("proc").unwrap();
    writer.add_symlink("sbin/init", "bin/busybox").unwrap();
    writer.add_device("dev/console", 5, 1, 0o0600).unwrap();

    let mut buf = Vec::new();
    let size = writer.write(&mut buf).unwrap();

    assert!(size > 0);
    assert!(buf.starts_with(b"070701")); // newc magic
}
```

### 3.3 Async Tests

```rust
#[tokio::test]
async fn test_event_bus_async() {
    let mut bus = EventBus::new();
    let received = Arc::new(Mutex::new(None));

    {
        let received = received.clone();
        bus.subscribe(EventType::Surge, move |event| {
            if let CaveEvent::SurgeStarted { arch, .. } = event {
                *received.lock().unwrap() = Some(arch.clone());
            }
        });
    }

    bus.emit(CaveEvent::SurgeStarted {
        arch: "x86_64".to_string(),
        kernel: PathBuf::from("/boot/vmlinuz"),
    });

    // Give async handlers time to run
    tokio::time::sleep(Duration::from_millis(10)).await;

    assert_eq!(*received.lock().unwrap(), Some("x86_64".to_string()));
}
```

### 3.4 Test Utilities

```rust
// tests/common/mod.rs

use cave_core::cpio::CpioWriter;
use std::fs;

pub fn create_test_initramfs() -> Vec<u8> {
    let mut writer = CpioWriter::new(CpioFormat::Newc);
    writer.add_file("init", b"#!/bin/sh\necho test").unwrap();
    writer.add_dir("dev").unwrap();
    let mut buf = Vec::new();
    writer.write(&mut buf).unwrap();
    buf
}

pub fn with_temp_dir<F>(f: F)
where
    F: FnOnce(&std::path::Path),
{
    let dir = tempfile::tempdir().unwrap();
    f(dir.path());
}
```

---

## 4. Golden Tests for CPIO

Golden tests compare generated output against known-good reference files.

### 4.1 Golden Test Structure

```
crates/cave-kindle/tests/
└── golden/
    ├── minimal.cpio.gz
    ├── minimal.cpio.gz.sha256
    ├── full.cpio.gz
    ├── full.cpio.gz.sha256
    └── ...
```

### 4.2 Golden Test Implementation

```rust
// tests/golden_tests.rs

use cave_core::cpio::{CpioWriter, CpioFormat};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;

fn generate_minimal_initramfs() -> Vec<u8> {
    let mut writer = CpioWriter::new(CpioFormat::Newc);
    writer.add_file("init", b"#!/bin/sh\necho hello").unwrap();
    writer.add_dir("dev").unwrap();

    let mut buf = Vec::new();
    writer.write(&mut buf).unwrap();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&buf).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn test_golden_minimal_initramfs() {
    let generated = generate_minimal_initramfs();
    let golden_path = "tests/golden/minimal.cpio.gz";

    // Update golden file if --update-golden flag is set
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(golden_path, &generated).unwrap();
        fs::write(format!("{}.sha256", golden_path),
            format!("{:x}\n", sha2::Sha256::digest(&generated)))
            .unwrap();
        return;
    }

    // Verify against golden
    let expected = fs::read(golden_path).unwrap();
    assert_eq!(generated, expected, "Initramfs output does not match golden file. Run with UPDATE_GOLDEN=1 to update.");
}
```

### 4.3 Updating Golden Files

```bash
# Update all golden files
UPDATE_GOLDEN=1 cargo test --test golden_tests

# Update specific golden
UPDATE_GOLDEN=1 cargo test test_golden_minimal_initramfs
```

### 4.4 Golden File Checksums

Store SHA-256 checksums alongside golden files:

```
# minimal.cpio.gz.sha256
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  minimal.cpio.gz
```

Verify checksums in CI:

```bash
cd tests/golden
sha256sum -c *.sha256
```

---

## 5. Integration Tests

### 5.1 End-to-End Test Structure

```
tests/
├── e2e/
│   ├── mod.rs
│   ├── surge_e2e.rs
│   ├── kindle_e2e.rs
│   └── forge_e2e.rs
├── common/
│   ├── mod.rs
│   └── fixtures.rs
└── cpio_integration.rs
```

### 5.2 Surge E2E Test

```rust
// tests/e2e/surge_e2e.rs

use cave_surge::QemuRunner;
use std::path::PathBuf;

#[test]
fn test_surge_help() {
    let result = cave_cli::run(&["surge", "--help"]);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("run"));
}

#[test]
fn test_surge_run_validation() {
    // Should fail with missing kernel
    let result = QemuRunner::new()
        .kernel(PathBuf::from("/nonexistent/vmlinuz"))
        .run();
    assert!(result.is_err());
}
```

### 5.3 Kindle E2E Test

```rust
// tests/e2e/kindle_e2e.rs

use cave_kindle::Generator;
use std::fs;

#[test]
fn test_kindle_build_minimal() {
    let config = r#"
[general]
output = "/tmp/test-initramfs.cpio.gz"
compression = "gzip"

[files]
binaries = []
"#;

    let gen = Generator::from_toml(config).unwrap();
    let output = gen.generate().unwrap();

    assert!(output.exists());
    assert!(fs::metadata(&output).unwrap().len() > 0);

    // Cleanup
    let _ = fs::remove_file(output);
}
```

### 5.4 Forge E2E Test

```rust
// tests/e2e/forge_e2e.rs

use cave_forge::{Pipeline, PipelineBuilder};

#[test]
fn test_forge_pipeline_validation() {
    let pipeline = Pipeline::load("tests/fixtures/valid_pipeline.toml").unwrap();
    assert_eq!(pipeline.stages.len(), 3);
}

#[test]
fn test_forge_invalid_pipeline() {
    let result = Pipeline::load("tests/fixtures/invalid_pipeline.toml");
    assert!(result.is_err());
}
```

### 5.5 Test Fixtures

```
tests/fixtures/
├── valid_pipeline.toml
├── invalid_pipeline.toml
├── minimal_init.toml
├── sample_kernel/
│   └── vmlinuz (small test kernel)
└── expected_outputs/
    └── minimal.cpio.gz
```

---

## 6. CI/CD Pipeline

### 6.1 GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Run golden tests
        run: cargo test --test golden_tests
        env:
          UPDATE_GOLDEN: ${{ github.event_name == 'push' }}

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release --all-features

      - name: Upload binary
        uses: actions/upload-artifact@v4
        with:
          name: cave-x86_64
          path: target/release/cave

  cross-compile:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [aarch64-unknown-linux-gnu, riscv64gc-unknown-linux-gnu]
    steps:
      - uses: actions/checkout@v4

      - name: Install target
        run: rustup target add ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
```

### 6.2 Pre-commit Hooks

```yaml
# .pre-commit-config.yaml

repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-toml

  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: cargo-fmt
      - id: cargo-clippy
        args: ["--all-targets", "--all-features", "-D", "warnings"]
```

### 6.3 Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --out Html --output-dir coverage/

# View report
open coverage/tarpaulin-report.html
```

### 6.4 Performance Regression Testing

```bash
# Install cargo-criterion
cargo install cargo-criterion

# Run benchmarks
cargo criterion

# Compare against baseline
cargo criterion --baseline main
```

---

_End of Testing Guide_
