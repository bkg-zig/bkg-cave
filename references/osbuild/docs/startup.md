# startup.md — osbuild Startsequenz

> **Ziel**: Genauer Ablauf eines osbuild-Runs als Grundlage für die Rust-Portierung in `bkg-cave-build`.
> **Namespace**: `bkg_cave_build`

---







## 1. CLI-Start

### 1.1 Aufruf

```text
bkg-cave-build <manifest.json> [Optionen]
bkg-cave-build -                         # Manifest aus STDIN
```

### 1.2 Schritte

```text
osbuild_cli()
│
├── 1. Argumente parsen (argparse / clap)
│   ├── manifest_path
│   ├── --cache / --store (Standard: ./.osbuild)
│   ├── --libdir (Standard: /usr/lib/osbuild)
│   ├── --cache-max-size
│   ├── --checkpoint
│   ├── --export / --in-vm / --output-directory
│   ├── --json / --inspect
│   ├── --monitor / --monitor-fd
│   ├── --stage-timeout
│   ├── --break <stage>       (debug shell)
│   ├── --quiet / -q
│   └── --version
│
├── 2. Manifest laden
│   └── parse_manifest(path)  → json::Value / stdin
│
├── 3. Manifest validieren
│   ├── Index(libdir) aus meta.py erzeugen
│   ├── detect_format_info(manifest) → FormatInfo (v1 oder v2)
│   ├── validate(manifest) → ValidationResult
│   └── Fehler? → Fehler ausgeben, Exit 2
│
├── 4. Manifest deserialisieren
│   └── fmt.load(desc, index) → Manifest (Python-Objekt)
│
├── 5. Export-IDs prüfen
│   └── Existieren alle --export-Namen im Manifest?
│
├── 6. Checkpoints markieren
│   └── mark_checkpoints(patterns) → HashSet
│
└── 7. ObjectStore + Monitor öffnen
    └── with ObjectStore(cache_dir) as store:
```

---







## 2. Build-Ausführung

### 2.1 Manifest.build()

```text
Manifest.build()
│
├── 1. download(store, monitor)
│   └── Jeden Source herunterladen/cachen
│
└── 2. depsolve(store, targets)
    │   Reihenfolge berechnen (topologische Sortierung)
    │
    └── 3. build() je Pipeline
        │
        └── Pipeline.run()
            │
            └── 4. build_stages()
                │
                ├── 5. Baum im ObjectStore reservieren (Object.new)
                │
                ├── 6. Build-Baum bestimmen
                │   ├── build=None → HostTree
                │   └── build=<pipeline-id> → ObjectStore.get()
                │
                ├── 7. Checkpoint-Suche (rekursiv von hinten)
                │
                └── 8. Stage-Schleife (todo-Deque)
                    │
                    └── Stage.run()
                        ├── BuildRoot.__enter__()
                        │   ├── /dev, /tmp, /run, /var tmpfs
                        │   ├── /usr, /boot read-only
                        │   ├── /proc override
                        │   └── APIs binden
                        │
                        ├── Input/Device/Mount mappen
                        │
                        ├── prepare_arguments(args, args_path)
                        │
                        ├── Stage in Sandbox ausführen
                        │   ├── bwrap + stage binary + JSON-args
                        │   └── stdout/stderr → Monitor
                        │
                        └── BuildRoot.__exit__()
```

### 2.2 Stage.run() Details

```text
Stage.run(tree_path, meta_name, runner, build_tree, store, monitor, ...)
│
├── 1. BuildRoot erstellen
│   └── Bubblewrap-Sandbox mit:
│       ├── IPC/PID/Netzwerk unshare
│       ├── /run/osbuild/lib bind-mount (stage binary + module)
│       ├── /run/osbuild/tree bind-mount
│       ├── /run/osbuild/meta bind-mount
│       ├── /run/osbuild/mounts bind-mount
│       ├── /run/osbuild/inputs bind-mount
│       └── /run/osbuild/api bind-mount
│
├── 2. Input-Manager
│   └── Inputs auflösen (Pipeline-Referenzen / Store-Objekte)
│
├── 3. Device-Manager
│   └── Loop-/Block-Devices öffnen
│
├── 4. Mount-Manager
│   └── Dateisysteme einhängen
│
├── 5. API-Endpoints registrieren
│   ├── JSON-API (Store, Build)
│   └── Remoteloop-Server
│
├── 6. Stage ausführen
│   └── bwrap mit:
│       ├── --ro-bind <stage> /run/osbuild/bin/<name>
│       ├── --ro-bind <input> /run/osbuild/inputs/...
│       ├── --bind <meta> /run/osbuild/api/arguments
│       ├── --bind <mount> /run/osbuild/mounts/...
│       ├── --bind <tree> /run/osbuild/tree
│       ├── --bind <build_tree> (optional)
│       └── runner /run/osbuild/runner/<name> <stage-args>
│
├── 7. Ergebnis sammeln
│   ├── returncode
│   ├── stdout+stderr
│   └── api.error (falls Exception via IPC)
│
└── 8. Räumen
    └── BuildRoot.__exit__() + ExitStack aufräumen
```

---







## 3. ObjectStore-Start

```text
ObjectStore(path)
│
├── __enter__()
│   ├── FsCache("osbuild", path) öffnen
│   ├── /tmp erstellen
│   ├── objects/ erstellen
│   └── HostTree initialisieren
│
└── __exit__()
    └── cleanup()
        ├── HostTree aufräumen
        └── Alle offenen Objects schließen
```

---







## 4. Monitor-Start

```text
make(monitor_name, monitor_fd, total_steps)
│
├── NullMonitor  → keine Ausgabe
├── LogMonitor   → stdout
└── JsonMonitor  → strukturiertes JSON auf FD
```

---







## 5. Relevante Umgebungsvariablen

| Variable | Zweck |
|----------|-------|
| `OSBUILD_EXPORT_FORCE_NO_PRESERVE_OWNER=1` | Ownership bei Export nicht bewahren |
| `SOURCE_DATE_EPOCH` | Reproduzierbare Builds |
| `container=bwrap-osbuild` | Sandbox-Umgebungsmarker |
| `PYTHONUNBUFFERED=1` | Unbuffered Output in Sandbox |

---







## 6. Rust-Äquivalente (bkg-cave-build)

| Schritt | Rust-Entität |
|---------|-------------|
| CLI-Parsing | `clap` → `BkgCaveBuildArgs` |
| Manifest laden | `serde_json::from_reader` |
| Manifest validieren | `bkg_cave_build_meta::Index::validate` |
| Format erkennen | `Index::detect_format_info` |
| ObjectStore | `bkg_cave_build_objectstore::ObjectStore` |
| BuildRoot | `bkg_cave_build_buildroot::BuildRoot` |
| bwrap-Aufruf | `std::process::Command` (bwrap) |
| JSON-IPC | `bkg_cave_build_api::ApiServer` |
| Monitor | `bkg_cave_build_monitor::Monitor` (Trait) |
| Logging | `tracing` subscribers |


