//! General-purpose utilities for bkg-cave.

use std::io;
use std::path::{Path, PathBuf};

/// Find a binary by walking the `PATH` environment variable.
///
/// Returns `Some(PathBuf)` with the full path if found, or `None` otherwise.
///
/// # Example
///
/// ```ignore
/// if let Some(path) = find_binary("bash") {
///     println!("bash is at {}", path.display());
/// }
/// ```
pub fn find_binary(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").as_ref().and_then(|path| {
        std::env::split_paths(path).find_map(|dir| {
            let candidate = dir.join(name);
            if candidate.is_file() {
                // On Linux we don't strictly need to check executable,
                // but it's a good sanity check.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = candidate.metadata() {
                        if meta.permissions().mode() & 0o111 != 0 {
                            return Some(candidate);
                        }
                    }
                    None
                }
                #[cfg(not(unix))]
                {
                    Some(candidate)
                }
            } else {
                None
            }
        })
    })
}

/// Quote a kernel argument for use with QEMU's `-append` option.
///
/// If the argument contains whitespace, quotes, or other shell-significant
/// characters it is wrapped in double quotes; embedded double quotes and
/// backslashes are escaped.
pub fn quote_karg(arg: &str) -> String {
    if arg.contains(|c: char| c.is_whitespace() || c == '"' || c == '\\' || c == '\'') {
        let escaped = arg.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    } else {
        arg.to_owned()
    }
}

/// Convert a filesystem path to a kernel argument string.
///
/// Strips a leading `/` if present so that `/root/foo` becomes `root/foo`.
pub fn path_to_karg(path: &Path) -> String {
    let s = path.display().to_string();
    if let Some(stripped) = s.strip_prefix('/') {
        stripped.to_owned()
    } else {
        s
    }
}

/// Recursively copy a directory tree from `src` to `dst`.
///
/// Existing files at `dst` are overwritten.  The `dst` directory is created if
/// it does not exist.
pub fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("source is not a directory: {}", src.display()),
        ));
    }
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let rel = src_path.strip_prefix(src).unwrap_or(&src_path);
        let dst_path = dst.join(rel);

        if file_type.is_dir() {
            copy_recursively(&src_path, &dst_path)?;
        } else if file_type.is_symlink() {
            let target = std::fs::read_link(&src_path)?;
            std::os::unix::fs::symlink(target, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Walk a directory tree and return a list of all regular files underneath it.
///
/// Entries are returned as absolute (or relative, matching the input) paths.
/// Symlinks to files are included; directories and other special entries are
/// skipped.  Errors for individual entries are propagated.
pub fn walk_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("not a directory: {}", dir.display()),
        ));
    }
    _walk_files(dir, &mut files)?;
    Ok(files)
}

fn _walk_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            _walk_files(&path, files)?;
        } else if file_type.is_file() || file_type.is_symlink() {
            files.push(path);
        }
    }
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn find_binary_known() {
        // `sh` should always exist in PATH on any Unix system.
        let result = find_binary("sh");
        assert!(result.is_some(), "expected to find 'sh' in PATH");
        let path = result.unwrap();
        assert!(path.is_file());
    }

    #[test]
    fn find_binary_unknown() {
        assert!(find_binary("this-binary-hopefully-does-not-exist-12345").is_none());
    }

    #[test]
    fn quote_karg_already_safe() {
        assert_eq!(quote_karg("root=/dev/sda1"), "root=/dev/sda1");
    }

    #[test]
    fn quote_karg_with_spaces() {
        let q = quote_karg("console=ttyS0,115200 root=/dev/sda1");
        assert_eq!(q, "\"console=ttyS0,115200 root=/dev/sda1\"");
    }

    #[test]
    fn quote_karg_with_quotes() {
        let q = quote_karg("arg=\"value\"");
        assert_eq!(q, "\"arg=\\\"value\\\"\"");
    }

    #[test]
    fn path_to_karg_strips_leading_slash() {
        assert_eq!(path_to_karg(Path::new("/root/foo")), "root/foo");
    }

    #[test]
    fn path_to_karg_relative_unchanged() {
        assert_eq!(path_to_karg(Path::new("root/foo")), "root/foo");
    }

    #[test]
    fn copy_recursively_works() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        // Create a small tree
        fs::write(src.path().join("a.txt"), b"hello").unwrap();
        fs::create_dir(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/b.txt"), b"world").unwrap();

        let dst_root = dst.path().join("copied");
        copy_recursively(src.path(), &dst_root).unwrap();

        assert!(dst_root.join("a.txt").is_file());
        assert!(dst_root.join("sub/b.txt").is_file());
        assert_eq!(fs::read_to_string(dst_root.join("a.txt")).unwrap(), "hello");
    }

    #[test]
    fn walk_files_works() {
        let dir = TempDir::new().unwrap();

        fs::write(dir.path().join("f1.txt"), b"").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/f2.txt"), b"").unwrap();
        fs::create_dir(dir.path().join("sub/empty")).unwrap();

        let files = walk_files(dir.path()).unwrap();
        // Collect just the file names (relative to dir)
        let names: Vec<String> = files
            .iter()
            .map(|p| p.strip_prefix(dir.path()).unwrap().display().to_string())
            .collect();

        assert!(names.contains(&"f1.txt".into()));
        assert!(names.contains(&"sub/f2.txt".into()));
        // Directories should NOT be in the list
        assert!(!names.contains(&"sub".into()));
        assert!(!names.contains(&"sub/empty".into()));
    }

    #[test]
    fn copy_recursively_source_not_dir() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("file.txt");
        let out = tmp.path().join("out");
        fs::write(&file, b"").unwrap();

        let result = copy_recursively(&file, &out);
        assert!(result.is_err());
    }

    #[test]
    fn walk_files_source_not_dir() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("file.txt");
        fs::write(&file, b"").unwrap();

        let result = walk_files(&file);
        assert!(result.is_err());
    }
}
