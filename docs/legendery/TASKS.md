# Task Registry — bkg-cave (Cave-Theme, Plugin-Architektur)

> Jeder Task = ein Commit = ein abgeschlossener Schritt.
> Tasks innerhalb eines Milestones in Reihenfolge.
> Nach jedem Task: `cargo build`, `cargo test`, `cargo clippy`.

---

## M1 — cave-core (a.0.1)

### T1.1 Workspace + cave-core Crate

- **Files**: `Cargo.toml`, `crates/cave-core/Cargo.toml`, `crates/cave-core/src/lib.rs`
- **Deps**: thiserror, tracing, serde
- **Verify**: `cargo build --workspace`

### T1.2 error.rs

- **File**: `crates/cave-core/src/error.rs`
- **Enum**:
  - `Io(std::io::Error)`
  - `Cpio(String)` — CPIO-Format
  - `BinaryNotFound(String)` — Externes Tool nicht gefunden
  - `Config(String)` — Config-Fehler
  - `Validation(String)` — Validierungsfehler
  - `Plugin(String)` — Plugin-Ladefehler
  - `Silent` — Stiller Exit
  - `Cli(String)` — CLI-Fehler
  - `Event(String)` — EventBus-Fehler
- **Verify**: `cargo test`

### T1.3 util.rs — Basis-Util

- **File**: `crates/cave-core/src/util.rs`
- **Funcs**:
  - `find_binary(names, root, use_path) -> Option<PathBuf>` — 6 Pfade: [usr/local, usr, ""] × [bin, sbin]
  - `find_binary_or_raise(...) -> Result<PathBuf>`
  - `is_file_more_recent(a, b) -> Result<bool>`
  - `quote_karg(arg) -> Result<String>` — `"` check, mit spaces → `"arg"`
  - `sanitize_disk_args(func, arg) -> Result<(String, String)>`
- **Tests**: Siehe T1.9

### T1.4 cpio.rs — CpioWriter

- **File**: `crates/cave-core/src/cpio.rs`
- **Struct**: `CpioWriter<W: Write>`
- **Const**: `TYPE_DIR=0o040000, TYPE_REG=0o100000, TYPE_SYMLINK=0o120000, TYPE_CHRDEV=0o020000, TYPE_MASK=0o170000`
- **Funcs**:
  - `new(w)` — init mit auto-ino
  - `write_object(name, body, mode, ino, nlink, uid, gid, mtime, devmajor, devminor, rdevmajor, rdevminor)`
  - `write_trailer()` — TRAILER!!! + 512-Pad
  - `mkdir(name, mode)` — TYPE_DIR
  - `symlink(src, dst)` — TYPE_SYMLINK
  - `write_file(name, body, mode)` — TYPE_REG
  - `mkchardev(name, (major,minor), mode)` — TYPE_CHRDEV
  - `len() -> u64` — total bytes
- **Binary-identisch zu Python**:
  - Header: `070701` + 13 × `%08X`
  - Namen-Padding: `(2 - namesize) % 4`
  - Body-Padding: `(-filesize).rem_euclid(4)`
  - Trailer-Padding: `(-totalsize).rem_euclid(512)`
  - ino: auto-increment ab 0
  - nlink: TYPE_DIR = 2, sonst = 1
- **Tests**: T1.10

### T1.5 shell.rs — Shell-Code-Generator-Basis

- **File**: `crates/cave-core/src/shell.rs`
- **Funcs**: `shell_escape(s)`, `indent(lines, level)`, `comment(text)`, `shebang()`
- **Verify**: `cargo build`

### T1.6 event.rs — EventBus

- **File**: `crates/cave-core/src/event.rs`
- **Struct**: `EventBus` — Plugin → Plugin Kommunikation
- **Funcs**: `publish(event)`, `subscribe(handler)`, `unsubscribe(id)`
- **Verify**: `cargo test`

### T1.7 config.rs — ConfigRegistry

- **File**: `crates/cave-core/src/config.rs`
- **Struct**: `ConfigRegistry` — zentrale, versionierte Config
- **Funcs**: `set(key, value)`, `get(key) -> Option<Value>`, `load_file(path)`, `watch(path)`
- **Verify**: `cargo test`

### T1.8 plugin.rs — Plugin-System

