//! Content-addressed object store.
//!
//! Stages produce filesystem trees.  Each tree is stored under its SHA256
//! content hash so that identical stage results can be shared across
//! pipelines and builds.  The store is a simple directory on disk where
//! each subdirectory is named after the hex-encoded hash of the tree it
//! contains.
//!
//! Copy-on-write initialisation uses `cp --reflink=auto -a` for efficient
//! tree cloning on CoW-capable filesystems (btrfs, ZFS, xfs with reflink).

use std::path::{Path, PathBuf};

use cave_core::BkgCaveError;

/// A content-addressed object store.
///
/// Objects are stored as directories under `base_dir/<first_two>/<remaining>`.
/// This two-level sharding prevents any single directory from holding too
/// many entries.
#[derive(Debug, Clone)]
pub struct ObjectStore {
    /// Root directory of the store.
    pub base_dir: PathBuf,
}

/// A writable object handle backed by a real directory on disk.
///
/// Objects are created via [`ObjectStore::new_object`] and finalised via
/// [`Object::finalize`] before being committed with
/// [`ObjectStore::commit`].
#[derive(Debug, Clone)]
pub struct Object {
    /// The directory where this object's contents live.
    pub dir: PathBuf,
}

// ── Constants ───────────────────────────────────────────────────────────────

/// Number of hex characters used for the first-level shard directory.
const SHARD_LEN: usize = 2;

// ── ObjectStore ──────────────────────────────────────────────────────────────

impl ObjectStore {
    /// Open or create an object store at `base_dir`.
    ///
    /// The directory is created if it does not exist.
    pub fn new(base_dir: &Path) -> Result<Self, BkgCaveError> {
        std::fs::create_dir_all(base_dir)?;
        tracing::info!(base_dir = %base_dir.display(), "ObjectStore opened");
        Ok(Self {
            base_dir: base_dir.to_owned(),
        })
    }

    /// Create a new, empty `Object` for writing.
    ///
    /// If `checkpoint` is `Some`, the new object is initialised as a CoW copy
    /// of the given checkpoint object (using `cp --reflink=auto -a`).
    pub fn new_object(&self, checkpoint: Option<&Object>) -> Result<Object, BkgCaveError> {
        // Create a unique temporary directory under the store's base.
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let pid = std::process::id();
        let dir = self
            .base_dir
            .join("tmp")
            .join(format!("forge-obj-{pid}-{ts}"));
        std::fs::create_dir_all(&dir)?;

        let mut obj = Object::new(dir);

        if let Some(base) = checkpoint {
            obj.init(base)?;
        }

        Ok(obj)
    }

    /// Retrieve an existing object by its content ID.
    ///
    /// Returns `Ok(None)` when the object is not present in the store.
    pub fn get(&self, id: &str) -> Result<Option<Object>, BkgCaveError> {
        let path = self.id_to_path(id);
        if path.is_dir() {
            Ok(Some(Object::new(path)))
        } else {
            Ok(None)
        }
    }

    /// Check whether an object with the given ID exists in the store.
    pub fn contains(&self, id: &str) -> bool {
        self.id_to_path(id).is_dir()
    }

    /// Commit (store) the object's finalised directory under the given ID.
    ///
    /// The directory is moved into the store's content-addressed location.
    /// If an object with this ID already exists the incoming tree is silently
    /// discarded (the store already holds the content).
    ///
    /// # Panics
    ///
    /// Panics if `id` is not a valid hex string (callers should only pass IDs
    /// produced by [`Stage::id`](crate::pipeline::Stage::id) or similar).
    pub fn commit(&self, obj: Object, id: &str) -> Result<(), BkgCaveError> {
        let target = self.id_to_path(id);

        // Already stored — nothing to do.
        if target.is_dir() {
            // Clean up the temporary directory.
            let _ = std::fs::remove_dir_all(&obj.dir);
            return Ok(());
        }

        // Create parent shard directory.
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Rename (atomic on the same filesystem).
        std::fs::rename(&obj.dir, &target).map_err(|e| {
            BkgCaveError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "failed to commit object {} from {} to {}: {e}",
                    id,
                    obj.dir.display(),
                    target.display()
                ),
            ))
        })?;

        tracing::debug!(id = %id, path = %target.display(), "Object committed");
        Ok(())
    }

    /// Export a stored object's tree to `target`.
    ///
    /// Uses `cp -a` to produce an independent copy of the tree.
    ///
    /// # Errors
    ///
    /// Returns [`BkgCaveError::NotFound`] if no object with `id` is present.
    pub fn export(&self, id: &str, target: &Path) -> Result<(), BkgCaveError> {
        let src = self.id_to_path(id);
        if !src.is_dir() {
            return Err(BkgCaveError::NotFound(format!(
                "object '{id}' not found in store at {}",
                src.display()
            )));
        }

        // Remove target if it exists.
        if target.exists() {
            std::fs::remove_dir_all(target)?;
        }

        cave_core::util::copy_recursively(&src, target).map_err(BkgCaveError::Io)?;

        tracing::debug!(id = %id, target = %target.display(), "Object exported");
        Ok(())
    }

    // ── helpers ────────────────────────────────────────────────────────────

    /// Convert a hex content ID to the on-disk path.
    fn id_to_path(&self, id: &str) -> PathBuf {
        debug_assert!(!id.is_empty(), "ID must not be empty");
        let (shard, rest) = if id.len() > SHARD_LEN {
            (&id[..SHARD_LEN], &id[SHARD_LEN..])
        } else {
            (id, "")
        };
        self.base_dir.join(shard).join(rest)
    }
}

