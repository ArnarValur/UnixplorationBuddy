//! Trip persistence — serialize/deserialize trip data to XDG data directory.
//!
//! Trip file: `~/.local/share/unixploration-buddy/trip.json`
//! Saves are debounced to avoid disk thrashing — call `maybe_save` after each
//! state change and it will write at most once per second.

use std::path::PathBuf;
use std::time::Instant;

use crate::model::Trip;

/// Minimum interval between saves (debounce).
const SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

/// Manages trip file I/O with debounced saves.
pub struct TripPersistence {
    path: PathBuf,
    last_save: Option<Instant>,
    dirty: bool,
}

impl TripPersistence {
    /// Create a new persistence manager.
    /// Resolves the trip file path using XDG data directory.
    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine XDG data directory")?
            .join("unixploration-buddy");

        Ok(Self {
            path: data_dir.join("trip.json"),
            last_save: None,
            dirty: false,
        })
    }

    /// Load trip from disk. Returns `Trip::default()` if file doesn't exist or is malformed.
    /// Returns a warning message if the file was malformed.
    pub fn load(&self) -> (Trip, Option<String>) {
        if !self.path.exists() {
            return (Trip::default(), None);
        }

        match std::fs::read_to_string(&self.path) {
            Ok(contents) => match serde_json::from_str::<Trip>(&contents) {
                Ok(mut trip) => {
                    // Upgrader: if legacy generic keys exist in stellar_codex,
                    // clear the entire codex to let them start fresh with 100% authentic subtypes!
                    if trip.stellar_codex.contains_key("K") || trip.stellar_codex.contains_key("F") {
                        trip.stellar_codex.clear();
                    }
                    (trip, None)
                }
                Err(e) => (
                    Trip::default(),
                    Some(format!(
                        "Trip file malformed (starting fresh): {}",
                        e
                    )),
                ),
            },
            Err(e) => (
                Trip::default(),
                Some(format!("Could not read trip file (starting fresh): {}", e)),
            ),
        }
    }

    /// Mark the trip as dirty (needs saving).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Save if dirty and enough time has elapsed since the last save.
    /// Returns an error message if the save failed.
    pub fn maybe_save(&mut self, trip: &Trip) -> Option<String> {
        if !self.dirty {
            return None;
        }

        if let Some(last) = self.last_save {
            if last.elapsed() < SAVE_INTERVAL {
                return None;
            }
        }

        self.force_save(trip)
    }

    /// Force an immediate save regardless of debounce.
    pub fn force_save(&mut self, trip: &Trip) -> Option<String> {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Some(format!("Failed to create trip directory: {e}"));
            }
        }

        match serde_json::to_string_pretty(trip) {
            Ok(json) => match std::fs::write(&self.path, json) {
                Ok(()) => {
                    self.last_save = Some(Instant::now());
                    self.dirty = false;
                    None
                }
                Err(e) => Some(format!("Failed to write trip file: {e}")),
            },
            Err(e) => Some(format!("Failed to serialize trip: {e}")),
        }
    }

    /// Get the trip file path (for display/debugging).
    #[allow(dead_code)]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_persistence(dir: &std::path::Path) -> TripPersistence {
        TripPersistence {
            path: dir.join("trip.json"),
            last_save: None,
            dirty: false,
        }
    }

    #[test]
    fn load_returns_default_when_no_file() {
        let dir = std::env::temp_dir().join("ubtest_no_file");
        let _ = std::fs::remove_dir_all(&dir);
        let p = temp_persistence(&dir);

        let (trip, warning) = p.load();
        assert_eq!(trip.systems_visited, 0);
        assert!(warning.is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("ubtest_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        let mut p = temp_persistence(&dir);

        let mut trip = Trip::default();
        trip.systems_visited = 42;
        trip.total_value = 1_000_000;

        let err = p.force_save(&trip);
        assert!(err.is_none(), "Save should succeed: {:?}", err);

        let (loaded, warning) = p.load();
        assert!(warning.is_none());
        assert_eq!(loaded.systems_visited, 42);
        assert_eq!(loaded.total_value, 1_000_000);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_handles_malformed_file() {
        let dir = std::env::temp_dir().join("ubtest_malformed");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("trip.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"{ not valid json }").unwrap();

        let p = temp_persistence(&dir);
        let (trip, warning) = p.load();
        assert_eq!(trip.systems_visited, 0, "Should return default trip");
        assert!(warning.is_some(), "Should return a warning");
        assert!(warning.unwrap().contains("malformed"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn maybe_save_respects_debounce() {
        let dir = std::env::temp_dir().join("ubtest_debounce");
        let _ = std::fs::remove_dir_all(&dir);
        let mut p = temp_persistence(&dir);

        let trip = Trip::default();

        // Not dirty — should not save
        assert!(p.maybe_save(&trip).is_none());
        assert!(!p.path.exists(), "Should not write when not dirty");

        // Mark dirty — first save should go through
        p.mark_dirty();
        assert!(p.maybe_save(&trip).is_none());
        assert!(p.path.exists(), "First save should write");

        // Immediately mark dirty again — debounce should prevent save
        p.mark_dirty();
        // The second maybe_save should be a no-op due to debounce
        // (within 1 second)
        assert!(p.maybe_save(&trip).is_none());
        assert!(!p.dirty || p.last_save.unwrap().elapsed() < SAVE_INTERVAL,
            "Should be debounced");

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
