use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    tasks_completed: AtomicU64,
    tasks_failed: AtomicU64,
    peers_connected: AtomicU64,
    jobs_active: AtomicU64,
    inference_requests: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            peers_connected: AtomicU64::new(0),
            jobs_active: AtomicU64::new(0),
            inference_requests: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        }
    }

    pub fn inc_tasks_completed(&self) {
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_tasks_failed(&self) {
        self.tasks_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_peers_connected(&self, n: u64) {
        self.peers_connected.store(n, Ordering::Relaxed);
    }

    pub fn set_jobs_active(&self, n: u64) {
        self.jobs_active.store(n, Ordering::Relaxed);
    }

    pub fn inc_inference_requests(&self) {
        self.inference_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes_sent(&self, n: u64) {
        self.bytes_sent.fetch_add(n, Ordering::Relaxed);
    }

    pub fn add_bytes_received(&self, n: u64) {
        self.bytes_received.fetch_add(n, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            tasks_completed: self.tasks_completed.load(Ordering::Relaxed),
            tasks_failed: self.tasks_failed.load(Ordering::Relaxed),
            peers_connected: self.peers_connected.load(Ordering::Relaxed),
            jobs_active: self.jobs_active.load(Ordering::Relaxed),
            inference_requests: self.inference_requests.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricsSnapshot {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub peers_connected: u64,
    pub jobs_active: u64,
    pub inference_requests: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_metrics_are_zero() {
        let m = Metrics::new();
        let snap = m.snapshot();
        assert_eq!(snap.tasks_completed, 0);
        assert_eq!(snap.tasks_failed, 0);
        assert_eq!(snap.peers_connected, 0);
    }

    #[test]
    fn increment_tasks() {
        let m = Metrics::new();
        m.inc_tasks_completed();
        m.inc_tasks_completed();
        m.inc_tasks_failed();
        let snap = m.snapshot();
        assert_eq!(snap.tasks_completed, 2);
        assert_eq!(snap.tasks_failed, 1);
    }

    #[test]
    fn set_peers() {
        let m = Metrics::new();
        m.set_peers_connected(5);
        assert_eq!(m.snapshot().peers_connected, 5);
    }

    #[test]
    fn bytes_accumulate() {
        let m = Metrics::new();
        m.add_bytes_sent(100);
        m.add_bytes_sent(200);
        assert_eq!(m.snapshot().bytes_sent, 300);
    }

    #[test]
    fn inference_requests() {
        let m = Metrics::new();
        m.inc_inference_requests();
        m.inc_inference_requests();
        m.inc_inference_requests();
        assert_eq!(m.snapshot().inference_requests, 3);
    }

    #[test]
    fn snapshot_is_consistent() {
        let m = Metrics::new();
        m.inc_tasks_completed();
        m.set_peers_connected(2);
        m.add_bytes_received(512);
        let snap = m.snapshot();
        assert_eq!(snap.tasks_completed, 1);
        assert_eq!(snap.peers_connected, 2);
        assert_eq!(snap.bytes_received, 512);
        assert_eq!(snap.bytes_sent, 0);
    }
}
