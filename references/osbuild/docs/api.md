# api.md — osbuild Öffentliche Rust-APIs (bkg-cave-build)

> Diese Datei beschreibt die öffentlichen APIs der geplanten Rust-Portierung.
> Namensraum: `bkg_cave_build`. Migration aus: `osbuild` (Python).

---







## 1. CLI-API

### `bkg_cave_build_cli::main() -> Result<i32, BkgCaveBuildError>`

Hauptentrypoint des Binarys `bkg-cave-build`.

- **Parameter**: liest `std::env::args()`
- **Rückgabe**: Exit-Code (0 = Erfolg)
- **Fehlerfälle**: `InvalidManifest`, `SchemaValidationFailed`, `PipelineBuildFailed`, `StoreError`

### `bkg_cave_build_cli::parse_manifest(path: &str) -> Result<Value, Error>`

Liest ein Manifest aus einem Pfad oder STDIN (`"-"`).

- **Parameter**: Pfad zur JSON-Datei oder `"-"` für STDIN
- **Rückgabe**: `serde_json::Value`
- **Fehlerfälle**: `IoError`, `JsonParseError`

---







## 2. Pipeline-API

### `bkg_cave_build_pipeline::Manifest`

Gesamtdarstellung einer Build-Pipeline.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `new()` | `-> Self` | Leeres Manifest |
| `add_pipeline` | `(&mut self, name: &str, runner: Runner, build: Option<&str>, source_epoch: Option<i64>) -> &mut Pipeline` | Pipeline hinzufügen |
| `add_source` | `(&mut self, info: ModuleInfo, items: Vec<Item>, options: Value) -> &mut Source` | Source hinzufügen |
| `download` | `(&self, store: &ObjectStore, monitor: &dyn Monitor) -> Result<DownloadResult>` | Alle Sources herunterladen |
| `depsolve` | `(&self, store: &ObjectStore, targets: &[&str]) -> Result<Vec<String>>` | Abhängigkeiten auflösen |
| `build` | `(&self, store: &ObjectStore, pipelines: &[String], monitor: &dyn Monitor, libdir: &str, ...) -> Result<ManifestBuildResult>` | Alle anstehenden Pipelines bauen |
| `mark_checkpoints` | `(&mut self, patterns: &[&str]) -> HashSet<String>` | Checkpoints per Glob setzen |

### `bkg_cave_build_pipeline::Pipeline`

Einzelne Pipeline (geordnete Liste von Stages).

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `id()` | `-> &str` | Inhaltlicher Hash der Pipeline |
| `add_stage` | `(&mut self, info: ModuleInfo, options: Value, sources_options: Option<Value>) -> &mut Stage` | Stage anhängen |
| `build_stages` | `(&self, store: &ObjectStore, monitor: &dyn Monitor, ...) -> Result<Vec<StageResult>>` | Alle Stages sequentiell ausführen |

### `bkg_cave_build_pipeline::Stage`

Einzelner Ausführungsschritt mit deterministischem ID-Hash.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `id()` | `-> &str` | SHA-256 über name + build + base + options + inputs + mounts |
| `dependencies()` | `-> impl Iterator<Item=&str>` | IDs der Vorgänger-Pipelines |
| `run` | `(&self, tree_path: &Path, meta_name: &Path, ...) -> Result<StageResult>` | Stage in Sandbox ausführen |

---







## 3. BuildRoot-API

### `bkg_cave_build_buildroot::BuildRoot`

bwrap-basierte Sandbox für Stage-Ausführung.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `new` | `(root: &Path, runner: &Path, libdir: &Path, var: &Path) -> Self` | Konstruktor |
| `register_api` | `(&mut self, api: impl ApiEndpoint)` | JSON-IPC-Endpoint anmelden |
| `run` | `(&self, argv: &[&str], monitor: &dyn Monitor, ...) -> Result<CompletedBuild>` | Kommando in der Sandbox ausführen |
| `build_capabilities_args` | `(&self) -> Vec<&'static str>` | Capability-Argumente für bwrap |

### `bkg_cave_build_buildroot::CompletedBuild`

Ergebnis einer Sandbox-Ausführung.

| Feld | Typ | Beschreibung |
|------|-----|-------------|
| `returncode` | `i32` | Exit-Code des Prozesses |
| `stdout` | `String` | Gesammelte Stdout+Stderr-Ausgabe |
| `output` | `String` | Alias für stdout |

---







## 4. ObjectStore-API

### `bkg_cave_build_objectstore::ObjectStore`

Content-adressierter Speicher-Cache.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `open` | `(path: &Path, read_only: bool) -> Result<Self>` | Store öffnen |
| `contains` | `(&self, object_id: &str, only_cached: bool) -> bool` | Objekt vorhanden? |
| `get` | `(&self, object_id: &str) -> Option<Object>` | Objekt lesen |
| `new` | `(&mut self, object_id: &str) -> Object` | Neues Schreib-Objekt |
| `commit` | `(&mut self, obj: &Object, object_id: &str)` | In Cache übernehmen |
| `host_tree` | `-> &HostTree` | Read-Only-Host-View |
| `export_floating` | `-> Vec<ObjectRef>` | Aktive Objekte exportieren (VM-Migration) |
| `load_floating` | `(&mut self, refs: &[ObjectRef])` | Floating-Objekte einlesen |