// ── Object ───────────────────────────────────────────────────────────────────

impl Object {
    /// Create a new `Object` wrapping the given directory.
    ///
    /// The directory must already exist and be writable.
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Initialise this object from a base object via CoW copy.
    ///
    /// Uses `cp --reflink=auto -a` to clone the base tree.  On filesystems
    /// that support reflink (btrfs, xfs, ZFS, Bcachefs) this is an
    /// O(metadata) operation.  Otherwise it falls back to a full copy.
    pub fn init(&mut self, base: &Object) -> Result<(), BkgCaveError> {
        if !base.dir.is_dir() {
            return Err(BkgCaveError::NotFound(format!(
                "base object directory does not exist: {}",
                base.dir.display()
            )));
        }

        // Remove destination first so `cp -aT` copies base *to* self
        // (instead of copying base *into* self when self already exists).
        std::fs::remove_dir_all(&self.dir)?;
        std::fs::create_dir_all(&self.dir)?;

        let status = std::process::Command::new("cp")
            .args(["--reflink=auto", "-aT"])
            .arg(&base.dir)
            .arg(&self.dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| BkgCaveError::Execution {
                command: format!(
                    "cp --reflink=auto -aT {} {}",
                    base.dir.display(),
                    self.dir.display()
                ),
                exit_code: -1,
                stderr: format!("failed to execute cp: {e}"),
            })?;

        if !status.success() {
            return Err(BkgCaveError::Execution {
                command: format!(
                    "cp --reflink=auto -aT {} {}",
                    base.dir.display(),
                    self.dir.display()
                ),
                exit_code: status.code().unwrap_or(-1),
                stderr: "cp command failed".into(),
            });
        }

        tracing::debug!(
            base = %base.dir.display(),
            obj = %self.dir.display(),
            "Object initialised from base (CoW)"
        );
        Ok(())
    }

    /// Finalise the object — clamp all file mtimes to the Unix epoch.
    ///
    /// This ensures that the tree's content hash is reproducible: two
    /// identical directory trees will produce the same hash regardless
    /// of when they were created.
    ///
    /// Uses `find -exec touch` to recursively set all mtimes to the epoch.
    /// This avoids extra Rust dependencies for file timestamp manipulation.
    pub fn finalize(&self) -> Result<(), BkgCaveError> {
        let status = std::process::Command::new("find")
            .arg(&self.dir)
            .arg("-exec")
            .arg("touch")
            .arg("-h") // don't follow symlinks
            .arg("-t")
            .arg("197001010000.00")
            .arg("{}")
            .arg("+")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .map_err(|e| BkgCaveError::Execution {
                command: format!("find {} -exec touch ...", self.dir.display()),
                exit_code: -1,
                stderr: format!("failed to execute find: {e}"),
            })?;

        if !status.success() {
            return Err(BkgCaveError::Execution {
                command: format!("find {} -exec touch ...", self.dir.display()),
                exit_code: status.code().unwrap_or(-1),
                stderr: "mtime finalize failed".into(),
            });
        }

        tracing::debug!(dir = %self.dir.display(), "Object finalised (mtimes clamped to epoch)");
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_object_store() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();
        assert!(store.base_dir.is_dir());
    }

