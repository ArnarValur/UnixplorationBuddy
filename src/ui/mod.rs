//! TUI rendering for UnixplorationBuddy.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::{App, Tab};

/// Elite Dangerous signature orange.
const ELITE_ORANGE: Color = Color::Rgb(255, 147, 0);
/// Dimmer orange for less important text.
const ELITE_DIM: Color = Color::Rgb(180, 100, 0);
/// Dark background matching the cockpit aesthetic.
const BG_DARK: Color = Color::Rgb(10, 10, 10);
/// Highlight color for the selected row.
const HIGHLIGHT: Color = Color::Rgb(50, 35, 0);

/// Root draw function — called once per frame from the event loop.
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // System header
            Constraint::Min(0),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);

    match app.active_tab {
        Tab::Bodies => draw_bodies(frame, app, chunks[1]),
        Tab::History => draw_history(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);
}

/// One-line system status bar at the top.
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let system_name = if app.system.name.is_empty() {
        "No system"
    } else {
        &app.system.name
    };

    let header = Line::from(vec![
        Span::styled(
            format!(" {} ", system_name),
            Style::default()
                .fg(ELITE_ORANGE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("— ", Style::default().fg(ELITE_DIM)),
        Span::styled(
            format!(
                "{} of {} bodies",
                app.system.body_count_discovered, app.system.body_count_total
            ),
            Style::default().fg(ELITE_ORANGE),
        ),
        Span::styled(" — ", Style::default().fg(ELITE_DIM)),
        Span::styled(
            format!("{} cr", format_credits(app.system.total_value)),
            Style::default().fg(ELITE_ORANGE),
        ),
    ]);

    let widget = Paragraph::new(header).style(Style::default().bg(BG_DARK));
    frame.render_widget(widget, area);
}

/// Bodies tab — hierarchical body table.
fn draw_bodies(frame: &mut Frame, app: &App, area: Rect) {
    if app.body_display_order.is_empty() {
        let content = Paragraph::new(" No bodies discovered yet")
            .style(Style::default().fg(ELITE_DIM).bg(BG_DARK))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(tab_title("Bodies", Tab::Bodies, app.active_tab))
                    .style(Style::default().fg(ELITE_ORANGE)),
            );
        frame.render_widget(content, area);
        return;
    }

    // Build table rows from display order
    let rows: Vec<Row> = app
        .body_display_order
        .iter()
        .enumerate()
        .map(|(i, (body_id, depth))| {
            let body = app.bodies.get(body_id);
            let indent = "  ".repeat(*depth as usize);

            let (name, body_type, atmo, dist, scan, value, bio, geo) = match body {
                Some(b) => (
                    format!("{}{}", indent, b.short_name),
                    format_body_type(b.body_type),
                    b.atmosphere
                        .as_deref()
                        .unwrap_or("—")
                        .to_string(),
                    b.distance_ls
                        .map(|d| format!("{:.1}", d))
                        .unwrap_or_else(|| "—".into()),
                    b.scan_state.icon().to_string(),
                    if b.calculated_value > 0 {
                        format_credits(b.calculated_value)
                    } else {
                        "—".into()
                    },
                    if b.bio_signals > 0 {
                        b.bio_signals.to_string()
                    } else {
                        "—".into()
                    },
                    if b.geo_signals > 0 {
                        b.geo_signals.to_string()
                    } else {
                        "—".into()
                    },
                ),
                None => (
                    format!("{}?", indent),
                    "?".into(),
                    "—".into(),
                    "—".into(),
                    "○".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                ),
            };

            let style = if i == app.selected_body_index {
                Style::default().fg(ELITE_ORANGE).bg(HIGHLIGHT)
            } else {
                Style::default().fg(ELITE_ORANGE)
            };

            Row::new(vec![name, body_type, atmo, dist, scan, value, bio, geo]).style(style)
        })
        .collect();

    let header = Row::new(vec![
        "Name", "Type", "Atmosphere", "Dist (Ls)", "Scan", "Value", "Bio", "Geo",
    ])
    .style(
        Style::default()
            .fg(ELITE_ORANGE)
            .add_modifier(Modifier::BOLD),
    );

    let widths = [
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(16),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Length(4),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title("Bodies", Tab::Bodies, app.active_tab))
                .style(Style::default().fg(ELITE_ORANGE)),
        )
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(table, area);
}

/// History tab — trip statistics overview.
fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let trip = &app.trip;

    let stats = vec![
        ("Systems Visited", trip.systems_visited.to_string()),
        ("Bodies Scanned (FSS)", trip.bodies_scanned_fss.to_string()),
        ("Bodies Mapped (DSS)", trip.bodies_mapped_dss.to_string()),
        ("First Discoveries", trip.first_discoveries.to_string()),
        ("First Mappings", trip.first_mappings.to_string()),
        ("Bio Signals Detected", trip.bio_detected.to_string()),
        ("Bio Analysed", trip.bio_analysed.to_string()),
        (
            "Total Value",
            format!("{} cr", format_credits(trip.total_value)),
        ),
    ];

    let rows: Vec<Row> = stats
        .into_iter()
        .map(|(label, value)| {
            Row::new(vec![
                Span::styled(format!("  {}", label), Style::default().fg(ELITE_DIM)),
                Span::styled(value, Style::default().fg(ELITE_ORANGE)),
            ])
        })
        .collect();

    let widths = [Constraint::Length(25), Constraint::Min(15)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title("History", Tab::History, app.active_tab))
                .style(Style::default().fg(ELITE_ORANGE)),
        )
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(table, area);
}

/// Status bar at the bottom.
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = app
        .status_message
        .as_deref()
        .unwrap_or("q: quit | Tab: switch view | ↑↓: navigate");

    let bar = Paragraph::new(format!(" {status}"))
        .style(Style::default().fg(ELITE_DIM).bg(BG_DARK));
    frame.render_widget(bar, area);
}

/// Format a tab title with active indicator.
fn tab_title(name: &str, tab: Tab, active: Tab) -> String {
    let num = match tab {
        Tab::Bodies => "1",
        Tab::History => "2",
    };
    if tab == active {
        format!("▸ [{}] {} ", num, name)
    } else {
        format!("  [{}] {} ", num, name)
    }
}

/// Format a body type for display.
fn format_body_type(bt: crate::model::BodyType) -> String {
    match bt {
        crate::model::BodyType::Star => "Star".into(),
        crate::model::BodyType::Planet => "Planet".into(),
        crate::model::BodyType::Moon => "Moon".into(),
        crate::model::BodyType::BeltCluster => "Belt".into(),
        crate::model::BodyType::Unknown => "?".into(),
    }
}

/// Format credits with thousands separators.
fn format_credits(value: u64) -> String {
    if value == 0 {
        return "0".into();
    }
    let s = value.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_credits_works() {
        assert_eq!(format_credits(0), "0");
        assert_eq!(format_credits(100), "100");
        assert_eq!(format_credits(1000), "1,000");
        assert_eq!(format_credits(1234567), "1,234,567");
        assert_eq!(format_credits(10_000_000), "10,000,000");
    }
}
