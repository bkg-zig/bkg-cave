//! Pipeline manifest data model — osbuild-compatible JSON representation.
//!
//! A [`Manifest`] contains one or more [`Pipeline`]s, each of which is a
//! sequence of [`Stage`]s that run inside a (possibly chained) build root.
//! Stages can reference inputs, devices, and mounts to wire up complex
//! build scenarios.

use std::collections::{HashMap, HashSet, VecDeque};

use cave_core::BkgCaveError;

// ── Core types ───────────────────────────────────────────────────────────────

/// An osbuild-compatible build manifest.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Manifest {
    /// Ordered list of pipelines to execute.
    pub pipelines: Vec<Pipeline>,
    /// Optional source definitions (org.osbuild.curl items, etc.).
    pub sources: Option<Vec<Source>>,
}

/// A single pipeline — a sequence of stages and an optional assembler.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Pipeline {
    /// Human-readable / symbolic name (used for dependency references).
    pub name: String,
    /// Optional runner override for this pipeline (e.g. `"org.osbuild.linux"`).
    pub runner: Option<String>,
    /// Reference to another pipeline whose build root this pipeline uses.
    #[serde(default)]
    pub build: Option<StageRef>,
    /// Ordered list of stages in this pipeline.
    pub stages: Vec<Stage>,
    /// Optional assembler that produces the final output artifact.
    #[serde(default)]
    pub assembler: Option<Assembler>,
}

/// A single stage within a pipeline.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Stage {
    /// Fully-qualified stage name (e.g. `"org.osbuild.curl"`).
    pub name: String,
    /// Optional per-stage build-root override.
    #[serde(default)]
    pub build: Option<StageRef>,
    /// Stage-specific options as a free-form JSON object.
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
    /// Named inputs that feed into this stage.
    #[serde(default)]
    pub inputs: HashMap<String, Input>,
    /// Named devices attached to this stage.
    #[serde(default)]
    pub devices: HashMap<String, Device>,
    /// Mount points set up inside the build root before the stage runs.
    #[serde(default)]
    pub mounts: Vec<Mount>,
}

/// Reference to another stage or pipeline.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StageRef {
    /// Name of the referenced pipeline.
    pub name: String,
    /// Optional stage name within the referenced pipeline.
    #[serde(default)]
    pub stage: Option<String>,
    /// Runner override for the referenced build root.
    #[serde(default)]
    pub runner: Option<String>,
    /// Options overrides for the referenced build.
    #[serde(default)]
    pub options: Option<HashMap<String, serde_json::Value>>,
    /// Input overrides for the referenced build.
    #[serde(default)]
    pub inputs: Option<HashMap<String, Input>>,
    /// Device overrides for the referenced build.
    #[serde(default)]
    pub devices: Option<HashMap<String, Device>>,
    /// Mount overrides for the referenced build.
    #[serde(default)]
    pub mounts: Option<Vec<Mount>>,
}

/// A source definition — describes external data needed by the manifest.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Source {
    /// Source name (e.g. `"org.osbuild.curl"`).
    pub name: String,
    /// Items keyed by identifier, each a map of free-form options.
    #[serde(default)]
    pub items: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// The assembler produces the final artifact from a built pipeline tree.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Assembler {
    /// Fully-qualified assembler name (e.g. `"org.osbuild.qemu"`).
    pub name: String,
    /// Assembler-specific options.
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
    /// Inputs for the assembler.
    #[serde(default)]
    pub inputs: Option<HashMap<String, Input>>,
    /// Devices for the assembler.
    #[serde(default)]
    pub devices: Option<HashMap<String, Device>>,
    /// Mounts for the assembler.
    #[serde(default)]
    pub mounts: Option<Vec<Mount>>,
}

/// An input reference — wires a named input to a source or pipeline output.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Input {
    /// Input type discriminator (e.g. `"org.osbuild.files"`, `"org.osbuild.tree"`).
    #[serde(rename = "type")]
    pub input_type: String,
    /// Optional origin reference.
    #[serde(default)]
    pub origin: Option<String>,
    /// Optional list of reference strings.
    #[serde(default)]
    pub references: Option<Vec<String>>,
}

/// A device attached to the build sandbox.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Device {
    /// Device name.
    pub name: String,
    /// Device type (e.g. `"org.osbuild.loopback"`).
    #[serde(rename = "type")]
    pub device_type: String,
    /// Device options.
    #[serde(default)]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// A mount point inside the build root.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Mount {
    /// Mount name.
    pub name: String,
    /// Mount type (e.g. `"org.osbuild.bind"`).
    #[serde(rename = "type")]
    pub mount_type: String,
    /// Source path (inside the build root or host).
    pub source: String,
    /// Target mount point (inside the build root).
    pub target: String,
    /// Mount options.
    #[serde(default)]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

// ── Implementations ──────────────────────────────────────────────────────────

