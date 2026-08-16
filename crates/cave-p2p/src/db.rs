use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static INSTANCE: OnceLock<Mutex<Database>> = OnceLock::new();

fn global_db() -> &'static Mutex<Database> {
    INSTANCE.get_or_init(|| Mutex::new(Database::new()))
}

#[derive(Default)]
pub struct Database {
    settings: HashMap<String, String>,
    runs: HashMap<String, Vec<String>>,
}

impl Database {
    fn new() -> Self {
        Self::default()
    }

    pub fn open() -> &'static Mutex<Database> {
        global_db()
    }

    pub fn store_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }

    pub fn get_setting(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }

    pub fn list_run_ids(&self) -> Vec<String> {
        self.runs.keys().cloned().collect()
    }

    pub fn get_run(&self, id: &str) -> Option<&Vec<String>> {
        self.runs.get(id)
    }

    pub fn list_run_events(&self, id: &str) -> Vec<String> {
        self.runs.get(id).cloned().unwrap_or_default()
    }

    pub fn append_run_event(&mut self, run_id: String, event: String) {
        self.runs.entry(run_id).or_default().push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_retrieve_setting() {
        let mut db = Database::new();
        db.store_setting("key1".into(), "val1".into());
        assert_eq!(db.get_setting("key1"), Some("val1"));
    }

    #[test]
    fn missing_setting_returns_none() {
        let db = Database::new();
        assert_eq!(db.get_setting("nonexistent"), None);
    }

    #[test]
    fn overwrite_setting() {
        let mut db = Database::new();
        db.store_setting("k".into(), "v1".into());
        db.store_setting("k".into(), "v2".into());
        assert_eq!(db.get_setting("k"), Some("v2"));
    }

    #[test]
    fn run_events() {
        let mut db = Database::new();
        db.append_run_event("run-1".into(), "start".into());
        db.append_run_event("run-1".into(), "complete".into());
        assert_eq!(db.list_run_ids(), vec!["run-1"]);
        assert_eq!(db.list_run_events("run-1"), vec!["start", "complete"]);
    }

    #[test]
    fn no_run_events_for_missing() {
        let db = Database::new();
        let events: Vec<String> = db.list_run_events("missing");
        assert!(events.is_empty());
    }

    #[test]
    fn global_singleton() {
        {
            let mut db = Database::open().lock().unwrap();
            db.store_setting("global_test".into(), "works".into());
        }
        {
            let db = Database::open().lock().unwrap();
            assert_eq!(db.get_setting("global_test"), Some("works"));
        }
    }
}
