# files.md — osbuild Dateiinventar (bkg-cave-build)

> Bezugnehmend auf die Analyse in `analysis.md` wird jedes Quellmodul auf sein
> Rust-Zielmodul im Crate `bkg-cave-build` abgebildet.

---







## 1. Python-Originaldateien

### 1.1 Hauptmodule

| Originaldatei | Zeilen | Verantwortung |
|---------------|--------|---------------|
| `osbuild/__init__.py` | 21 | Paketmarker, Version |
| `osbuild/__main__.py` | — | `python -m osbuild` |
| `osbuild/main_cli.py` | 232 | CLI-Entrypoint, Manifest laden, Build starten |
| `osbuild/pipeline.py` | 732 | Manifest, Pipeline, Stage, Runner, Executor |
| `osbuild/buildroot.py` | 393 | bwrap-Sandbox (BuildRoot) |
| `osbuild/objectstore.py` | 635 | Content-Store (Object, ObjectStore, HostTree) |
| `osbuild/meta.py` | 815 | Index, Schema, Validierung, Runner-Erkennung |
| `osbuild/api.py` | 212 | JSON-IPC-Endpoint (BaseAPI, API) |
| `osbuild/monitor.py` | — | Monitor (Null/Log/Json) |
| `osbuild/mixins.py` | — | Shared Mixins |
| `osbuild/host.py` | — | Service-Manager |
| `osbuild/inputs.py` | — | Input-Manager |
| `osbuild/devices.py` | — | Device-Manager |
| `osbuild/mounts.py` | — | Mount-Manager |
| `osbuild/sources.py` | — | Source-Download/Copy |
| `osbuild/qemu.py` | — | QEMU-VM für In-VM-Builds |
| `osbuild/loop.py` | — | Loop-Device-Management |
| `osbuild/vm.py` | 315 | VM-Integration |
| `osbuild/remoteloop.py` | — | Remote-Loop-Server |
| `osbuild/formats/__init__.py` | — | Format-Registry |
| `osbuild/formats/v1.py` | 311 | Manifest-Format Version 1 |
| `osbuild/formats/v2.py` | — | Manifest-Format Version 2 |
| `osbuild/solver/__init__.py` | — | Solver-Paket |
| `osbuild/solver/model.py` | 620 | Abhängigkeitsmodell |
| `osbuild/solver/dnf.py` | 522 | DNF4-Solver |
| `osbuild/solver/dnf5.py` | 560 | DNF5-Solver |
| `osbuild/solver/exceptions.py` | — | Solver-Fehler |
| `osbuild/solver/request.py` | 276 | Solver-Request |
| `osbuild/solver/api/v1.py` | — | Solver-REST-API v1 |
| `osbuild/solver/api/v2.py` | — | Solver-REST-API v2 |
| `osbuild/util/__init__.py` | — | Util-Paket |
| `osbuild/util/fscache.py` | 1278 | Dateisystem-Cache (FsCache) |
| `osbuild/util/lvm2.py` | 625 | LVM2-Hilfsroutinen |
| `osbuild/util/lorax.py` | 315 | Lorax-Integration |
| `osbuild/util/linux.py` | 437 | Linux-Hilfsmittel |
| `osbuild/util/jsoncomm.py` | — | JSON-Socket-Kommunikation |
| `osbuild/util/mnt.py` | — | Mount-Hilfen |
| `osbuild/util/osrelease.py` | — | os-release-Parser |
| `osbuild/util/path.py` | — | Pfad-Hilfen |
| `osbuild/util/sbom/__init__.py` | — | SBOM-Paket |
| `osbuild/util/sbom/spdx2/model.py` | — | SPDX-2-Modell |
| `osbuild/testutil/__init__.py` | — | Test-Utilities |
| `osbuild/testutil/atomic.py` | — | Atomare Operationen (Tests) |
| `osbuild/testutil/dnf4.py` | — | DNF4-Test-Hilfe |
| `osbuild/testutil/imports.py` | — | Import-Test-Hilfe |
| `osbuild/testutil/net.py` | — | Netzwerk-Test-Hilfe |
| `test/conftest.py` | — | Test-Fixtures |
| `test/test.py` | — | Integrationstests |
| `test/testvm.py` | — | VM-Tests |
| `test/mod/test_*.py` | — | Unit-Tests (Solver, Formate, Util) |
| `tools/osbuild-mpp` | — | Manifest-Pruning-Policy |
| `tools/osbuild-dev` | — | Entwickler-Helfer |
| `tools/osbuild-image-info` | — | Image-Info-Abfrage |
| `assemblers/*` | — | Assembler-Module |
| `devices/*` | — | Device-Module |
| `inputs/*` | — | Input-Module |
| `mounts/*` | — | Mount-Module |
| `sources/*` | — | Source-Module |
| `docs/osbuild.1.rst` | — | Manpage |
| `docs/manifest.md` | — | Manifest-Dokumentation |
| `docs/osbuild-manifest.5.rst` | — | Manifest-Schema-Doku |

