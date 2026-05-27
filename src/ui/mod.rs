//! TUI rendering for UnixplorationBuddy.
//!
//! Elite orange-on-black aesthetic with color-coded body types,
//! scrollable table, and clear visual hierarchy.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

use crate::app::{App, Tab};
use crate::model::{BodyType, ScanState};

// ── Color palette ────────────────────────────────────────────────
/// Elite Dangerous signature orange.
const ELITE_ORANGE: Color = Color::Rgb(255, 147, 0);
/// Dimmer orange for secondary text.
const ELITE_DIM: Color = Color::Rgb(140, 85, 0);
/// Dark background matching the cockpit aesthetic.
const BG_DARK: Color = Color::Rgb(10, 10, 10);
/// Highlight color for the selected row.
const HIGHLIGHT_BG: Color = Color::Rgb(50, 35, 0);
/// Star body color — warm yellow.
const COLOR_STAR: Color = Color::Rgb(255, 220, 100);
/// Planet body color — steely blue.
const COLOR_PLANET: Color = Color::Rgb(100, 170, 255);
/// Moon body color — soft grey.
const COLOR_MOON: Color = Color::Rgb(170, 170, 190);
/// Belt cluster color — muted.
const COLOR_BELT: Color = Color::Rgb(120, 110, 90);
/// High-value highlight — green tint for valuable bodies.
const COLOR_VALUE_HIGH: Color = Color::Rgb(80, 220, 80);
/// Bio signal color.
const COLOR_BIO: Color = Color::Rgb(80, 230, 160);
/// Geo signal color.
const COLOR_GEO: Color = Color::Rgb(230, 140, 60);
/// First discovery / first mapping marker.
const COLOR_FIRST: Color = Color::Rgb(255, 215, 0);

/// Value threshold for "high value" highlighting (credits).
const HIGH_VALUE_THRESHOLD: u64 = 100_000;

// ── Root draw ────────────────────────────────────────────────────

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

// ── System header ────────────────────────────────────────────────

/// Slim single-line system header: system name · body count · total value.
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let system_name = if app.system.name.is_empty() {
        "No system"
    } else {
        &app.system.name
    };

    let body_progress = if app.system.body_count_total > 0 {
        format!(
            "{}/{}",
            app.system.body_count_discovered, app.system.body_count_total
        )
    } else {
        format!("{}", app.system.body_count_discovered)
    };

    let header = Line::from(vec![
        Span::styled(
            format!(" {} ", system_name),
            Style::default()
                .fg(ELITE_ORANGE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(ELITE_DIM)),
        Span::styled(
            format!("{} bodies", body_progress),
            Style::default().fg(ELITE_ORANGE),
        ),
        Span::styled(" │ ", Style::default().fg(ELITE_DIM)),
        Span::styled(
            format!("{} cr", format_credits(app.system.total_value)),
            Style::default()
                .fg(if app.system.total_value >= HIGH_VALUE_THRESHOLD {
                    COLOR_VALUE_HIGH
                } else {
                    ELITE_ORANGE
                }),
        ),
    ]);

    let widget = Paragraph::new(header).style(Style::default().bg(BG_DARK));
    frame.render_widget(widget, area);
}

// ── Bodies tab ───────────────────────────────────────────────────

