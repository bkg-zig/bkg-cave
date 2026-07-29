//! Simple configuration registry backed by TOML values.
//!
//! [`ConfigRegistry`] provides a flat, key-oriented view over a set of TOML
//! documents.  It is intentionally simple — no schema, no validation — and
//! is meant to be wrapped by higher-level configuration logic.

use std::collections::HashMap;
use std::path::Path;

use crate::error::BkgCaveError;

/// A key-value configuration registry backed by [`toml::Value`].
///
/// Keys use a dotted-path convention (`"build.kernel.args"`) for accessing
/// nested tables.
///
/// # Example
///
/// ```ignore
/// use cave_core::ConfigRegistry;
///
/// let mut reg = ConfigRegistry::new();
/// reg.load_file("/path/to/config.toml")?;
///
/// if let Some(val) = reg.get_str("build.kernel") {
///     println!("kernel: {val}");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConfigRegistry {
    /// The merged configuration tree (a single `toml::Value::Table`).
    config: HashMap<String, toml::Value>,
    /// Optional path from which the config was last loaded.
    config_path: Option<std::path::PathBuf>,
}

impl ConfigRegistry {
    /// Create a new, empty configuration registry.
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            config_path: None,
        }
    }

    /// Load a TOML file and deep-merge it into the current configuration.
    ///
    /// Existing keys are **overwritten** by the loaded file.  Tables are
    /// merged recursively.
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), BkgCaveError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(BkgCaveError::Io)?;
        let value: toml::Value = content.parse::<toml::Value>()?;

        self.merge(&value);
        self.config_path = Some(path.to_owned());
        Ok(())
    }

    /// Deep-merge a `toml::Value` into the current config.
    fn merge(&mut self, value: &toml::Value) {
        match value {
            toml::Value::Table(table) => {
                for (key, val) in table {
                    match val {
                        toml::Value::Table(sub) => {
                            let entry = self
                                .config
                                .entry(key.clone())
                                .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
                            if let toml::Value::Table(ref mut existing) = entry {
                                for (sk, sv) in sub {
                                    existing.insert(sk.clone(), sv.clone());
                                }
                            }
                        }
                        other => {
                            self.config.insert(key.clone(), other.clone());
                        }
                    }
                }
            }
            other => {
                // Not a table — replace entire config?  For now treat as
                // setting a single key "_root".
                self.config.insert("_root".into(), other.clone());
            }
        }
    }

    /// Get a string value by key.
    ///
    /// Returns `None` if the key is absent or the value is not a string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.config.get(key).and_then(|v| v.as_str())
    }

    /// Get a boolean value by key.
    ///
    /// Returns `None` if the key is absent or the value is not a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.config.get(key).and_then(|v| v.as_bool())
    }

    /// Get a `u64` value by key.
    ///
    /// Returns `None` if the key is absent or the value is not an integer.
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.config
            .get(key)
            .and_then(|v| v.as_integer())
            .map(|i| i as u64)
    }

    /// Get the inner TOML table for a key.
    ///
    /// Returns `None` if the key is absent or the value is not a table.
    pub fn get_table(&self, key: &str) -> Option<&toml::value::Table> {
        self.config.get(key).and_then(|v| v.as_table())
    }

    /// Set a key to an arbitrary `toml::Value`.
    ///
    /// If the key already exists it is overwritten.
    pub fn set(&mut self, key: &str, value: toml::Value) {
        self.config.insert(key.to_owned(), value);
    }

    /// Check whether a key exists in the registry.
    pub fn has(&self, key: &str) -> bool {
        self.config.contains_key(key)
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn empty_registry() {
        let reg = ConfigRegistry::new();
        assert!(!reg.has("anything"));
        assert!(reg.get_str("x").is_none());
    }

    #[test]
    fn set_and_get_str() {
        let mut reg = ConfigRegistry::new();
        reg.set("name", toml::Value::String("hello".into()));
        assert!(reg.has("name"));
        assert_eq!(reg.get_str("name"), Some("hello"));
    }

    #[test]
    fn set_and_get_bool() {
        let mut reg = ConfigRegistry::new();
        reg.set("enabled", toml::Value::Boolean(true));
        assert_eq!(reg.get_bool("enabled"), Some(true));
    }

    #[test]
    fn set_and_get_u64() {
        let mut reg = ConfigRegistry::new();
        reg.set("count", toml::Value::Integer(42));
        assert_eq!(reg.get_u64("count"), Some(42));
    }

    #[test]
    fn set_and_get_table() {
        let mut reg = ConfigRegistry::new();
        let mut table = toml::value::Table::new();
        table.insert("inner".into(), toml::Value::String("val".into()));
        reg.set("outer", toml::Value::Table(table));
        let t = reg.get_table("outer");
        assert!(t.is_some());
        assert_eq!(
            t.unwrap().get("inner").and_then(|v| v.as_str()),
            Some("val")
        );
    }

    #[test]
    fn load_file_merges() {
        let dir = TempDir::new().unwrap();
        let cfg_path = dir.path().join("test.toml");
        std::fs::write(
            &cfg_path,
            r#"
            key1 = "value1"
            [section]
            inner = 99
            "#,
        )
        .unwrap();

        let mut reg = ConfigRegistry::new();
        reg.load_file(&cfg_path).unwrap();

        assert_eq!(reg.get_str("key1"), Some("value1"));
        // "inner" is nested inside [section] — access via get_table
        let tbl = reg.get_table("section").expect("section table exists");
        assert_eq!(tbl.get("inner").and_then(|v| v.as_integer()), Some(99));
    }

    #[test]
    fn load_file_nonexistent() {
        let mut reg = ConfigRegistry::new();
        let result = reg.load_file("/tmp/does_not_exist_12345.toml");
        assert!(result.is_err());
    }

    #[test]
    fn load_file_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let cfg_path = dir.path().join("bad.toml");
        std::fs::write(&cfg_path, "[[[broken]]").unwrap();

        let mut reg = ConfigRegistry::new();
        let result = reg.load_file(&cfg_path);
        assert!(result.is_err());
    }
}