- **File**: `crates/cave-core/src/plugin.rs`
- **Trait**:
  ```rust
  pub trait CavePlugin: Send + Sync {
      fn id(&self) -> &'static str;
      fn name(&self) -> &str;
      fn version(&self) -> &str;
      fn kind(&self) -> PluginKind;
      fn init(&mut self, ctx: &PluginContext) -> Result<()>;
      fn commands(&self) -> Vec<RegisteredCommand>;
      fn hot_reloadable(&self) -> bool { true }
  }
  ```
- **Enums**: `PluginKind { Surge, Kindle, Forge, Utility }`
- **Structs**: `PluginContext { config, events, plugin_dir }`, `RegisteredCommand { name, about, run }`
- **Manager**:
  ```rust
  pub struct PluginManager {
      plugins: HashMap<&'static str, Box<dyn CavePlugin>>,
      event_bus: EventBus,
      config: ConfigRegistry,
  }
  ```
  - `new() -> Self`
  - `load_static(&mut self, plugin: Box<dyn CavePlugin>)` — fest einkompiliert
  - `load_dynamic(&mut self, path: &Path) -> Result<()>` — via libloading
  - `unload(&mut self, id: &str) -> Result<()>` — Plugin entladen
  - `reload(&mut self, id: &str) -> Result<()>` — Hot-Reload
  - `find_command(&self, name: &str) -> Option<&RegisteredCommand>`
  - `execute(&self, name: &str, args: &[String]) -> Result<i32>`
- **Verify**: Tests für load/unload/find/execute

### T1.9 lib.rs

- **File**: `crates/cave-core/src/lib.rs`
- **Re-exports**: `pub use error::*; pub use plugin::*; pub use event::*; pub use config::*; pub use cpio::*; pub use util::*; pub use shell::*;`
- **Verify**: `cargo doc --no-deps`

### T1.10 Python-Reference CPIO

- **Dir**: `tests/test_data/`
- **Action**: Python-Skript das CPIO-Archive erzeugt (Python-cpiowriter-Logik)
- **Output**: `cpio_ref_single.bin`, `cpio_ref_dir.bin`, `cpio_ref_symlink.bin`, `cpio_ref_chardev.bin`, `cpio_ref_multi.bin`

### T1.11 Tests

- **File**: `crates/cave-core/tests/test_cpio.rs`
- **Golden Tests** (binary-diff gegen Python-Reference):
  - `cpio_single_file`
  - `cpio_directory`
  - `cpio_symlink`
  - `cpio_chardev`
  - `cpio_multi_objects`
- **Edge Cases**:
  - `cpio_empty_only_trailer`
  - `cpio_ino_auto_increment`
  - `cpio_nlink_dir_is_2`
  - `cpio_padding_1_byte_name`
  - `cpio_padding_3_byte_body`

- **File**: `crates/cave-core/tests/test_util.rs`
  - `find_binary_path_first`, `find_binary_root_fallback`, `find_binary_all_paths`
  - `find_binary_not_found`, `find_binary_or_raise_ok/err`
  - `is_file_more_recent_true/false`
  - `quote_karg_with_spaces`, `quote_karg_no_spaces`, `quote_karg_has_double_quote_err`
  - `sanitize_disk_args_valid`, `sanitize_disk_args_invalid`

- **File**: `crates/cave-core/tests/test_plugin.rs`
  - `plugin_load_static`
  - `plugin_find_command`
  - `plugin_execute_command`
  - `plugin_unload`

- **File**: `crates/cave-core/tests/test_event.rs`
  - `event_publish_subscribe`
  - `event_multiple_subscribers`

- **Verify**: `cargo test --workspace`

### M1 Release Gate

- `cargo fmt --check`
- `cargo clippy --all -- -D warnings`
- `cargo test --all`
- `cargo doc --no-deps`

---

## M2 — cave-surge (b.0.2)

### T2.1 Crate-Struktur

- **Files**: `crates/cave-surge/Cargo.toml`, `crates/cave-surge/src/lib.rs`
- **Deps**: cave-core, clap 4
- **Plugin-Export**:
  ```rust
  pub fn plugin() -> impl CavePlugin { SurgePlugin::new() }
  ```

