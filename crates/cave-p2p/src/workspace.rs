use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub auto_save: bool,
    pub max_history: usize,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            auto_save: true,
            max_history: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub path: PathBuf,
    pub config: WorkspaceConfig,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            config: WorkspaceConfig::default(),
        }
    }

    pub fn open(path: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&path);
        let config = Self::load_workspace_config(&path);
        Self { path, config }
    }

    fn load_workspace_config(path: &std::path::Path) -> WorkspaceConfig {
        let config_path = path.join("workspace.toml");
        if let Ok(data) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<WorkspaceConfigSerializable>(&data) {
                return WorkspaceConfig {
                    auto_save: config.auto_save,
                    max_history: config.max_history,
                };
            }
        }
        WorkspaceConfig::default()
    }
}

#[derive(serde::Deserialize)]
struct WorkspaceConfigSerializable {
    #[serde(default = "default_true")]
    auto_save: bool,
    #[serde(default = "default_history")]
    max_history: usize,
}

fn default_true() -> bool {
    true
}

fn default_history() -> usize {
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workspace_uses_default_config() {
        let ws = Workspace::new(PathBuf::from("/tmp/test-ws"));
        assert!(ws.config.auto_save);
        assert_eq!(ws.config.max_history, 100);
    }

    #[test]
    fn open_creates_directory() {
        let tmp = std::env::temp_dir().join("bkg-test-workspace");
        let _ = std::fs::remove_dir_all(&tmp);
        let ws = Workspace::open(tmp.clone());
        assert!(tmp.exists());
        assert!(ws.config.auto_save);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_config_from_file() {
        let tmp = std::env::temp_dir().join("bkg-test-ws-config");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("workspace.toml"),
            "auto_save = false\nmax_history = 50\n",
        )
        .unwrap();
        let ws = Workspace::open(tmp.clone());
        assert!(!ws.config.auto_save);
        assert_eq!(ws.config.max_history, 50);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn missing_config_uses_defaults() {
        let tmp = std::env::temp_dir().join("bkg-test-ws-missing");
        let _ = std::fs::remove_dir_all(&tmp);
        let ws = Workspace::open(tmp.clone());
        assert!(ws.config.auto_save);
        assert_eq!(ws.config.max_history, 100);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn default_config_values() {
        let config = WorkspaceConfig::default();
        assert!(config.auto_save);
        assert_eq!(config.max_history, 100);
    }
}
