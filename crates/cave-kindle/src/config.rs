//! Initramfs configuration management.
//!
//! [`InitramfsConfig`] holds the user-supplied parameters as well as
//! per-module configuration.  It supports a deferred-processing mechanism so
//! that parameters can be set before their values are known, then finalised
//! once all modules have been loaded.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::error::KindleError;

/// The lifecycle stage of configuration processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigStage {
    /// Early — before modules have been loaded.
    Early,
    /// Late — after modules have been loaded but before validation.
    Late,
    /// Final — configuration is locked and validated.
    Final,
}

/// Holds all initramfs-related configuration values.
///
/// # Layering
///
/// 1. **Built-in parameters** — known keys that have special handling
///    (e.g. `"compression"`, `"loglevel"`).
/// 2. **User-supplied parameters** — from the TOML config file.
/// 3. **Module-supplied parameters** — from each module's `<module>.toml`.
///
/// Values set later override earlier ones.  Deferred values are replayed
/// when `process_deferred` is called.
#[derive(Debug, Clone)]
pub struct InitramfsConfig {
    /// Flat key → value mapping of all resolved parameters.
    params: HashMap<String, toml::Value>,

    /// Unprocessed / deferred per-key queues.
    processing: HashMap<String, VecDeque<toml::Value>>,

    /// Keys that are recognised as built-in.
    builtin_params: HashSet<String>,

    /// Current processing stage.
    stage: ConfigStage,

    /// Whether `validate()` has been called successfully.
    validated: bool,
}

impl Default for InitramfsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl InitramfsConfig {
    /// Create a new empty configuration.
    ///
    /// Built-in parameters are pre-populated with sensible defaults.
    pub fn new() -> Self {
        let mut builtin_params = HashSet::new();
        for key in Self::builtin_keys() {
            builtin_params.insert(key.to_string());
        }

        let mut params = HashMap::new();
        params.insert("compression".into(), toml::Value::String("xz".into()));
        params.insert("loglevel".into(), toml::Value::Integer(3));
        params.insert("timeout".into(), toml::Value::Integer(30));
        params.insert("no_base".into(), toml::Value::Boolean(false));

        Self {
            params,
            processing: HashMap::new(),
            builtin_params,
            stage: ConfigStage::Early,
            validated: false,
        }
    }

