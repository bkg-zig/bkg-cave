//! Module index — discovers and validates available stages, assemblers, and
//! runners from a library directory.
//!
//! The module index reads JSON metadata files from subdirectories of `libdir`:
//!
//! * `libdir/stages/*.json` — stage definitions
//! * `libdir/assemblers/*.json` — assembler definitions
//! * `libdir/runners/*.json` — runner definitions (executor environments)
//!
//! Each JSON file contains name, description, and optional metadata that the
//! pipeline engine uses to validate manifests before building.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cave_core::BkgCaveError;

use crate::pipeline::Manifest;

/// Metadata index of all available forge modules.
#[derive(Debug, Clone)]
pub struct Index {
    /// The library directory this index was built from.
    pub libdir: PathBuf,
    /// Discovered stage modules, keyed by name.
    pub stages: HashMap<String, StageInfo>,
    /// Discovered assembler modules, keyed by name.
    pub assemblers: HashMap<String, StageInfo>,
    /// Discovered runner modules, keyed by name.
    pub runners: HashMap<String, RunnerInfo>,
}

/// Information about a single stage or assembler module.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StageInfo {
    /// Fully-qualified module name (e.g. `"org.osbuild.curl"`).
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Path to the module file (set by the index scanner, not from JSON).
    #[serde(skip)]
    pub path: PathBuf,
}

/// Information about a runner module (execution environment).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RunnerInfo {
    /// Runner name (e.g. `"org.osbuild.linux"`).
    pub name: String,
    /// Path to the runner implementation.
    #[serde(skip)]
    pub path: PathBuf,
    /// Runner version string.
    #[serde(default)]
    pub version: String,
}

impl Index {
    /// Scan a library directory and build a module index.
    ///
    /// The directory is expected to contain `stages/`, `assemblers/`, and
    /// `runners/` subdirectories.  Any subdirectory that is missing is simply
    /// skipped (the corresponding map remains empty).
    ///
    /// # Errors
    ///
    /// Returns I/O errors from reading directories and JSON parse errors from
    /// malformed module descriptions.
    pub fn new(libdir: &Path) -> Result<Self, BkgCaveError> {
        let mut index = Index {
            libdir: libdir.to_owned(),
            stages: HashMap::new(),
            assemblers: HashMap::new(),
            runners: HashMap::new(),
        };

        // Scan stages.
        let stages_dir = libdir.join("stages");
        if stages_dir.is_dir() {
            for entry in std::fs::read_dir(&stages_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(&path)?;
                    let mut info: StageInfo = serde_json::from_str(&content)?;
                    info.path = path;
                    index.stages.insert(info.name.clone(), info);
                }
            }
        }

