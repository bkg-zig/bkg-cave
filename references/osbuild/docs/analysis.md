# analysis.md — osbuild

> **Analyseziel**: Verhaltensgleiche Rust-Portierung als `bkg-cave-build`.
> **Quelle**: https://github.com/osbuild/osbuild (Apache-2.0)

---

## 1. Gesamtübersicht

osbuild (Version 189) ist ein pipeline-basiertes Build-System für Betriebssystem-Images.
Stages, Assembler und Sources werden in einer Manifest-Datei (JSON) beschrieben und
von einem Pipeline-Executor ausgeführt — isoliert in einer `bubblewrap`-Sandbox.

| Merkmal | Wert |
|---------|------|
| Originalsprache | Python 3.6+ |
| Lizenz | Apache-2.0 |
| Zeilen gesamt (Python) | ~48.600 (inkl. Tests) |
| Hauptmodul | `osbuild` |
| Entrypoint | `osbuild.main_cli:osbuild_cli` |
| Kernidee | Pipeline / Stages / Buildgraph |

---

## 2. Architektur

### 2.1 Schichten

```
┌─────────────────────────────────────────────────┐
│  main_cli.py                                       │  CLI-Entrypoint
│  ├── argparse                                       │
│  ├── Manifest laden/validieren                      │
│  └── ObjectStore + Monitor öffnen                   │
└────────────────────┬──────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────┐
│  pipeline.py                                        │
│  ├── Manifest (pipelines, sources)                  │
│  ├── Pipeline → Stage[] build_stages()              │
│  │     ├── buildroot.BuildRoot (bwrap)              │
│  │     ├── Input/Device/Mount Manager               │
│  │     └── stage.run() → sandboxed execution        │
│  ├── Stage.id (SHA-256 aus Optionen+Inputs+Mounts)  │
│  └── Runner / Result-Typen                          │
└────────────────────┬──────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────┐
│  objectstore.py                                     │
│  ├── ObjectStore (content-adressierter Cache)       │
│  ├── Object READ/WRITE + Metadata                   │
│  ├── FsCache + staging/commit                       │
│  ├── HostTree (read-only Host-View)                 │
│  └── StoreServer/StoreClient (JSON-IPC)             │
└────────────────────┬──────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────┐
│  meta.py                                           │
│  ├── Index (stages/assemblers/schemas laden)       │
│  ├── ModuleInfo + Schema (JSON-Schema-Validierung) │
│  ├── FormatInfo (Manifest-Format v1/v2)            │
│  └── Runner-Erkennung (os-release)                 │
└──────────────────────────────────────────────────┘
```

### 2.2 Wichtige Klassen

| Klasse | Datei | Verantwortung |
|--------|-------|----------------|
| `Manifest` | `pipeline.py` | Pipelines + Sources verwalten |
| `Pipeline` | `pipeline.py` | Reihenfolge der Stages planen |
| `Stage` | `pipeline.py` | Einzelner Ausführungsschritt |
| `Runner` | `pipeline.py` | Betriebssystem-spezifischer Runner |
| `BuildRoot` | `buildroot.py` | bwrap-Sandbox aufsetzen |
| `ObjectStore` | `objectstore.py` | Content-Store (Cache) |
| `Object` | `objectstore.py` | Store-Objekt READ/WRITE |
| `HostTree` | `objectstore.py` | Host-FS Read-Only-View |
| `Index` | `meta.py` | Modul-/Format-/Schema-Index |
| `Schema` | `meta.py` | JSON-Schema-Validierung |
| `API` | `api.py` | JSON-IPC-Endpoint für Sandbox |
| `StageResult` | `pipeline.py` | Erfolg/Fehler pro Stage |

### 2.3 Datenfluss beim Build

```
Manifest (JSON)
├── sources     → download → ObjectStore
├── pipelines   → depsolve → Reihenfolge berechnen
│                  └── benötigte Vorgänger bauen
└── build()
    └── Pipeline.run()
        └── stage.run()
            ├── BuildRoot.__enter__() (bwrap vorbereiten)
            ├── Input/Device/Mount mappen
            ├── Stage in Sandbox ausführen
            └── BuildRoot.__exit__() (räumen)
```

### 2.4 Sandbox-Umfang (BuildRoot)

`bubblewrap` (bwrap) wird verwendet, um jede Stage in einer privaten Umgebung
laufen zu lassen:

- Unshare: IPC, PID, Netzwerk
- Versiegelte Verzeichnisse: `/usr`, `/boot` read-only
- `/dev`, `/tmp`, `/run`, `/var` als tmpfs / bind-mounts
- `/proc`-Overrides
- Runner + Stage-Binary werden read-only eingehängt
- Capabilities können eingeschränkt werden (`build_capabilities_args`)

### 2.5 Objekt-Store

- Content-addressed in `.osbuild/store/`
- Jedes Objekt hat einen READ- und WRITE-Modus
- `FsCache` verwaltet staging → commit → Laden
- HostTree: Read-Only-View des Host-Dateisystems

---

## 3. Module (osbuild)

### 3.1 Hauptmodule

| Modul | Zeilen | Aufgabe |
|-------|--------|---------|
| `pipeline.py` | 732 | Manifest, Pipeline, Stage, Executor |
| `buildroot.py` | 393 | bwrap-Sandbox |
| `objectstore.py` | 635 | Content-Store |
| `meta.py` | 815 | Index, Schema, Validierung |
| `main_cli.py` | 232 | CLI (argparse) |

### 3.2 Util-Module

| Modul | Zeilen | Aufgabe |
|-------|--------|---------|
| `util/fscache.py` | 1278 | Dateisystem-Cache |
| `util/lvm2.py` | 625 | LVM2-Hilfsroutinen |
| `util/sbom/spdx2/model.py` | 397 | SPDX-2-Datenmodell |
| `util/lorax.py` | 315 | Lorax-Integration |
| `util/linux.py` | 437 | Linux-spezifische Hilfsmittel |

### 3.3 Solver-Module

| Modul | Zeilen | Aufgabe |
|-------|--------|---------|
| `solver/model.py` | 620 | Abhängigkeitsmodell |
| `solver/dnf5.py` | 560 | DNF5-Solver (neu) |
| `solver/dnf.py` | 522 | DNF4-Solver (legacy) |
| `solver/api/v2.py` | 476 | REST-API für Solver v2 |
| `solver/api/v1.py` | 303 | REST-API für Solver v1 … |