### T2.2 arch.rs — Arch-Trait + 8 Impl

- **Trait**:
  ```rust
  pub trait Arch: Send + Sync {
      fn virtmename(&self) -> &str;
      fn qemuname(&self) -> &str;
      fn linuxname(&self) -> &str;
      fn gccname(&self) -> &str;
      fn defconfig_target(&self) -> &str;
      fn serial_dev_name(&self, idx: u32) -> String;
      fn qemuargs(&self, is_native: bool) -> Vec<String>;
      fn virtio_dev_type(&self, kind: &str) -> String;
      fn earlyconsole_args(&self) -> Vec<String>;
      fn serial_console_args(&self) -> Vec<String>;
      fn qemu_nodisplay_args(&self) -> Vec<String>;
      fn config_base(&self) -> Vec<String>;
      fn kimg_path(&self) -> String;
      fn dtb_path(&self) -> Option<String>;
  }
  ```
- **8 Impls**: `ArchX86` (x86_64 + i386), `ArchArm`, `ArchAarch64`, `ArchPpc64`, `ArchRiscv64`, `ArchSparc64`, `ArchS390x`, `ArchUnknown`
- **Factory**: `fn arch_from_name(name: &str) -> Box<dyn Arch>`

### T2.3 qemu.rs — QEMU-Discovery

- **Struct**:
  ```rust
  pub struct Qemu {
      pub qemubin: PathBuf,
      pub version: Option<String>,
      pub cannot_overmount_virtfs: bool,
      pub has_multidevs: bool,
  }
  ```
- **Funcs**: `Qemu::new(arch)`, `probe(&mut self)`, `quote_optarg(&self, a)`, `export_virtfs(&self, ...)`
- **Version-Parsing**: Regex für `version X.Y.Z`, determination von cannot_overmount (< 1.6) und has_multidevs (>= 4.2)

### T2.4 modules.rs — Modulfinder

- **Funcs**: `resolve_dep(modalias, root, kver, moddir)`, `merge_mods(lists)`, `find_from_install(aliases, ...)`
- **Mechanismus**: `modprobe --show-depends -C /var/empty [-d root] [-S kver] [--moddir dir] -- {alias}`

### T2.5 modaliases.rs

- **Const**: `pub const MODALIASES: &[&str] = &["fs-9p", "virtio:...", "pci:...", "unix", "i8042", "atkbd"]`

### T2.6 initramfs.rs — Initramfs-Builder

- **Struct Config**: `modfiles`, `virtme_data`, `busybox`, `access { Ro, Rw }`
- **Funcs**:
  - `find_busybox(root, is_native)` — sucht busybox-static, busybox.static, busybox
  - `make_base_layout(cw)` — 10 dirs + symlinks
  - `make_dev_nodes(cw)` — /dev/null, kmsg, console
  - `install_busybox(cw, config)` — binary + symlinks
  - `install_modprobe(cw)` — stub
  - `install_modules(cw, modfiles)` — module + load_all.sh
  - `generate_init(config) -> Vec<u8>` — Shell-Init aus _INIT-Template
  - `mkinitramfs(out, config)` — gesamte Pipeline

### T2.7 cli.rs — CLI-Args

- **clap-Builder**: 35 Argumente, 8 Groups:
  - `--installed-kernel`, `--kimg`, `--kdir` (mutex)
  - `--mods none|use|auto`, `--kopt`, `--xen`
  - `--root`, `--rw`, `--graphics`, `--net`, `--balloon`
  - `--disk`, `--blk-disk`, `--memory`, `--cpus`, `--name`
  - `--script-sh`, `--script-exec`
  - `--arch`, `--busybox`
  - `--qemu-opt`, `--qemu-opts`
  - `--force-initramfs`, `--dry-run`, `--show-command`
  - `--save-initramfs`, `--show-boot-console`
  - `--pwd`, `--cwd`
  - `--rwdir`, `--rodir`

### T2.8 lib.rs — Orchestrierung

- **Struct**: `SurgePlugin` implementiert `CavePlugin`
- **Kernel-Struct**: `kimg`, `dtb`, `modfiles`, `moddir`, `use_root_mods`, `config`
- **Funcs**:
  - `find_kernel(args) -> Result<Kernel>` — 3 Modi
  - `execute(args) -> Result<i32>` — gesamte do_it-Logik
