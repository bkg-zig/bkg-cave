//! [`InitramfsProtocol`] trait — the abstraction through which hooks and
//! module generators interact with the build environment.
//!
//! Every hook function receives a `&dyn InitramfsProtocol` so it can query the
//! configuration, create files and directories in the build tree, copy host
//! files, and invoke sub-commands.

use std::collections::HashMap;
use std::path::Path;

use crate::config::InitramfsConfig;
use crate::error::KindleError;

/// Protocol provided to every hook and module generator.
///
/// Implementations are provided by [`Generator`](crate::generator::Generator)
/// during the build so that hooks can:
///
/// * Read configuration (via [`config`](Self::config)).
/// * Inspect which shell functions have already been included.
/// * Invoke sub-hooks.
/// * Manipulate the build directory tree.
pub trait InitramfsProtocol {
    /// Return a shared reference to the current initramfs configuration.
    fn config(&self) -> &InitramfsConfig;

    /// Return a map of hook-name → list-of-shell-function-names that have
    /// already been included in the init script.
    fn included_functions(&self) -> &HashMap<String, Vec<String>>;

    /// Invoke all hooks registered under `name`, collecting their shell
    /// code output.
    ///
    /// `force_include` causes masked hooks to fire anyway; `force_exclude`
    /// prevents the named hooks from firing.
    fn run_hook(
        &self,
        name: &str,
        force_include: bool,
        force_exclude: &[&str],
    ) -> Result<Vec<String>, KindleError>;

    /// Create a directory (and any parents) inside the build tree.
    fn mkdir(&self, path: &Path) -> Result<(), KindleError>;

    /// Write `content` to a file inside the build tree with the given
    /// Unix permission bits.
    fn write(&self, name: &str, content: &str, mode: u32) -> Result<(), KindleError>;

    /// Copy a file or directory from the host filesystem into the build
    /// tree.
    fn copy(&self, src: &Path, dst: &Path) -> Result<(), KindleError>;

    /// Create a symbolic link inside the build tree.
    fn symlink(&self, target: &str, link: &Path) -> Result<(), KindleError>;

    /// Execute an external command and return its stdout.
    fn run(&self, cmd: &str) -> Result<String, KindleError>;
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the trait is object-safe (required for `&dyn InitramfsProtocol`).
    #[test]
    fn trait_is_object_safe() {
        // Compile-time check: if the trait weren't object-safe this wouldn't
        // compile.  We just declare a function that takes the trait object.
        fn _take_trait_object(_p: &dyn InitramfsProtocol) {}
        _ = _take_trait_object;
    }
}
