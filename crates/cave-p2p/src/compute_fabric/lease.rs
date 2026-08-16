use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::Result;

/// Unique identifier for a lease.
pub type LeaseId = String;

/// Status of a resource lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseStatus {
    /// Lease is active and the worker is reserved.
    Active,
    /// Lease was released before expiration.
    Released,
    /// Lease expired due to timeout.
    Expired,
    /// Lease was cancelled.
    Cancelled,
}

/// A resource lease reserves a worker for a specific job.
#[derive(Debug, Clone)]
pub struct Lease {
    pub id: LeaseId,
    pub worker_id: String,
    pub job_id: String,
    pub status: LeaseStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub ttl_secs: u64,
}

impl Lease {
    /// Check if the lease has expired.
    pub fn is_expired(&self) -> bool {
        now_secs() >= self.expires_at
    }

    /// Remaining time until expiration in seconds.
    pub fn remaining_secs(&self) -> u64 {
        self.expires_at.saturating_sub(now_secs())
    }
}

/// Manages resource leases for workers.
#[derive(Debug, Default)]
pub struct LeaseManager {
    leases: Vec<Lease>,
    next_id: u64,
}

impl LeaseManager {
    /// Create a new lease manager.
    pub fn new() -> Self {
        Self {
            leases: Vec::new(),
            next_id: 0,
        }
    }

    /// Create a new lease for a worker.
    pub fn create_lease(
        &mut self,
        worker_id: impl Into<String>,
        job_id: impl Into<String>,
        ttl_secs: u64,
    ) -> LeaseId {
        let id = self.generate_id();
        let now = now_secs();
        let lease = Lease {
            id: id.clone(),
            worker_id: worker_id.into(),
            job_id: job_id.into(),
            status: LeaseStatus::Active,
            created_at: now,
            expires_at: now + ttl_secs,
            ttl_secs,
        };
        self.leases.push(lease);
        id
    }

    /// Release a lease early.
    pub fn release_lease(&mut self, lease_id: &str) -> Result<()> {
        let lease = self
            .leases
            .iter_mut()
            .find(|l| l.id == lease_id)
            .ok_or_else(|| crate::Error::Other(format!("lease {lease_id} not found")))?;
        if lease.status != LeaseStatus::Active {
            return Err(crate::Error::Other(format!(
                "lease {lease_id} is not active (status: {:?})",
                lease.status
            )));
        }
        lease.status = LeaseStatus::Released;
        Ok(())
    }

    /// Cancel a lease.
    pub fn cancel_lease(&mut self, lease_id: &str) -> Result<()> {
        let lease = self
            .leases
            .iter_mut()
            .find(|l| l.id == lease_id)
            .ok_or_else(|| crate::Error::Other(format!("lease {lease_id} not found")))?;
        if lease.status != LeaseStatus::Active {
            return Err(crate::Error::Other(format!(
                "lease {lease_id} is not active (status: {:?})",
                lease.status
            )));
        }
        lease.status = LeaseStatus::Cancelled;
        Ok(())
    }

    /// Get a lease by ID.
    pub fn get(&self, lease_id: &str) -> Option<&Lease> {
        self.leases.iter().find(|l| l.id == lease_id)
    }

    /// Get all active leases.
    pub fn active_leases(&self) -> Vec<&Lease> {
        self.leases
            .iter()
            .filter(|l| l.status == LeaseStatus::Active && !l.is_expired())
            .collect()
    }

    /// Get all leases for a specific worker.
    pub fn leases_for_worker(&self, worker_id: &str) -> Vec<&Lease> {
        self.leases
            .iter()
            .filter(|l| l.worker_id == worker_id)
            .collect()
    }

    /// Get all leases for a specific job.
    pub fn leases_for_job(&self, job_id: &str) -> Vec<&Lease> {
        self.leases
            .iter()
            .filter(|l| l.job_id == job_id)
            .collect()
    }

