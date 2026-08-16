use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PeerReputation {
    pub peer_id: String,
    pub score: f64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub avg_response_time: f64,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Untrusted,
    Low,
    Medium,
    High,
    Trusted,
}

impl TrustLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 {
            TrustLevel::Trusted
        } else if score >= 60.0 {
            TrustLevel::High
        } else if score >= 40.0 {
            TrustLevel::Medium
        } else if score >= 20.0 {
            TrustLevel::Low
        } else {
            TrustLevel::Untrusted
        }
    }
}

#[derive(Default)]
pub struct ReputationStore {
    peers: HashMap<String, PeerReputation>,
}

impl ReputationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, peer_id: &str) -> Option<&PeerReputation> {
        self.peers.get(peer_id)
    }

    pub fn record_success(&mut self, peer_id: &str, response_time_secs: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = self.peers.entry(peer_id.to_string()).or_insert_with(|| {
            let mut r = Self::new_entry(peer_id);
            r.last_seen = now;
            r
        });
        let total = entry.completed_jobs + entry.failed_jobs;
        entry.score = if total == 0 {
            50.0
        } else {
            let success_rate = entry.completed_jobs as f64 / (total as f64);
            let time_factor = (100.0 / (response_time_secs + 1.0)).min(30.0);
            (success_rate * 70.0 + time_factor).min(100.0)
        };
        entry.completed_jobs += 1;
        entry.last_seen = now;
        entry.avg_response_time = if entry.completed_jobs > 0 {
            ((entry.avg_response_time * (entry.completed_jobs - 1) as f64) + response_time_secs)
                / entry.completed_jobs as f64
        } else {
            response_time_secs
        };
    }

    pub fn record_failure(&mut self, peer_id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = self.peers.entry(peer_id.to_string()).or_insert_with(|| {
            let mut r = Self::new_entry(peer_id);
            r.last_seen = now;
            r
        });
        let total = entry.completed_jobs + entry.failed_jobs;
        entry.score = if total == 0 {
            50.0
        } else {
            let success_rate = entry.completed_jobs as f64 / ((total + 1) as f64);
            (success_rate * 70.0).min(100.0)
        };
        entry.failed_jobs += 1;
        entry.last_seen = now;
    }

    pub fn trust_level(&self, peer_id: &str) -> TrustLevel {
        self.peers
            .get(peer_id)
            .map(|p| TrustLevel::from_score(p.score))
            .unwrap_or(TrustLevel::Untrusted)
    }

    pub fn best_workers(&self, limit: usize) -> Vec<&PeerReputation> {
        let mut peers: Vec<&PeerReputation> = self.peers.values().collect();
        peers.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        peers.truncate(limit);
        peers
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    fn new_entry(peer_id: &str) -> PeerReputation {
        PeerReputation {
            peer_id: peer_id.to_string(),
            score: 50.0,
            completed_jobs: 0,
            failed_jobs: 0,
            avg_response_time: 0.0,
            last_seen: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store_is_empty() {
        let store = ReputationStore::new();
        assert_eq!(store.peer_count(), 0);
    }

    #[test]
    fn success_increases_score() {
        let mut store = ReputationStore::new();
        store.record_success("peer-1", 1.0);
        let rep = store.get("peer-1").unwrap();
        assert_eq!(rep.completed_jobs, 1);
        assert!(rep.score >= 50.0);
    }

    #[test]
    fn failure_adds_peer() {
        let mut store = ReputationStore::new();
        store.record_failure("peer-2");
        assert_eq!(store.peer_count(), 1);
        assert_eq!(store.get("peer-2").unwrap().failed_jobs, 1);
    }

    #[test]
    fn trust_level_untrusted_for_unknown() {
        let store = ReputationStore::new();
        assert_eq!(store.trust_level("unknown"), TrustLevel::Untrusted);
    }

    #[test]
    fn trust_level_depends_on_score() {
        let mut store = ReputationStore::new();
        store.record_success("good-peer", 0.1);
        store.record_success("good-peer", 0.1);
        store.record_success("good-peer", 0.1);
        let rep = store.get("good-peer").unwrap();
        assert!(rep.score > 60.0);
        let level = store.trust_level("good-peer");
        assert!(level == TrustLevel::High || level == TrustLevel::Trusted);
    }

    #[test]
    fn best_workers_returns_sorted() {
        let mut store = ReputationStore::new();
        store.record_success("fast", 0.1);
        store.record_success("fast", 0.1);
        store.record_failure("slow");
        let best = store.best_workers(1);
        assert_eq!(best.len(), 1);
        assert_eq!(best[0].peer_id, "fast");
    }
}
