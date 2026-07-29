# User Guide

## bkg-cave — End-User Documentation

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [What is bkg-cave?](#what-is-bkg-cave)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [cave surge — VM Running](#cave-surge--vm-running)
5. [cave kindle — Initramfs Generation](#cave-kindle--initramfs-generation)
6. [cave forge — OS Image Building](#cave-forge--os-image-building)
7. [Plugin Management](#plugin-management)
8. [Configuration Files](#configuration-files)
9. [Troubleshooting](#troubleshooting)

---

## 1. What is bkg-cave?

`bkg-cave` is a unified toolchain for building, running, and packaging operating-system artifacts. It combines three powerful tools into one binary:

- **cave-surge**: Run kernels in QEMU with custom initramfs
- **cave-kindle**: Generate initramfs images from TOML configuration
- **cave-forge**: Build OS images using pipeline stages

All functionality is extensible via a plugin system.

---

## 2. Installation

### 2.1 From crates.io (Recommended)

```bash
cargo install bkg-cave
```

This installs the `cave` binary with default features.

### 2.2 From Source

```bash
git clone https://github.com/example/bkg-cave.git
cd bkg-cave
cargo install --path crates/cave
```

### 2.3 Pre-built Binary

Download from the [releases page](https://github.com/example/bkg-cave/releases):

```bash
# Linux x86_64
curl -LO https://github.com/example/bkg-cave/releases/download/v0.1.0/cave-x86_64-unknown-linux-gnu.tar.gz
tar xzf cave-x86_64-unknown-linux-gnu.tar.gz
sudo mv cave /usr/local/bin/
```

### 2.4 Verify Installation

```bash
cave --version
# cave 0.1.0

cave --help
```

---

## 3. Quick Start

```bash
# Initialize configuration
cave init

# Run a kernel in QEMU
cave surge run --kernel /boot/vmlinuz-linux

# Generate an initramfs
cave kindle build --config init.toml --output initramfs.cpio.gz

# Build an OS image
cave forge build --pipeline pipeline.toml --output disk.img

# Manage plugins
cave plugin list
cave plugin load /path/to/plugin.so
```

---

## 4. cave surge — VM Running

### 4.1 Run a Kernel

```bash
cave surge run --kernel /boot/vmlinuz-linux
```

### 4.2 Run with Custom Memory and CPU

```bash
cave surge run \
  --kernel /boot/vmlinuz-linux \
  --memory 4G \
  --cpu host \
  --smp 4
```

### 4.3 Run with Custom Initramfs

```bash
cave kindle build --config init.toml --output /tmp/my-initramfs.cpio.gz
cave surge run \
  --kernel /boot/vmlinuz-linux \
  --initrd /tmp/my-initramfs.cpio.gz \
  --append "root=/dev/sda1 console=ttyS0"
```

### 4.4 Run with Kernel Modules

```bash
cave surge run \
  --kernel /boot/vmlinuz-linux \
  --modules /lib/modules/6.1.0-arch1-1 \
  --append "modprobe.blacklist=radeon"
```

### 4.5 Run with Custom QEMU Arguments

```bash
cave surge run \
  --kernel /boot/vmlinuz-linux \
  --qemu-args "-device virtio-gpu-pci" \
  --qemu-args "-display sdl,gl=on"
```

### 4.6 Supported Architectures

| Architecture | QEMU Binary         | Notes                  |
| ------------ | ------------------- | ---------------------- |
| x86_64       | qemu-system-x86_64  | Default, KVM supported |
| i386         | qemu-system-i386    | 32-bit x86             |
| aarch64      | qemu-system-aarch64 | ARM 64-bit             |
| arm          | qemu-system-arm     | ARM 32-bit             |
| riscv64      | qemu-system-riscv64 | RISC-V 64-bit          |
| ppc64le      | qemu-system-ppc64   | PowerPC                |
| s390x        | qemu-system-s390x   | IBM s390x              |
| mips64       | qemu-system-mips64  | MIPS 64-bit            |

### 4.7 Common Kernel Arguments

```bash
# Serial console
--append "console=ttyS0"

# Root filesystem
--append "root=/dev/sda1"

# Init system override
--append "init=/bin/bash"

# Debug initramfs
--append "rd.debug"

# Network boot
--append "ip=dhcp root=/dev/nfs nfsroot=192.168.1.1:/srv/nfs"
```

---

## 5. cave kindle — Initramfs Generation

### 5.1 Basic Initramfs Build

```bash
cave kindle build --config init.toml --output initramfs.cpio.gz
```

### 5.2 Configuration File (init.toml)

```toml
[general]
output = "initramfs.cpio.gz"
compression = "gzip"
compression_level = 6

[hooks]
directory = "/etc/cave/hooks"
order = ["prep", "modules", "files", "post"]

[files]
binaries = ["/bin/busybox", "/sbin/udevd"]
libraries = ["/lib/ld-linux-x86-64.so.2", "/lib/libc.so.6"]
configs = ["/etc/resolv.conf", "/etc/fstab"]

[modules]
include = ["ext4", "vfat", "nfs", "overlay"]
exclude = ["radeon", "nouveau"]

[devices]
create = ["/dev/console", "/dev/null", "/dev/random", "/dev/urandom"]
```

### 5.3 Build with Verbose Output

```bash
cave kindle build --config init.toml --output initramfs.cpio.gz --verbose
```

### 5.4 Build for Specific Kernel

```bash
cave kindle build \
  --config init.toml \
  --output initramfs-6.1.cpio.gz \
  --kernel-version 6.1.0-arch1-1
```

### 5.5 Inspect Generated Initramfs

```bash
# List contents
lsinitrd initramfs.cpio.gz

# Or manually
zcat initramfs.cpio.gz | cpio -t

# Extract
mkdir /tmp/initramfs-extract
cd /tmp/initramfs-extract
zcat /path/to/initramfs.cpio.gz | cpio -idmv
```

### 5.6 Hook Scripts

Place executable scripts in the hooks directory:

```bash
#!/bin/bash
# /etc/cave/hooks/10-custom.sh

# Add custom files
mkdir -p ${initramfs_dir}/custom
cp /usr/local/bin/my-tool ${initramfs_dir}/custom/

# Load modules
modprobe -d ${initramfs_dir} my_custom_module
```

Hooks are executed in alphabetical order by filename.

---

## 6. cave forge — OS Image Building

### 6.1 Basic Build

```bash
cave forge build --pipeline pipeline.toml --output disk.img
```

### 6.2 Pipeline Configuration (pipeline.toml)

```toml
[build]
name = "my-os-image"
version = "1.0.0"

[stages]

[[stages]]
name = "mount"
type = "org.osbuild.mount"
inputs = { tree = { type = "org.osbuild.tree" } }
outputs = { mount = { type = "org.osbuild.directory" } }

[[stages]]
name = "packages"
type = "org.osbuild.packages"
inputs = { mount = { type = "org.osbuild.directory" } }
options = { packages = ["bash", "coreutils", "systemd"] }
outputs = { tree = { type = "org.osbuild.tree" } }

[[stages]]
name = "grub"
type = "org.osbuild.grub2"
inputs = { tree = { type = "org.osbuild.tree" } }
options = { kernel = "/boot/vmlinuz", cmdline = "root=/dev/sda1" }
outputs = { disk = { type = "org.osbuild.image" } }
```

### 6.3 Build with Custom Build Root

```bash
cave forge build \
  --pipeline pipeline.toml \
  --output disk.img \
  --build-root /var/lib/cave-forge/buildroot
```

### 6.4 Output Formats

```bash
# QCOW2 (default, compressed)
cave forge build --pipeline pipeline.toml --output disk.qcow2 --format qcow2

# Raw (uncompressed)
cave forge build --pipeline pipeline.toml --output disk.raw --format raw

# ISO
cave forge build --pipeline pipeline.toml --output disk.iso --format iso
```

### 6.5 Inspect Build Artifacts

```bash
# List stages
cave forge stages --pipeline pipeline.toml

# Show build tree
cave forge tree --build-root /var/lib/cave-forge/buildroot

# Export object store
cave forge export --object-store /var/cache/cave-forge --output artifacts.tar
```

---

## 7. Plugin Management

### 7.1 List Loaded Plugins

```bash
cave plugin list
```

Output:

```
Loaded plugins:
  cave-surge-qemu     v0.1.0  [surge]   Static
  cave-surge-modules  v0.1.0  [surge]   Static
  cave-kindle-hooks   v0.1.0  [kindle]  Static
  cave-forge-grub     v0.1.0  [forge]   Static
```

### 7.2 Load a Plugin

```bash
# Load from path
cave plugin load /usr/lib/cave/plugins/myplugin.so

# Load with name override
cave plugin load /path/to/plugin.so --name my-custom-name
```

### 7.3 Unload a Plugin

```bash
cave plugin unload myplugin
```

### 7.4 Reload a Plugin

```bash
# Reload specific plugin (hot-reload)
cave plugin reload myplugin

# Reload all plugins of a kind
cave plugin reload --kind surge

# Reload all plugins
cave plugin reload --all
```

### 7.5 Plugin Information

```bash
# Show detailed info about a plugin
cave plugin info myplugin

# Output:
# Name: cave-surge-myplugin
# Version: 0.1.0
# Kind: surge
# Type: Dynamic
# Path: /usr/lib/cave/plugins/myplugin.so
# Commands: my-action, my-other-action
# State: Loaded
```

---

## 8. Configuration Files

### 8.1 Configuration File Locations

bkg-cave searches for configuration in the following order (later overrides earlier):

1. `/etc/cave/config.toml` — System-wide
2. `~/.config/cave/config.toml` — User-specific
3. `./cave.toml` — Project-specific
4. `$CAVE_CONFIG` — Environment variable override

### 8.2 Complete Configuration Example

```toml
# /etc/cave/config.toml

[global]
data_dir = "/var/lib/cave"
cache_dir = "/var/cache/cave"
log_level = "info"
max_parallel_jobs = 4
log_file = "/var/log/cave.log"

[surge]
qemu_path = "/usr/bin/qemu-system-x86_64"
default_memory = "2G"
default_cpu = "host"
kernel_dir = "/boot"
module_dir = "/lib/modules"
default_append = "console=ttyS0"

[kindle]
compressor = "gzip"
compression_level = 6
hook_dir = "/etc/cave/hooks"
default_binaries = ["busybox"]
include_modules = ["ext4", "vfat", "nfs", "overlay"]
exclude_modules = ["radeon", "nouveau"]

[forge]
object_store = "/var/cache/cave-forge"
build_root = "/var/lib/cave-forge/buildroot"
default_output_format = "qcow2"
max_build_size = "50G"

[plugins]
directories = [
    "/usr/lib/cave/plugins",
    "/usr/local/lib/cave/plugins",
    "~/.local/lib/cave/plugins",
]
auto_load = true
allow_untrusted = false

[logging]
level = "info"
format = "text"  # or "json"
file = "/var/log/cave.log"
max_size = "100M"
max_files = 5
```

### 8.3 Environment Variables

| Variable          | Description         | Default                 |
| ----------------- | ------------------- | ----------------------- |
| `CAVE_CONFIG`     | Path to config file | (search paths)          |
| `CAVE_DATA_DIR`   | Data directory      | `/var/lib/cave`         |
| `CAVE_CACHE_DIR`  | Cache directory     | `/var/cache/cave`       |
| `CAVE_LOG_LEVEL`  | Log level           | `info`                  |
| `CAVE_PLUGIN_DIR` | Plugin directory    | `/usr/lib/cave/plugins` |
| `RUST_LOG`        | Rust log filter     | (none)                  |

---

## 9. Troubleshooting

### 9.1 QEMU Won't Start

**Symptom:** `cave surge run` fails with "Permission denied" or "Could not open '/dev/kvm'"

**Solution:**

```bash
# Check KVM availability
ls -la /dev/kvm

# Add user to kvm group
sudo usermod -aG kvm $USER

# Or run without KVM (slower)
cave surge run --kernel vmlinuz --no-kvm
```

### 9.2 Initramfs Boot Fails

**Symptom:** Kernel panics with "Unable to mount root fs"

**Debug steps:**

```bash
# 1. Enable debug output
cave kindle build --config init.toml --output initramfs.cpio.gz --verbose

# 2. Add rd.debug to kernel args
cave surge run --kernel vmlinuz --initrd initramfs.cpio.gz \
  --append "rd.debug root=/dev/sda1"

# 3. Inspect initramfs contents
lsinitrd initramfs.cpio.gz

# 4. Check for missing libraries
chroot /path/to/rootfs ldd /sbin/init
```

### 9.3 Plugin Fails to Load

**Symptom:** `cave plugin load myplugin.so` fails

**Debug steps:**

```bash
# Check plugin dependencies
ldd /usr/lib/cave/plugins/myplugin.so

# Verify ABI compatibility
cave plugin info myplugin.so

# Check for missing symbols
nm -D /usr/lib/cave/plugins/myplugin.so | grep cave_plugin

# Enable debug logging
RUST_LOG=debug cave plugin load myplugin.so
```

### 9.4 Build Fails with "No Space Left"

**Symptom:** `cave forge build` fails with I/O error

**Solution:**

```bash
# Check build root space
df -h /var/lib/cave-forge/buildroot

# Clean old builds
cave forge clean --build-root /var/lib/cave-forge/buildroot

# Or use a different build root
cave forge build --pipeline pipeline.toml --output disk.img \
  --build-root /path/to/larger/drive
```

### 9.5 Slow Builds

**Solution:**

```bash
# Increase parallel jobs
cave forge build --pipeline pipeline.toml --jobs $(nproc)

# Use faster compression
cave kindle build --config init.toml --compressor zstd --compression-level 1

# Enable object store caching
cave forge build --pipeline pipeline.toml --cache-dir /ssd/cache
```

### 9.6 Getting Help

```bash
# General help
cave --help

# Subcommand help
cave surge --help
cave kindle --help
cave forge --help

# Plugin help
cave plugin --help

# Version info
cave --version
cave doctor  # Check system dependencies
```

---

_End of User Guide_
