use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum RoutineAction {
    ResourceBroadcast,
    ProviderRefresh,
    PeerHealthCheck,
    WalletSweep,
}

impl RoutineAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoutineAction::ResourceBroadcast => "resource_broadcast",
            RoutineAction::ProviderRefresh => "provider_refresh",
            RoutineAction::PeerHealthCheck => "peer_health_check",
            RoutineAction::WalletSweep => "wallet_sweep",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Routine {
    pub name: String,
    pub interval_secs: u64,
    pub last_run: u64,
    pub action: RoutineAction,
}

impl Routine {
    pub fn new(name: String, interval_secs: u64, action: RoutineAction) -> Self {
        Self {
            name,
            interval_secs,
            last_run: 0,
            action,
        }
    }

    pub fn is_due(&self, now: u64) -> bool {
        now.saturating_sub(self.last_run) >= self.interval_secs
    }
}

#[derive(Default)]
pub struct RoutineEngine {
    routines: Vec<Routine>,
}

impl RoutineEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_routine(&mut self, routine: Routine) {
        self.routines.push(routine);
    }

    pub fn check_due(&self) -> Vec<&Routine> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.routines.iter().filter(|r| r.is_due(now)).collect()
    }

    pub fn mark_run(&mut self, name: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(r) = self.routines.iter_mut().find(|r| r.name == name) {
            r.last_run = now;
        }
    }

    pub fn add_test_routines(&mut self) {
        self.register_routine(Routine::new(
            "test_health".into(),
            60,
            RoutineAction::PeerHealthCheck,
        ));
        self.register_routine(Routine::new(
            "test_broadcast".into(),
            300,
            RoutineAction::ResourceBroadcast,
        ));
    }

    pub fn routines(&self) -> &[Routine] {
        &self.routines
    }

    pub fn routine_count(&self) -> usize {
        self.routines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine_has_no_routines() {
        let engine = RoutineEngine::new();
        assert_eq!(engine.routine_count(), 0);
    }

    #[test]
    fn register_routine() {
        let mut engine = RoutineEngine::new();
        engine.register_routine(Routine::new(
            "check".into(),
            60,
            RoutineAction::PeerHealthCheck,
        ));
        assert_eq!(engine.routine_count(), 1);
    }

    #[test]
    fn routine_is_due_after_interval() {
        let r = Routine::new("test".into(), 10, RoutineAction::PeerHealthCheck);
        let now = 100;
        assert!(r.is_due(now));
    }

    #[test]
    fn routine_not_due_within_interval() {
        let mut r = Routine::new("test".into(), 100, RoutineAction::PeerHealthCheck);
        r.last_run = 50;
        assert!(!r.is_due(55));
    }

    #[test]
    fn mark_run_updates_last_run() {
        let mut engine = RoutineEngine::new();
        engine.register_routine(Routine::new(
            "timer".into(),
            10,
            RoutineAction::ResourceBroadcast,
        ));
        engine.mark_run("timer");
        let r = engine
            .routines()
            .iter()
            .find(|r| r.name == "timer")
            .unwrap();
        assert!(r.last_run > 0);
    }

    #[test]
    fn add_test_routines() {
        let mut engine = RoutineEngine::new();
        engine.add_test_routines();
        assert_eq!(engine.routine_count(), 2);
    }

    #[test]
    fn check_due_returns_due_routines() {
        let mut engine = RoutineEngine::new();
        engine.register_routine(Routine::new(
            "immediate".into(),
            0,
            RoutineAction::PeerHealthCheck,
        ));
        let mut later = Routine::new("later".into(), 999999, RoutineAction::WalletSweep);
        later.last_run = u64::MAX;
        engine.register_routine(later);
        let due = engine.check_due();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, "immediate");
    }
}
