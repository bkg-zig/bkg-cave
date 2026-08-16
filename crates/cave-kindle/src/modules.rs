//! Module discovery and parameter resolution.
//!
//! Modules are TOML files that define hook functions, parameters, and
//! default values for the initramfs generator.  They live in a well-known
//! directory (e.g. `/usr/share/cave-kindle/modules/` or a user-specified
//! path).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

use crate::config::InitramfsConfig;
use crate::error::KindleError;

/// Parameter type classification.
///
/// Each parameter in a module's TOML definition can be annotated with a
/// type hint.  This determines how the value is validated and processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// A single filesystem path.
    Path,
    /// A list of strings with no duplicates.
    NoDupFlatList,
    /// An integer value.
    Int,
    /// A boolean flag.
    Bool,
    /// A single string.
    String,
    /// A list of strings (duplicates allowed).
    StringList,
}

/// Read and parse a module's TOML definition from disk.
///
/// The file should be a valid TOML document with optional `[param]` and
/// `[hook]` tables.
///
/// # Errors
///
/// Returns `KindleError::Module` if the file cannot be read or parsed.
pub fn read_module(path: &Path) -> Result<toml::Value> {
    debug!(path = %path.display(), "reading module TOML");
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read module: {}", path.display()))?;
    let value: toml::Value = content
        .parse()
        .with_context(|| format!("failed to parse module TOML: {}", path.display()))?;
    Ok(value)
}

/// Discover module TOML files in a base directory.
///
/// Scans `base` for files matching `*.toml` (non-recursive by default).
///
/// # Errors
///
/// Returns an error if `base` is not a directory.
pub fn get_module_paths(base: &Path) -> Result<Vec<PathBuf>> {
    if !base.is_dir() {
        anyhow::bail!(KindleError::Module(format!(
            "modules directory does not exist: {}",
            base.display()
        )));
    }

    let mut paths = Vec::new();
    for entry in std::fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            paths.push(path);
        }
    }

    paths.sort();
    debug!(
        base = %base.display(),
        count = paths.len(),
        "discovered module paths"
    );
    Ok(paths)
}

/// Return the list of parameter names that have been set (or deferred)
/// in the configuration.
pub fn get_parameters(config: &InitramfsConfig) -> Vec<String> {
    config.keys()
}

/// Resolve the [`ParamType`] for a parameter name.
///
/// Uses hard-coded heuristics:
///
/// * Keys ending with `_dir` or `_path` → `Path`
/// * Keys starting with `no_` or ending with `_enable` → `Bool`
/// * Keys ending with `_list` → `NoDupFlatList`
/// * Keys ending with `_level`, `_count`, `_size`, `timeout` → `Int`
/// * All others → `String`
pub fn resolve_type(name: &str) -> ParamType {
    if name.ends_with("_dir") || name.ends_with("_path") || name == "output" {
        ParamType::Path
    } else if name.starts_with("no_")
        || name.ends_with("_enable")
        || name == "debug"
        || name == "dry_run"
    {
        ParamType::Bool
    } else if name.ends_with("_list") {
        ParamType::NoDupFlatList
    } else if name.ends_with("_level")
        || name.ends_with("_count")
        || name.ends_with("_size")
        || name == "timeout"
        || name == "loglevel"
    {
        ParamType::Int
    } else if name.ends_with("_args") || name.ends_with("_opts") {
        ParamType::StringList
    } else {
        ParamType::String
    }
}