/// Bodies tab — hierarchical, scrollable body table with color-coded types.
fn draw_bodies(frame: &mut Frame, app: &App, area: Rect) {
    if app.body_display_order.is_empty() {
        let content = Paragraph::new(" No bodies discovered yet")
            .style(Style::default().fg(ELITE_DIM).bg(BG_DARK))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(tab_title("Bodies", Tab::Bodies, app.active_tab))
                    .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
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

            match body {
                Some(b) => {
                    let type_color = body_type_color(b.body_type);
                    let is_selected = i == app.selected_body_index;

                    // Name with hierarchy indentation
                    let name = format!("{}{}", indent, b.short_name);

                    // Body type label
                    let body_type_str = format_body_type(b.body_type);

                    // Atmosphere (shortened)
                    let atmo = b
                        .atmosphere
                        .as_deref()
                        .unwrap_or("—")
                        .to_string();

                    // Distance from arrival
                    let dist = b
                        .distance_ls
                        .map(|d| {
                            if d >= 10_000.0 {
                                format!("{:.0}", d)
                            } else if d >= 100.0 {
                                format!("{:.1}", d)
                            } else {
                                format!("{:.2}", d)
                            }
                        })
                        .unwrap_or_else(|| "—".into());

                    // Scan state icon
                    let scan = b.scan_state.icon().to_string();

                    // Value display — show mapped_value for DSS'd bodies
                    let value = format_body_value(b);

                    // Bio/Geo signal counts
                    let bio = if b.bio_signals > 0 {
                        b.bio_signals.to_string()
                    } else {
                        "—".into()
                    };
                    let geo = if b.geo_signals > 0 {
                        b.geo_signals.to_string()
                    } else {
                        "—".into()
                    };

                    // First discovery/mapping indicators
                    let first = format_first_indicators(b);

                    let row_style = if is_selected {
                        Style::default().fg(type_color).bg(HIGHLIGHT_BG)
                    } else {
                        Style::default().fg(type_color)
                    };

                    Row::new(vec![
                        name, body_type_str, atmo, dist, scan, value, bio, geo, first,
                    ])
                    .style(row_style)
                }
                None => {
                    let style = if i == app.selected_body_index {
                        Style::default().fg(ELITE_DIM).bg(HIGHLIGHT_BG)
                    } else {
                        Style::default().fg(ELITE_DIM)
                    };
                    Row::new(vec![
                        format!("{}?", indent),
                        "?".into(),
                        "—".into(),
                        "—".into(),
                        "○".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "".into(),
                    ])
                    .style(style)
                }
            }
        })
        .collect();

    let header = Row::new(vec![
        "Name", "Type", "Atmosphere", "Dist(Ls)", "Scan", "Value(cr)", "Bio", "Geo", "",
    ])
    .style(
        Style::default()
            .fg(ELITE_ORANGE)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )
    .bottom_margin(0);

    let widths = [
        Constraint::Min(18),       // Name — flexible
        Constraint::Length(8),     // Type
        Constraint::Length(14),    // Atmosphere
        Constraint::Length(10),    // Distance
        Constraint::Length(4),     // Scan icon
        Constraint::Length(14),    // Value
        Constraint::Length(4),     // Bio
        Constraint::Length(4),     // Geo
        Constraint::Length(3),     // First disc/map indicators
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title("Bodies", Tab::Bodies, app.active_tab))
                .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
        )
        .row_highlight_style(Style::default().bg(HIGHLIGHT_BG))
        .style(Style::default().bg(BG_DARK));

    // Use StatefulWidget for scrollable selection
    let mut table_state = TableState::default().with_selected(Some(app.selected_body_index));
    frame.render_stateful_widget(table, area, &mut table_state);

    // Scrollbar for large systems
    if app.body_display_order.len() > (area.height as usize).saturating_sub(4) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(app.body_display_order.len())
            .position(app.selected_body_index);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

// ── History tab ──────────────────────────────────────────────────

/// History tab — trip statistics overview with clean layout.
fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let trip = &app.trip;

    let stats = vec![
        ("Systems Visited", trip.systems_visited.to_string(), ELITE_ORANGE),
        ("Bodies Scanned (FSS)", trip.bodies_scanned_fss.to_string(), ELITE_ORANGE),
        ("Bodies Mapped (DSS)", trip.bodies_mapped_dss.to_string(), ELITE_ORANGE),
        ("First Discoveries", trip.first_discoveries.to_string(), COLOR_FIRST),
        ("First Mappings", trip.first_mappings.to_string(), COLOR_FIRST),
        ("Bio Signals Detected", trip.bio_detected.to_string(), COLOR_BIO),
        ("Bio Analysed", trip.bio_analysed.to_string(), COLOR_BIO),
        (
            "Total Value",
            format!("{} cr", format_credits(trip.total_value)),
            if trip.total_value >= HIGH_VALUE_THRESHOLD {
                COLOR_VALUE_HIGH
            } else {
                ELITE_ORANGE
            },
        ),
    ];

    let rows: Vec<Row> = stats
        .into_iter()
        .map(|(label, value, color)| {
            Row::new(vec![
                Span::styled(format!("  {}", label), Style::default().fg(ELITE_DIM)),
                Span::styled(value, Style::default().fg(color)),
            ])
        })
        .collect();

    let widths = [Constraint::Length(25), Constraint::Min(15)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title("History", Tab::History, app.active_tab))
                .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
        )
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(table, area);
}

