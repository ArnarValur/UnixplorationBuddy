//! UnixplorationBuddy — A TUI exploration companion for Elite Dangerous on Linux.

mod app;
mod journal;
mod model;
mod ui;

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::App;
use journal::JournalUpdate;

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
            while let Ok(update) = rx.try_recv() {
                match update {
                    JournalUpdate::Event(event) => {
                        journal::process_event(&mut app, &event);
                        app.rebuild_display_order();
                    }
                    JournalUpdate::Error(e) => {
                        app.status_message = Some(format!("Journal error: {e}"));
                    }
                }
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
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
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