    /// Return the list of recognised built-in parameter names.
    fn builtin_keys() -> &'static [&'static str] {
        &[
            "compression",
            "loglevel",
            "timeout",
            "no_base",
            "compress",
            "output",
            "debug",
            "dry_run",
            "modules_dir",
            "include_binary",
            "include_dir",
            "include_file",
        ]
    }

    /// Load configuration from a parsed TOML value (typically the user's
    /// main config file).
    ///
    /// Values are deep-merged into the existing configuration.
    pub fn load_toml(&mut self, toml_value: &toml::Value) -> Result<()> {
        match toml_value {
            toml::Value::Table(table) => {
                for (key, value) in table {
                    self.set(key, value.clone());
                }
            }
            other => {
                // Non-table TOML — store under a synthetic root key.
                self.set("_root", other.clone());
            }
        }
        Ok(())
    }

    /// Load module-specific configuration.
    ///
    /// Module TOML files can declare parameters, defaults, and hook
    /// overrides.  This method processes them and places unrecognised
    /// keys into the deferred queue.
    pub fn load_module(&mut self, module_toml: &toml::Value) -> Result<()> {
        let table = module_toml
            .as_table()
            .context("module TOML must be a table")?;

        for (key, value) in table {
            if self.builtin_params.contains(key) {
                // Override builtin if already set.
                self.set(key, value.clone());
            } else if self.params.contains_key(key) {
                // Already known — override.
                self.set(key, value.clone());
            } else {
                // Unknown parameter — defer.
                self.processing
                    .entry(key.clone())
                    .or_default()
                    .push_back(value.clone());
            }
        }

        Ok(())
    }

    /// Retrieve a deserialised configuration value by key.
    ///
    /// Returns `Ok(None)` if the key does not exist.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.params.get(key) {
            Some(value) => {
                let deserialized = value.clone().try_into::<T>().map_err(|e| {
                    KindleError::Config(format!("failed to deserialize '{key}': {e}"))
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Set a configuration value.
    ///
    /// This overwrites any existing value for the key.
    pub fn set(&mut self, key: &str, value: toml::Value) {
        debug!(key = %key, value = ?value, "InitramfsConfig::set");
        self.params.insert(key.to_owned(), value);
    }

    /// Check whether a key exists.
    pub fn has(&self, key: &str) -> bool {
        self.params.contains_key(key) || self.processing.contains_key(key)
    }

    /// Return all known parameter keys (both resolved and deferred).
    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.params.keys().cloned().collect();
        for k in self.processing.keys() {
            if !self.params.contains_key(k) {
                keys.push(k.clone());
            }
        }
        keys.sort();
        keys
    }

    /// Process all deferred values.
    ///
    /// For each deferred key, if a corresponding built-in or already-known
    /// parameter exists the deferred value is applied (last wins).  After
    /// this call the deferred queue is cleared.
    pub fn process_deferred(&mut self) -> Result<()> {
        let pending: Vec<(String, VecDeque<toml::Value>)> = self.processing.drain().collect();
        for (key, mut queue) in pending {
            // Apply queued values in order; last wins.
            while let Some(value) = queue.pop_front() {
                if self.builtin_params.contains(&key) {
                    self.params.insert(key.clone(), value);
                } else {
                    // Still not recognised — keep as a regular param.
                    self.params.insert(key.clone(), value);
                }
            }
        }
        Ok(())
    }

    /// Advance the configuration stage.
    pub fn set_stage(&mut self, stage: ConfigStage) {
        self.stage = stage;
    }

    /// Return the current stage.
    pub fn stage(&self) -> ConfigStage {
        self.stage
    }

    /// Validate the configuration.
    ///
    /// Checks that required keys exist and have sensible values.
    pub fn validate(&mut self) -> Result<()> {
        // Check compression is one of the recognised formats.
        if let Some(comp) = self.params.get("compression").and_then(|v| v.as_str()) {
            match comp {
                "none" | "gzip" | "xz" | "zstd" | "lz4" | "lzo" | "bzip2" => {}
                other => {
                    anyhow::bail!(KindleError::Validation(format!(
                        "unknown compression format: '{other}'"
                    )));
                }
            }
        }

        // Log level must be 0–7 (syslog levels).
        if let Some(level) = self.params.get("loglevel").and_then(|v| v.as_integer()) {
            if !(0..=7).contains(&level) {
                anyhow::bail!(KindleError::Validation(format!(
                    "loglevel must be 0–7, got {level}"
                )));
            }
        }

        self.validated = true;
        Ok(())
    }

    /// Whether validation has completed successfully.
    pub fn is_validated(&self) -> bool {
        self.validated
    }

    // ── internal helpers ────────────────────────────────────────────────────

    /// Return a reference to the underlying parameter map (for tests / hooks).
    pub fn raw_params(&self) -> &HashMap<String, toml::Value> {
        &self.params
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_has_defaults() {
        let cfg = InitramfsConfig::new();
        assert!(cfg.has("compression"));
        assert!(cfg.has("loglevel"));
        assert!(cfg.has("timeout"));
        assert!(cfg.has("no_base"));
        assert_eq!(cfg.stage(), ConfigStage::Early);
        assert!(!cfg.is_validated());
    }

    #[test]
    fn set_and_get_string() {
        let mut cfg = InitramfsConfig::new();
        cfg.set("output", toml::Value::String("/tmp/initrd.cpio".into()));
        let val: Option<String> = cfg.get("output").unwrap();
        assert_eq!(val, Some("/tmp/initrd.cpio".into()));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let cfg = InitramfsConfig::new();
        let val: Option<String> = cfg.get("does_not_exist").unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn load_toml_from_table() {
        let toml_str = r#"
            compression = "zstd"
            loglevel = 5
            custom_key = "hello"
        "#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let mut cfg = InitramfsConfig::new();
        cfg.load_toml(&value).unwrap();

        let comp: Option<String> = cfg.get("compression").unwrap();
        assert_eq!(comp, Some("zstd".into()));

        let custom: Option<String> = cfg.get("custom_key").unwrap();
        assert_eq!(custom, Some("hello".into()));
    }

    #[test]
    fn load_module_deferred_key() {
        let mut cfg = InitramfsConfig::new();

        let module_toml: toml::Value = toml::from_str(
            r#"
            some_module_key = 42
            "#,
        )
        .unwrap();

        cfg.load_module(&module_toml).unwrap();
        // The key was deferred since it's not builtin.
        // It should appear in keys().
        assert!(cfg.keys().contains(&"some_module_key".into()));
    }
    #[test]
    fn process_deferred_resolves_unknown_key() {
        let mut cfg = InitramfsConfig::new();

        // Load a module with an unknown key that should be deferred.
        let module_toml: toml::Value = toml::from_str(r#"custom_module_key = "hello""#).unwrap();
        cfg.load_module(&module_toml).unwrap();

        // Unknown keys are deferred — they don't appear in get() yet.
        let val: Option<String> = cfg.get("custom_module_key").unwrap();
        assert!(val.is_none());

        cfg.process_deferred().unwrap();

        // After processing, the deferred value should be available.
        let val: Option<String> = cfg.get("custom_module_key").unwrap();
        assert_eq!(val, Some("hello".into()));
    }

    #[test]
    fn load_module_overrides_builtin_immediately() {
        let mut cfg = InitramfsConfig::new();

        // Builtin keys are set immediately by load_module.
        let module_toml: toml::Value = toml::from_str(r#"compression = "lz4""#).unwrap();
        cfg.load_module(&module_toml).unwrap();

        // Since "compression" is a builtin, load_module sets it immediately.
        let comp: Option<String> = cfg.get("compression").unwrap();
        assert_eq!(comp, Some("lz4".into()));
    }

    #[test]
    fn validate_accepts_known_compression() {
        let mut cfg = InitramfsConfig::new();
        cfg.set("compression", toml::Value::String("zstd".into()));
        assert!(cfg.validate().is_ok());
        assert!(cfg.is_validated());
    }

    #[test]
    fn validate_rejects_unknown_compression() {
        let mut cfg = InitramfsConfig::new();
        cfg.set("compression", toml::Value::String("rar".into()));
        let err = cfg.validate();
        assert!(err.is_err());
        assert!(!cfg.is_validated());
    }

    #[test]
    fn validate_rejects_bad_loglevel() {
        let mut cfg = InitramfsConfig::new();
        cfg.set("loglevel", toml::Value::Integer(99));
        let err = cfg.validate();
        assert!(err.is_err());
    }

    #[test]
    fn validate_accepts_loglevel_0_to_7() {
        for level in 0..=7 {
            let mut cfg = InitramfsConfig::new();
            cfg.set("loglevel", toml::Value::Integer(level));
            assert!(cfg.validate().is_ok(), "loglevel {level} should be valid");
        }
    }

    #[test]
    fn keys_returns_sorted() {
        let cfg = InitramfsConfig::new();
        let keys = cfg.keys();
        // Should be sorted.
        for w in keys.windows(2) {
            assert!(w[0] <= w[1], "keys should be sorted");
        }
    }

    #[test]
    fn set_stage_transitions() {
        let mut cfg = InitramfsConfig::new();
        assert_eq!(cfg.stage(), ConfigStage::Early);
        cfg.set_stage(ConfigStage::Late);
        assert_eq!(cfg.stage(), ConfigStage::Late);
        cfg.set_stage(ConfigStage::Final);
        assert_eq!(cfg.stage(), ConfigStage::Final);
    }
}
