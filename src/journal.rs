//! Journal ingestion — reads Elite Dangerous journal files and populates application state.
//!
//! Architecture:
//! - `discover_journal_dir` resolves the journal directory from CLI args or default Steam Proton path
//! - `replay_session` reads all journal files and processes events to reconstruct current state
//! - `process_event` is the single event handler shared by both replay and live watching
//! - `start_live_watcher` spawns a thread that feeds new events back to the main loop

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use ed_journals::logs::blocking::LiveLogDirReader;
use ed_journals::logs::{LogDir, LogEvent, LogEventContent};

use crate::app::App;
use crate::model::{Body, BodyType, ScanState, System};

/// Discover the journal directory, preferring an explicit path over the default.
pub fn discover_journal_dir(explicit_path: Option<&str>) -> Result<PathBuf, String> {
    if let Some(path) = explicit_path {
        let p = PathBuf::from(path);
        if p.is_dir() {
            return Ok(p);
        }
        return Err(format!("Journal path does not exist: {}", path));
    }

    // Default: Steam Proton path on Linux
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let default = home.join(
        ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/\
         compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/\
         Frontier Developments/Elite Dangerous",
    );

    if default.is_dir() {
        Ok(default)
    } else {
        Err(format!(
            "Default journal directory not found: {}\n\
             Use --journal-path <dir> to specify the journal location.",
            default.display()
        ))
    }
}

/// Replay all journal files from the directory, processing events to build current state.
/// Returns the number of events processed.
pub fn replay_session(app: &mut App, journal_dir: &Path) -> Result<u32, String> {
    let log_dir = LogDir::new(journal_dir.to_path_buf());
    let logs = log_dir
        .journal_logs_oldest_first()
        .map_err(|e| format!("Failed to read journal directory: {e}"))?;

    if logs.is_empty() {
        return Err(format!(
            "No journal files found in: {}",
            journal_dir.display()
        ));
    }

    let mut event_count: u32 = 0;

    for log_file in &logs {
        let reader = log_file
            .create_blocking_reader()
            .map_err(|e| format!("Failed to open journal file: {e}"))?;

        for entry_result in reader {
            match entry_result {
                Ok(event) => {
                    process_event(app, &event);
                    event_count += 1;
                }
                Err(e) => {
                    // Log parse errors but don't halt — some events may be
                    // unsupported by the crate's current version.
                    eprintln!("Warning: skipping unparseable journal entry: {e}");
                }
            }
        }
    }

    // Rebuild display hierarchy after processing all events.
    app.rebuild_display_order();

    Ok(event_count)
}

/// Event type sent from the live watcher thread to the main event loop.
pub enum JournalUpdate {
    /// A new journal event was received.
    Event(LogEvent),
    /// The watcher encountered an error.
    Error(String),
}

/// Spawn a background thread that watches the journal directory for new events.
/// Returns a receiver channel that the main loop can poll.
pub fn start_live_watcher(
    journal_dir: PathBuf,
) -> Result<mpsc::Receiver<JournalUpdate>, String> {
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("journal-watcher".into())
        .spawn(move || {
            let reader = match LiveLogDirReader::open(&journal_dir) {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(JournalUpdate::Error(format!(
                        "Failed to start journal watcher: {e}"
                    )));
                    return;
                }
            };

            for entry_result in reader {
                match entry_result {
                    Ok(event) => {
                        if tx.send(JournalUpdate::Event(event)).is_err() {
                            // Main thread dropped the receiver — exit quietly.
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: journal watcher error: {e}");
                    }
                }
            }
        })
        .map_err(|e| format!("Failed to spawn journal watcher thread: {e}"))?;

    Ok(rx)
}

