//! Hook registry — the extensibility mechanism for initramfs generation.
//!
//! Modules and the core generator register functions under named hooks
//! (e.g. `"init"`, `"pre_build"`, `"post_build"`, `"modules_load"`).
//! When a hook is executed, all registered functions fire in order, and
//! their returned shell code lines are collected.
//!
//! Ordering is controlled via [`HookFn::before`] and [`HookFn::after`]
//! fields, which are resolved by [`HookRegistry::sort_hooks`].

use std::collections::{HashMap, HashSet};

use tracing::debug;

use crate::error::KindleError;
use crate::protocol::InitramfsProtocol;

/// A single hook function registered by a module.
///
/// The `func` field is a function pointer that receives an
/// [`InitramfsProtocol`] trait object and returns optional shell code.
#[derive(Clone)]
pub struct HookFn {
    /// Human-readable name for this hook function (used in masks).
    pub name: String,

    /// Name of the module that registered this hook.
    pub module: String,

    /// The actual function pointer.
    #[allow(clippy::type_complexity)]
    pub func: fn(&dyn InitramfsProtocol) -> Result<Option<Vec<String>>, KindleError>,

    /// Only run *after* these hook names have been executed.
    pub before: Vec<String>,

    /// Only run *before* these hook names have been executed.
    pub after: Vec<String>,
}

impl std::fmt::Debug for HookFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookFn")
            .field("name", &self.name)
            .field("module", &self.module)
            .field("before", &self.before)
            .field("after", &self.after)
            .finish()
    }
}

/// Registry that holds and executes hook functions.
///
/// # Thread-safety
///
/// `HookRegistry` is neither `Send` nor `Sync` because `HookFn` contains
/// a raw function pointer that is safe to call.  The registry is used
/// from a single thread during the build.
#[derive(Debug, Clone)]
pub struct HookRegistry {
    /// hook_name → ordered list of HookFn.
    hooks: HashMap<String, Vec<HookFn>>,

    /// hook_name → set of function names to skip (masked).
    masks: HashMap<String, HashSet<String>>,

    /// Order in which hooks were imported (for stable iteration).
    import_order: Vec<String>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    /// Create a new, empty hook registry.
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            masks: HashMap::new(),
            import_order: Vec::new(),
        }
    }

    /// Register a hook function under the given hook name.
    ///
    /// If this is the first registration for `hook_name`, it is appended to
    /// the import order.
    pub fn register(&mut self, hook_name: &str, hook: HookFn) {
        debug!(
            hook_name = %hook_name,
            fn_name = %hook.name,
            module = %hook.module,
            "registering hook"
        );
        let entry = self.hooks.entry(hook_name.to_owned()).or_default();
        if entry.is_empty() {
            self.import_order.push(hook_name.to_owned());
        }
        entry.push(hook);
    }

    /// Mask (disable) a specific hook function under the given hook name.
    ///
    /// Masked functions are skipped during [`run`](Self::run) unless
    /// `force_include` is `true`.
    pub fn mask(&mut self, hook_name: &str, fn_name: &str) {
        debug!(
            hook_name = %hook_name,
            fn_name = %fn_name,
            "masking hook function"
        );
        self.masks
            .entry(hook_name.to_owned())
            .or_default()
            .insert(fn_name.to_owned());
    }

    /// Run all hooks registered under `hook_name`.
    ///
    /// # Arguments
    ///
    /// * `hook_name` — The hook point to fire.
    /// * `protocol` — Protocol object for hooks to interact with the build.
    /// * `force_include` — If `true`, masked functions are still executed.
    /// * `force_exclude` — Function names to skip regardless of masking.
    ///
    /// # Returns
    ///
    /// A flat `Vec<String>` of shell-code lines produced by all hooks.
    pub fn run(
        &self,
        hook_name: &str,
        protocol: &dyn InitramfsProtocol,
        force_include: bool,
        force_exclude: &[&str],
    ) -> Result<Vec<String>, KindleError> {
        let mut result = Vec::new();

        let Some(hooks) = self.hooks.get(hook_name) else {
            debug!(hook_name = %hook_name, "no hooks registered");
            return Ok(result);
        };

        let hook_masks = self.masks.get(hook_name);
        let exclude_set: HashSet<&&str> = force_exclude.iter().collect();

        for hook in hooks {
            // Check masking.
            if !force_include {
                if let Some(masks) = hook_masks {
                    if masks.contains(&hook.name) {
                        debug!(
                            hook_name = %hook_name,
                            fn_name = %hook.name,
                            "hook function is masked, skipping"
                        );
                        continue;
                    }
                }
            }

            // Check force_exclude.
            if exclude_set.contains(&&hook.name.as_str()) {
                debug!(
                    hook_name = %hook_name,
                    fn_name = %hook.name,
                    "hook function is force-excluded, skipping"
                );
                continue;
            }

            // Execute the hook function.
            debug!(
                hook_name = %hook_name,
                fn_name = %hook.name,
                module = %hook.module,
                "executing hook function"
            );
            match (hook.func)(protocol) {
                Ok(Some(lines)) => {
                    result.extend(lines);
                }
                Ok(None) => {
                    // Hook chose to contribute nothing this round.
                }
                Err(e) => {
                    return Err(KindleError::Hook(format!(
                        "hook '{}' (module '{}') failed: {e}",
                        hook.name, hook.module
                    )));
                }
            }
        }

        Ok(result)
    }

    /// Sort hook functions within each hook point based on their
    /// `before` / `after` constraints using a simple topological sort.
    ///
    /// This should be called after all modules have registered their hooks
    /// and before the first `run` call.
    pub fn sort_hooks(&mut self) -> Result<(), KindleError> {
        for hook_name in self.import_order.clone() {
            if let Some(hooks) = self.hooks.get(&hook_name) {
                let sorted = topological_sort(hooks)?;
                self.hooks.insert(hook_name, sorted);
            }
        }
        Ok(())
    }

    /// Return the set of hook names that have registered functions.
    pub fn hook_names(&self) -> Vec<String> {
        self.import_order.clone()
    }

    /// Check if a specific hook function is masked.
    pub fn is_masked(&self, hook_name: &str, fn_name: &str) -> bool {
        self.masks
            .get(hook_name)
            .is_some_and(|m| m.contains(fn_name))
    }

    /// Return the number of registered hook points.
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

