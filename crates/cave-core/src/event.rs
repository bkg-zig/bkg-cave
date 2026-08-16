//! Simple synchronous event bus for bkg-cave.
//!
//! [`EventBus`] allows components to subscribe to named events and emit
//! [`CaveEvent`] values. Handlers are called synchronously on the thread
//! that calls [`EventBus::emit`].

use std::sync::{Arc, Mutex};

/// Events that can flow through the bkg-cave system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaveEvent {
    /// A plugin with the given ID was loaded.
    PluginLoaded(String),
    /// A plugin with the given ID was unloaded.
    PluginUnloaded(String),
    /// A configuration value changed (key name).
    ConfigChanged(String),
    /// A build was started (build name).
    BuildStarted(String),
    /// A build completed with its success status.
    BuildComplete {
        /// Build / target name.
        name: String,
        /// Whether the build succeeded.
        success: bool,
    },
    /// A VM was started, yielding its PID.
    VmStarted(u32),
    /// A VM stopped with the given exit code.
    VmStopped(i32),
}

/// Type alias for event handler callbacks.
///
/// Internally wrapped in [`Arc`] so handlers can be cloned out of the lock.
pub type EventHandler = Arc<dyn Fn(&CaveEvent) + Send + Sync>;

/// A simple synchronous event bus.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use cave_core::event::{EventBus, CaveEvent};
///
/// let bus = EventBus::new();
/// bus.subscribe("logger", Arc::new(|event| {
///     println!("event: {event:?}");
/// }));
/// bus.emit(CaveEvent::BuildStarted("kernel".into()));
/// ```
pub struct EventBus {
    handlers: Mutex<Vec<(String, EventHandler)>>,
}

impl EventBus {
    /// Create a new, empty event bus.
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }

    /// Subscribe a handler under the given name.
    ///
    /// If a handler with the same name already exists it will **not** be
    /// replaced — both will fire. Use [`unsubscribe`](Self::unsubscribe) to
    /// remove a previous subscription first.
    pub fn subscribe(&self, name: &str, handler: EventHandler) {
        if let Ok(mut guard) = self.handlers.lock() {
            guard.push((name.to_owned(), handler));
        }
    }

    /// Remove **all** handlers with the given name.
    pub fn unsubscribe(&self, name: &str) {
        if let Ok(mut guard) = self.handlers.lock() {
            guard.retain(|(n, _)| n != name);
        }
    }

    /// Emit an event, calling every registered handler synchronously.
    ///
    /// If a handler panics the panic is forwarded to the caller; the event
    /// bus does **not** catch panics.
    pub fn emit(&self, event: CaveEvent) {
        // Clone the handler list so we don't hold the lock during callbacks,
        // preventing potential deadlocks if a handler re-enters the bus.
        let snapshot: Vec<(String, EventHandler)> = {
            let guard = match self.handlers.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            guard.clone()
        };
        for (_, handler) in &snapshot {
            handler(&event);
        }
    }

    /// Return the number of registered handlers.
    pub fn handler_count(&self) -> usize {
        self.handlers.lock().map(|g| g.len()).unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn emit_calls_handlers() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        bus.subscribe(
            "test",
            Arc::new(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            }),
        );

        bus.emit(CaveEvent::BuildStarted("x".into()));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn unsubscribe_removes_handlers() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        bus.subscribe(
            "test",
            Arc::new(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            }),
        );

        bus.unsubscribe("test");
        bus.emit(CaveEvent::BuildStarted("x".into()));
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn handler_count_works() {
        let bus = EventBus::new();
        assert_eq!(bus.handler_count(), 0);

        bus.subscribe("a", Arc::new(|_| {}));
        bus.subscribe("b", Arc::new(|_| {}));
        assert_eq!(bus.handler_count(), 2);
    }

    #[test]
    fn snapshot_prevents_deadlock_on_reentrant_emit() {
        let bus = Arc::new(EventBus::new());
        let weak = Arc::downgrade(&bus);
        let call_count = Arc::new(AtomicUsize::new(0));

        let count = call_count.clone();
        bus.subscribe(
            "outer",
            Arc::new(move |_event| {
                let prev = count.fetch_add(1, Ordering::SeqCst);
                // Re-emit on the first call only — avoids infinite recursion.
                if prev == 0 {
                    if let Some(b) = weak.upgrade() {
                        b.emit(CaveEvent::VmStopped(0));
                    }
                }
            }),
        );

        bus.emit(CaveEvent::VmStarted(42));
        // The handler was called twice (initial + re-entrant), proving the lock
        // was released before the re-entrant emit.
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn default_is_empty() {
        let bus = EventBus::default();
        assert_eq!(bus.handler_count(), 0);
    }
}