// ── Status bar ───────────────────────────────────────────────────

/// Status bar at the bottom with keybinding hints.
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        match app.active_tab {
            Tab::Bodies => "q: quit │ Tab/1/2: switch view │ ↑↓: navigate".to_string(),
            Tab::History => "q: quit │ Tab/1/2: switch view".to_string(),
        }
    };

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {status}"), Style::default().fg(ELITE_DIM)),
    ]))
    .style(Style::default().bg(BG_DARK));

    frame.render_widget(bar, area);
}

// ── Helpers ──────────────────────────────────────────────────────

/// Format a tab title with active indicator and number key hint.
fn tab_title(name: &str, tab: Tab, active: Tab) -> String {
    let num = match tab {
        Tab::Bodies => "1",
        Tab::History => "2",
    };
    if tab == active {
        format!(" ▸ [{}] {} ", num, name)
    } else {
        format!("   [{}] {} ", num, name)
    }
}

/// Color for a body type.
fn body_type_color(bt: BodyType) -> Color {
    match bt {
        BodyType::Star => COLOR_STAR,
        BodyType::Planet => COLOR_PLANET,
        BodyType::Moon => COLOR_MOON,
        BodyType::BeltCluster => COLOR_BELT,
        BodyType::Unknown => ELITE_DIM,
    }
}

/// Format a body type for display.
fn format_body_type(bt: BodyType) -> String {
    match bt {
        BodyType::Star => "Star".into(),
        BodyType::Planet => "Planet".into(),
        BodyType::Moon => "Moon".into(),
        BodyType::BeltCluster => "Belt".into(),
        BodyType::Unknown => "?".into(),
    }
}

/// Format the value display for a body.
/// Shows mapped_value for DSS'd bodies, FSS value otherwise.
fn format_body_value(b: &crate::model::Body) -> String {
    if b.scan_state >= ScanState::DSSMapped && b.mapped_value > 0 {
        format_credits(b.mapped_value)
    } else if b.calculated_value > 0 {
        format_credits(b.calculated_value)
    } else {
        "—".into()
    }
}