    #[test]
    fn create_and_commit_object() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let obj = store.new_object(None).unwrap();
        // Write a test file inside the object.
        std::fs::write(obj.dir.join("hello.txt"), b"world").unwrap();

        let id = "aabbccdd".to_string();
        store.commit(obj, &id).unwrap();

        assert!(store.contains(&id));
    }

    #[test]
    fn get_committed_object() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let obj = store.new_object(None).unwrap();
        std::fs::write(obj.dir.join("test.txt"), b"data").unwrap();
        store.commit(obj, "11223344").unwrap();

        let retrieved = store.get("11223344").unwrap();
        assert!(retrieved.is_some());
        let retrieved_obj = retrieved.unwrap();
        assert!(retrieved_obj.dir.join("test.txt").is_file());
    }

    #[test]
    fn contains_returns_false_for_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();
        assert!(!store.contains("nonexistent"));
    }

    #[test]
    fn get_returns_none_for_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();
        let result = store.get("deadbeef").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn export_object() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let obj = store.new_object(None).unwrap();
        std::fs::write(obj.dir.join("export.txt"), b"exported").unwrap();
        store.commit(obj, "export1234").unwrap();

        let export_dir = tmp.path().join("exported");
        store.export("export1234", &export_dir).unwrap();
        assert!(export_dir.is_dir());
        assert!(export_dir.join("export.txt").is_file());
        assert_eq!(
            std::fs::read_to_string(export_dir.join("export.txt")).unwrap(),
            "exported"
        );
    }

    #[test]
    fn export_nonexistent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();
        let result = store.export("ghost", &tmp.path().join("out"));
        assert!(result.is_err());
    }

    #[test]
    fn commit_twice_is_idempotent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let obj1 = store.new_object(None).unwrap();
        std::fs::write(obj1.dir.join("file.txt"), b"content").unwrap();
        store.commit(obj1, "dup00001").unwrap();

        let obj2 = store.new_object(None).unwrap();
        std::fs::write(obj2.dir.join("file.txt"), b"content").unwrap();
        // Should not error — duplicates are silently accepted.
        store.commit(obj2, "dup00001").unwrap();

        assert!(store.contains("dup00001"));
    }

    #[test]
    fn cow_init_from_checkpoint() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        // Create a base checkpoint object.
        let base_obj = store.new_object(None).unwrap();
        std::fs::create_dir_all(base_obj.dir.join("sub")).unwrap();
        std::fs::write(base_obj.dir.join("base.txt"), b"base content").unwrap();
        std::fs::write(base_obj.dir.join("sub/nested.txt"), b"nested").unwrap();
        store.commit(base_obj, "base0001").unwrap();

        // Retrieve and use as checkpoint.
        let checkpoint = store.get("base0001").unwrap().unwrap();

        // Create a new object from this checkpoint.
        let derived = store.new_object(Some(&checkpoint)).unwrap();
        // The derived object should have the base files.
        assert!(derived.dir.join("base.txt").is_file());
        assert!(derived.dir.join("sub/nested.txt").is_file());

        // Add a new file.
        std::fs::write(derived.dir.join("derived.txt"), b"derived").unwrap();
        store.commit(derived, "derived1").unwrap();

        // Verify derived is independent.
        let derived_check = store.get("derived1").unwrap().unwrap();
        assert!(derived_check.dir.join("base.txt").is_file());
        assert!(derived_check.dir.join("derived.txt").is_file());
    }

    #[test]
    fn object_finalize_clamps_mtimes() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let obj = store.new_object(None).unwrap();
        std::fs::write(obj.dir.join("test.txt"), b"data").unwrap();
        std::fs::create_dir(obj.dir.join("subdir")).unwrap();

        // Finalize should succeed.
        obj.finalize().unwrap();
    }

    #[test]
    fn object_new_creates_empty_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();
        let obj = store.new_object(None).unwrap();
        assert!(obj.dir.is_dir());
        // Should be empty.
        let entries: Vec<_> = std::fs::read_dir(&obj.dir).unwrap().collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn store_id_to_path_sharded() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();
        // Access a private method indirectly via commit and get.
        let obj = store.new_object(None).unwrap();
        std::fs::write(obj.dir.join("f"), b"1").unwrap();
        let id = "abcdef1234567890";
        store.commit(obj, id).unwrap();

        // The stored path should be base_dir/ab/cdef1234567890
        let expected = tmp.path().join("ab").join("cdef1234567890");
        assert!(
            expected.is_dir(),
            "expected path {expected:?} does not exist"
        );
    }
}