/// Process a single journal event and update application state.
pub fn process_event(app: &mut App, event: &LogEvent) {
    match &event.content {
        // --- System transitions ---
        LogEventContent::FSDJump(e) => {
            let sys_name = e.system_info.star_system.clone();
            let sys_addr = e.system_info.system_address;
            app.system = System::new(sys_name, sys_addr);
            app.bodies.clear();
            app.body_display_order.clear();
            app.selected_body_index = 0;
            app.system.body_count_discovered = 0;

            // Increment trip counter
            app.trip.systems_visited += 1;
        }

        LogEventContent::Location(e) => {
            // Location fires on game load — set current system without incrementing trip.
            let sys_name = e.location_info.star_system.clone();
            let sys_addr = e.location_info.system_address;
            if app.system.system_address != sys_addr {
                app.system = System::new(sys_name, sys_addr);
                app.bodies.clear();
                app.body_display_order.clear();
                app.selected_body_index = 0;
            }
        }

        // --- Discovery scan (honk) ---
        LogEventContent::FSSDiscoveryScan(e) => {
            app.system.body_count_total = u32::from(e.body_count);
        }

        // --- Body scan (FSS detailed or auto scan) ---
        LogEventContent::Scan(e) => {
            let body_id = u32::from(e.body_id);

            let body = app.bodies.entry(body_id).or_insert_with(|| {
                let mut b = Body::new(body_id, e.body_name.clone());
                // Derive short name by stripping system name prefix
                b.short_name = strip_system_prefix(&e.body_name, &app.system.name);
                b
            });

            // Update scan state (only upgrade, never downgrade)
            if body.scan_state < ScanState::FSSScanned {
                body.scan_state = ScanState::FSSScanned;
                app.trip.bodies_scanned_fss += 1;
            }

            body.distance_ls = Some(e.distance_from_arrival.as_ls() as f64);
            body.was_discovered = e.was_discovered;
            body.was_mapped = e.was_mapped;

            // Track first discovery
            if !e.was_discovered && body.scan_state == ScanState::FSSScanned {
                app.trip.first_discoveries += 1;
            }

            // Determine body type and extract type-specific data
            match &e.kind {
                ed_journals::logs::scan_event::ScanEventKind::Star(star) => {
                    body.body_type = BodyType::Star;
                    body.mass = Some(star.stellar_mass as f64);
                    body.atmosphere = None;
                    body.terraformable = false;
                }
                ed_journals::logs::scan_event::ScanEventKind::Planet(planet) => {
                    // Determine if this is a planet or moon from parents
                    body.body_type = if has_planet_parent(&e.parents) {
                        BodyType::Moon
                    } else {
                        BodyType::Planet
                    };
                    body.mass = Some(planet.mass_em as f64);
                    body.atmosphere = Some(format!("{:?}", planet.atmosphere.kind));
                    body.terraformable = matches!(
                        planet.terraform_state,
                        ed_journals::galaxy::TerraformState::Terraformable
                            | ed_journals::galaxy::TerraformState::Terraforming
                    );
                }
                ed_journals::logs::scan_event::ScanEventKind::BeltCluster(_) => {
                    body.body_type = BodyType::BeltCluster;
                }
            }

            // Set parent_id from the journal's Parents array
            body.parent_id = extract_parent_id(&e.parents);

            // Update discovered count
            app.system.body_count_discovered = app.bodies.len() as u32;
        }

        // --- Body signals (bio/geo counts from FSS) ---
        LogEventContent::FSSBodySignals(e) => {
            let body_id = u32::from(e.body_id);
            let body = app
                .bodies
                .entry(body_id)
                .or_insert_with(|| Body::new(body_id, e.body_name.clone()));

            for signal in &e.signals {
                match signal.kind {
                    ed_journals::exploration::PlanetarySignalType::Biological => {
                        body.bio_signals = u32::from(signal.count);
                        app.trip.bio_detected += u32::from(signal.count);
                    }
                    ed_journals::exploration::PlanetarySignalType::Geological => {
                        body.geo_signals = u32::from(signal.count);
                    }
                    _ => {} // Human, Thargoid, Guardian, etc. — not tracked in Phase 1
                }
            }
        }

        // --- DSS mapping complete ---
        LogEventContent::SAAScanComplete(e) => {
            let body_id = u32::from(e.body_id);
            if let Some(body) = app.bodies.get_mut(&body_id) {
                if body.scan_state < ScanState::DSSMapped {
                    body.scan_state = ScanState::DSSMapped;
                    app.trip.bodies_mapped_dss += 1;

                    if !body.was_mapped {
                        app.trip.first_mappings += 1;
                    }
                }
            }
        }

        // --- DSS surface signals (can provide updated bio/geo counts) ---
        LogEventContent::SAASignalsFound(e) => {
            let body_id = u32::from(e.body_id);
            if let Some(body) = app.bodies.get_mut(&body_id) {
                for signal in &e.signals {
                    match signal.kind {
                        ed_journals::exploration::PlanetarySignalType::Biological => {
                            body.bio_signals = u32::from(signal.count);
                        }
                        ed_journals::exploration::PlanetarySignalType::Geological => {
                            body.geo_signals = u32::from(signal.count);
                        }
                        _ => {}
                    }
                }
            }
        }

        // All other events — ignored for Phase 1
        _ => {}
    }
}

/// Strip the system name prefix from a body name to get the short display name.
/// e.g., "Prudgeou VD-B e1 9" with system "Prudgeou VD-B e1" → "9"
fn strip_system_prefix(body_name: &str, system_name: &str) -> String {
    if body_name.starts_with(system_name) && body_name.len() > system_name.len() {
        body_name[system_name.len()..].trim().to_string()
    } else {
        body_name.to_string()
    }
}

/// Check if any parent in the chain is a planet (which makes this body a moon).
fn has_planet_parent(
    parents: &[ed_journals::logs::scan_event::ScanEventParent],
) -> bool {
    parents.iter().any(|p| {
        matches!(
            p,
            ed_journals::logs::scan_event::ScanEventParent::Planet(_)
        )
    })
}

/// Extract the immediate parent body_id from the Parents array.
/// The first entry is the immediate parent.
fn extract_parent_id(
    parents: &[ed_journals::logs::scan_event::ScanEventParent],
) -> Option<u32> {
    parents.first().map(|p| match p {
        ed_journals::logs::scan_event::ScanEventParent::Null(id) => u32::from(*id),
        ed_journals::logs::scan_event::ScanEventParent::Star(id) => u32::from(*id),
        ed_journals::logs::scan_event::ScanEventParent::Ring(id) => u32::from(*id),
        ed_journals::logs::scan_event::ScanEventParent::Planet(id) => u32::from(*id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_system_prefix_works() {
        assert_eq!(
            strip_system_prefix("Prudgeou VD-B e1 9", "Prudgeou VD-B e1"),
            "9"
        );
        assert_eq!(
            strip_system_prefix("Sol", "Sol"),
            "Sol" // Body name == system name → keep full name
        );
        assert_eq!(
            strip_system_prefix("Prudgeou VD-B e1 A 1 a", "Prudgeou VD-B e1"),
            "A 1 a"
        );
    }
}