/// Validate that a value matches its expected [`ParamType`].
///
/// Returns `Ok(())` if the value is compatible, or a [`KindleError`]
/// describing the mismatch.
pub fn validate_param(name: &str, value: &toml::Value) -> Result<(), KindleError> {
    let ptype = resolve_type(name);
    match ptype {
        ParamType::Path | ParamType::String => {
            if !value.is_str() {
                return Err(KindleError::Validation(format!(
                    "parameter '{name}' expects a string, got {value:?}"
                )));
            }
        }
        ParamType::Bool => {
            if !value.is_bool() {
                return Err(KindleError::Validation(format!(
                    "parameter '{name}' expects a boolean, got {value:?}"
                )));
            }
        }
        ParamType::Int => {
            if !value.is_integer() {
                return Err(KindleError::Validation(format!(
                    "parameter '{name}' expects an integer, got {value:?}"
                )));
            }
        }
        ParamType::NoDupFlatList | ParamType::StringList => {
            if !value.is_array() {
                return Err(KindleError::Validation(format!(
                    "parameter '{name}' expects an array, got {value:?}"
                )));
            }
        }
    }
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_module_valid_toml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        std::fs::write(
            &path,
            r#"
            name = "test-module"
            [param]
            key = "value"
            "#,
        )
        .unwrap();

        let value = read_module(&path).unwrap();
        assert!(value.is_table());
        let table = value.as_table().unwrap();
        assert_eq!(
            table.get("name").and_then(|v| v.as_str()),
            Some("test-module")
        );
    }

    #[test]
    fn read_module_nonexistent() {
        let result = read_module(Path::new("/tmp/does_not_exist_12345.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn get_module_paths_finds_toml_files() {
        let dir = TempDir::new().unwrap();

        // Create some .toml files and a non-.toml file.
        std::fs::write(dir.path().join("base.toml"), b"").unwrap();
        std::fs::write(dir.path().join("network.toml"), b"").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"").unwrap();

        let paths = get_module_paths(dir.path()).unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.ends_with("base.toml")));
        assert!(paths.iter().any(|p| p.ends_with("network.toml")));
    }

    #[test]
    fn get_module_paths_nonexistent_dir() {
        let result = get_module_paths(Path::new("/tmp/does_not_exist_12345"));
        assert!(result.is_err());
    }

    #[test]
    fn get_parameters_returns_keys() {
        let cfg = InitramfsConfig::new();
        let params = get_parameters(&cfg);
        assert!(params.contains(&"compression".into()));
        assert!(params.contains(&"loglevel".into()));
    }

    #[test]
    fn resolve_type_path() {
        assert_eq!(resolve_type("modules_dir"), ParamType::Path);
        assert_eq!(resolve_type("output_path"), ParamType::Path);
        assert_eq!(resolve_type("output"), ParamType::Path);
    }

    #[test]
    fn resolve_type_bool() {
        assert_eq!(resolve_type("no_base"), ParamType::Bool);
        assert_eq!(resolve_type("debug"), ParamType::Bool);
        assert_eq!(resolve_type("compress_enable"), ParamType::Bool);
    }

    #[test]
    fn resolve_type_int() {
        assert_eq!(resolve_type("loglevel"), ParamType::Int);
        assert_eq!(resolve_type("timeout"), ParamType::Int);
        assert_eq!(resolve_type("retry_count"), ParamType::Int);
    }

    #[test]
    fn resolve_type_list() {
        assert_eq!(resolve_type("module_list"), ParamType::NoDupFlatList);
        assert_eq!(resolve_type("kernel_args"), ParamType::StringList);
    }

    #[test]
    fn resolve_type_default_string() {
        assert_eq!(resolve_type("root_device"), ParamType::String);
        assert_eq!(resolve_type("fstype"), ParamType::String);
    }

    #[test]
    fn validate_param_string_accepts_string() {
        assert!(validate_param("root_device", &toml::Value::String("/dev/sda1".into())).is_ok());
    }

    #[test]
    fn validate_param_string_rejects_integer() {
        assert!(validate_param("root_device", &toml::Value::Integer(42)).is_err());
    }

    #[test]
    fn validate_param_bool_accepts_bool() {
        assert!(validate_param("debug", &toml::Value::Boolean(true)).is_ok());
    }

    #[test]
    fn validate_param_bool_rejects_string() {
        assert!(validate_param("debug", &toml::Value::String("true".into())).is_err());
    }

    #[test]
    fn validate_param_int_accepts_int() {
        assert!(validate_param("timeout", &toml::Value::Integer(30)).is_ok());
    }

    #[test]
    fn validate_param_array_accepts_array() {
        let arr = toml::Value::Array(vec!["a".into(), "b".into()]);
        assert!(validate_param("module_list", &arr).is_ok());
    }

    #[test]
    fn validate_param_array_rejects_string() {
        assert!(validate_param("module_list", &toml::Value::String("a".into())).is_err());
    }
}