/// Topological sort based on `before` / `after` constraints.
///
/// Uses Kahn's algorithm.  If there are cycles, the sort falls back to the
/// insertion order for conflicting items.
fn topological_sort(hooks: &[HookFn]) -> Result<Vec<HookFn>, KindleError> {
    let n = hooks.len();
    if n <= 1 {
        return Ok(hooks.to_vec());
    }

    // Build name → index mapping.
    let name_to_idx: HashMap<&str, usize> = hooks
        .iter()
        .enumerate()
        .map(|(i, h)| (h.name.as_str(), i))
        .collect();

    // Build adjacency list and in-degree count.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree = vec![0usize; n];

    for (i, hook) in hooks.iter().enumerate() {
        // "before" means hook_i should come before the named hooks.
        // So edges go from hook_i → before_targets.
        for before_name in &hook.before {
            if let Some(&j) = name_to_idx.get(before_name.as_str()) {
                adj[i].push(j);
                in_degree[j] += 1;
            }
        }
        // "after" means hook_i should come after the named hooks.
        // So edges go from after_targets → hook_i.
        for after_name in &hook.after {
            if let Some(&j) = name_to_idx.get(after_name.as_str()) {
                adj[j].push(i);
                in_degree[i] += 1;
            }
        }
    }

    // Kahn's algorithm.
    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut sorted = Vec::with_capacity(n);

    while let Some(i) = queue.pop() {
        sorted.push(hooks[i].clone());
        for &j in &adj[i] {
            in_degree[j] -= 1;
            if in_degree[j] == 0 {
                queue.push(j);
            }
        }
    }

    // If there are cycles, append remaining items in original order.
    if sorted.len() < n {
        // Collect names first to avoid borrow conflict with push below.
        let sorted_names: Vec<String> = sorted.iter().map(|h| h.name.clone()).collect();
        for hook in hooks {
            if !sorted_names.contains(&hook.name) {
                sorted.push(hook.clone());
            }
        }
    }

    Ok(sorted)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;

    // A minimal protocol implementation for testing.
    struct TestProtocol;

    impl InitramfsProtocol for TestProtocol {
        fn config(&self) -> &InitramfsConfig {
            unimplemented!("not needed in tests")
        }
        fn included_functions(&self) -> &HashMap<String, Vec<String>> {
            unimplemented!("not needed in tests")
        }
        fn run_hook(
            &self,
            _name: &str,
            _force_include: bool,
            _force_exclude: &[&str],
        ) -> Result<Vec<String>, KindleError> {
            Ok(vec![])
        }
        fn mkdir(&self, _path: &Path) -> Result<(), KindleError> {
            Ok(())
        }
        fn write(&self, _name: &str, _content: &str, _mode: u32) -> Result<(), KindleError> {
            Ok(())
        }
        fn copy(&self, _src: &Path, _dst: &Path) -> Result<(), KindleError> {
            Ok(())
        }
        fn symlink(&self, _target: &str, _link: &Path) -> Result<(), KindleError> {
            Ok(())
        }
        fn run(&self, _cmd: &str) -> Result<String, KindleError> {
            Ok(String::new())
        }
    }

    // Need to reference InitramfsConfig for the trait impl.
    use crate::config::InitramfsConfig;

    fn dummy_hook_fn(_p: &dyn InitramfsProtocol) -> Result<Option<Vec<String>>, KindleError> {
        Ok(Some(vec!["echo 'hello from hook'".into()]))
    }

    #[test]
    fn empty_registry() {
        let reg = HookRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn register_and_run() {
        let mut reg = HookRegistry::new();
        reg.register(
            "init",
            HookFn {
                name: "test_fn".into(),
                module: "test".into(),
                func: dummy_hook_fn,
                before: vec![],
                after: vec![],
            },
        );

        let protocol = TestProtocol;
        let lines = reg.run("init", &protocol, false, &[]).unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("hello from hook"));
    }

    #[test]
    fn run_unknown_hook_returns_empty() {
        let reg = HookRegistry::new();
        let protocol = TestProtocol;
        let lines = reg.run("nonexistent", &protocol, false, &[]).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn mask_suppresses_hook() {
        let mut reg = HookRegistry::new();
        reg.register(
            "init",
            HookFn {
                name: "masked_fn".into(),
                module: "test".into(),
                func: dummy_hook_fn,
                before: vec![],
                after: vec![],
            },
        );
        reg.mask("init", "masked_fn");

        let protocol = TestProtocol;
        let lines = reg.run("init", &protocol, false, &[]).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn force_include_overrides_mask() {
        let mut reg = HookRegistry::new();
        reg.register(
            "init",
            HookFn {
                name: "masked_fn".into(),
                module: "test".into(),
                func: dummy_hook_fn,
                before: vec![],
                after: vec![],
            },
        );
        reg.mask("init", "masked_fn");

        let protocol = TestProtocol;
        let lines = reg.run("init", &protocol, true, &[]).unwrap();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn force_exclude_skips_hook() {
        let mut reg = HookRegistry::new();
        reg.register(
            "init",
            HookFn {
                name: "excluded_fn".into(),
                module: "test".into(),
                func: dummy_hook_fn,
                before: vec![],
                after: vec![],
            },
        );

        let protocol = TestProtocol;
        let lines = reg.run("init", &protocol, false, &["excluded_fn"]).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn sort_hooks_respects_before() {
        let mut reg = HookRegistry::new();

        let fn_a = HookFn {
            name: "a".into(),
            module: "test".into(),
            func: dummy_hook_fn,
            before: vec!["b".into()],
            after: vec![],
        };
        let fn_b = HookFn {
            name: "b".into(),
            module: "test".into(),
            func: dummy_hook_fn,
            before: vec![],
            after: vec![],
        };

        reg.register("init", fn_a);
        reg.register("init", fn_b);
        reg.sort_hooks().unwrap();

        let hooks = reg.hooks.get("init").unwrap();
        let names: Vec<&str> = hooks.iter().map(|h| h.name.as_str()).collect();
        // a should come before b
        let pos_a = names.iter().position(|&n| n == "a").unwrap();
        let pos_b = names.iter().position(|&n| n == "b").unwrap();
        assert!(pos_a < pos_b, "a should be before b");
    }

    #[test]
    fn sort_hooks_respects_after() {
        let mut reg = HookRegistry::new();

        let fn_a = HookFn {
            name: "a".into(),
            module: "test".into(),
            func: dummy_hook_fn,
            before: vec![],
            after: vec![],
        };
        let fn_b = HookFn {
            name: "b".into(),
            module: "test".into(),
            func: dummy_hook_fn,
            before: vec![],
            after: vec!["a".into()],
        };

        reg.register("init", fn_a);
        reg.register("init", fn_b);
        reg.sort_hooks().unwrap();

        let hooks = reg.hooks.get("init").unwrap();
        let names: Vec<&str> = hooks.iter().map(|h| h.name.as_str()).collect();
        // a should come before b
        let pos_a = names.iter().position(|&n| n == "a").unwrap();
        let pos_b = names.iter().position(|&n| n == "b").unwrap();
        assert!(pos_a < pos_b, "a should be before b");
    }

    #[test]
    fn hook_names_returns_registered() {
        let mut reg = HookRegistry::new();
        reg.register(
            "pre_build",
            HookFn {
                name: "fn1".into(),
                module: "test".into(),
                func: dummy_hook_fn,
                before: vec![],
                after: vec![],
            },
        );
        reg.register(
            "init",
            HookFn {
                name: "fn2".into(),
                module: "test".into(),
                func: dummy_hook_fn,
                before: vec![],
                after: vec![],
            },
        );

        let names = reg.hook_names();
        assert!(names.contains(&"pre_build".into()));
        assert!(names.contains(&"init".into()));
    }
}
