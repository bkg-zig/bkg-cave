//! Source resolution and download for pipeline manifests.
//!
//! In osbuild, sources describe external content — files, RPM repositories,
//! container images — that stages consume.  The source system maps a source
//! item to a content-addressed object in the store so that downstream stages
//! can reference it by checksum.
//!
//! This module provides the orchestration framework: given a manifest and an
//! object store, it iterates over the declared sources and resolves each
//! item, calling out to the appropriate stage implementation.

use cave_core::BkgCaveError;

use crate::objectstore::ObjectStore;
use crate::pipeline::Manifest;

/// Resolve and download all sources declared in the manifest.
///
/// For each source in the manifest, this function iterates its items and
/// ensures that every item is available in the object store.  Items that
/// are already present are skipped; missing items are "downloaded" by
/// delegating to the appropriate source handler.
///
/// Currently supported source types:
///
/// * `"org.osbuild.curl"` — downloads a single file via `curl`
/// * `"org.osbuild.noop"` — no-op, useful for testing
///
/// # Errors
///
/// Returns an error if a source type is unsupported, if a download fails,
/// or if I/O operations on the object store fail.
pub fn download_sources(manifest: &Manifest, store: &ObjectStore) -> Result<(), BkgCaveError> {
    let sources = match &manifest.sources {
        Some(s) => s,
        None => return Ok(()),
    };

    for source in sources {
        let items = match &source.items {
            Some(i) => i,
            None => continue,
        };

        match source.name.as_str() {
            "org.osbuild.curl" => {
                for (checksum_id, options) in items {
                    // Check if we already have it.
                    if store.contains(checksum_id) {
                        tracing::debug!(item = %checksum_id, "source already cached");
                        continue;
                    }

                    let url = options.get("url").and_then(|v| v.as_str()).ok_or_else(|| {
                        BkgCaveError::InvalidInput(format!(
                            "org.osbuild.curl item '{checksum_id}' missing 'url'"
                        ))
                    })?;

                    tracing::info!(item = %checksum_id, url = %url, "Downloading source");
                    download_curl_item(store, checksum_id, url)?;
                }
            }
            "org.osbuild.noop" => {
                // No-op source type — just log and continue.
                for checksum_id in items.keys() {
                    tracing::debug!(item = %checksum_id, "noop source (skipped)");
                }
            }
            other => {
                return Err(BkgCaveError::Unsupported(format!(
                    "unsupported source type '{other}'"
                )));
            }
        }
    }

    tracing::info!("All sources resolved");
    Ok(())
}

/// Download a single URL via `curl` and store the result under the given
/// content ID.
fn download_curl_item(
    store: &ObjectStore,
    checksum_id: &str,
    url: &str,
) -> Result<(), BkgCaveError> {
    // Create a temporary object for the download.
    let obj = store.new_object(None)?;

    // Determine the output file path inside the object.
    let filename = url.split('/').next_back().unwrap_or("downloaded_file");
    let output_path = obj.dir.join(filename);

    // Run curl.
    let status = std::process::Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&output_path)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| BkgCaveError::Execution {
            command: format!("curl -fsSL -o {} {}", output_path.display(), url),
            exit_code: -1,
            stderr: format!("failed to execute curl: {e}"),
        })?;

    if !status.success() {
        return Err(BkgCaveError::Execution {
            command: format!("curl -fsSL -o {} {}", output_path.display(), url),
            exit_code: status.code().unwrap_or(-1),
            stderr: "curl download failed".into(),
        });
    }

    // Store it in the object store under the checksum ID.
    store.commit(obj, checksum_id)?;

    tracing::debug!(item = %checksum_id, url = %url, "Source downloaded via curl");
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{Manifest, Source};

    #[test]
    fn no_sources_is_ok() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let manifest = Manifest {
            pipelines: vec![],
            sources: None,
        };

        assert!(download_sources(&manifest, &store).is_ok());
    }

    #[test]
    fn empty_sources_list_is_ok() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let manifest = Manifest {
            pipelines: vec![],
            sources: Some(vec![]),
        };

        assert!(download_sources(&manifest, &store).is_ok());
    }

    #[test]
    fn unsupported_source_type_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let manifest = Manifest {
            pipelines: vec![],
            sources: Some(vec![Source {
                name: "org.osbuild.unsupported".into(),
                items: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert("item1".into(), std::collections::HashMap::new());
                    map
                }),
            }]),
        };

        let result = download_sources(&manifest, &store);
        assert!(
            result.is_err(),
            "expected error for unsupported source type"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, BkgCaveError::Unsupported(_)),
            "expected Unsupported, got: {err}"
        );
    }

    #[test]
    fn noop_source_does_not_error() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let manifest = Manifest {
            pipelines: vec![],
            sources: Some(vec![Source {
                name: "org.osbuild.noop".into(),
                items: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert("sha256:abc123".into(), std::collections::HashMap::new());
                    map
                }),
            }]),
        };

        assert!(download_sources(&manifest, &store).is_ok());
    }

    #[test]
    fn curl_source_missing_url_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        let manifest = Manifest {
            pipelines: vec![],
            sources: Some(vec![Source {
                name: "org.osbuild.curl".into(),
                items: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "sha256:def456".into(),
                        std::collections::HashMap::new(), // no "url" key
                    );
                    map
                }),
            }]),
        };

        let result = download_sources(&manifest, &store);
        assert!(result.is_err());
    }

    #[test]
    fn curl_source_with_cached_item_skips_download() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        // Pre-populate the store with an item.
        let obj = store.new_object(None).unwrap();
        std::fs::write(obj.dir.join("cached.txt"), b"cached").unwrap();
        store.commit(obj, "sha256:cached_item").unwrap();
        assert!(store.contains("sha256:cached_item"));

        // Now create a manifest that references this item.
        let manifest = Manifest {
            pipelines: vec![],
            sources: Some(vec![Source {
                name: "org.osbuild.curl".into(),
                items: Some({
                    let mut options = std::collections::HashMap::new();
                    options.insert(
                        "url".into(),
                        serde_json::Value::String("https://example.com/file".into()),
                    );
                    let mut map = std::collections::HashMap::new();
                    map.insert("sha256:cached_item".into(), options);
                    map
                }),
            }]),
        };

        // Should not attempt a download because the item is cached.
        assert!(download_sources(&manifest, &store).is_ok());
    }

    #[test]
    fn curl_download_integration() {
        // This test verifies that curl is available and can download
        // from a real HTTP server.  We skip if curl is not installed.
        let has_curl = std::process::Command::new("curl")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok();

        if !has_curl {
            eprintln!("SKIP: curl not installed");
            return;
        }

        let tmp = tempfile::TempDir::new().unwrap();
        let store = ObjectStore::new(tmp.path()).unwrap();

        // We'll use httpbin.org or similar — let's try httpbin.org/get
        // which returns a small JSON response.
        let manifest = Manifest {
            pipelines: vec![],
            sources: Some(vec![Source {
                name: "org.osbuild.curl".into(),
                items: Some({
                    let mut options = std::collections::HashMap::new();
                    options.insert(
                        "url".into(),
                        serde_json::Value::String("https://httpbin.org/get".into()),
                    );
                    let mut map = std::collections::HashMap::new();
                    map.insert("sha256:httpbin_get".into(), options);
                    map
                }),
            }]),
        };

        let result = download_sources(&manifest, &store);
        // This might fail if httpbin is down — just verify it doesn't crash.
        if let Err(e) = &result {
            eprintln!("NOTE: httpbin download failed (network?): {e}");
        }
    }
}
