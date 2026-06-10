//! UnixplorationBuddy — A TUI exploration companion for Elite Dangerous on Linux.

mod app;
mod journal;
mod model;
mod persistence;
mod state;
mod ui;

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind, EnableMouseCapture, DisableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::DefaultTerminal;

use app::App;
use journal::JournalUpdate;
use persistence::TripPersistence;
use state::StatePersistence;

fn main() -> io::Result<()> {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();
    let journal_path = parse_journal_path(&args);

    // Discover journal directory
    let journal_dir = match journal::discover_journal_dir(journal_path) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // Initialize terminal
    let mut terminal = ratatui::init();
    // Enable mouse capture for scroll support
    execute!(io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, &journal_dir);
    // Disable mouse capture before restoring terminal
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

/// Main event loop.
fn run(terminal: &mut DefaultTerminal, journal_dir: &std::path::Path) -> io::Result<()> {
    let mut app = App::new();

    // Initialize trip persistence
    let mut trip_persistence = match TripPersistence::new() {
        Ok(p) => Some(p),
        Err(e) => {
            app.status_message = Some(format!("Warning: trip persistence disabled: {e}"));
            None
        }
    };

    // Load saved trip data
    if let Some(ref persistence) = trip_persistence {
        let (trip, warning) = persistence.load();
        app.trip = trip;
        if let Some(w) = warning {
            app.status_message = Some(w);
        }
    }

    // State snapshot persistence (instant cold start)
    let state_persistence = match StatePersistence::new() {
        Ok(p) => Some(p),
        Err(e) => {
            app.status_message = Some(format!("Warning: state persistence disabled: {e}"));
            None
        }
    };

    // Load saved state snapshot for instant display
    let skip_info = if let Some(ref sp) = state_persistence {
        if let Some(snapshot) = sp.load() {
            app.system = snapshot.system;
            app.bodies = snapshot.bodies;
            app.body_display_order = snapshot.body_display_order;
            // Show instantly
            terminal.draw(|frame| ui::draw(frame, &app))?;
            // Prepare skip info for replay
            snapshot.last_journal_file.map(|f| (f, snapshot.last_journal_event_count))
        } else {
            None
        }
    } else {
        None
    };

    // Show loading indicator during journal replay
    app.status_message = Some("Catching up on journal...".to_string());
    terminal.draw(|frame| ui::draw(frame, &app))?;

    // Replay journal files, skipping already-processed events
    let skip_ref = skip_info.as_ref().map(|(f, c)| (f.as_str(), *c));
    let mut last_journal_file: Option<String> = None;
    let mut last_file_event_count: u32 = 0;
    match journal::replay_session(&mut app, journal_dir, skip_ref) {
        Ok(result) => {
            last_journal_file = result.last_journal_file;
            last_file_event_count = result.last_file_event_count;
            let body_count = app.bodies.len();
            let system = if app.system.name.is_empty() {
                "no system".to_string()
            } else {
                app.system.name.clone()
            };
            let skipped = if skip_info.is_some() { " (resumed)" } else { "" };
            app.status_message = Some(format!(
                "Replayed {count} events{skipped} — {system}, {body_count} bodies — watching for new activity",
                count = result.event_count,
            ));
        }
        Err(e) => {
            app.status_message = Some(format!("Warning: {e}"));
            if let Some(ref e_str) = app.status_message {
                eprintln!("{e_str}");
            }
        }
    }

    // Bootstrap Status.json on startup
    let status_path = journal_dir.join("Status.json");
    if let Ok(content) = std::fs::read_to_string(&status_path) {
        if let Ok(status) = serde_json::from_str::<model::Status>(&content) {
            app.last_latitude = status.latitude;
            app.last_longitude = status.longitude;
            app.last_heading = status.heading;

            if let Some(ref dest) = status.destination {
                if dest.system == app.system.system_address {
                    app.targeted_body_id = Some(dest.body);
                    if let Some(pos) = app.body_display_order.iter().position(|&(id, _)| id == dest.body) {
                        app.selected_body_index = pos;
                    }
                }
            }
        }
    }

    // Bootstrap NavRoute.json on startup
    let nav_route_path = journal_dir.join("NavRoute.json");
    if let Ok(content) = std::fs::read_to_string(&nav_route_path) {
        if let Ok(nav_route) = serde_json::from_str::<model::NavRoute>(&content) {
            app.plotted_route = Some(nav_route);
        }
    }

    // Start live journal watcher
    let (journal_rx, edsm_tx) = match journal::start_live_watcher(journal_dir.to_path_buf()) {
        Ok((rx, tx)) => (Some(rx), Some(tx)),
        Err(e) => {
            app.status_message = Some(format!("Warning: live watcher failed: {e}"));
            (None, None)
        }
    };

    // Queue initial EDSM queries on startup
    if let Some(ref tx) = edsm_tx {
        queue_edsm_requests(&app, tx);
    }

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Drain any pending journal events (non-blocking)
        if let Some(ref rx) = journal_rx {
            let mut state_changed = false;
            while let Ok(update) = rx.try_recv() {
                match update {
                    JournalUpdate::Event(event) => {
                        journal::process_event(&mut app, &event, true);
                        state_changed = true;
                        if let Some(ref tx) = edsm_tx {
                            queue_edsm_requests(&app, tx);
                        }
                    }
                    JournalUpdate::StatusUpdate(status) => {
                        app.last_latitude = status.latitude;
                        app.last_longitude = status.longitude;
                        app.last_heading = status.heading;

                        if let Some(ref dest) = status.destination {
                            if dest.system == app.system.system_address {
                                app.targeted_body_id = Some(dest.body);
                                if let Some(pos) = app.body_display_order.iter().position(|&(id, _)| id == dest.body) {
                                    app.selected_body_index = pos;
                                }
                            }
                        } else {
                            app.targeted_body_id = None;
                        }
                    }
                    JournalUpdate::NavRouteUpdate(nav_route) => {
                        app.plotted_route = Some(nav_route);
                        if let Some(ref tx) = edsm_tx {
                            queue_edsm_requests(&app, tx);
                        }
                    }
                    JournalUpdate::EdsmPayload(data) => {
                        app.edsm_cache.insert(data.name.clone(), data);
                    }
                    JournalUpdate::Error(e) => {
                        app.status_message = Some(format!("Journal error: {e}"));
                    }
                }
            }
            if state_changed {
                app.rebuild_display_order();
                if let Some(ref mut p) = trip_persistence {
                    p.mark_dirty();
                }
            }
        }

        // Debounced trip save
        if let Some(ref mut p) = trip_persistence {
            if let Some(err) = p.maybe_save(&app.trip) {
                app.status_message = Some(err);
            }
        }

        // Poll for keyboard events (faster 100ms ticks for smooth orbit graphics)
        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            match ev {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    // Any key press clears transient status messages
                    app.status_message = None;

                    // If help overlay is showing, any key dismisses it
                    if app.show_help {
                        app.show_help = false;
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                            KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.quit();
                            }
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::Char('1') => app.active_tab = app::Tab::Bodies,
                            KeyCode::Char('2') => app.active_tab = app::Tab::History,
                            KeyCode::Up => {
                                if app.active_tab == app::Tab::History {
                                    app.select_previous_codex_row();
                                } else {
                                    app.select_previous_body();
                                }
                            }
                            KeyCode::Down => {
                                if app.active_tab == app::Tab::History {
                                    app.select_next_codex_row();
                                } else {
                                    app.select_next_body();
                                }
                            }
                            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                                if app.active_tab == app::Tab::History {
                                    app.prev_codex_tab();
                                } else if app.active_tab == app::Tab::Bodies {
                                    app.bodies_subtab = app::BodiesSubTab::Table;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                                if app.active_tab == app::Tab::History {
                                    app.next_codex_tab();
                                } else if app.active_tab == app::Tab::Bodies {
                                    app.bodies_subtab = app::BodiesSubTab::Route;
                                }
                            }
                            KeyCode::Char('?') => app.show_help = true,
                            KeyCode::Char('r')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                // Ctrl+R: reset trip
                                app.trip.reset();
                                app.status_message =
                                    Some("Trip statistics reset".to_string());
                                if let Some(ref mut p) = trip_persistence {
                                    p.mark_dirty();
                                    if let Some(err) = p.force_save(&app.trip) {
                                        app.status_message = Some(err);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            if app.active_tab == app::Tab::History {
                                app.select_previous_codex_row();
                            } else {
                                app.select_previous_body();
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.active_tab == app::Tab::History {
                                app.select_next_codex_row();
                            } else {
                                app.select_next_body();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            // Final save before exit
            if let Some(ref mut p) = trip_persistence {
                if let Some(err) = p.force_save(&app.trip) {
                    eprintln!("Warning: final trip save failed: {err}");
                }
            }
            // Save state snapshot for instant cold start
            if let Some(ref sp) = state_persistence {
                let snapshot = state::StateSnapshot {
                    system: app.system.clone(),
                    bodies: app.bodies.clone(),
                    body_display_order: app.body_display_order.clone(),
                    last_journal_file: last_journal_file.clone(),
                    last_journal_event_count: last_file_event_count,
                };
                if let Err(err) = sp.save(&snapshot) {
                    eprintln!("Warning: state snapshot save failed: {err}");
                }
            }
            break;
        }
    }

    Ok(())
}

/// Parse --journal-path from CLI arguments.
fn parse_journal_path(args: &[String]) -> Option<&str> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--journal-path" {
            return args.get(i + 1).map(|s| s.as_str());
        }
    }
    None
}

/// Queue EDSM queries for the current system and waypoints on the plotted route.
fn queue_edsm_requests(app: &App, edsm_tx: &std::sync::mpsc::Sender<String>) {
    if !app.system.name.is_empty() && !app.edsm_cache.contains_key(&app.system.name) {
        let _ = edsm_tx.send(app.system.name.clone());
    }
    if let Some(ref route) = app.plotted_route {
        for entry in &route.route {
            if !app.edsm_cache.contains_key(&entry.star_system) {
                let _ = edsm_tx.send(entry.star_system.clone());
            }
        }
    }
}
