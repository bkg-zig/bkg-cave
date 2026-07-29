# Migration Guide

## bkg-cave — Migration from Legacy Tools

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [Overview](#overview)
2. [From virtme to cave-surge](#from-virtme-to-cave-surge)
3. [From ugrd to cave-kindle](#from-ugrd-to-cave-kindle)
4. [From osbuild to cave-forge](#from-osbuild-to-cave-forge)
5. [Configuration Migration](#configuration-migration)
6. [Breaking Changes](#breaking-changes)
7. [Migration Checklist](#migration-checklist)

---

## 1. Overview

This guide helps you migrate from the legacy tools (`virtme`, `ugrd`, `osbuild`) to `bkg-cave`.

| Legacy Tool | bkg-cave Equivalent | Migration Complexity             |
| ----------- | ------------------- | -------------------------------- |
| `virtme`    | `cave surge`        | Low — CLI is similar             |
| `ugrd`      | `cave kindle`       | Medium — Config format changed   |
| `osbuild`   | `cave forge`        | Medium — Pipeline format changed |

---

## 2. From virtme to cave-surge

### 2.1 Command Mapping

| virtme                                  | cave-surge                                              | Notes             |
| --------------------------------------- | ------------------------------------------------------- | ----------------- |
| `virtme-run --kernel /path/to/vmlinuz`  | `cave surge run --kernel /path/to/vmlinuz`              | Direct equivalent |
| `virtme-run --kdir /path/to/kernel/dir` | `cave surge run --kernel-dir /path/to/kernel/dir`       |                   |
| `virtme-run --mods /path/to/modules`    | `cave surge run --modules /path/to/modules`             |                   |
| `virtme-run --memory 4G`                | `cave surge run --memory 4G`                            |                   |
| `virtme-run --cpu host`                 | `cave surge run --cpu host`                             |                   |
| `virtme-run --qemu-args "-nographic"`   | `cave surge run --qemu-args "-nographic"`               |                   |
| `virtme-run --arch x86_64`              | `cave surge run --arch x86_64`                          |                   |
| `virtme --kdir /path/to/kernel/dir`     | `cave surge initramfs --kernel-dir /path/to/kernel/dir` |                   |

### 2.2 virtme-run Example → cave-surge

**Before (virtme):**

```bash
virtme-run \
  --kdir /usr/lib/modules/6.1.0-arch1-1 \
  --memory 4G \
  --cpu host \
  --qemu-args "-nographic" \
  --append "console=ttyS0 root=/dev/sda1"
```

**After (cave-surge):**

```bash
cave surge run \
  --kernel-dir /usr/lib/modules/6.1.0-arch1-1 \
  --memory 4G \
  --cpu host \
  --qemu-args "-nographic" \
  --append "console=ttyS0 root=/dev/sda1"
```

### 2.3 virtme Config → cave Config

**Before (virtme config):**

```bash
# ~/.config/virtme/config
KERNEL_DIR=/usr/lib/modules/$(uname -r)
QEMU_ARGS="-m 2G -smp 2"
```

**After (cave config):**

```toml
# ~/.config/cave/config.toml
[surge]
kernel_dir = "/usr/lib/modules/6.1.0-arch1-1"
default_memory = "2G"
default_cpu = "host"
```

### 2.4 virtme-init → cave-kindle

**Before (virtme):**

```bash
virtme --kdir /usr/lib/modules/6.1.0-arch1-1 --initramfs
```

**After (cave):**

```bash
cave kindle build --config init.toml --output initramfs.cpio.gz
cave surge run --kernel /boot/vmlinuz --initrd initramfs.cpio.gz
```

---

## 3. From ugrd to cave-kindle

### 3.1 ugrd Config → cave-kindle Config

**Before (ugrd config):**

```bash
# /etc/ugrd.conf
MODULES="ext4 vfat nfs overlay"
BINARIES="/bin/busybox /sbin/udevd"
HOOK_DIR="/etc/ugrd/hooks"
COMPRESSION="gzip"
```

**After (cave-kindle config):**

```toml
# /etc/cave/init.toml
[modules]
include = ["ext4", "vfat", "nfs", "overlay"]

[files]
binaries = ["/bin/busybox", "/sbin/udevd"]

[hooks]
directory = "/etc/cave/hooks"

[general]
compression = "gzip"
```

### 3.2 ugrd Hooks → cave-kindle Hooks

**Before (ugrd hook):**

```bash
#!/bin/bash
# /etc/ugrd/hooks/10-custom.sh
# ugrd hook format

ugrd_add_file /usr/local/bin/my-tool /custom/my-tool
ugrd_load_module my_module
```

**After (cave-kindle hook):**

```bash
#!/bin/bash
# /etc/cave/hooks/10-custom.sh
# cave-kindle hook format

# Environment variables provided:
#   ${initramfs_dir} — root of initramfs being built
#   ${config_file}   — path to config.toml

mkdir -p ${initramfs_dir}/custom
cp /usr/local/bin/my-tool ${initramfs_dir}/custom/

# Load module into initramfs
modprobe -d ${initramfs_dir} my_module
```

### 3.3 ugrd CLI → cave-kindle CLI

| ugrd                        | cave-kindle                                      | Notes     |
| --------------------------- | ------------------------------------------------ | --------- |
| `ugrd -c /etc/ugrd.conf`    | `cave kindle build --config /etc/cave/init.toml` |           |
| `ugrd -o initramfs.cpio.gz` | `cave kindle build --output initramfs.cpio.gz`   |           |
| `ugrd --list`               | `cave kindle list`                               | (planned) |
| `ugrd --verbose`            | `cave kindle build --verbose`                    |           |

### 3.4 ugrd Module Handling

**Before (ugrd):**

```bash
# ugrd.conf
MODULES="ext4 vfat nfs"
MOD_BLACKLIST="radeon nouveau"
```

**After (cave-kindle):**

```toml
# init.toml
[modules]
include = ["ext4", "vfat", "nfs"]
exclude = ["radeon", "nouveau"]
```

---

## 4. From osbuild to cave-forge

### 4.1 osbuild Manifest → cave-forge Pipeline

**Before (osbuild manifest):**

```json
{
  "version": "2.0.0",
  "pipelines": [
    {
      "name": "os",
      "stages": [
        {
          "type": "org.osbuild.mount",
          "inputs": {
            "tree": { "type": "org.osbuild.tree" }
          },
          "outputs": {
            "mount": { "type": "org.osbuild.directory" }
          }
        },
        {
          "type": "org.osbuild.packages",
          "inputs": {
            "mount": { "type": "org.osbuild.directory" }
          },
          "options": {
            "packages": ["bash", "coreutils"]
          },
          "outputs": {
            "tree": { "type": "org.osbuild.tree" }
          }
        }
      ]
    }
  ]
}
```

**After (cave-forge pipeline):**

```toml
# pipeline.toml
[build]
name = "os"
version = "2.0.0"

[[stages]]
name = "mount"
type = "org.osbuild.mount"
inputs = { tree = { type = "org.osbuild.tree" } }
outputs = { mount = { type = "org.osbuild.directory" } }

[[stages]]
name = "packages"
type = "org.osbuild.packages"
inputs = { mount = { type = "org.osbuild.directory" } }
options = { packages = ["bash", "coreutils"] }
outputs = { tree = { type = "org.osbuild.tree" } }
```

### 4.2 osbuild CLI → cave-forge CLI

| osbuild               | cave-forge                                                    | Notes     |
| --------------------- | ------------------------------------------------------------- | --------- |
| `osbuild -o tree.tar` | `cave forge build --pipeline pipeline.toml --output tree.tar` |           |
| `osbuild-manifest`    | `cave forge stages --pipeline pipeline.toml`                  |           |
| `osbuild-store`       | `cave forge store`                                            | (planned) |

### 4.3 osbuild Object Store → cave-forge Object Store

**Before (osbuild):**

```bash
# osbuild stores objects in /var/lib/osbuild/store
# with content-addressed paths:
# /var/lib/osbuild/store/objects/<prefix>/<hash>
```

**After (cave-forge):**

```bash
# cave-forge stores objects in /var/cache/cave-forge
# with the same content-addressed scheme:
# /var/cache/cave-forge/<prefix>/<hash>

# Migration:
sudo mv /var/lib/osbuild/store /var/cache/cave-forge
```

### 4.4 osbuild Build Root → cave-forge Build Root

**Before (osbuild):**

```bash
# osbuild uses /var/lib/osbuild/buildroot by default
```

**After (cave-forge):**

```bash
# cave-forge uses /var/lib/cave-forge/buildroot by default

# Migration:
sudo mv /var/lib/osbuild/buildroot /var/lib/cave-forge/buildroot
```

---

## 5. Configuration Migration

### 5.1 Global Config Migration

**Before (multiple config files):**

```bash
# ~/.config/virtme/config
KERNEL_DIR=/usr/lib/modules/$(uname -r)

# /etc/ugrd.conf
MODULES="ext4 vfat"

# /etc/osbuild.conf
STORAGE=/var/lib/osbuild/store
```

**After (single config file):**

```toml
# ~/.config/cave/config.toml
[global]
data_dir = "/var/lib/cave"
cache_dir = "/var/cache/cave"

[surge]
kernel_dir = "/usr/lib/modules/6.1.0-arch1-1"

[kindle]
include_modules = ["ext4", "vfat"]

[forge]
object_store = "/var/cache/cave-forge"
```

### 5.2 Environment Variables

| Legacy              | bkg-cave            | Notes |
| ------------------- | ------------------- | ----- |
| `VIRTME_KERNEL_DIR` | `CAVE_KERNEL_DIR`   |       |
| `UGRD_CONFIG`       | `CAVE_CONFIG`       |       |
| `OSBUILD_STORAGE`   | `CAVE_OBJECT_STORE` |       |

### 5.3 Data Directory Migration

```bash
# Migrate virtme data
sudo mv /var/lib/virtme /var/lib/cave

# Migrate ugrd data
sudo mv /var/lib/ugrd /var/lib/cave

# Migrate osbuild data
sudo mv /var/lib/osbuild /var/lib/cave-forge
sudo mv /var/cache/osbuild /var/cache/cave-forge
```

---

## 6. Breaking Changes

### 6.1 CLI Changes

| Change               | Before                      | After                      |
| -------------------- | --------------------------- | -------------------------- |
| Binary name          | `virtme`, `ugrd`, `osbuild` | `cave`                     |
| Subcommand structure | Separate binaries           | Unified `cave <domain>`    |
| Config flag          | `-c /path/to/config`        | `--config /path/to/config` |

### 6.2 Config Format Changes

| Change         | Before                  | After                 |
| -------------- | ----------------------- | --------------------- |
| Format         | Shell variables, JSON   | TOML                  |
| Module lists   | Space-separated strings | TOML arrays           |
| Hook execution | Inline scripts          | External script files |

### 6.3 API Changes (for plugin developers)

| Change           | Before           | After                        |
| ---------------- | ---------------- | ---------------------------- |
| Plugin interface | N/A (no plugins) | `CavePlugin` trait           |
| Plugin loading   | N/A              | `dlopen` / WASM              |
| Event system     | N/A              | `EventBus` with typed events |

### 6.4 Removed Features

| Feature           | Status  | Alternative                 |
| ----------------- | ------- | --------------------------- |
| virtme `--script` | Removed | Use cave-kindle hooks       |
| ugrd `--net`      | Removed | Use cave-surge network args |
| osbuild `--json`  | Removed | Use `--output json`         |

---

## 7. Migration Checklist

Use this checklist to track your migration progress:

### Pre-Migration

- [ ] Back up existing configurations
- [ ] Back up existing initramfs images
- [ ] Back up existing build artifacts
- [ ] Document current virtme/ugrd/osbuild versions

### Installation

- [ ] Install bkg-cave (`cargo install bkg-cave` or binary)
- [ ] Verify `cave --version` works
- [ ] Run `cave doctor` to check dependencies

### Configuration

- [ ] Create `~/.config/cave/config.toml`
- [ ] Migrate surge settings from virtme config
- [ ] Migrate kindle settings from ugrd config
- [ ] Migrate forge settings from osbuild config

### virtme → cave-surge

- [ ] Test `cave surge run --help`
- [ ] Run a test kernel with `cave surge run`
- [ ] Verify KVM acceleration works
- [ ] Test custom kernel arguments
- [ ] Test module loading

### ugrd → cave-kindle

- [ ] Convert ugrd.conf to init.toml
- [ ] Convert ugrd hooks to cave-kindle hooks
- [ ] Test `cave kindle build`
- [ ] Verify generated initramfs boots
- [ ] Test compression options

### osbuild → cave-forge

- [ ] Convert osbuild manifest to pipeline.toml
- [ ] Migrate object store
- [ ] Migrate build root
- [ ] Test `cave forge build`
- [ ] Verify output image boots

### Post-Migration

- [ ] Remove old virtme/ugrd/osbuild packages
- [ ] Update scripts and CI/CD pipelines
- [ ] Train team on new CLI
- [ ] Document project-specific configurations
- [ ] Set up monitoring/logging

---

## 8. Getting Help

If you encounter issues during migration:

1. Check the [Troubleshooting section in USER_GUIDE.md](USER_GUIDE.md#troubleshooting)
2. Search [existing issues](https://github.com/example/bkg-cave/issues)
3. Open a new issue with the `migration` label
4. Join our [Matrix/Discord channel](https://example.com/chat)

---

_End of Migration Guide_
