//! UnixplorationBuddy — A TUI exploration companion for Elite Dangerous on Linux.

mod app;
mod journal;
mod model;
mod persistence;
mod ui;

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::App;
use journal::JournalUpdate;
use persistence::TripPersistence;

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
    let result = run(&mut terminal, &journal_dir);
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

    // Show loading indicator during journal replay
    app.status_message = Some("Replaying journal files...".to_string());
    terminal.draw(|frame| ui::draw(frame, &app))?;

    // Replay existing journal files to reconstruct state
    match journal::replay_session(&mut app, journal_dir) {
        Ok(count) => {
            let body_count = app.bodies.len();
            let system = if app.system.name.is_empty() {
                "no system".to_string()
            } else {
                app.system.name.clone()
            };
            app.status_message = Some(format!(
                "Replayed {count} events — {system}, {body_count} bodies — watching for new activity",
            ));
        }
        Err(e) => {
            app.status_message = Some(format!("Warning: {e}"));
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

        // Poll for keyboard events
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
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
                            KeyCode::Char('3') => app.active_tab = app::Tab::Route,
                            KeyCode::Up => app.select_previous_body(),
                            KeyCode::Down => app.select_next_body(),
                            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') if app.active_tab == app::Tab::History => {
                                app.prev_codex_tab();
                            }
                            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') if app.active_tab == app::Tab::History => {
                                app.next_codex_tab();
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
            }
        }

        if app.should_quit {
            // Final save before exit
            if let Some(ref mut p) = trip_persistence {
                if let Some(err) = p.force_save(&app.trip) {
                    eprintln!("Warning: final trip save failed: {err}");
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