        // Scan assemblers.
        let assemblers_dir = libdir.join("assemblers");
        if assemblers_dir.is_dir() {
            for entry in std::fs::read_dir(&assemblers_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(&path)?;
                    let mut info: StageInfo = serde_json::from_str(&content)?;
                    info.path = path;
                    index.assemblers.insert(info.name.clone(), info);
                }
            }
        }

        // Scan runners.
        let runners_dir = libdir.join("runners");
        if runners_dir.is_dir() {
            for entry in std::fs::read_dir(&runners_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(&path)?;
                    let mut info: RunnerInfo = serde_json::from_str(&content)?;
                    info.path = path;
                    index.runners.insert(info.name.clone(), info);
                }
            }
        }

        tracing::debug!(
            stages = index.stages.len(),
            assemblers = index.assemblers.len(),
            runners = index.runners.len(),
            libdir = %libdir.display(),
            "Module index built"
        );

        Ok(index)
    }

    /// Validate that every stage and assembler referenced in the manifest
    /// is present in the index.
    ///
    /// # Errors
    ///
    /// Returns [`BkgCaveError::NotFound`] for each unknown stage, assembler,
    /// or runner.
    pub fn validate(&self, manifest: &Manifest) -> Result<(), BkgCaveError> {
        for pipeline in &manifest.pipelines {
            // Validate runner if specified.
            if let Some(ref runner) = pipeline.runner {
                if !self.runners.contains_key(runner) {
                    return Err(BkgCaveError::NotFound(format!(
                        "pipeline '{}' references unknown runner '{}'",
                        pipeline.name, runner
                    )));
                }
            }

            // Validate each stage.
            for stage in &pipeline.stages {
                if !self.stages.contains_key(&stage.name) {
                    return Err(BkgCaveError::NotFound(format!(
                        "pipeline '{}' references unknown stage '{}'",
                        pipeline.name, stage.name
                    )));
                }

                // Validate per-stage runner override.
                if let Some(ref build) = stage.build {
                    if let Some(ref runner) = build.runner {
                        if !self.runners.contains_key(runner) {
                            return Err(BkgCaveError::NotFound(format!(
                                "pipeline '{}' stage '{}' references unknown runner '{}'",
                                pipeline.name, stage.name, runner
                            )));
                        }
                    }
                }
            }

            // Validate assembler if present.
            if let Some(ref asm) = pipeline.assembler {
                if !self.assemblers.contains_key(&asm.name) {
                    return Err(BkgCaveError::NotFound(format!(
                        "pipeline '{}' references unknown assembler '{}'",
                        pipeline.name, asm.name
                    )));
                }
            }
        }

        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal stage JSON metadata file.
    fn make_stage_json(dir: &Path, name: &str, description: &str) -> PathBuf {
        let stages_dir = dir.join("stages");
        std::fs::create_dir_all(&stages_dir).unwrap();
        let path = stages_dir.join(format!("{name}.json"));
        let content = serde_json::json!({
            "name": name,
            "description": description,
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    /// Create a minimal assembler JSON metadata file.
    fn make_assembler_json(dir: &Path, name: &str, description: &str) -> PathBuf {
        let assemblers_dir = dir.join("assemblers");
        std::fs::create_dir_all(&assemblers_dir).unwrap();
        let path = assemblers_dir.join(format!("{name}.json"));
        let content = serde_json::json!({
            "name": name,
            "description": description,
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    /// Create a minimal runner JSON metadata file.
    fn make_runner_json(dir: &Path, name: &str, version: &str) -> PathBuf {
        let runners_dir = dir.join("runners");
        std::fs::create_dir_all(&runners_dir).unwrap();
        let path = runners_dir.join(format!("{name}.json"));
        let content = serde_json::json!({
            "name": name,
            "version": version,
        });
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    #[test]
    fn index_empty_libdir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let index = Index::new(tmp.path()).unwrap();
        assert!(index.stages.is_empty());
        assert!(index.assemblers.is_empty());
        assert!(index.runners.is_empty());
    }

    #[test]
    fn index_scans_stages() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_stage_json(tmp.path(), "org.osbuild.curl", "Download files via curl");
        make_stage_json(tmp.path(), "org.osbuild.tar", "Create a tar archive");

        let index = Index::new(tmp.path()).unwrap();
        assert_eq!(index.stages.len(), 2);
        assert!(index.stages.contains_key("org.osbuild.curl"));
        assert!(index.stages.contains_key("org.osbuild.tar"));
    }

    #[test]
    fn index_scans_assemblers() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_assembler_json(tmp.path(), "org.osbuild.qemu", "QEMU disk image assembler");

        let index = Index::new(tmp.path()).unwrap();
        assert_eq!(index.assemblers.len(), 1);
        assert!(index.assemblers.contains_key("org.osbuild.qemu"));
    }

    #[test]
    fn index_scans_runners() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_runner_json(tmp.path(), "org.osbuild.linux", "2.6");

        let index = Index::new(tmp.path()).unwrap();
        assert_eq!(index.runners.len(), 1);
        assert!(index.runners.contains_key("org.osbuild.linux"));
        assert_eq!(index.runners["org.osbuild.linux"].version, "2.6");
    }

    #[test]
    fn validate_known_stage() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_stage_json(tmp.path(), "org.osbuild.curl", "curl stage");

        let index = Index::new(tmp.path()).unwrap();
        let manifest = Manifest::from_json(
            r#"{
            "pipelines": [{
                "name": "build",
                "stages": [{
                    "name": "org.osbuild.curl",
                    "options": {},
                    "inputs": {},
                    "devices": {},
                    "mounts": []
                }]
            }]
        }"#,
        )
        .unwrap();

        assert!(index.validate(&manifest).is_ok());
    }

    #[test]
    fn validate_unknown_stage_fails() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Don't create any stage files — index is empty.
        let index = Index::new(tmp.path()).unwrap();
        let manifest = Manifest::from_json(
            r#"{
            "pipelines": [{
                "name": "build",
                "stages": [{
                    "name": "org.osbuild.nonexistent",
                    "options": {},
                    "inputs": {},
                    "devices": {},
                    "mounts": []
                }]
            }]
        }"#,
        )
        .unwrap();

        let result = index.validate(&manifest);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, BkgCaveError::NotFound(_)),
            "expected NotFound, got: {err}"
        );
    }

    #[test]
    fn validate_unknown_assembler_fails() {
        let tmp = tempfile::TempDir::new().unwrap();
        let index = Index::new(tmp.path()).unwrap();
        let manifest = Manifest::from_json(
            r#"{
            "pipelines": [{
                "name": "image",
                "stages": [],
                "assembler": {
                    "name": "org.osbuild.mkimage",
                    "options": {}
                }
            }]
        }"#,
        )
        .unwrap();

        let result = index.validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn validate_unknown_runner_fails() {
        let tmp = tempfile::TempDir::new().unwrap();
        let index = Index::new(tmp.path()).unwrap();
        let manifest = Manifest::from_json(
            r#"{
            "pipelines": [{
                "name": "build",
                "runner": "org.osbuild.norunner",
                "stages": []
            }]
        }"#,
        )
        .unwrap();

        let result = index.validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn validate_known_assembler_succeeds() {
        let tmp = tempfile::TempDir::new().unwrap();
        make_assembler_json(tmp.path(), "org.osbuild.qemu", "QEMU assembler");

        let index = Index::new(tmp.path()).unwrap();
        let manifest = Manifest::from_json(
            r#"{
            "pipelines": [{
                "name": "image",
                "stages": [],
                "assembler": {
                    "name": "org.osbuild.qemu",
                    "options": {}
                }
            }]
        }"#,
        )
        .unwrap();

        assert!(index.validate(&manifest).is_ok());
    }

    #[test]
    fn stage_info_has_path_set() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = make_stage_json(tmp.path(), "org.osbuild.curl", "curl");

        let index = Index::new(tmp.path()).unwrap();
        let info = index.stages.get("org.osbuild.curl").unwrap();
        assert_eq!(info.path, path);
        assert_eq!(info.description, "curl");
    }

    #[test]
    fn runner_info_has_path_and_version() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = make_runner_json(tmp.path(), "org.osbuild.linux", "6.1");

        let index = Index::new(tmp.path()).unwrap();
        let info = index.runners.get("org.osbuild.linux").unwrap();
        assert_eq!(info.path, path);
        assert_eq!(info.version, "6.1");
    }

    #[test]
    fn index_ignores_non_json_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let stages_dir = tmp.path().join("stages");
        std::fs::create_dir_all(&stages_dir).unwrap();
        // Create a non-JSON file.
        std::fs::write(stages_dir.join("readme.txt"), b"hello").unwrap();

        let index = Index::new(tmp.path()).unwrap();
        assert!(index.stages.is_empty());
    }
}
