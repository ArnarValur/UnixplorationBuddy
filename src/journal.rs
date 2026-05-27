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
use std::time::Duration;

use ed_journals::logs::blocking::LiveLogDirReader;
use ed_journals::logs::{LogDir, LogEvent, LogEventContent};

use crate::app::App;
use crate::model::naming::parse_body_name;
use crate::model::valuation::{calculate_planet_value, calculate_star_value};
use crate::model::{Body, BodyType, ScanState, System, Status, NavRoute};

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
                    process_event(app, &event, false);
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
    /// A real-time cockpit status update was parsed from Status.json.
    StatusUpdate(Status),
    /// A plotted navigation route update was parsed from NavRoute.json.
    NavRouteUpdate(NavRoute),
    /// The watcher encountered an error.
    Error(String),
}

/// Spawn background threads that watch the journal directory for new events and status/route updates.
/// Returns a receiver channel that the main loop can poll.
pub fn start_live_watcher(
    journal_dir: PathBuf,
) -> Result<mpsc::Receiver<JournalUpdate>, String> {
    let (tx, rx) = mpsc::channel();

    // Spawn journal watcher thread
    let tx_journal = tx.clone();
    let journal_dir_copy1 = journal_dir.clone();
    thread::Builder::new()
        .name("journal-watcher".into())
        .spawn(move || {
            let reader = match LiveLogDirReader::open(&journal_dir_copy1) {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx_journal.send(JournalUpdate::Error(format!(
                        "Failed to start journal watcher: {e}"
                    )));
                    return;
                }
            };

            for entry_result in reader {
                match entry_result {
                    Ok(event) => {
                        if tx_journal.send(JournalUpdate::Event(event)).is_err() {
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

    // Spawn Status.json and NavRoute.json watcher thread
    let tx_status_nav = tx.clone();
    let journal_dir_copy2 = journal_dir.clone();
    thread::Builder::new()
        .name("status-nav-watcher".into())
        .spawn(move || {
            watch_status_and_nav_route(journal_dir_copy2, tx_status_nav);
        })
        .map_err(|e| format!("Failed to spawn status/nav watcher thread: {e}"))?;

    Ok(rx)
}

/// Periodically poll Status.json and NavRoute.json for changes.
fn watch_status_and_nav_route(journal_dir: PathBuf, tx: mpsc::Sender<JournalUpdate>) {
    let status_path = journal_dir.join("Status.json");
    let nav_route_path = journal_dir.join("NavRoute.json");

    let mut last_status_mtime = None;
    let mut last_nav_route_mtime = None;

    loop {
        thread::sleep(Duration::from_millis(250));

        // Check Status.json
        if let Ok(metadata) = std::fs::metadata(&status_path) {
            if let Ok(mtime) = metadata.modified() {
                if last_status_mtime != Some(mtime) {
                    last_status_mtime = Some(mtime);
                    if let Ok(content) = std::fs::read_to_string(&status_path) {
                        if !content.trim().is_empty() {
                            match serde_json::from_str::<Status>(&content) {
                                Ok(status) => {
                                    if tx.send(JournalUpdate::StatusUpdate(status)).is_err() {
                                        break; // Channel closed, exit thread
                                    }
                                }
                                Err(e) => {
                                    // Gracefully ignore half-written json
                                    eprintln!("Warning: failed to parse Status.json: {e}");
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check NavRoute.json
        if let Ok(metadata) = std::fs::metadata(&nav_route_path) {
            if let Ok(mtime) = metadata.modified() {
                if last_nav_route_mtime != Some(mtime) {
                    last_nav_route_mtime = Some(mtime);
                    if let Ok(content) = std::fs::read_to_string(&nav_route_path) {
                        if !content.trim().is_empty() {
                            match serde_json::from_str::<NavRoute>(&content) {
                                Ok(nav_route) => {
                                    if tx.send(JournalUpdate::NavRouteUpdate(nav_route)).is_err() {
                                        break; // Channel closed, exit thread
                                    }
                                }
                                Err(e) => {
                                    // Gracefully ignore half-written json
                                    eprintln!("Warning: failed to parse NavRoute.json: {e}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Process a single journal event and update application state.
///
/// When `track_trip` is `true`, trip counters are incremented (live events).
/// When `false`, only system/body state is reconstructed (replay on startup).
pub fn process_event(app: &mut App, event: &LogEvent, track_trip: bool) {
    match &event.content {
        // --- System transitions ---
        LogEventContent::FSDJump(e) => {
            if track_trip {
                // Bank the departing system's value into the trip total
                app.trip.total_value += app.system.total_value;
                app.trip.systems_visited += 1;
            }

            let sys_name = e.system_info.star_system.clone();
            let sys_addr = e.system_info.system_address;
            app.system = System::new(sys_name, sys_addr);
            app.bodies.clear();
            app.body_display_order.clear();
            app.selected_body_index = 0;
            app.system.body_count_discovered = 0;
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
                // Parse naming convention for hierarchy sort ordering
                let pos = parse_body_name(&b.short_name);
                b.sort_key = pos.sort_key;
                b
            });

            // Update scan state (only upgrade, never downgrade)
            if body.scan_state < ScanState::FSSScanned {
                body.scan_state = ScanState::FSSScanned;
                if track_trip {
                    app.trip.bodies_scanned_fss += 1;
                }
            }

            body.distance_ls = Some(e.distance_from_arrival.as_ls() as f64);
            body.was_discovered = e.was_discovered;
            body.was_mapped = e.was_mapped;

            // Track first discovery
            if track_trip && !e.was_discovered && body.scan_state == ScanState::FSSScanned {
                app.trip.first_discoveries += 1;
            }

            // Determine body type, extract data, and calculate exploration value
            match &e.kind {
                ed_journals::logs::scan_event::ScanEventKind::Star(star) => {
                    body.body_type = BodyType::Star;
                    body.mass = Some(star.stellar_mass as f64);
                    body.atmosphere = None;
                    body.terraformable = false;
                    body.star_type = Some(format!("{}", star.star_type));
                    body.star_class_enum = Some(star.star_type.clone());
                    body.temperature = Some(star.surface_temperature as f64);

                    let first_disc = !body.was_discovered;
                    let value = calculate_star_value(
                        &star.star_type,
                        star.stellar_mass as f64,
                        first_disc,
                    );
                    body.calculated_value = value.fss_value;
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
                    body.planet_class = Some(format!("{}", planet.planet_class));
                    body.planet_class_enum = Some(planet.planet_class.clone());
                    body.gravity = Some(planet.surface_gravity.0 as f64);
                    body.temperature = Some(planet.surface_temperature as f64);
                    body.landable = planet.landable;

                    let first_disc = !body.was_discovered;
                    let first_map = !body.was_mapped;
                    let value = calculate_planet_value(
                        &planet.planet_class,
                        planet.mass_em as f64,
                        body.terraformable,
                        first_disc,
                        first_map,
                        false, // efficiency unknown until DSS
                    );
                    body.calculated_value = value.fss_value;
                    body.mapped_value = value.mapped_value;
                }
                ed_journals::logs::scan_event::ScanEventKind::BeltCluster(_) => {
                    body.body_type = BodyType::BeltCluster;
                }
            }

            // Set parent_id from the journal's Parents array
            body.parent_id = extract_parent_id(&e.parents);

            // Update discovered count
            app.system.body_count_discovered = app.bodies.len() as u32;

            // Recompute system total value
            app.system.total_value = aggregate_system_value(&app.bodies);
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
                        if track_trip {
                            app.trip.bio_detected += u32::from(signal.count);
                        }
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
                    if track_trip {
                        app.trip.bodies_mapped_dss += 1;

                        if !body.was_mapped {
                            app.trip.first_mappings += 1;
                        }
                    }
                }

                // Track probe efficiency and recalculate mapped value
                let efficient = e.probes_used <= e.efficiency_target;
                body.probes_efficient = efficient;

                if let Some(ref pc) = body.planet_class_enum {
                    let first_disc = !body.was_discovered;
                    let first_map = !body.was_mapped;
                    let mass = body.mass.unwrap_or(1.0);
                    let value = calculate_planet_value(
                        pc,
                        mass,
                        body.terraformable,
                        first_disc,
                        first_map,
                        efficient,
                    );
                    body.mapped_value = value.mapped_value;
                }
            }

            // Recompute system total value
            app.system.total_value = aggregate_system_value(&app.bodies);
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

        LogEventContent::ScanOrganic(e) => {
            if track_trip {
                if matches!(e.scan_type, ed_journals::logs::scan_organic_event::ScanOrganicEventScanType::Analyse) {
                    app.trip.bio_analysed += 1;
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

/// Aggregate total system value from all discovered bodies.
/// Uses mapped_value for DSS'd bodies (higher payout), calculated_value (FSS) for others.
fn aggregate_system_value(bodies: &std::collections::HashMap<u32, Body>) -> u64 {
    bodies.values().map(|b| {
        if b.scan_state >= ScanState::DSSMapped && b.mapped_value > 0 {
            b.mapped_value
        } else {
            b.calculated_value
        }
    }).sum()
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
    use crate::app::App;
    use ed_journals::logs::LogEvent;

    /// Parse a journal JSON line into a LogEvent.
    fn parse_event(json: &str) -> LogEvent {
        serde_json::from_str(json).expect("Failed to parse test journal event")
    }

    // ---------------------------------------------------------------
    // Fixtures: real journal JSON from actual game sessions
    // ---------------------------------------------------------------

    const FSDJUMP_JSON: &str = r#"{ "timestamp":"2026-05-26T15:57:40Z", "event":"FSDJump", "Taxi":false, "Multicrew":false, "StarSystem":"Prudgeou VD-B e1", "SystemAddress":4997497796, "StarPos":[-23117.25,-209.53125,-4787.25], "SystemAllegiance":"", "SystemEconomy":"$economy_None;", "SystemEconomy_Localised":"None", "SystemSecondEconomy":"$economy_None;", "SystemSecondEconomy_Localised":"None", "SystemGovernment":"$government_None;", "SystemGovernment_Localised":"None", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;", "SystemSecurity_Localised":"Anarchy", "Population":0, "Body":"Prudgeou VD-B e1", "BodyID":0, "BodyType":"Star", "JumpDist":75.834, "FuelUsed":4.144365, "FuelLevel":11.855635 }"#;

    const LOCATION_JSON: &str = r#"{ "timestamp":"2026-05-26T15:55:51Z", "event":"Location", "DistFromStarLS":278.271102, "Docked":false, "Taxi":false, "Multicrew":false, "StarSystem":"Prudgaei OD-B e0", "SystemAddress":706724804, "StarPos":[-23060.6875,-200.96875,-4837.03125], "SystemAllegiance":"", "SystemEconomy":"$economy_None;", "SystemEconomy_Localised":"None", "SystemSecondEconomy":"$economy_None;", "SystemSecondEconomy_Localised":"None", "SystemGovernment":"$government_None;", "SystemGovernment_Localised":"None", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;", "SystemSecurity_Localised":"Anarchy", "Population":0, "Body":"Prudgaei OD-B e0 AB", "BodyID":0, "BodyType":"Null" }"#;

    const FSSDISCOVERYSCAN_JSON: &str = r#"{ "timestamp":"2026-05-26T15:57:00Z", "event":"FSSDiscoveryScan", "Progress":1.0, "BodyCount":2, "NonBodyCount":0, "SystemName":"Prudgaei OD-B e0", "SystemAddress":706724804 }"#;

    const SCAN_STAR_JSON: &str = r#"{ "timestamp":"2026-05-26T15:57:45Z", "event":"Scan", "ScanType":"AutoScan", "BodyName":"Prudgeou VD-B e1", "BodyID":0, "StarSystem":"Prudgeou VD-B e1", "SystemAddress":4997497796, "DistanceFromArrivalLS":0.0, "StarType":"B", "Subclass":9, "StellarMass":3.183594, "Radius":1481800320.0, "AbsoluteMagnitude":0.313232, "Age_MY":332, "SurfaceTemperature":11197.0, "Luminosity":"Vab", "RotationPeriod":63915.422108, "AxialTilt":0.0, "WasDiscovered":false, "WasMapped":false, "WasFootfalled":false }"#;

    const SCAN_PLANET_JSON: &str = r#"{ "timestamp":"2026-05-26T15:58:08Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Prudgeou VD-B e1 9", "BodyID":61, "Parents":[ {"Null":52}, {"Star":0} ], "StarSystem":"Prudgeou VD-B e1", "SystemAddress":4997497796, "DistanceFromArrivalLS":3378.469848, "TidalLock":false, "TerraformState":"", "PlanetClass":"High metal content body", "Atmosphere":"hot thick water atmosphere", "AtmosphereType":"Water", "AtmosphereComposition":[ { "Name":"Water", "Percent":94.319878 }, { "Name":"CarbonDioxide", "Percent":5.07096 }, { "Name":"Ammonia", "Percent":0.168935 } ], "Volcanism":"major silicate vapour geysers volcanism", "MassEM":12.588833, "Radius":12146220.0, "SurfaceGravity":34.01046, "SurfaceTemperature":2282.107422, "SurfacePressure":3268144640.0, "Landable":false, "Composition":{ "Ice":0.0, "Rock":0.671181, "Metal":0.328819 }, "SemiMajorAxis":37443051338.195801, "Eccentricity":0.057579, "OrbitalInclination":-2.784278, "Periapsis":115.153042, "OrbitalPeriod":48399391.770363, "AscendingNode":-28.509869, "MeanAnomaly":30.528101, "RotationPeriod":155371.757687, "AxialTilt":0.230221, "WasDiscovered":false, "WasMapped":false, "WasFootfalled":false }"#;

    const SCAN_MOON_JSON: &str = r#"{ "timestamp":"2026-05-26T15:58:31Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Prudgeou VD-B e1 8 a", "BodyID":56, "Parents":[ {"Planet":53}, {"Null":52}, {"Star":0} ], "StarSystem":"Prudgeou VD-B e1", "SystemAddress":4997497796, "DistanceFromArrivalLS":3327.220644, "TidalLock":true, "TerraformState":"", "PlanetClass":"Rocky body", "Atmosphere":"", "AtmosphereType":"None", "Volcanism":"", "MassEM":0.002607, "Radius":966100.625, "SurfaceGravity":1.113483, "SurfaceTemperature":352.186981, "SurfacePressure":0.0, "Landable":true, "Materials":[ { "Name":"iron", "Percent":19.464247 } ], "Composition":{ "Ice":0.0, "Rock":0.910762, "Metal":0.089238 }, "SemiMajorAxis":1042846441.268921, "Eccentricity":0.000917, "OrbitalInclination":-0.059785, "Periapsis":25.16838, "OrbitalPeriod":223709.55348, "AscendingNode":-104.957454, "MeanAnomaly":63.712812, "RotationPeriod":217485.66769, "AxialTilt":0.266921, "WasDiscovered":false, "WasMapped":false, "WasFootfalled":false }"#;

    const FSSBODYSIGNALS_BIO_JSON: &str = r#"{ "timestamp":"2026-05-26T11:15:38Z", "event":"FSSBodySignals", "BodyName":"Hypua Phroo QU-O e6-1 CDE 2", "BodyID":25, "SystemAddress":5001692092, "Signals":[ { "Type":"$SAA_SignalType_Biological;", "Type_Localised":"Biological", "Count":1 } ] }"#;

    const FSSBODYSIGNALS_GEO_JSON: &str = r#"{ "timestamp":"2026-05-26T15:59:45Z", "event":"FSSBodySignals", "BodyName":"Prudgeou VD-B e1 5 c", "BodyID":27, "SystemAddress":4997497796, "Signals":[ { "Type":"$SAA_SignalType_Geological;", "Type_Localised":"Geological", "Count":3 } ] }"#;

    const SAASCANCOMPLETE_JSON: &str = r#"{ "timestamp":"2026-05-26T16:09:16Z", "event":"SAAScanComplete", "BodyName":"Prudgeou VD-B e1 1", "SystemAddress":4997497796, "BodyID":1, "ProbesUsed":6, "EfficiencyTarget":7 }"#;

    /// Inefficient DSS: probes_used > efficiency_target
    const SAASCANCOMPLETE_INEFFICIENT_JSON: &str = r#"{ "timestamp":"2026-05-26T16:10:00Z", "event":"SAAScanComplete", "BodyName":"Prudgeou VD-B e1 1", "SystemAddress":4997497796, "BodyID":1, "ProbesUsed":10, "EfficiencyTarget":7 }"#;

    const SAASIGNALSFOUND_JSON: &str = r#"{ "timestamp":"2026-05-26T16:09:16Z", "event":"SAASignalsFound", "BodyName":"Prudgeou VD-B e1 1", "SystemAddress":4997497796, "BodyID":1, "Signals":[ { "Type":"$SAA_SignalType_Geological;", "Type_Localised":"Geological", "Count":2 } ], "Genuses":[] }"#;

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    fn app_with_system(name: &str, addr: u64) -> App {
        let mut app = App::new();
        app.system = System::new(name.into(), addr);
        app
    }

    fn app_in_prudgeou() -> App {
        app_with_system("Prudgeou VD-B e1", 4997497796)
    }

    // ---------------------------------------------------------------
    // strip_system_prefix (existing)
    // ---------------------------------------------------------------

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

    // ---------------------------------------------------------------
    // FSDJump
    // ---------------------------------------------------------------

    #[test]
    fn fsdjump_resets_system_and_clears_bodies() {
        let mut app = app_with_system("Old System", 111);
        app.bodies.insert(1, Body::new(1, "Old Body".into()));
        app.body_display_order.push((1, 0));
        app.selected_body_index = 5;

        let event = parse_event(FSDJUMP_JSON);
        process_event(&mut app, &event, false);

        assert_eq!(app.system.name, "Prudgeou VD-B e1");
        assert_eq!(app.system.system_address, 4997497796);
        assert!(app.bodies.is_empty(), "Bodies should be cleared on jump");
        assert!(app.body_display_order.is_empty());
        assert_eq!(app.selected_body_index, 0);
        assert_eq!(app.system.body_count_discovered, 0);
    }

    #[test]
    fn fsdjump_increments_trip_only_when_live() {
        let mut app = App::new();
        let event = parse_event(FSDJUMP_JSON);

        // Replay: trip should NOT increment
        process_event(&mut app, &event, false);
        assert_eq!(app.trip.systems_visited, 0);

        // Live: trip SHOULD increment
        process_event(&mut app, &event, true);
        assert_eq!(app.trip.systems_visited, 1);
    }

    // ---------------------------------------------------------------
    // Location
    // ---------------------------------------------------------------

    #[test]
    fn location_sets_system_without_trip_increment() {
        let mut app = App::new();
        let event = parse_event(LOCATION_JSON);

        process_event(&mut app, &event, true); // even with live=true
        assert_eq!(app.system.name, "Prudgaei OD-B e0");
        assert_eq!(app.system.system_address, 706724804);
        assert_eq!(app.trip.systems_visited, 0, "Location should not increment trip");
    }

    #[test]
    fn location_clears_bodies_when_system_changes() {
        let mut app = app_with_system("Different System", 999);
        app.bodies.insert(1, Body::new(1, "Old Body".into()));

        let event = parse_event(LOCATION_JSON);
        process_event(&mut app, &event, false);

        assert!(app.bodies.is_empty(), "Bodies should be cleared when system changes");
    }

    #[test]
    fn location_preserves_bodies_when_same_system() {
        let mut app = app_with_system("Prudgaei OD-B e0", 706724804);
        app.bodies.insert(1, Body::new(1, "Existing Body".into()));

        let event = parse_event(LOCATION_JSON);
        process_event(&mut app, &event, false);

        assert_eq!(app.bodies.len(), 1, "Bodies should be preserved for same system");
    }

    // ---------------------------------------------------------------
    // FSSDiscoveryScan (honk)
    // ---------------------------------------------------------------

    #[test]
    fn fssdiscoveryscan_sets_body_count_total() {
        let mut app = App::new();
        let event = parse_event(FSSDISCOVERYSCAN_JSON);

        process_event(&mut app, &event, false);
        assert_eq!(app.system.body_count_total, 2);
    }

    // ---------------------------------------------------------------
    // Scan — Star
    // ---------------------------------------------------------------

    #[test]
    fn scan_creates_star_with_correct_fields() {
        let mut app = app_in_prudgeou();
        let event = parse_event(SCAN_STAR_JSON);

        process_event(&mut app, &event, false);

        let body = app.bodies.get(&0).expect("Star body should exist");
        assert_eq!(body.body_type, BodyType::Star);
        assert_eq!(body.scan_state, ScanState::FSSScanned);
        assert!((body.mass.unwrap() - 3.183594).abs() < 0.001, "Star mass should be solar masses");
        assert!(body.atmosphere.is_none(), "Stars have no atmosphere");
        assert!(!body.terraformable);
        assert_eq!(body.distance_ls.unwrap(), 0.0);
        assert_eq!(body.temperature.unwrap(), 11197.0);
        assert!(body.gravity.is_none());
        assert!(!body.landable);
    }

    // ---------------------------------------------------------------
    // Scan — Planet
    // ---------------------------------------------------------------

    #[test]
    fn scan_creates_planet_with_correct_fields() {
        let mut app = app_in_prudgeou();
        let event = parse_event(SCAN_PLANET_JSON);

        process_event(&mut app, &event, false);

        let body = app.bodies.get(&61).expect("Planet body should exist");
        assert_eq!(body.body_type, BodyType::Planet);
        assert_eq!(body.scan_state, ScanState::FSSScanned);
        assert_eq!(body.short_name, "9");
        assert!((body.mass.unwrap() - 12.588833).abs() < 0.001);
        assert!(body.atmosphere.is_some());
        assert!(!body.terraformable, "Empty TerraformState = not terraformable");
        assert!((body.distance_ls.unwrap() - 3378.469848).abs() < 0.01);
        assert!(!body.was_discovered);
        assert!(!body.was_mapped);
        assert!((body.gravity.unwrap() - 34.01046).abs() < 0.0001);
        assert!((body.temperature.unwrap() - 2282.107422).abs() < 0.0001);
        assert!(!body.landable);
    }

    // ---------------------------------------------------------------
    // Scan — Moon (has Planet parent)
    // ---------------------------------------------------------------

    #[test]
    fn scan_detects_moon_via_planet_parent() {
        let mut app = app_in_prudgeou();
        let event = parse_event(SCAN_MOON_JSON);

        process_event(&mut app, &event, false);

        let body = app.bodies.get(&56).expect("Moon body should exist");
        assert_eq!(body.body_type, BodyType::Moon, "Body with Planet parent should be Moon");
        assert_eq!(body.short_name, "8 a");
        assert_eq!(body.parent_id, Some(53), "Immediate parent should be the planet");
        assert!((body.gravity.unwrap() - 1.113483).abs() < 0.0001);
        assert!((body.temperature.unwrap() - 352.186981).abs() < 0.0001);
        assert!(body.landable);
    }

    // ---------------------------------------------------------------
    // Scan — state transitions
    // ---------------------------------------------------------------

    #[test]
    fn scan_state_only_upgrades_never_downgrades() {
        let mut app = app_in_prudgeou();

        // First scan → FSSScanned
        let event = parse_event(SCAN_STAR_JSON);
        process_event(&mut app, &event, false);
        assert_eq!(app.bodies.get(&0).unwrap().scan_state, ScanState::FSSScanned);

        // Scanning again should NOT change state
        process_event(&mut app, &event, false);
        assert_eq!(app.bodies.get(&0).unwrap().scan_state, ScanState::FSSScanned);
    }

    #[test]
    fn scan_tracks_first_discovery_only_when_live() {
        let mut app = app_in_prudgeou();
        // SCAN_STAR_JSON has WasDiscovered:false
        let event = parse_event(SCAN_STAR_JSON);

        // Replay: no trip tracking
        process_event(&mut app, &event, false);
        assert_eq!(app.trip.first_discoveries, 0);

        // Reset for live test — need a fresh body
        app.bodies.clear();
        process_event(&mut app, &event, true);
        assert_eq!(app.trip.first_discoveries, 1);
    }

    #[test]
    fn scan_fss_increments_trip_only_when_live() {
        let mut app = app_in_prudgeou();
        let event = parse_event(SCAN_PLANET_JSON);

        // Replay
        process_event(&mut app, &event, false);
        assert_eq!(app.trip.bodies_scanned_fss, 0);

        // Live — need fresh body
        app.bodies.clear();
        process_event(&mut app, &event, true);
        assert_eq!(app.trip.bodies_scanned_fss, 1);
    }

    // ---------------------------------------------------------------
    // FSSBodySignals
    // ---------------------------------------------------------------

    #[test]
    fn fssbodysignals_sets_bio_count() {
        let mut app = App::new();
        let event = parse_event(FSSBODYSIGNALS_BIO_JSON);

        process_event(&mut app, &event, false);

        let body = app.bodies.get(&25).expect("Body should be created");
        assert_eq!(body.bio_signals, 1);
        assert_eq!(body.geo_signals, 0);
    }

    #[test]
    fn fssbodysignals_sets_geo_count() {
        let mut app = App::new();
        let event = parse_event(FSSBODYSIGNALS_GEO_JSON);

        process_event(&mut app, &event, false);

        let body = app.bodies.get(&27).expect("Body should be created");
        assert_eq!(body.bio_signals, 0);
        assert_eq!(body.geo_signals, 3);
    }

    #[test]
    fn fssbodysignals_bio_increments_trip_only_when_live() {
        let mut app = App::new();
        let event = parse_event(FSSBODYSIGNALS_BIO_JSON);

        process_event(&mut app, &event, false);
        assert_eq!(app.trip.bio_detected, 0, "Replay should not count bio");

        // Can't easily test live here since the body already exists
        // and bio_signals is set (not additive). Test with a new app.
        let mut app2 = App::new();
        process_event(&mut app2, &event, true);
        assert_eq!(app2.trip.bio_detected, 1);
    }

    // ---------------------------------------------------------------
    // SAAScanComplete
    // ---------------------------------------------------------------

    #[test]
    fn saascancomplete_upgrades_to_dss_mapped() {
        let mut app = app_in_prudgeou();

        // Pre-create the body with FSSScanned state (as would happen in real flow)
        let mut body = Body::new(1, "Prudgeou VD-B e1 1".into());
        body.scan_state = ScanState::FSSScanned;
        body.was_mapped = false;
        app.bodies.insert(1, body);

        let event = parse_event(SAASCANCOMPLETE_JSON);
        process_event(&mut app, &event, true);

        let body = app.bodies.get(&1).unwrap();
        assert_eq!(body.scan_state, ScanState::DSSMapped);
        assert_eq!(app.trip.bodies_mapped_dss, 1);
        assert_eq!(app.trip.first_mappings, 1, "was_mapped=false → first mapping");
    }

    #[test]
    fn saascancomplete_trip_only_when_live() {
        let mut app = app_in_prudgeou();
        let mut body = Body::new(1, "Prudgeou VD-B e1 1".into());
        body.scan_state = ScanState::FSSScanned;
        app.bodies.insert(1, body);

        let event = parse_event(SAASCANCOMPLETE_JSON);
        process_event(&mut app, &event, false);

        assert_eq!(app.bodies.get(&1).unwrap().scan_state, ScanState::DSSMapped,
            "State should upgrade even during replay");
        assert_eq!(app.trip.bodies_mapped_dss, 0, "Trip should NOT count during replay");
        assert_eq!(app.trip.first_mappings, 0);
    }

    #[test]
    fn saascancomplete_ignores_unknown_body() {
        let mut app = App::new();
        // No body with ID 1 exists — SAAScanComplete should be silently ignored
        let event = parse_event(SAASCANCOMPLETE_JSON);
        process_event(&mut app, &event, true);
        assert_eq!(app.trip.bodies_mapped_dss, 0);
    }

    #[test]
    fn saascancomplete_sets_probes_efficient_when_optimal() {
        let mut app = app_in_prudgeou();

        // Scan first to populate planet_class_enum and mass
        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);

        // DSS with probes_used=6 <= efficiency_target=7
        let event = parse_event(SAASCANCOMPLETE_JSON);
        // Insert body 1 for the SAAScanComplete event (body_id=1 in fixture)
        let body_61 = app.bodies.get(&61).unwrap().clone();
        let mut body_1 = body_61;
        body_1.body_id = 1;
        body_1.scan_state = ScanState::FSSScanned;
        app.bodies.insert(1, body_1);

        process_event(&mut app, &event, false);

        let body = app.bodies.get(&1).unwrap();
        assert!(body.probes_efficient, "probes_used(6) <= efficiency_target(7) should be efficient");
        assert!(body.mapped_value > 0, "Mapped value should be recalculated");
    }

    #[test]
    fn saascancomplete_inefficient_probes_no_bonus() {
        let mut app = app_in_prudgeou();

        // Scan to populate planet data
        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);

        // Create body for DSS with same planet data
        let body_61 = app.bodies.get(&61).unwrap().clone();
        let mut body_eff = body_61.clone();
        body_eff.body_id = 1;
        body_eff.scan_state = ScanState::FSSScanned;
        app.bodies.insert(1, body_eff);

        // Efficient mapping
        process_event(&mut app, &parse_event(SAASCANCOMPLETE_JSON), false);
        let efficient_value = app.bodies.get(&1).unwrap().mapped_value;

        // Reset for inefficient test
        let mut body_ineff = body_61;
        body_ineff.body_id = 1;
        body_ineff.scan_state = ScanState::FSSScanned;
        app.bodies.insert(1, body_ineff);

        // Inefficient mapping (probes_used=10 > efficiency_target=7)
        process_event(&mut app, &parse_event(SAASCANCOMPLETE_INEFFICIENT_JSON), false);
        let body = app.bodies.get(&1).unwrap();
        assert!(!body.probes_efficient, "probes_used(10) > efficiency_target(7) should be inefficient");
        assert!(body.mapped_value < efficient_value,
            "Inefficient mapping ({}) should be less than efficient ({})",
            body.mapped_value, efficient_value);
    }

    #[test]
    fn saascancomplete_recalculates_mapped_value_with_efficiency() {
        use crate::model::valuation::calculate_planet_value;

        let mut app = app_in_prudgeou();

        // Scan the planet to get real data
        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);
        let planet = app.bodies.get(&61).unwrap();
        let pc = planet.planet_class_enum.clone().unwrap();
        let mass = planet.mass.unwrap();
        let tf = planet.terraformable;

        // Calculate expected efficient value
        let expected_eff = calculate_planet_value(&pc, mass, tf, true, true, true);

        // Create body for DSS
        let mut body = app.bodies.get(&61).unwrap().clone();
        body.body_id = 1;
        body.scan_state = ScanState::FSSScanned;
        app.bodies.insert(1, body);

        // Efficient DSS
        process_event(&mut app, &parse_event(SAASCANCOMPLETE_JSON), false);
        assert_eq!(app.bodies.get(&1).unwrap().mapped_value, expected_eff.mapped_value,
            "Efficient mapped value should match valuation calculation");
    }

    // ---------------------------------------------------------------
    // System value aggregation
    // ---------------------------------------------------------------

    #[test]
    fn system_total_value_aggregates_body_values() {
        let mut app = app_in_prudgeou();

        // Scan two bodies
        process_event(&mut app, &parse_event(SCAN_STAR_JSON), false);
        let star_value = app.bodies.get(&0).unwrap().calculated_value;

        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);
        let planet_value = app.bodies.get(&61).unwrap().calculated_value;

        assert_eq!(app.system.total_value, star_value + planet_value,
            "System value should be sum of body values");
        assert!(app.system.total_value > 0, "Total value should be non-zero");
    }

    #[test]
    fn system_total_value_uses_mapped_value_for_dss_bodies() {
        let mut app = app_in_prudgeou();

        // Scan planet
        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);
        let fss_only_total = app.system.total_value;

        // DSS the planet (body_id 61 → need fixture with body_id 61)
        // Use a direct body mutation to set up for DSS
        let planet = app.bodies.get_mut(&61).unwrap();
        planet.body_id = 61;

        // Create SAAScanComplete for body 61
        let dss_json = r#"{ "timestamp":"2026-05-26T16:09:16Z", "event":"SAAScanComplete", "BodyName":"Prudgeou VD-B e1 9", "SystemAddress":4997497796, "BodyID":61, "ProbesUsed":5, "EfficiencyTarget":7 }"#;
        process_event(&mut app, &parse_event(dss_json), false);

        let mapped_total = app.system.total_value;
        assert!(mapped_total > fss_only_total,
            "Total after DSS ({}) should exceed FSS-only ({})",
            mapped_total, fss_only_total);
    }

    #[test]
    fn system_total_value_resets_on_fsdjump() {
        let mut app = app_in_prudgeou();

        // Scan a body to have non-zero value
        process_event(&mut app, &parse_event(SCAN_STAR_JSON), false);
        assert!(app.system.total_value > 0);

        // Jump to new system — resets
        process_event(&mut app, &parse_event(FSDJUMP_JSON), false);
        assert_eq!(app.system.total_value, 0, "System value should reset on jump");
    }

    // ---------------------------------------------------------------
    // SAASignalsFound
    // ---------------------------------------------------------------

    #[test]
    fn saasignalsfound_updates_geo_count() {
        let mut app = app_in_prudgeou();
        app.bodies.insert(1, Body::new(1, "Prudgeou VD-B e1 1".into()));

        let event = parse_event(SAASIGNALSFOUND_JSON);
        process_event(&mut app, &event, false);

        let body = app.bodies.get(&1).unwrap();
        assert_eq!(body.geo_signals, 2);
    }

    #[test]
    fn saasignalsfound_ignores_unknown_body() {
        let mut app = App::new();
        let event = parse_event(SAASIGNALSFOUND_JSON);
        process_event(&mut app, &event, false);
        // Should not panic or create a body
        assert!(app.bodies.is_empty());
    }

    // ---------------------------------------------------------------
    // Trip gating: comprehensive
    // ---------------------------------------------------------------

    #[test]
    fn replay_mode_never_touches_trip_counters() {
        let mut app = App::new();

        // Simulate a full replay sequence
        process_event(&mut app, &parse_event(FSDJUMP_JSON), false);
        process_event(&mut app, &parse_event(FSSDISCOVERYSCAN_JSON), false);
        process_event(&mut app, &parse_event(SCAN_STAR_JSON), false);
        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);
        process_event(&mut app, &parse_event(FSSBODYSIGNALS_BIO_JSON), false);

        // Pre-create body for DSS
        let mut b = Body::new(1, "Test".into());
        b.scan_state = ScanState::FSSScanned;
        app.bodies.insert(1, b);
        process_event(&mut app, &parse_event(SAASCANCOMPLETE_JSON), false);

        // ALL trip counters should be 0
        assert_eq!(app.trip.systems_visited, 0);
        assert_eq!(app.trip.bodies_scanned_fss, 0);
        assert_eq!(app.trip.bodies_mapped_dss, 0);
        assert_eq!(app.trip.first_discoveries, 0);
        assert_eq!(app.trip.first_mappings, 0);
        assert_eq!(app.trip.bio_detected, 0);
    }

    // ---------------------------------------------------------------
    // Body discovery count tracking
    // ---------------------------------------------------------------

    #[test]
    fn body_count_discovered_tracks_body_map_size() {
        let mut app = app_in_prudgeou();

        process_event(&mut app, &parse_event(SCAN_STAR_JSON), false);
        assert_eq!(app.system.body_count_discovered, 1);

        process_event(&mut app, &parse_event(SCAN_PLANET_JSON), false);
        assert_eq!(app.system.body_count_discovered, 2);

        process_event(&mut app, &parse_event(SCAN_MOON_JSON), false);
        assert_eq!(app.system.body_count_discovered, 3);
    }

    #[test]
    fn scanorganic_analyse_increments_trip_bio_analysed_only_when_live() {
        let mut app = App::new();
        
        let scan_organic_json = r#"{ "timestamp":"2026-05-26T16:00:00Z", "event":"ScanOrganic", "ScanType":"Analyse", "Genus":"$Codex_Ent_Bacterial_Genus_Name;", "Genus_Localised":"Bacterial Colonies", "Species":"$Codex_Ent_Bacterial_01_Name;", "Species_Localised":"Bacterial Species 1", "Variant":null, "SystemAddress":4997497796, "Body":61 }"#;
        let event = parse_event(scan_organic_json);

        // Replay mode: should NOT increment trip bio_analysed
        process_event(&mut app, &event, false);
        assert_eq!(app.trip.bio_analysed, 0);

        // Live mode: SHOULD increment trip bio_analysed
        process_event(&mut app, &event, true);
        assert_eq!(app.trip.bio_analysed, 1);
    }
}