- **execv**: `std::os::unix::process::CommandExt::exec()`

### T2.9 Gast-Tools embedded

- **Files**: `crates/cave-surge/src/guest/init.sh`, `udhcpc.sh`
- **Embed**: `include_str!("guest/init.sh")` in `lib.rs`
- **Shell-Code**: 1:1 aus Reference (virtme-init, 251 Zeilen)

### T2.10 Tests

- **Golden Tests**: QEMU-Argument-Liste == Python-Output
- **Unit**: Arch-Eigenschaften, QEMU-Version-Parsing, Initramfs-Aufbau
- **Integration**: `cave surge --dry-run --kimg /path --show-command`
- **Verify**: `cargo test --workspace`

---

## M3 — cave-kindle (c.0.3)

### T3.1 Crate-Struktur

- **Files**: `crates/cave-kindle/Cargo.toml`, `crates/cave-kindle/src/lib.rs`
- **Deps**: cave-core, toml, clap
- **Plugin-Export**: `pub fn plugin() -> impl CavePlugin`

### T3.2 error.rs

- **Enum**: `ValidationError(String)`, `AutodetectError(String)`, `ConfigError(String)`, `HookError(String)`, `ModuleError(String)`

### T3.3 config.rs — InitramfsConfig

- **Struct**: `InitramfsConfig` mit HashMap-Basis
- **SetItem-Logik**: builtin vs custom vs deferred (Queue)
- **Processing**: `_processing: HashMap<String, VecDeque<Value>>`
- **Late Args**: `["binaries", "binary_dir", "kernel_version", ...]`
- **Stage**: `early → late → final`, `validated`-Flag
- **Modul-Ladung**: `_load_module(toml)` — imports, hooks, provides, needs

### T3.4 modules.rs — Modul-Discovery

- **Funcs**: `read_module(path) -> Value`, `get_module_paths(base) -> Vec<PathBuf>`, `get_parameters(config) -> Vec<String>`, `resolve_type(name) -> Type`
- **Types**: `Path`, `NoDupFlatList`, `int`, `bool`, `string`, `string_list`

### T3.5 protocol.rs + hooks.rs

- **Protocol**:
  ```rust
  pub trait InitramfsProtocol {
      fn config_dict(&self) -> &InitramfsConfig;
      fn included_functions(&self) -> &HashMap<String, Vec<String>>;
      fn run_hook(&self, name: &str, force_include: bool, force_exclude: &[&str]) -> Result<Vec<String>>;
      fn _mkdir(&self, path: &Path) -> Result<()>;
      fn _write(&self, name: &str, content: &str, mode: u32) -> Result<()>;
      fn _copy(&self, src: &Path, dst: &Path) -> Result<()>;
      fn _symlink(&self, target: &str, link: &Path) -> Result<()>;
      fn _run(&self, cmd: &str) -> Result<String>;
  }
  ```
- **HookRegistry**: `hooks: HashMap<String, Vec<HookFn>>`, `masks`, `import_order`
- **HookFn**: `{ name, func: fn(&dyn InitramfsProtocol) -> Result<Option<Vec<String>>>, module }`
- **Sortierung**: `before/after` Konvergenz (max 100 Iterationen)
- **Masking**: Funktionen pro Hook deaktivierbar

### T3.6 helpers.rs — GeneratorHelpers

- **Funcs**: `_mkdir`, `_write`, `_copy`, `_symlink`, `_rotate_old`, `_run`
- **Path**: `_get_build_path(name)`, `_get_out_path(name)`

### T3.7 base/base.rs — Shell-Basis

- **Funcs** (Shell-Code-Generatoren):
  - `set_loglevel()`, `einfo(msg)`, `ewarn(msg)`, `eerror(msg)`
  - `setvar(name, value)`, `readvar(name)`
  - `find_root(possible)`, `mount_base()`
  - `do_switch_root()`, `run_switch_root(target, init)`

### T3.8 base/core.rs — Build-Kern

- **Funcs**:
  - `clean_build_dir()`, `set_shebang()`
  - `deploy_binaries()`, `deploy_dependencies()`, `deploy_symlinks()`, `deploy_nodes()`
  - `generate_structure()`, `regen_ld_so_cache()`, `generate_fstab()`

