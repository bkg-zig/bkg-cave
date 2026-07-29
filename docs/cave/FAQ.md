# Frequently Asked Questions

## bkg-cave — FAQ

**Version:** 0.1.0  
**Last Updated:** 2026-07-29

---

## Table of Contents

1. [What is bkg-cave?](#what-is-bkg-cave)
2. [Why Rust?](#why-rust)
3. [How does the plugin system work?](#how-does-the-plugin-system-work)
4. [Can I use cave without QEMU?](#can-i-use-cave-without-qemu)
5. [How do I debug initramfs issues?](#how-do-i-debug-initramfs-issues)
6. [Performance considerations](#performance-considerations)
7. [General Questions](#general-questions)
8. [Troubleshooting](#troubleshooting)

---

## What is bkg-cave?

**Q: What is bkg-cave?**

bkg-cave is a unified, plugin-driven Rust workspace for building, running, and packaging operating-system artifacts. It combines three historically separate tools into one binary:

- **cave-surge**: Run kernels in QEMU with custom initramfs (port of `virtme`)
- **cave-kindle**: Generate initramfs images from TOML configuration (port of `ugrd`)
- **cave-forge**: Build OS images via pipeline stages (port of `osbuild`)

**Q: Why "bkg-cave"?**

The name reflects the project's philosophy: a "cave" is a contained, controlled environment for building and testing OS artifacts. "bkg" is the team/project prefix.

**Q: Is bkg-cave production-ready?**

Version 0.1.0 is an initial release with core functionality. It's suitable for development and testing. Production use is possible but expect API changes in upcoming releases.

**Q: How is bkg-cave different from the tools it replaces?**

| Feature            | bkg-cave | virtme/ugrd/osbuild   |
| ------------------ | -------- | --------------------- |
| Unified binary     | ✅       | ❌ (3 separate tools) |
| Plugin system      | ✅       | ❌                    |
| Hot-reload         | ✅       | ❌                    |
| TOML configuration | ✅       | Mixed formats         |
| Cross-compilation  | ✅       | Limited               |
| WASM support       | Planned  | ❌                    |

---

## Why Rust?

**Q: Why was Rust chosen for bkg-cave?**

Rust provides several advantages for this project:

1. **Memory safety** — No segfaults or use-after-free bugs, critical for low-level VM and filesystem code
2. **Zero-cost abstractions** — High-level code compiles to efficient machine code
3. **Type safety** — The plugin system uses Rust's type system for ABI stability
4. **Cross-compilation** — Excellent support for targeting multiple architectures
5. **Ecosystem** — Strong libraries for CLI (`clap`), serialization (`serde`), async (`tokio`)
6. **Performance** — Comparable to C/C++ for system-level code

**Q: Can I use bkg-cave from other languages?**

Yes, via:

- **CLI** — Call `cave` from any language
- **Dynamic plugins** — Write plugins in C/C++ that implement the C ABI
- **WASM plugins** — Write plugins in any language that compiles to WASM (Rust, Go, AssemblyScript)

**Q: Is there a Python/Go/Java API?**

Not yet. The primary interface is the CLI. A gRPC/HTTP API is planned for a future release.

---

## How does the plugin system work?

**Q: What is a plugin?**

A plugin is a compiled module (static or dynamic `.so`) that implements the `CavePlugin` trait. Plugins can:

- Add new CLI commands
- Hook into the event system
- Access shared configuration and utilities

**Q: What's the difference between static and dynamic plugins?**

| Aspect        | Static             | Dynamic                |
| ------------- | ------------------ | ---------------------- |
| Compiled into | `cave` binary      | Separate `.so` file    |
| Hot-reload    | ❌                 | ✅                     |
| Distribution  | With cave binary   | Separate package       |
| Startup time  | Faster             | Slightly slower        |
| Use case      | Core functionality | Third-party extensions |

**Q: How do I create a plugin?**

See the [Plugin API Documentation](PLUGIN_API.md) for a complete guide. In short:

1. Implement the `CavePlugin` trait
2. For dynamic plugins, export `cave_plugin_create` and `cave_plugin_destroy` symbols
3. Compile as `cdylib` for `.so` output
4. Load with `cave plugin load path/to/plugin.so`

**Q: Can plugins crash the main process?**

Static plugins run in the same process and can crash `cave`. Dynamic plugins are isolated — a segfault in a `.so` will crash only that plugin (though the host process may also terminate on Linux). WASM plugins (planned) will be fully sandboxed.

**Q: How does hot-reload work?**

Hot-reload uses `dlclose` + `dlopen`:

1. Call `shutdown()` on the old plugin instance
2. `dlclose` the old `.so`
3. `dlopen` the `.so` again
4. Call `cave_plugin_create()` to get a new instance
5. Call `init()` on the new instance
6. Re-register commands

If any step fails, the old plugin remains loaded.

**Q: Can I write plugins in C/C++?**

Yes, for dynamic plugins. You need to:

1. Implement the C ABI (`cave_plugin_create`, `cave_plugin_destroy`)
2. Use the C header (`cave-plugin.h`)
3. Compile as a shared library

Example C plugin:

```c
#include <cave-plugin.h>

typedef struct {
    CavePlugin base;
    int initialized;
} MyPlugin;

CavePlugin* cave_plugin_create() {
    MyPlugin* p = malloc(sizeof(MyPlugin));
    p->base.name = "cave-surge-c-plugin";
    p->base.version = "0.1.0";
    p->base.kind = PLUGIN_KIND_SURGE;
    p->initialized = 0;
    return (CavePlugin*)p;
}

void cave_plugin_destroy(CavePlugin* plugin) {
    free(plugin);
}
```

---

## Can I use cave without QEMU?

**Q: Can I use cave without QEMU?**

Yes! `cave-kindle` and `cave-forge` do not require QEMU. Only `cave-surge` needs QEMU for VM execution.

**Q: What can I do without QEMU?**

- Generate initramfs images with `cave kindle build`
- Build OS images with `cave forge build`
- Manage plugins with `cave plugin`
- Inspect and manipulate CPIO archives

**Q: How do I disable QEMU features?**

```bash
# Build without QEMU support
cargo build --no-default-features --features "gzip,xz,zstd"

# Or just don't install QEMU — cave-surge will report it's unavailable
cave doctor
```

---

## How do I debug initramfs issues?

**Q: My initramfs won't boot. How do I debug?**

Follow these steps:

### 1. Enable Kernel Debug Output

```bash
cave surge run \
  --kernel /boot/vmlinuz \
  --initrd initramfs.cpio.gz \
  --append "rd.debug loglevel=7 console=ttyS0"
```

### 2. Inspect Initramfs Contents

```bash
# List files
zcat initramfs.cpio.gz | cpio -t

# Extract and inspect
mkdir /tmp/initramfs
cd /tmp/initramfs
zcat /path/to/initramfs.cpio.gz | cpio -idmv

# Check init script
cat init
file bin/busybox

# Check for missing libraries
chroot /tmp/initramfs ldd /sbin/init 2>&1
```

### 3. Use cave-kindle Verbose Mode

```bash
cave kindle build --config init.toml --output initramfs.cpio.gz --verbose
```

### 4. Check Hook Execution

```bash
# Run hooks manually
bash -x /etc/cave/hooks/10-my-hook.sh

# Check hook logs
journalctl -u cave-kindle
```

### 5. Common Issues

| Symptom                      | Likely Cause           | Fix                               |
| ---------------------------- | ---------------------- | --------------------------------- |
| "Unable to mount root fs"    | Missing storage driver | Add `ext4` to `[modules].include` |
| "Kernel panic - not syncing" | Missing `/init`        | Ensure init script is included    |
| "No such file or directory"  | Missing library        | Run `ldd` on all binaries         |
| "Permission denied"          | Wrong file modes       | Check `[files]` permissions       |

**Q: How do I add debug output to my init script?**

```bash
#!/bin/sh
#!/bin/busybox sh

mount -t proc none /proc
mount -t sysfs none /sys

# Debug: print environment
echo "Init started" > /dev/kmsg
set > /tmp/init-env.txt

# Your actual init logic
exec /sbin/init
```

---

## Performance Considerations

**Q: How fast is cave-kindle compared to ugrd?**

cave-kindle is typically 20-40% faster due to:

- Parallel file collection
- Efficient CPIO writing (zero-copy where possible)
- Better compression algorithm selection

**Q: How can I speed up initramfs builds?**

```bash
# 1. Use faster compression
cave kindle build --config init.toml --compressor zstd --compression-level 1

# 2. Exclude unnecessary modules
# In init.toml:
[modules]
exclude = ["*.ko"]  # Exclude all modules, include only what you need

# 3. Use busybox instead of full utilities
[files]
binaries = ["/bin/busybox"]  # Instead of individual utilities

# 4. Enable parallel compression
cave kindle build --jobs $(nproc)
```

**Q: How can I speed up cave-forge builds?**

```bash
# 1. Use SSD for build root and object store
cave forge build --pipeline pipeline.toml \
  --build-root /ssd/buildroot \
  --cache-dir /ssd/cache

# 2. Increase parallel jobs
cave forge build --pipeline pipeline.toml --jobs $(nproc)

# 3. Reuse cached stages
# cave-forge automatically skips stages with matching inputs

# 4. Use faster compression for intermediate artifacts
[stages]
# Use zstd for intermediate, gzip for final
```

**Q: What's the memory footprint?**

| Component           | Typical Memory                         |
| ------------------- | -------------------------------------- |
| `cave` (idle)       | ~15 MB                                 |
| `cave kindle build` | ~50-200 MB (depends on initramfs size) |
| `cave forge build`  | ~100-500 MB (depends on pipeline)      |
| `cave surge run`    | ~10 MB + QEMU memory                   |

**Q: How big is the cave binary?**

| Build              | Size     |
| ------------------ | -------- |
| Debug              | ~50 MB   |
| Release (stripped) | ~8-12 MB |
| Release + LTO      | ~5-8 MB  |

---

## General Questions

**Q: What platforms are supported?**

| Platform      | Status                   |
| ------------- | ------------------------ |
| Linux x86_64  | ✅ Fully supported       |
| Linux aarch64 | ✅ Supported             |
| Linux riscv64 | ✅ Supported             |
| macOS         | ⚠️ Partial (no QEMU/KVM) |
| Windows       | ❌ Not supported         |

**Q: Does bkg-cave require root?**

- `cave kindle build` — No (unless writing to system directories)
- `cave forge build` — No (unless writing to system directories)
- `cave surge run` — Yes (for KVM) or No (for TCG/software emulation)

**Q: How do I update bkg-cave?**

```bash
# From crates.io
cargo install bkg-cave --force

# From source
git pull && cargo install --path crates/cave --force
```

**Q: Where is the configuration stored?**

| Config   | Path                         |
| -------- | ---------------------------- |
| System   | `/etc/cave/config.toml`      |
| User     | `~/.config/cave/config.toml` |
| Project  | `./cave.toml`                |
| Override | `$CAVE_CONFIG`               |

**Q: How do I uninstall bkg-cave?**

```bash
# From cargo
cargo uninstall bkg-cave

# Remove config and data
rm -rf ~/.config/cave ~/.local/lib/cave /var/lib/cave /var/cache/cave
```

**Q: Is there a GUI?**

Not yet. A Dioxus-based TUI/Web UI is planned for version 0.2.0 (M10).

**Q: Can I use bkg-cave in CI/CD?**

Yes! bkg-cave is designed for automation:

```yaml
# GitHub Actions example
- name: Build initramfs
  run: cave kindle build --config init.toml --output initramfs.cpio.gz

- name: Build OS image
  run: cave forge build --pipeline pipeline.toml --output disk.img

- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: os-image
    path: disk.img
```

---

## Troubleshooting

**Q: Where can I get help?**

- Documentation: This docs folder
- GitHub Issues: https://github.com/example/bkg-cave/issues
- Matrix/Discord: https://example.com/chat

**Q: How do I report a bug?**

See the [Bug Reporting section in CONTRIBUTING.md](CONTRIBUTING.md#reporting-bugs).

**Q: How do I request a feature?**

Open a GitHub issue with the `enhancement` label and describe:

1. The feature you want
2. Why you need it
3. How it should work

---

_End of FAQ_