### 1.2 Rust-Zielmodule (bkg-cave-build)

| Rust-Modul | Originaldatei | Zeilen (ca.) |
|-----------|--------------|--------------|
| `bkg_cave_build_cli` | `main_cli.py` | ~240 |
| `bkg_cave_build_pipeline` | `pipeline.py` | ~740 |
| `bkg_cave_build_buildroot` | `buildroot.py` | ~400 |
| `bkg_cave_build_objectstore` | `objectstore.py` | ~640 |
| `bkg_cave_build_meta` | `meta.py` | ~820 |
| `bkg_cave_build_api` | `api.py` | ~215 |
| `bkg_cave_build_monitor` | `monitor.py` | — |
| `bkg_cave_build_host` | `host.py` | — |
| `bkg_cave_build_inputs` | `inputs.py` | — |
| `bkg_cave_build_devices` | `devices.py` | — |
| `bkg_cave_build_mounts` | `mounts.py` | — |
| `bkg_cave_build_sources` | `sources.py` | — |
| `bkg_cave_build_qemu` | `qemu.py` | — |
| `bkg_cave_build_loop` | `loop.py` | — |
| `bkg_cave_build_vm` | `vm.py` | ~320 |
| `bkg_cave_build_remoteloop` | `remoteloop.py` | — |
| `bkg_cave_build_formats` | `formats/` | — |
| `bkg_cave_build_solver` | `solver/` | ~2000 |
| `bkg_cave_build_util_fscache` | `util/fscache.py` | ~1280 |
| `bkg_cave_build_util_lvm2` | `util/lvm2.py` | ~630 |
| `bkg_cave_build_util_lorax` | `util/lorax.py` | ~320 |
| `bkg_cave_build_util_linux` | `util/linux.py` | ~440 |
| `bkg_cave_build_util_sbom` | `util/sbom/` | — |

---







## 2. Import-Graph (osbuild)

```
main_cli.py
├── pipeline.py (Manifest, Pipeline, Stage)
│   ├── buildroot.py (BuildRoot, bwrap)
│   ├── objectstore.py (ObjectStore, Object)
│   ├── api.py (API, BaseAPI)
│   ├── host.py (ServiceManager)
│   ├── inputs.py (InputManager)
│   ├── devices.py (DeviceManager)
│   ├── mounts.py (MountManager)
│   ├── sources.py (Source)
│   ├── qemu.py (Qemu)
│   └── remoteloop.py (LoopServer)
├── meta.py (Index, ModuleInfo, Schema)
│   └── util/osrelease.py
├── monitor.py (Monitor-Trait)
└── formats/v1.py + v2.py
    └── meta.py, pipeline.py

objectstore.py
└── util/fscache.py (FsCache)

pipeline.py
└── buildroot.py → bwrap (subprocess)
```

---







## 3. Crate-Zuordnung (bkg-cave-build)

```
bkg-cave/
├── crates/
│   ├── bkg-cave-build/            # Hauptcrate (CLI + Pipeline + Meta)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── cli.rs             # main_cli
│   │   │   ├── pipeline.rs        # pipeline (Manifest, Pipeline, Stage)
│   │   │   ├── buildroot.rs       # buildroot
│   │   │   ├── objectstore.rs     # objectstore
│   │   │   ├── meta.rs            # meta (Index, Schema)
│   │   │   ├── api.rs             # JSON-IPC
│   │   │   ├── monitor.rs         # Monitor-Trait + Impls
│   │   │   ├── formats.rs         # v1 + v2
│   │   │   └── runner.rs          # Runner-Erkennung
│   │   └── Cargo.toml
│   │
│   ├── bkg-cave-build-solver/     # Solver (separates Crate)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── model.rs
│   │   │   ├── dnf.rs
│   │   │   ├── dnf5.rs
│   │   │   ├── request.rs
│   │   │   └── api/
│   │   └── Cargo.toml
│   │
│   └── bkg-cave-build-util/       # Utility-Crate
│       ├── src/
│       │   ├── lib.rs
│       │   ├── fscache.rs
│       │   ├── lvm2.rs
│       │   ├── lorax.rs
│       │   ├── linux.rs
│       │   ├── jsoncomm.rs
│       │   ├── osrelease.rs
│       │   └── sbom.rs
│       └── Cargo.toml
│
└── src/bin/
    ├── bkg-cave-build/
    │   └── main.rs
    └── bkg-cave-build-quiet/
        └── main.rs
```


