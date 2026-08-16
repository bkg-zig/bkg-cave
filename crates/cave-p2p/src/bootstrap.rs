use std::path::PathBuf;

const APP_NAME: &str = "bkg-cave";

fn base_data_dir() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join(APP_NAME)
    } else {
        PathBuf::from(".").join(format!(".local/share/{APP_NAME}"))
    }
}

pub fn base_dir() -> PathBuf {
    base_data_dir()
}

pub fn data_dir() -> PathBuf {
    base_dir().join("data")
}

pub fn models_dir() -> PathBuf {
    base_dir().join("models")
}

pub fn skills_dir() -> PathBuf {
    base_dir().join("skills")
}

pub fn tools_dir() -> PathBuf {
    base_dir().join("tools")
}

pub fn prompts_dir() -> PathBuf {
    base_dir().join("prompts")
}

pub fn identity_path() -> PathBuf {
    base_dir().join("identity.json")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir())?;
    std::fs::create_dir_all(models_dir())?;
    std::fs::create_dir_all(skills_dir())?;
    std::fs::create_dir_all(tools_dir())?;
    std::fs::create_dir_all(prompts_dir())?;
    Ok(())
}

pub fn ensure_dirs_in(base: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(base.join("data"))?;
    std::fs::create_dir_all(base.join("models"))?;
    std::fs::create_dir_all(base.join("skills"))?;
    std::fs::create_dir_all(base.join("tools"))?;
    std::fs::create_dir_all(base.join("prompts"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_absolute() {
        assert!(base_dir().is_absolute());
    }

    #[test]
    fn data_dir_is_child_of_base() {
        assert!(data_dir().starts_with(base_dir()));
    }

    #[test]
    fn models_dir_is_child_of_base() {
        assert!(models_dir().starts_with(base_dir()));
    }

    #[test]
    fn skills_dir_is_child_of_base() {
        assert!(skills_dir().starts_with(base_dir()));
    }

    #[test]
    fn identity_path_ends_with_json() {
        assert_eq!(identity_path().extension().unwrap(), "json");
    }

    #[test]
    fn ensure_dirs_in_temp() {
        let tmp = std::env::temp_dir().join("bkg-test-bootstrap");
        let _ = std::fs::remove_dir_all(&tmp);
        let result = ensure_dirs_in(&tmp);
        assert!(result.is_ok());
        assert!(tmp.join("data").exists());
        assert!(tmp.join("models").exists());
        assert!(tmp.join("skills").exists());
        assert!(tmp.join("tools").exists());
        assert!(tmp.join("prompts").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