/// Format first discovery / first mapping indicators.
fn format_first_indicators(b: &crate::model::Body) -> String {
    let mut indicators = String::new();
    if !b.was_discovered {
        indicators.push('◆'); // First discovery
    }
    if !b.was_mapped && b.body_type != BodyType::Star && b.body_type != BodyType::BeltCluster {
        indicators.push('◇'); // First mapping opportunity
    }
    indicators
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
    use crate::model::{Body, BodyType, ScanState, System};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Render the full UI to a TestBackend and return the buffer content as a string.
    fn render_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut output = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                output.push_str(buf[(x, y)].symbol());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn format_credits_works() {
        assert_eq!(format_credits(0), "0");
        assert_eq!(format_credits(100), "100");
        assert_eq!(format_credits(1000), "1,000");
        assert_eq!(format_credits(1234567), "1,234,567");
        assert_eq!(format_credits(10_000_000), "10,000,000");
    }

    #[test]
    fn header_renders_system_name_and_body_count() {
        let mut app = App::new();
        app.system = System::new("Sagittarius A*".into(), 123);
        app.system.body_count_discovered = 5;
        app.system.body_count_total = 10;

        let output = render_to_string(&app, 80, 10);
        assert!(
            output.contains("Sagittarius A*"),
            "Header should contain system name.\nOutput:\n{output}"
        );
        assert!(
            output.contains("5/10 bodies"),
            "Header should show body count.\nOutput:\n{output}"
        );
    }

    #[test]
    fn empty_bodies_shows_placeholder() {
        let app = App::new();
        let output = render_to_string(&app, 80, 10);
        assert!(
            output.contains("No bodies discovered yet"),
            "Should show placeholder when no bodies.\nOutput:\n{output}"
        );
    }

    #[test]
    fn history_tab_renders_trip_stats() {
        let mut app = App::new();
        app.active_tab = Tab::History;
        app.trip.systems_visited = 42;
        app.trip.bodies_scanned_fss = 17;

        let output = render_to_string(&app, 80, 15);
        assert!(
            output.contains("Systems Visited"),
            "History should show stat labels.\nOutput:\n{output}"
        );
        assert!(
            output.contains("42"),
            "History should show trip counter values.\nOutput:\n{output}"
        );
        assert!(
            output.contains("17"),
            "History should show FSS scan count.\nOutput:\n{output}"
        );
    }

    #[test]
    fn bodies_tab_renders_body_rows() {
        let mut app = App::new();
        app.system = System::new("Test System".into(), 1);

        let mut star = Body::new(0, "Test System".into());
        star.short_name = "Test System".into();
        star.body_type = BodyType::Star;
        star.scan_state = ScanState::FSSScanned;
        star.distance_ls = Some(0.0);
        app.bodies.insert(0, star);

        let mut planet = Body::new(1, "Test System 1".into());
        planet.short_name = "1".into();
        planet.body_type = BodyType::Planet;
        planet.scan_state = ScanState::DSSMapped;
        planet.distance_ls = Some(123.4);
        planet.parent_id = Some(0);
        app.bodies.insert(1, planet);

        app.rebuild_display_order();

        let output = render_to_string(&app, 100, 12);
        assert!(
            output.contains("Star"),
            "Should show Star body type.\nOutput:\n{output}"
        );
        assert!(
            output.contains("Planet"),
            "Should show Planet body type.\nOutput:\n{output}"
        );
        // Check scan state icons
        assert!(
            output.contains("●"), // FSSScanned icon
            "Should show FSS scan icon.\nOutput:\n{output}"
        );
        assert!(
            output.contains("★"), // DSSMapped icon
            "Should show DSS mapped icon.\nOutput:\n{output}"
        );
    }

    #[test]
    fn header_shows_total_value() {
        let mut app = App::new();
        app.system = System::new("Rich System".into(), 42);
        app.system.total_value = 1_234_567;

        let output = render_to_string(&app, 80, 10);
        assert!(
            output.contains("1,234,567 cr"),
            "Header should show formatted total value.\nOutput:\n{output}"
        );
    }

    #[test]
    fn format_body_value_shows_mapped_for_dss() {
        let mut b = Body::new(1, "Test".into());
        b.scan_state = ScanState::DSSMapped;
        b.calculated_value = 1000;
        b.mapped_value = 5000;

        assert_eq!(format_body_value(&b), "5,000");
    }

    #[test]
    fn format_body_value_shows_fss_for_scanned() {
        let mut b = Body::new(1, "Test".into());
        b.scan_state = ScanState::FSSScanned;
        b.calculated_value = 1000;
        b.mapped_value = 0;

        assert_eq!(format_body_value(&b), "1,000");
    }

    #[test]
    fn first_indicators_show_discovery_and_mapping() {
        let mut b = Body::new(1, "Test".into());
        b.body_type = BodyType::Planet;
        b.was_discovered = false;
        b.was_mapped = false;

        let indicators = format_first_indicators(&b);
        assert!(indicators.contains('◆'), "Should show first discovery marker");
        assert!(indicators.contains('◇'), "Should show first mapping marker");
    }

    #[test]
    fn first_indicators_empty_for_known_body() {
        let mut b = Body::new(1, "Test".into());
        b.body_type = BodyType::Planet;
        b.was_discovered = true;
        b.was_mapped = true;

        let indicators = format_first_indicators(&b);
        assert!(indicators.is_empty(), "Known body should have no indicators");
    }

    #[test]
    fn status_bar_shows_keybindings() {
        let app = App::new(); // Default tab is Bodies
        let output = render_to_string(&app, 80, 5);
        assert!(
            output.contains("quit") && output.contains("navigate"),
            "Status bar should show keybinding hints.\nOutput:\n{output}"
        );
    }
}