### T3.9 base/cmdline.rs + base/debug.rs

- **Cmdline**: `parse_cmdline_bool()`, `parse_cmdline_str()`, `export_cmdline()`, `set_loglevel_from_cmd()`
- **Debug**: `start_debug_shell()`, `edit_config_file()`

### T3.10 TOML-Modul-Definitionen

- **Dir**: `crates/cave-kindle/data/`
- **Files**: `base.toml`, `core.toml`, `cmdline.toml`, `debug.toml`, `kmod.toml`, etc.
- **Content**: `imports`, `custom_parameters`, `provides`, `needs`, `masks`, `import_order`

### T3.11 cli.rs + lib.rs — CLI + Orchestrierung

- **CLI**: `--config PATH`, `--no-base`, `--validate-only`, `--output PATH`, `--compress xz|zstd|gzip|none`, `--debug`, `--dry-run`
- **KindlePlugin**: `build(config_path) -> Result<PathBuf>`:
  1. Config laden (TOML)
  2. InitramfsConfig erstellen
  3. Generator instanziieren
  4. `run_build()` → 6 Build-Hooks
  5. `generate_init()` → init-Skript + profile
  6. `pack_build()` → CPIO (xz default)
  7. `run_checks()` + `run_tests()`

### T3.12 Tests

- **Unit**: TOML-Parsing, Hook-Sortierung, Config-Queue, Shell-Generatoren
- **Integration**: `cave kindle --config test.toml --dry-run` → CPIO-Output
- **Golden**: Generated init-Skript == Python-Version
- **Verify**: `cargo test --workspace`

---

## M4 — cave-forge (d.0.4)

### T4.1 Crate-Struktur

- **Files**: `crates/cave-forge/Cargo.toml`, `crates/cave-forge/src/lib.rs`
- **Deps**: cave-core, serde_json, sha2, clap
- **Plugin-Export**: `pub fn plugin() -> impl CavePlugin`

### T4.2 pipeline.rs — Core

- **Structs**:
  - `StageResult { name, id, success, output, error }`
  - `Stage { info, sources, build, base, options, inputs, devices, mounts, source_epoch }`
  - `Manifest { pipelines: Vec<Pipeline>, sources: Vec<Source> }`
  - `Pipeline { name, runner, build, stages, assembler, source_epoch }`
  - `Runner { info, path, name }`
- **Funcs**:
  - `Stage::id()` — SHA256(name + build + base + options + inputs + mounts)
  - `Manifest::depsolve()` — topologische Sortierung
  - `Manifest::build()` — Pipeline-Ausführung
  - `Pipeline::build_stages()` — Stage-Sequenz

### T4.3 buildroot.rs — Sandbox

- **Structs**: `BuildRoot`, `CompletedBuild { returncode, output }`
- **Funcs**:
  - `BuildRoot::new()` — tmpfs, device nodes
  - `run(args, binds, readonly_binds, caps, env, timeout)` — bwrap ausführen
  - Argumente: `--chdir / --die-with-parent --new-session --unshare-ipc --unshare-pid --unshare-net --ro-bind... --tmpfs... --cap-drop ALL +caps -- runner args`

### T4.4 objectstore.rs — Content-Addressed Store

- **Structs**: `Object`, `ObjectStore`, `HostTree`
- **Funcs**:
  - `ObjectStore::new(base_dir)` — FsCache initialisieren
  - `store.new(id) -> Object` — Write-Mode
  - `store.get(id) -> Object` — Read-Mode
  - `store.contains(id) -> bool` — Cache-Hit
  - `store.commit(obj, id)` — clamp_mtime + cache
  - `Object::init(base_obj)` — cp --reflink=auto -a
  - `Object::finalize()` — Datei-Timestamps setzen
- **FsCache**: LRU, flock-Locking, atomare Ops

### T4.5 meta.rs — Index

- **Structs**: `Index`, `ModuleInfo`, `Schema`, `RunnerInfo`, `FormatInfo`
- **Funcs**:
  - `Index::new(libdir)` — Module laden
  - `Index::get_module_info(type, name) -> ModuleInfo`
  - `Index::validate(manifest) -> Result`
  - `Index::detect_runner(os_release) -> RunnerInfo`

