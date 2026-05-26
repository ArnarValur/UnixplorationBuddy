//! TUI rendering for UnixplorationBuddy.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Tab};

/// Elite Dangerous signature orange.
const ELITE_ORANGE: Color = Color::Rgb(255, 147, 0);
/// Dark background matching the cockpit aesthetic.
const BG_DARK: Color = Color::Rgb(10, 10, 10);

/// Root draw function — called once per frame from the event loop.
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // System header
            Constraint::Min(0),   // Main content
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);

    match app.active_tab {
        Tab::Bodies => draw_bodies(frame, app, chunks[1]),
        Tab::History => draw_history(frame, app, chunks[1]),
    }
}

/// One-line system status bar at the top.
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let header_text = format!(
        " {} — {} of {} bodies — {} cr",
        app.system.name,
        app.system.body_count_discovered,
        app.system.body_count_total,
        app.system.total_value,
    );
    let header =
        Paragraph::new(header_text).style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK));
    frame.render_widget(header, area);
}

/// Bodies tab — placeholder until Phase 4 builds the tree view.
fn draw_bodies(frame: &mut Frame, _app: &App, area: Rect) {
    let content = Paragraph::new("Bodies view — coming in Phase 4")
        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[1] Bodies")
                .style(Style::default().fg(ELITE_ORANGE)),
        );
    frame.render_widget(content, area);
}

/// History tab — trip statistics overview.
fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let trip = &app.trip;
    let text = format!(
        "Systems: {}\nBodies FSS: {}\nBodies DSS: {}\nFirst Discoveries: {}\nFirst Mappings: {}\nTotal Value: {} cr",
        trip.systems_visited,
        trip.bodies_scanned_fss,
        trip.bodies_mapped_dss,
        trip.first_discoveries,
        trip.first_mappings,
        trip.total_value,
    );
    let content = Paragraph::new(text)
        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[2] History")
                .style(Style::default().fg(ELITE_ORANGE)),
        );
    frame.render_widget(content, area);
}
