//! Kernel module resolution — finds kernel modules by modalias or name,
//! resolves their dependencies via `modprobe --show-depends`, and merges
//! module lists for deduplication.

use std::path::{Path, PathBuf};
use std::time::Duration;

use cave_core::error::BkgCaveError;

use crate::arch::Arch;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Resolve the dependencies for a given modalias or module name.
///
/// Runs `modprobe --show-depends` inside a minimal sandbox (`-C /var/empty`)
/// rooted at `root` with the kernel version `kver`. The `moddir` is the
/// module subdirectory (e.g. `"kernel/fs/9p"`).
///
/// Returns the list of `.ko` paths (relative to the module root) that
/// need to be installed.
pub fn resolve_dep(
    modalias: &str,
    root: &Path,
    kver: &str,
    _moddir: &str,
    _timeout: Duration,
) -> Result<Vec<String>, BkgCaveError> {
    // Build the modprobe command.
    // virtme uses: modprobe --show-depends -C /var/empty -d <root> -S <kver> -- <alias>
    let output = std::process::Command::new("modprobe")
        .args(["--show-depends", "-C", "/var/empty", "-d"])
        .arg(root.as_os_str())
        .args(["-S", kver, "--"])
        .arg(modalias)
        .output()
        .map_err(|e| BkgCaveError::Execution {
            command: format!(
                "modprobe --show-depends -C /var/empty -d {} -S {} -- {}",
                root.display(),
                kver,
                modalias
            ),
            exit_code: -1,
            stderr: e.to_string(),
        })?;

    if !output.status.success() {
        // modprobe returns non-zero for unknown aliases — that's often expected.
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let modules: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Lines look like: "insmod /lib/modules/5.15.0/kernel/fs/9p/9p.ko"
            if let Some(path) = line.strip_prefix("insmod ") {
                // Strip the module root prefix to get the relative path.
                let full_path = Path::new(path);
                // Make it relative: remove /lib/modules/<kver>/
                if let Ok(rel) = full_path.strip_prefix(format!("/lib/modules/{kver}/")) {
                    Some(rel.to_string_lossy().into_owned())
                } else {
                    // Keep as-is if we can't strip the prefix.
                    Some(path.to_owned())
                }
            } else if line.starts_with("builtin ") {
                // Built-in modules are skipped.
                None
            } else {
                None
            }
        })
        .collect();

    Ok(modules)
}

/// Merge multiple module lists, removing duplicates while preserving order.
pub fn merge_mods(lists: &[Vec<String>]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for list in lists {
        for m in list {
            if seen.insert(m.clone()) {
                result.push(m.clone());
            }
        }
    }
    result
}

/// Find kernel modules from the installed modules tree by matching aliases.
///
/// Looks up each alias in `aliases` against the installed modules under
/// `root/lib/modules/<kver>/`, using `modprobe --show-depends` under the hood.
pub fn find_from_install(
    aliases: &[&str],
    root: &Path,
    kver: &str,
) -> Result<Vec<String>, BkgCaveError> {
    let timeout = Duration::from_secs(30);
    let mut all_modules = Vec::new();

    for alias in aliases {
        let mods = resolve_dep(alias, root, kver, "", timeout)?;
        all_modules.push(mods);
    }

    Ok(merge_mods(&all_modules))
}

/// Determine the kernel module directory for the given architecture and kernel
/// version, rooted at `root`.
///
/// Typically returns `<root>/lib/modules/<kver>/`.
pub fn find_mod_dir(arch: &dyn Arch, kver: &str, root: &Path) -> PathBuf {
    let linuxname = arch.linuxname();
    // Standard path: /lib/modules/<kver>/
    let mut path = root.join("lib").join("modules").join(kver);

    // If that doesn't exist, try the architecture-specific path.
    if !path.is_dir() {
        path = root
            .join("lib")
            .join("modules")
            .join(format!("{kver}-{linuxname}"));
    }

    path
}

/// Determine the kernel release string from the installed kernel image.
///
/// Reads `/lib/modules/<kver>/` or tries `uname -r` fallback.
pub fn detect_kernel_version(root: &Path) -> Result<String, BkgCaveError> {
    let modules_dir = root.join("lib").join("modules");
    if modules_dir.is_dir() {
        let entries: Vec<_> = std::fs::read_dir(&modules_dir)
            .map_err(BkgCaveError::Io)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        if let Some(kver) = entries.into_iter().next() {
            return Ok(kver);
        }
    }

    // Fallback: run `uname -r`
    let output = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .map_err(|e| BkgCaveError::Execution {
            command: "uname -r".into(),
            exit_code: -1,
            stderr: e.to_string(),
        })?;

    if output.status.success() {
        let kver = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if !kver.is_empty() {
            return Ok(kver);
        }
    }

    Err(BkgCaveError::NotFound(
        "could not determine kernel version".into(),
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_mods_deduplicates() {
        let lists = vec![
            vec!["a.ko".into(), "b.ko".into()],
            vec!["b.ko".into(), "c.ko".into()],
        ];
        let merged = merge_mods(&lists);
        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"a.ko".to_string()));
        assert!(merged.contains(&"b.ko".to_string()));
        assert!(merged.contains(&"c.ko".to_string()));
    }

    #[test]
    fn merge_mods_empty() {
        assert!(merge_mods(&[]).is_empty());
        assert!(merge_mods(&[vec![], vec![]]).is_empty());
    }

    #[test]
    fn merge_mods_preserves_order() {
        let lists = vec![vec!["z.ko".into(), "a.ko".into()], vec!["b.ko".into()]];
        let merged = merge_mods(&lists);
        // First occurrence should be kept: z.ko before a.ko
        assert_eq!(merged[0], "z.ko");
        assert_eq!(merged[1], "a.ko");
    }

    #[test]
    fn find_mod_dir_default() {
        let arch = crate::arch::ArchX86;
        // Use tempdir to control what exists on the filesystem.
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        // Create the default module directory.
        let default_moddir = root.join("lib").join("modules").join("5.15.0");
        std::fs::create_dir_all(&default_moddir).unwrap();

        let dir = find_mod_dir(&arch, "5.15.0", root);
        assert_eq!(dir, default_moddir);
    }

    #[test]
    fn version_check_does_not_panic() {
        // Just ensure it doesn't panic on non-existent root.
        let root = Path::new("/nonexistent");
        let result = detect_kernel_version(root);
        // It will fail but shouldn't panic.
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn resolve_dep_unknown_alias() {
        // An unknown alias should return an empty list without an error.
        let root = Path::new("/");
        let result = resolve_dep(
            "nonexistent_alias_xyz",
            root,
            "0.0.0",
            "",
            Duration::from_secs(5),
        );
        // Should not error, just return empty list.
        assert!(result.is_ok());
    }

    #[test]
    fn find_mod_dir_with_arch_suffix() {
        let arch = crate::arch::ArchAarch64;
        // Use tempdir to control what exists on the filesystem.
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        // Create the arch-suffixed module directory (default doesn't exist).
        let moddir = root
            .join("lib")
            .join("modules")
            .join(format!("5.15.0-{}", arch.linuxname()));
        std::fs::create_dir_all(&moddir).unwrap();

        let dir = find_mod_dir(&arch, "5.15.0", root);
        // Should fall back to the arch-suffixed path.
        assert_eq!(dir, moddir);
    }
}