### T4.6 cli.rs + lib.rs

- **CLI**: `--manifest PATH`, `--store PATH`, `--output-dir PATH`, `--export PIPELINE`, `--checkpoint`, `--in-vm`, `--libdir PATH`, `--cache-max SIZE`, `--monitor [log|null|json]`, `--break`
- **ForgePlugin**:
  1. Manifest parsen (JSON)
  2. Index erstellen
  3. Validieren
  4. ObjectStore öffnen
  5. Depsolve
  6. Sources downloaden
  7. Pipelines bauen
  8. Exportieren

### T4.7 Tests

- **Unit**: Pipeline-ID-Berechnung, Manifest-Parsing, Index-Loading
- **Integration**: `cave forge --manifest test.json --dry-run`
- **Verify**: `cargo test --workspace`

---

## M5 — cave-surge: Feature-Complete (e.0.5)

### T5.1 Rest-Args implementieren

- Alle 35 CLI-Argumente vollständig
- Edge-Cases: Xen, --mods auto, DTB-Pfade, Scripting

### T5.2 Gast-Tools vollständig

- `virtme-init` → `guest/init.sh` — Shell-Code identisch (251 Zeilen)
- `virtme-udhcpc-script` → `guest/udhcpc.sh` — Shell-Code identisch (24 Zeilen)

### T5.3 Integrationstests

- `test_surge_dry_run` — vollständiger Durchlauf
- `test_surge_qemu_args` — Argumente korrekt
- `test_surge_initramfs` — CPIO-Ausgabe korrekt
- `test_surge_guest_tools` — Shell-Skripte korrekt eingebettet

### T5.4 Verify

- `cargo test --workspace`
- `cave surge --help` alle Argumente sichtbar

---

## M6 — cave-kindle: Feature-Complete (f.0.6)

### T6.1 FS-Module

- `crates/cave-kindle/src/fs/mounts.rs` — mount-Info + fstab + root-mount Shell-Code
- `crates/cave-kindle/src/fs/ext4.rs` — ext4 Shell-Code
- `crates/cave-kindle/src/fs/btrfs.rs` — btrfs Shell-Code

### T6.2 KMod-Modul

- `crates/cave-kindle/src/kmod/kmod.rs` — module processing + firmware + aliases + load-all-init

### T6.3 Net-Modul

- `crates/cave-kindle/src/net/net.rs` — net-autodetect + mac

### T6.4 Crypto-Modul

- `crates/cave-kindle/src/crypto/cryptsetup.rs` — LUKS, key handling, shell-init

### T6.5 Integrationstests

- `test_kindle_full_build` — TOML → CPIO, vollständiger Durchlauf
- `test_kindle_init_shellcheck` — generiertes Init-Skript ist syntaktisch korrekt
- `test_kindle_cpio_valid` — Ausgabe ist gültiges CPIO

### T6.6 Verify

- `cargo test --workspace`

---

## M7 — cave-forge: Feature-Complete (g.0.7)

### T7.1 sources.rs

- **Source-Handler**: `Source::download(store)`, `SourceService`

### T7.2 devices.rs

- **Device-Manager**: `Device::open()` via Host-Service, device nodes

### T7.3 inputs.rs

- **Input-Manager**: Pipeline-Inputs auflösen, Source-Refs

### T7.4 mounts.rs

- **Mount-Manager**: Mounts via Host-Service

### T7.5 host.rs

- **Service-Framework**: `Service`, `ServiceClient`, `ServiceManager` (Unix-Socket-IPC)

### T7.6 api.rs + monitor.rs

- **API**: `BaseAPI`, Exception-Handling
- **Monitor**: `NullMonitor`, `LogMonitor`, `JSONSeqMonitor`

### T7.7 Integrationstests

- `test_forge_manifest_parse` — JSON korrekt parsen
- `test_forge_pipeline_id` — SHA256 korrekt
- `test_forge_depsolve` — Pipeline-Reihenfolge korrekt

### T7.8 Verify

- `cargo test --workspace`

---

## M8 — cave: Hauptbinary + Hot-Reload (h.0.8)

### T8.1 cave/Cargo.toml

- **Deps**: cave-core, cave-surge, cave-kindle, cave-forge, clap 4, libloading, notify

