//! State snapshot persistence — instant cold-start restoration.
//!
//! Saves the current system and bodies to `~/.local/share/unixploration-buddy/state.json`
//! on graceful exit. On startup, loads the snapshot for immediate display, then
//! replays only new journal events to catch up.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{Body, System};

/// Serializable snapshot of the current session state.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshot {
    /// Current star system.
    pub system: System,
    /// All known bodies in the current system, keyed by body_id.
    pub bodies: HashMap<u32, Body>,
    /// Display order: `(body_id, depth)` pairs.
    pub body_display_order: Vec<(u32, u32)>,
    /// The journal file name (basename) that was last fully processed.
    pub last_journal_file: Option<String>,
    /// Number of events processed in the last journal file.
    pub last_journal_event_count: u32,
}

/// Manages state snapshot I/O.
pub struct StatePersistence {
    path: PathBuf,
}

impl StatePersistence {
    /// Create a new state persistence manager.
    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine XDG data directory")?
            .join("unixploration-buddy");

        Ok(Self {
            path: data_dir.join("state.json"),
        })
    }

    /// Load a saved snapshot. Returns `None` if no snapshot exists or it's malformed.
    pub fn load(&self) -> Option<StateSnapshot> {
        if !self.path.exists() {
            return None;
        }

        let contents = std::fs::read_to_string(&self.path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Save a snapshot to disk.
    pub fn save(&self, snapshot: &StateSnapshot) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create state directory: {e}"))?;
        }

        let json = serde_json::to_string(snapshot)
            .map_err(|e| format!("Failed to serialize state: {e}"))?;

        std::fs::write(&self.path, json)
            .map_err(|e| format!("Failed to write state file: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("ubtest_state_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let p = StatePersistence {
            path: dir.join("state.json"),
        };

        let mut system = System::new("Test System".to_string(), 12345);
        system.body_count_total = 5;
        system.body_count_discovered = 3;

        let mut bodies = HashMap::new();
        let body = Body::new(1, "Test A".to_string());
        bodies.insert(1, body);

        let snapshot = StateSnapshot {
            system,
            bodies,
            body_display_order: vec![(1, 0)],
            last_journal_file: Some("Journal.2026-06-10.log".to_string()),
            last_journal_event_count: 500,
        };

        assert!(p.save(&snapshot).is_ok());

        let loaded = p.load().expect("Should load snapshot");
        assert_eq!(loaded.system.name, "Test System");
        assert_eq!(loaded.system.system_address, 12345);
        assert_eq!(loaded.bodies.len(), 1);
        assert_eq!(loaded.body_display_order.len(), 1);
        assert_eq!(loaded.last_journal_event_count, 500);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_returns_none_when_no_file() {
        let p = StatePersistence {
            path: std::env::temp_dir().join("ubtest_no_state/state.json"),
        };
        assert!(p.load().is_none());
    }
}
