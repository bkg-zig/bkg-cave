# api.md — ugrd Öffentliche Rust-APIs (bkg-cave-boot)

> Diese Datei beschreibt die öffentlichen APIs der geplanten Rust-Portierung.
> Namensraum: `bkg_cave_boot`. Migration aus: `ugrd` (Python, GPL-2.0).

---







## 1. CLI-API

### `bkg_cave_boot_cli::main() -> Result<i32, BkgCaveBootError>`

Hauptentrypoint des Binarys `bkg-cave-boot`.

- **Parameter**: liest `std::env::args()`
- **Rückgabe**: Exit-Code (0 = Erfolg)
- **Fehlerfälle**: `ConfigError`, `ValidationError`, `BuildFailed`, `AutodetectError`

### `bkg_cave_boot_cli::print_parameters()` → `Result<(), Error>`

Gibt alle verfügbaren Config-Parameter aus (für `--parameters`).

- **Rückgabe**: nichts
- **Seiteneffekte**: schreibt auf stdout

---







## 2. Kern-API (Initramfs-Bau)

### `bkg_cave_boot_core::InitramfsGenerator`

Haupt-Orchestrator — entspricht `ugrd.initramfs_generator.InitramfsGenerator`.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `new` | `(config: Option<Path>, args: HashMap<String, Value>) -> Result<Self, ConfigError>` | Generator initialisieren |
| `build` | `(&mut self) -> Result<(), BuildError>` | Kompletter Build |
| `run_build` | `(&mut self)` | Build-Hooks ausführen |
| `generate_init` | `(&mut self)` | `/init` + `/etc/profile` erzeugen |
| `pack_build` | `(&mut self)` | CPIO-Archiv packen |
| `run_hook` | `(&mut self, hook: &str) -> Result<Vec<String>, HookError>` | Einzelnen Hook ausführen |
| `run_func` | `(&mut self, func: &dyn Fn(&Self) -> Option<String>) -> Result<Option<String>, FuncError>` | Einzelne Hook-Funktion ausführen |
| `generate_profile` | `(&self) -> Vec<String>` | Shell-Profil (`/etc/profile`) |
| `generate_init_main` | `(&self) -> Vec<String>` | Hauptteil des Init-Skripts |
| `run_checks` | `(&mut self) -> Result<(), CheckError>` | Build-Checks |
| `run_tests` | `(&mut self) -> Result<(), TestError>` | Post-Build-Tests |

### Hook-Typen (Rust-Enum)

```rust
pub enum HookLevel {
    InitPre,
    InitDebug,
    InitMain,
    InitMount,
    InitFinal,
    BuildEnum,
    BuildPre,
    BuildTasks,
    BuildLate,
    BuildDeploy,
    BuildFinal,
    Pack,
    Checks,
    Tests,
    Functions,
}
```

---







## 3. Konfigurations-API

### `bkg_cave_boot_config::InitramfsConfig`

`HashMap`-Ersatz mit Typprüfung, Queues und Stage-Gating.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `new` | `(NO_BASE: bool, args: HashMap<String, Value>) -> Result<Self, ConfigError>` | Neue Konfiguration |
| `load_module` | `(&mut self, toml_path: &Path, module_name: &str) -> Result<(), ConfigError>` | TOML-Modul laden |
| `set` | `<T: Into<Value>>(&mut self, key: &str, value: T)` | Wert setzen (mit Queue/Typ-Prüfung) |
| `get` | `<T: From<Value>>(&self, key: &str) -> Option<T>` | Wert lesen |
| `validate` | `(&self) -> Result<(), ValidationError>` | Validierung in Stage `final` |
| `imports` | `-> &HashMap<HookLevel, Vec<HookFunction>>` | Registrierte Hook-Funktionen |
| `modules` | `-> &NoDupFlatList<String>` | Geladene Module |

### Config-Stages

```rust
pub enum Stage {
    Early,  // Basismodule laden, CLI-Args
    Late,   // Module einbinden, late_args
    Final,  // Validierung, Read-Only
}
```

---







## 4. Plugin-/Hook-System

Jedes Konfigurationsmodul liefert eine **Hook-Funktion**, die entweder:
- einen `&str` (einzige Code-Zeile)
- `Vec<String>` (mehrere Code-Zeilen)
- `()` (kein Output, nur Seiteneffekte)

```rust
pub type HookFunction = fn(&InitramfsGenerator) -> Option<HookOutput>;

pub enum HookOutput {
    Single(String),
    Multi(Vec<String>),
}
```

Module können deklarativ Abhängigkeiten ausdrücken:

```rust
pub struct ModuleManifest {
    pub name: &'static str,
    pub needs: Vec<&'static str>,   // required tags
    pub provides: Vec<&'static str>, // emitted tags
    pub imports: HashMap<HookLevel, Vec<&'static str>>,
    pub custom_parameters: HashMap<&'static str, ParamType>,
}
```

---







## 5. Shell-Template-API

### `bkg_cave_boot_shell::ShellBuilder`

Erzeugt die Shell-Ausgaben (ersetzt `base.py`/`core.py`).

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `einfo` | `(&self, msg: &str)` | Info-Log (grün) |
| `ewarn` | `(&self, msg: &str)` | Warnung (gelb) |
| `eerror` | `(&self, msg: &str)` | Fehler (rot) |
| `edebug` | `(&self, msg: &str)` | Debug (blau) |
| `setvar` | `(&self, name: &str, value: &str) -> String` | Variable in `/run/ugrd/<name>` |
| `readvar` | `(&self, name: &str, default: Option<&str>) -> String` | Variable lesen |
| `check_var` | `(&self, name: &str) -> String` | Kernel-Cmdline prüfen |
| `prompt_user` | `(&self, msg: &str, timeout: Option<u32>) -> Vec<String>` | Interaktiver Prompt |
| `retry` | `(&self, retries: i32, timeout: u32, cmd: &[&str]) -> String` | Retry-Loop |
| `do_switch_root` | `(&self) -> Vec<String>` | Finaler Root-Switch |
| `rd_fail` | `(&self, msg: Option<&str>) -> Vec<String>` | Fehlerbehandlung |
| `rd_restart` | `(&self) -> Vec<String>` | Init-Neustart |

---







## 6. Fehler-API

```rust
pub enum BkgCaveBootError {
    ConfigError { msg: String },
    ValidationError { msg: String },
    BuildError { msg: String },
    AutodetectError { msg: String },
    HookError { hook: String, msg: String },
    IoError { source: io::Error },
    TomlParseError { source: toml::de::Error },
    ModuleLoadError { module: String, msg: String },
    CompressionError { format: String },
}
```

---







## 7. Datei-API

### `bkg_cave_boot_fs`

Dateioperationen im Build-Dir (ersetzt `generator_helpers.py`).

| Funktion | Signatur | Zweck |
|----------|----------|-------|
| `mkdir` | `(base: &Path, rel: &Path, mode: u32)` | Verzeichnis erstellen |
| `copy` | `(src: &Path, dst: &Path)` | Datei kopieren |
| `symlink` | `(src: &Path, dst: &Path)` | Symlink erstellen |
| `write_file` | `(path: &Path, content: &str, mode: u32)` | Datei schreiben |
| `deploy_dependencies` | `(deps: &[Path], dst: &Path)` | Abhängigkeiten kopieren |
| `deploy_symlinks` | `(links: &[(Path, Path)], dst: &Path)` | Symlinks erstellen |
| `deploy_nodes` | `(nodes: &[DeviceNode], dst: &Path)` | Device-Nodes anlegen |
| `deploy_xz_deps` | `(deps: &[Path], dst: &Path)` | XZ dekomprimieren |
| `deploy_zstd_deps` | `(deps: &[Path], dst: &Path)` | ZSTD dekomprimieren |

---







## 8. Rust-Crate-Zuordnung (Target)

| Original-Python | Rust-Crate (bkg-cave) | Rust-Modul |
|-----------------|----------------------|-----------|
| `ugrd/main.py` | `bkg-cave-boot` | `bkg_cave_boot_cli` |
| `ugrd/initramfs_generator.py` | `bkg-cave-boot` | `bkg_cave_boot_core` |
| `ugrd/initramfs_config.py` | `bkg-cave-boot` | `bkg_cave_boot_config` |
| `ugrd/generator_helpers.py` | `bkg-cave-boot` | `bkg_cave_boot_shell` |
| `ugrd/initramfs_protocol.py` | `bkg-cave-boot` | `bkg_cave_boot_protocol` |
| `ugrd/exceptions.py` | `bkg-cave-core` | `bkg_cave_core::error::BootError` |
| `ugrd/config_helpers.py` | `bkg-cave-boot` | `bkg_cave_boot_config` |
| `ugrd/base/` (alle Module) | `bkg-cave-boot` | `bkg_cave_boot_base::*` |
| `ugrd/fs/*` | `bkg-cave-boot` | `bkg_cave_boot_fs::*` |
| `ugrd/kmod/*` | `bkg-cave-boot` | `bkg_cave_boot_kmod::*` |
| `ugrd/crypto/*` | `bkg-cave-boot` | `bkg_cave_boot_crypto::*` |
| `ugrd/net/*` | `bkg-cave-boot` | `bkg_cave_boot_net::*` |