### T8.2 main.rs

```rust
fn main() -> Result<i32> {
    let mut pm = PluginManager::new();
    pm.load_static(cave_surge::plugin());
    pm.load_static(cave_kindle::plugin());
    pm.load_static(cave_forge::plugin());
    // Optional: dynamische Plugins aus /usr/lib/cave/plugins/
    if let Ok(dir) = std::fs::read_dir("/usr/lib/cave/plugins/") {
        for entry in dir { pm.load_dynamic(entry?.path())?; }
    }
    // Hot-Reload Watcher
    pm.start_watcher("/usr/lib/cave/plugins/")?;

    // CLI-Routing
    let cli = Cli::parse();
    match cli {
        Cli::Surge(args) => pm.execute("surge", &args),
        Cli::Kindle(args) => pm.execute("kindle", &args),
        Cli::Forge(args) => pm.execute("forge", &args),
    }
}
```

### T8.3 Hot-Reload-Implementierung

- `PluginManager::load_dynamic(path)`:
  1. `unsafe { libloading::Library::new(path) }`
  2. `lib.get(b"cave_plugin_create")` → `extern fn() -> *mut dyn CavePlugin`
  3. Plugin in `Box::from_raw` → in `plugins`-Map
- `PluginManager::unload(id)`:
  1. Plugin aus Map entfernen
  2. Drop → `dlclose` (via Library-Drop)
- `PluginManager::reload(id)`:
  1. Pfad merken
  2. `unload(id)`
  3. `load_dynamic(path)`
  4. Config aus vorheriger Session übernehmen
- **File-Watcher**: `notify::Watcher` auf plugin-dir → bei modify → reload

### T8.4 Tests

- `test_cave_surge_subcommand` — `cave surge --help`
- `test_cave_kindle_subcommand` — `cave kindle --help`
- `test_cave_forge_subcommand` — `cave forge --help`
- `test_plugin_hot_reload` — dynamisches Laden + Entladen (falls .so verfügbar)

### T8.5 Verify

- `cargo build --workspace`
- `cave surge --dry-run --kimg /path/to/vmlinuz`
- `cave kindle --config test.toml --dry-run`
- `cave forge --manifest test.json --dry-run`

---

## M9 — QA (i.0.9)

### T9.1 cargo clippy

- `cargo clippy --all -- -D warnings`
- Alle Warnungen fixen

### T9.2 cargo fmt

- `cargo fmt --all --check`

### T9.3 cargo test

- `cargo test --all`
- Alle Tests grün

### T9.4 cargo doc

- `cargo doc --workspace --no-deps`
- Keine broken links, keine Warnings

---

## M10 — Dioxus/WASM (j.0.10, später)

### T10.1 cave-web Crate

- **Deps**: dioxus 0.5, cave-core
- **App**: Dioxus-Desktop + Web
- **Bridge**: PluginEngine via WASM-kompatiblen Wrapper

### T10.2 Web-Frontend

- Subcommand-Auswahl: Surge, Kindle, Forge
- Config-Editor (TOML/JSON)
- Output-Viewer
- Build-Status (via Monitor)

---

## Zusammenfassung

| MS  | Branch | Crate       | Tasks       | Status      |
| --- | ------ | ----------- | ----------- | ----------- |
| M1  | a.0.1  | cave-core   | T1.1–T1.11  | **DONE**    |
| M2  | b.0.2  | cave-surge  | T2.1–T2.10  | **PENDING** |
| M3  | c.0.3  | cave-kindle | T3.1–T3.12  | **PENDING** |
| M4  | d.0.4  | cave-forge  | T4.1–T4.7   | **PENDING** |
| M5  | e.0.5  | cave-surge  | T5.1–T5.4   | **PENDING** |
| M6  | f.0.6  | cave-kindle | T6.1–T6.6   | **PENDING** |
| M7  | g.0.7  | cave-forge  | T7.1–T7.8   | **PENDING** |
| M8  | h.0.8  | cave        | T8.1–T8.5   | **PENDING** |
| M9  | i.0.9  | —           | T9.1–T9.4   | **PENDING** |
| M10 | j.0.10 | cave-web    | T10.1–T10.2 | **SPÄTER**  |
