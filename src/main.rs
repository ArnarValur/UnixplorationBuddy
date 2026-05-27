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

    // Replay existing journal files to reconstruct state
    match journal::replay_session(&mut app, journal_dir) {
        Ok(count) => {
            app.status_message = Some(format!(
                "Replayed {} events — watching for new activity",
                count
            ));
        }
        Err(e) => {
            app.status_message = Some(format!("Warning: {e}"));
        }
    }

    // Start live journal watcher
    let journal_rx = match journal::start_live_watcher(journal_dir.to_path_buf()) {
        Ok(rx) => Some(rx),
        Err(e) => {
            app.status_message = Some(format!("Warning: live watcher failed: {e}"));
            None
        }
    };

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
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.quit();
                        }
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::Char('1') => app.active_tab = app::Tab::Bodies,
                        KeyCode::Char('2') => app.active_tab = app::Tab::History,
                        KeyCode::Up => app.select_previous_body(),
                        KeyCode::Down => app.select_next_body(),
                        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
