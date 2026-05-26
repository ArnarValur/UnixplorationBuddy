//! UnixplorationBuddy — A TUI exploration companion for Elite Dangerous on Linux.

mod app;
mod model;
mod ui;

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::App;


fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

/// Main event loop.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Poll with timeout so we can re-render on journal updates later.
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