    /// Sweep expired leases and mark them as expired.
    /// Returns the number of leases that were expired.
    pub fn sweep_expired(&mut self) -> usize {
        let now = now_secs();
        let mut count = 0;
        for lease in &mut self.leases {
            if lease.status == LeaseStatus::Active && lease.expires_at <= now {
                lease.status = LeaseStatus::Expired;
                count += 1;
            }
        }
        count
    }

    /// Remove all expired or completed leases.
    pub fn cleanup(&mut self) {
        self.leases
            .retain(|l| l.status == LeaseStatus::Active && !l.is_expired());
    }

    fn generate_id(&mut self) -> LeaseId {
        self.next_id += 1;
        format!("lease-{}", self.next_id)
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_lease() {
        let mut mgr = LeaseManager::new();
        let lease_id = mgr.create_lease("worker-1", "job-1", 3600);
        assert!(lease_id.starts_with("lease-"));
        let lease = mgr.get(&lease_id).unwrap();
        assert_eq!(lease.worker_id, "worker-1");
        assert_eq!(lease.job_id, "job-1");
        assert_eq!(lease.status, LeaseStatus::Active);
        assert!(lease.remaining_secs() <= 3600);
    }

    #[test]
    fn test_release_lease() {
        let mut mgr = LeaseManager::new();
        let lease_id = mgr.create_lease("worker-1", "job-1", 3600);
        assert!(mgr.release_lease(&lease_id).is_ok());
        let lease = mgr.get(&lease_id).unwrap();
        assert_eq!(lease.status, LeaseStatus::Released);
    }

    #[test]
    fn test_cancel_lease() {
        let mut mgr = LeaseManager::new();
        let lease_id = mgr.create_lease("worker-1", "job-1", 3600);
        assert!(mgr.cancel_lease(&lease_id).is_ok());
        let lease = mgr.get(&lease_id).unwrap();
        assert_eq!(lease.status, LeaseStatus::Cancelled);
    }

    #[test]
    fn test_release_nonexistent_lease_fails() {
        let mut mgr = LeaseManager::new();
        let result = mgr.release_lease("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_active_leases() {
        let mut mgr = LeaseManager::new();
        let id1 = mgr.create_lease("worker-1", "job-1", 3600);
        let id2 = mgr.create_lease("worker-2", "job-1", 3600);
        mgr.release_lease(&id1).unwrap();

        let active = mgr.active_leases();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, id2);
    }

    #[test]
    fn test_leases_for_worker() {
        let mut mgr = LeaseManager::new();
        mgr.create_lease("worker-1", "job-1", 3600);
        mgr.create_lease("worker-1", "job-2", 3600);
        mgr.create_lease("worker-2", "job-1", 3600);

        let worker1_leases = mgr.leases_for_worker("worker-1");
        assert_eq!(worker1_leases.len(), 2);
    }

    #[test]
    fn test_leases_for_job() {
        let mut mgr = LeaseManager::new();
        mgr.create_lease("worker-1", "job-1", 3600);
        mgr.create_lease("worker-2", "job-1", 3600);
        mgr.create_lease("worker-3", "job-2", 3600);

        let job1_leases = mgr.leases_for_job("job-1");
        assert_eq!(job1_leases.len(), 2);
    }

    #[test]
    fn test_sweep_expired() {
        let mut mgr = LeaseManager::new();
        // Create a lease with 0 TTL (already expired)
        let lease_id = mgr.create_lease("worker-1", "job-1", 0);
        let count = mgr.sweep_expired();
        assert_eq!(count, 1);
        let lease = mgr.get(&lease_id).unwrap();
        assert_eq!(lease.status, LeaseStatus::Expired);
    }

    #[test]
    fn test_cleanup_removes_expired() {
        let mut mgr = LeaseManager::new();
        mgr.create_lease("worker-1", "job-1", 0);
        mgr.create_lease("worker-2", "job-2", 3600);
        mgr.cleanup();

        assert_eq!(mgr.leases.len(), 1);
        assert_eq!(mgr.leases[0].worker_id, "worker-2");
    }
}
