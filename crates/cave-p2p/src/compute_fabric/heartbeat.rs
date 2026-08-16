use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::compute_fabric::worker::WorkerStatus;

/// Heartbeat timeout threshold in seconds.
pub const DEFAULT_HEARTBEAT_TIMEOUT_SECS: u64 = 30;

/// Heartbeat interval in seconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 10;

/// Current heartbeat timestamp for a worker.
#[derive(Debug, Clone, Copy)]
pub struct Heartbeat {
    pub worker_id: String,
    pub timestamp: u64,
}

impl Heartbeat {
    /// Create a new heartbeat for the given worker.
    pub fn new(worker_id: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            timestamp: now_secs(),
        }
    }

    /// Check if this heartbeat is still valid given a timeout.
    pub fn is_alive(&self, timeout_secs: u64) -> bool {
        now_secs() < self.timestamp + timeout_secs
    }
}

/// Monitors worker heartbeats and detects liveness failures.
#[derive(Debug, Default)]
pub struct HeartbeatMonitor {
    heartbeats: HashMap<String, Heartbeat>,
    timeout_secs: u64,
}

impl HeartbeatMonitor {
    /// Create a new heartbeat monitor with the given timeout.
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            heartbeats: HashMap::new(),
            timeout_secs,
        }
    }

    /// Create with default timeout (30 seconds).
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_HEARTBEAT_TIMEOUT_SECS)
    }

    /// Record a heartbeat from a worker.
    pub fn record(&mut self, worker_id: impl Into<String>) {
        let worker_id = worker_id.into();
        self.heartbeats
            .insert(worker_id.clone(), Heartbeat::new(worker_id));
    }

    /// Check if a worker is alive based on its last heartbeat.
    pub fn is_alive(&self, worker_id: &str) -> bool {
        self.heartbeats
            .get(worker_id)
            .map(|hb| hb.is_alive(self.timeout_secs))
            .unwrap_or(false)
    }

    /// Get all alive workers.
    pub fn alive_workers(&self) -> Vec<String> {
        self.heartbeats
            .iter()
            .filter(|(_, hb)| hb.is_alive(self.timeout_secs))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all dead workers (no heartbeat within timeout).
    pub fn dead_workers(&self) -> Vec<String> {
        self.heartbeats
            .iter()
            .filter(|(_, hb)| !hb.is_alive(self.timeout_secs))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Remove a worker from monitoring.
    pub fn remove(&mut self, worker_id: &str) {
        self.heartbeats.remove(worker_id);
    }

    /// Update the timeout threshold.
    pub fn set_timeout(&mut self, timeout_secs: u64) {
        self.timeout_secs = timeout_secs;
    }

    /// Get the current timeout threshold.
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_check_alive() {
        let mut monitor = HeartbeatMonitor::new(60);
        monitor.record("worker-1");
        assert!(monitor.is_alive("worker-1"));
    }

    #[test]
    fn test_dead_after_timeout() {
        let mut monitor = HeartbeatMonitor::new(0);
        monitor.record("worker-1");
        // With 0 timeout, the heartbeat is immediately expired
        assert!(!monitor.is_alive("worker-1"));
    }

    #[test]
    fn test_alive_workers() {
        let mut monitor = HeartbeatMonitor::new(60);
        monitor.record("worker-1");
        monitor.record("worker-2");
        monitor.record("worker-3");

        let alive = monitor.alive_workers();
        assert_eq!(alive.len(), 3);
    }

    #[test]
    fn test_dead_workers() {
        let mut monitor = HeartbeatMonitor::new(0);
        monitor.record("worker-1");
        monitor.record("worker-2");

        let dead = monitor.dead_workers();
        assert_eq!(dead.len(), 2);
    }

    #[test]
    fn test_remove_worker() {
        let mut monitor = HeartbeatMonitor::new(60);
        monitor.record("worker-1");
        monitor.remove("worker-1");
        assert!(!monitor.is_alive("worker-1"));
        assert!(monitor.alive_workers().is_empty());
    }

    #[test]
    fn test_set_timeout() {
        let mut monitor = HeartbeatMonitor::new(60);
        monitor.record("worker-1");
        monitor.set_timeout(0);
        assert!(!monitor.is_alive("worker-1"));
    }

    #[test]
    fn test_heartbeat_is_alive_with_future_timestamp() {
        // Create a heartbeat with a future timestamp
        let hb = Heartbeat {
            worker_id: "worker-1".into(),
            timestamp: now_secs() + 100,
        };
        assert!(hb.is_alive(200));
        assert!(!hb.is_alive(50));
    }

    #[test]
    fn test_with_defaults() {
        let monitor = HeartbeatMonitor::with_defaults();
        assert_eq!(monitor.timeout_secs(), DEFAULT_HEARTBEAT_TIMEOUT_SECS);
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