impl Stage {
    /// Compute a content-addressed identifier for this stage.
    ///
    /// The ID is `SHA256(name || build_json || options_json || inputs_json ||
    /// devices_json || mounts_json)` encoded as a lowercase hex string.
    /// This is deterministic across identical stage definitions and can be
    /// used for caching.
    pub fn id(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());

        if let Some(ref build) = self.build {
            let json = serde_json::to_string(build).unwrap_or_default();
            hasher.update(json.as_bytes());
        }

        {
            let json = serde_json::to_string(&self.options).unwrap_or_default();
            hasher.update(json.as_bytes());
        }

        {
            let json = serde_json::to_string(&self.inputs).unwrap_or_default();
            hasher.update(json.as_bytes());
        }

        {
            let json = serde_json::to_string(&self.devices).unwrap_or_default();
            hasher.update(json.as_bytes());
        }

        {
            let json = serde_json::to_string(&self.mounts).unwrap_or_default();
            hasher.update(json.as_bytes());
        }

        hex::encode(hasher.finalize())
    }
}

impl Manifest {
    /// Parse a [`Manifest`] from a JSON string.
    ///
    /// The JSON is expected to match the osbuild manifest schema
    /// (see [osbuild manifest docs](https://www.osbuild.org/)).
    pub fn from_json(json: &str) -> Result<Self, BkgCaveError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Resolve pipeline dependencies and return names in build order.
    ///
    /// Uses Kahn's algorithm to topologically sort pipelines so that every
    /// pipeline's build-root dependency (if any) appears before the pipeline
    /// that depends on it.
    ///
    /// # Errors
    ///
    /// Returns [`BkgCaveError::InvalidInput`] if a pipeline references an
    /// unknown build pipeline or if a dependency cycle is detected.
    pub fn depsolve(&self) -> Result<Vec<String>, BkgCaveError> {
        let names: HashSet<&str> = self.pipelines.iter().map(|p| p.name.as_str()).collect();

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialise all pipelines with degree 0.
        for p in &self.pipelines {
            in_degree.entry(p.name.as_str()).or_insert(0);
            adj.entry(p.name.as_str()).or_default();
        }

        // Build dependency edges: if B has build root A, then A must come
        // before B.  Edge direction: A -> B.
        for p in &self.pipelines {
            if let Some(ref build) = p.build {
                if !names.contains(build.name.as_str()) {
                    return Err(BkgCaveError::InvalidInput(format!(
                        "pipeline '{}' depends on unknown build pipeline '{}'",
                        p.name, build.name
                    )));
                }
                adj.entry(build.name.as_str())
                    .or_default()
                    .push(p.name.as_str());
                *in_degree.entry(p.name.as_str()).or_insert(0) += 1;
            }
        }

        // Collect zero-degree nodes.
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        let mut result: Vec<String> = Vec::with_capacity(self.pipelines.len());

        while let Some(name) = queue.pop_front() {
            result.push(name.to_owned());

            if let Some(neighbors) = adj.get(name) {
                for &nb in neighbors {
                    if let Some(deg) = in_degree.get_mut(nb) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(nb);
                        }
                    }
                }
            }
        }

        if result.len() != self.pipelines.len() {
            return Err(BkgCaveError::InvalidInput(
                "cycle detected in pipeline dependencies".into(),
            ));
        }

        Ok(result)
    }

    /// Look up a pipeline by name.
    pub fn get_pipeline(&self, name: &str) -> Option<&Pipeline> {
        self.pipelines.iter().find(|p| p.name == name)
    }

    /// Return the number of pipelines in this manifest.
    pub fn len(&self) -> usize {
        self.pipelines.len()
    }

    /// Whether this manifest is empty.
    pub fn is_empty(&self) -> bool {
        self.pipelines.is_empty()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "build",
                    "stages": []
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        assert_eq!(manifest.pipelines.len(), 1);
        assert_eq!(manifest.pipelines[0].name, "build");
    }

    #[test]
    fn parse_manifest_with_sources() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "build",
                    "stages": [
                        {
                            "name": "org.osbuild.curl",
                            "options": {
                                "url": "https://example.com/file"
                            },
                            "inputs": {},
                            "devices": {},
                            "mounts": []
                        }
                    ]
                }
            ],
            "sources": [
                {
                    "name": "org.osbuild.curl",
                    "items": {
                        "sha256:abc123": {
                            "url": "https://example.com/file"
                        }
                    }
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        assert_eq!(manifest.pipelines.len(), 1);
        assert!(manifest.sources.is_some());
        assert_eq!(manifest.sources.as_ref().unwrap().len(), 1);
        assert_eq!(
            manifest.sources.as_ref().unwrap()[0].name,
            "org.osbuild.curl"
        );
    }

    #[test]
    fn stage_id_is_deterministic() {
        let stage1 = Stage {
            name: "org.osbuild.curl".into(),
            build: None,
            options: {
                let mut m = HashMap::new();
                m.insert(
                    "url".into(),
                    serde_json::Value::String("https://example.com".into()),
                );
                m
            },
            inputs: HashMap::new(),
            devices: HashMap::new(),
            mounts: vec![],
        };
        let stage2 = Stage {
            name: "org.osbuild.curl".into(),
            build: None,
            options: {
                let mut m = HashMap::new();
                m.insert(
                    "url".into(),
                    serde_json::Value::String("https://example.com".into()),
                );
                m
            },
            inputs: HashMap::new(),
            devices: HashMap::new(),
            mounts: vec![],
        };
        assert_eq!(
            stage1.id(),
            stage2.id(),
            "identical stages must produce identical IDs"
        );
    }

    #[test]
    fn stage_id_differs_on_name() {
        let stage1 = Stage {
            name: "org.osbuild.curl".into(),
            build: None,
            options: HashMap::new(),
            inputs: HashMap::new(),
            devices: HashMap::new(),
            mounts: vec![],
        };
        let stage2 = Stage {
            name: "org.osbuild.tar".into(),
            build: None,
            options: HashMap::new(),
            inputs: HashMap::new(),
            devices: HashMap::new(),
            mounts: vec![],
        };
        assert_ne!(
            stage1.id(),
            stage2.id(),
            "different stages must produce different IDs"
        );
    }

    #[test]
    fn depsolve_no_deps() {
        let json = r#"{
            "pipelines": [
                { "name": "a", "stages": [] },
                { "name": "b", "stages": [] }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        let order = manifest.depsolve().unwrap();
        assert_eq!(order.len(), 2);
        assert!(order.contains(&"a".to_string()));
        assert!(order.contains(&"b".to_string()));
    }

    #[test]
    fn depsolve_with_deps() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "build",
                    "stages": []
                },
                {
                    "name": "image",
                    "build": { "name": "build" },
                    "stages": []
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        let order = manifest.depsolve().unwrap();
        assert_eq!(order.len(), 2);
        // "build" must come before "image"
        let pos_build = order.iter().position(|n| n == "build").unwrap();
        let pos_image = order.iter().position(|n| n == "image").unwrap();
        assert!(pos_build < pos_image, "build must come before image");
    }

    #[test]
    fn depsolve_unknown_dep() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "image",
                    "build": { "name": "nonexistent" },
                    "stages": []
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        let result = manifest.depsolve();
        assert!(result.is_err());
    }

    #[test]
    fn depsolve_cycle_detected() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "a",
                    "build": { "name": "b" },
                    "stages": []
                },
                {
                    "name": "b",
                    "build": { "name": "a" },
                    "stages": []
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        let result = manifest.depsolve();
        assert!(result.is_err());
    }

    #[test]
    fn invalid_json_is_rejected() {
        let result = Manifest::from_json("this is not valid JSON");
        assert!(result.is_err());
    }

    #[test]
    fn get_pipeline_by_name() {
        let json = r#"{
            "pipelines": [
                { "name": "build", "stages": [] },
                { "name": "image", "stages": [] }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        assert!(manifest.get_pipeline("build").is_some());
        assert!(manifest.get_pipeline("image").is_some());
        assert!(manifest.get_pipeline("nonexistent").is_none());
    }

    #[test]
    fn stage_with_build_ref() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "build",
                    "stages": [
                        {
                            "name": "org.osbuild.rpm",
                            "build": { "name": "build", "stage": "org.osbuild.rpm" },
                            "options": {},
                            "inputs": {},
                            "devices": {},
                            "mounts": []
                        }
                    ]
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        let stage = &manifest.pipelines[0].stages[0];
        assert!(stage.build.is_some());
        let build_ref = stage.build.as_ref().unwrap();
        assert_eq!(build_ref.name, "build");
        assert_eq!(build_ref.stage.as_deref(), Some("org.osbuild.rpm"));
    }

    #[test]
    fn assembler_roundtrip() {
        let json = r#"{
            "pipelines": [
                {
                    "name": "image",
                    "stages": [],
                    "assembler": {
                        "name": "org.osbuild.qemu",
                        "options": { "format": "qcow2" },
                        "inputs": {},
                        "devices": {},
                        "mounts": []
                    }
                }
            ]
        }"#;
        let manifest = Manifest::from_json(json).unwrap();
        let pipeline = &manifest.pipelines[0];
        assert!(pipeline.assembler.is_some());
        let asm = pipeline.assembler.as_ref().unwrap();
        assert_eq!(asm.name, "org.osbuild.qemu");
        assert_eq!(
            asm.options.get("format").and_then(|v| v.as_str()),
            Some("qcow2")
        );
    }

    #[test]
    fn manifest_len_and_empty() {
        let empty = Manifest {
            pipelines: vec![],
            sources: None,
        };
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let single = Manifest {
            pipelines: vec![Pipeline {
                name: "p".into(),
                runner: None,
                build: None,
                stages: vec![],
                assembler: None,
            }],
            sources: None,
        };
        assert!(!single.is_empty());
        assert_eq!(single.len(), 1);
    }
}