### `bkg_cave_build_objectstore::Object`

Einzelnes Store-Objekt (READ oder WRITE).

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `id()` | `-> &str` | Objekt-ID |
| `mode()` | `-> ObjectMode` | READ oder WRITE |
| `tree()` | `-> &Path` | Pfad zum Dateibaum |
| `meta()` | `-> &Metadata` | Metadaten-Zugriff |
| `init` | `(&self, base: &Object)` | Mit Basis-Objekt initialisieren |
| `finalize` | `(&mut self)` | WRITE → READ überführen |
| `export` | `(&self, dest: &Path)` | In externes Verzeichnis kopieren |

---







## 5. Meta-API

### `bkg_cave_build_meta::Index`

Modul-/Schema-/Format-Index.

| Methode | Signatz | Zweck |
|---------|----------|-------|
| `new` | `(libdir: &Path) -> Self` | Index aus Lib-Verzeichnis laden |
| `list_formats` | `() -> Vec<String>` | Unterstützte Manifest-Formate |
| `detect_format_info` | `(data: &Value) -> Option<FormatInfo>` | Format an Manifest-Inhalt erkennen |
| `get_module_info` | `(klass: &str, name: &str) -> Option<ModuleInfo>` | Meta-Daten zu Stage/Assembler/Source |
| `get_schema` | `(klass: &str, name: Option<&str>, version: &str) -> Schema` | JSON-Schema für Validierung |
| `list_runners` | `(distro: Option<&str>) -> Vec<RunnerInfo>` | Verfügbare Runner auflisten |
| `detect_host_runner` | `() -> Result<RunnerInfo>` | Runner anhand `/etc/os-release` |

### `bkg_cave_build_meta::Schema`

JSON-Schema-Validierung.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `validate` | `(&self, target: &Value) -> ValidationResult` | Daten gegen Schema prüfen |

---







## 6. Monitor-API

### `bkg_cave_build_monitor::Monitor` (Trait)

Ereignis-Reporting während des Builds.

| Methode | Signatur | Zweck |
|---------|----------|-------|
| `log` | `(&self, msg: &str, origin: &str)` | Lognachricht |
| `begin` | `(&self, item: &dyn MonitorItem)` | Start eines Pipeline/Stage/Source |
| `result` | `(&self, result: &dyn MonitorResult)` | Ergebnis eines Schritts |
| `finish` | `(&self, data: &Value)` | Abschluss eines Schritts |
| `error` | `(&self, msg: &str)` | Fehler melden |

Implementierungen:
- `NullMonitor` — kein Output
- `LogMonitor` — stdout/stderr
- `JsonMonitor` — strukturiertes JSON

---







## 7. Fehler-API (bkg-cave-build-Error)

```rust
pub enum BkgCaveBuildError {
    /// Manifest konnte nicht gelesen/geparst werden
    InvalidManifest { source: io::Error },

    /// JSON-Schema-Validierung fehlgeschlagen
    SchemaValidationFailed { errors: Vec<ValidationError> },

    /// Pipeline konnte nicht gebaut werden
    PipelineBuildFailed { pipeline: String, stage: String },

    /// Object-Store-Fehler
    StoreError { source: objectstore::StoreError },

    /// Sandbox-Fehler (bwrap)
    SandboxError { source: io::Error },

    /// Quelle nicht gefunden
    SourceNotFound { name: String },
}
```

---







## 8. Rust-Crate-Zuordnung (Target)

| Original-Python | Rust-Crate (bkg-cave) | Rust-Modul |
|-----------------|----------------------|-----------|
| `osbuild/main_cli.py` | `bkg-cave-build` | `bkg_cave_build_cli` |
| `osbuild/pipeline.py` | `bkg-cave-build` | `bkg_cave_build_pipeline` |
| `osbuild/objectstore.py` | `bkg-cave-build` | `bkg_cave_build_objectstore` |
| `osbuild/meta.py` | `bkg-cave-build` | `bkg_cave_build_meta` |
| `osbuild/buildroot.py` | `bkg-cave-build` | `bkg_cave_build_buildroot` |
| `osbuild/api.py` | `bkg-cave-build` | `bkg_cave_build_api` |
| `osbuild/monitor.py` | `bkg-cave-build` | `bkg_cave_build_monitor` |
| `osbuild/qemu.py` | `bkg-cave-build` | `bkg_cave_build_qemu` |
| `osbuild/sources.py` | `bkg-cave-build` | `bkg_cave_build_sources` |
| `osbuild/inputs.py` | `bkg-cave-build` | `bkg_cave_build_inputs` |
| `osbuild/devices.py` | `bkg-cave-build` | `bkg_cave_build_devices` |
| `osbuild/mounts.py` | `bkg-cave-build` | `bkg_cave_build_mounts` |
| `osbuild/solver/` | `bkg-cave-build-solver` | `bkg_cave_build_solver::*` |
| `osbuild/util/` | `bkg-cave-build-util` | `bkg_cave_build_util::*` |
| `osbuild/formats/` | `bkg-cave-build-formats` | `bkg_cave_build_formats::*` |


