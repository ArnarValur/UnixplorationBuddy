//! TUI rendering for UnixplorationBuddy.
//!
//! Elite orange-on-black aesthetic with color-coded body types,
//! scrollable table, and clear visual hierarchy.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{App, Tab};
use crate::model::{BodyType, ScanState};

// Register submodules
pub mod bodies;
pub mod inspector;
pub mod route;
pub mod history;

// ── Color palette ────────────────────────────────────────────────
/// Elite Dangerous signature orange.
pub const ELITE_ORANGE: Color = Color::Rgb(255, 147, 0);
/// Dimmer orange for secondary text.
pub const ELITE_DIM: Color = Color::Rgb(140, 85, 0);
/// Dark background matching the cockpit aesthetic.
pub const BG_DARK: Color = Color::Rgb(10, 10, 10);
/// Highlight color for the selected row.
pub const HIGHLIGHT_BG: Color = Color::Rgb(50, 35, 0);
/// Star body color — warm yellow.
pub const COLOR_STAR: Color = Color::Rgb(255, 220, 100);
/// Planet body color — steely blue.
pub const COLOR_PLANET: Color = Color::Rgb(100, 170, 255);
/// Moon body color — soft grey.
pub const COLOR_MOON: Color = Color::Rgb(170, 170, 190);
/// Belt cluster color — muted.
pub const COLOR_BELT: Color = Color::Rgb(120, 110, 90);
/// High-value highlight — green tint for valuable bodies.
pub const COLOR_VALUE_HIGH: Color = Color::Rgb(80, 220, 80);
/// Bio signal color.
pub const COLOR_BIO: Color = Color::Rgb(80, 230, 160);
/// First discovery / first mapping marker.
pub const COLOR_FIRST: Color = Color::Rgb(255, 215, 0);
/// Anomaly / POI highlight — magenta to stand out.
pub const COLOR_ANOMALY: Color = Color::Rgb(255, 100, 200);

/// Value threshold for "high value" highlighting (credits).
pub const HIGH_VALUE_THRESHOLD: u64 = 100_000;

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
        Tab::Bodies => bodies::draw_bodies(frame, app, chunks[1]),
        Tab::History => history::draw_history(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Settings overlay
    if app.show_settings {
        draw_settings_overlay(frame, app);
    }

    // Help overlay (rendered last to be on top)
    if app.show_help {
        draw_help_overlay(frame);
    }
}

// ── System header ────────────────────────────────────────────────

/// Slim single-line system header: system name · region · POI/anomaly summary.
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let system_name = if app.system.name.is_empty() {
        "No system"
    } else {
        &app.system.name
    };

    let mut spans = vec![
        Span::styled(
            format!(" {} ", system_name),
            Style::default()
                .fg(ELITE_ORANGE)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(ref region) = app.system.region {
        spans.push(Span::styled("│ ", Style::default().fg(ELITE_DIM)));
        spans.push(Span::styled(
            region.clone(),
            Style::default().fg(ELITE_DIM),
        ));
    }

    // Jumponium / Green System badge
    if let Some(ref jumpo) = app.jumponium {
        spans.push(Span::styled(" │ ", Style::default().fg(ELITE_DIM)));
        spans.push(Span::styled(
            format!("{} {}", jumpo.grade.icon(), jumpo.grade.label()),
            Style::default().fg(COLOR_VALUE_HIGH).add_modifier(Modifier::BOLD),
        ));
    }

    // POI / anomaly summary — collect unique anomaly kinds across all bodies
    if !app.anomalies.is_empty() {
        let mut kind_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for anomalies in app.anomalies.values() {
            for a in anomalies {
                *kind_counts.entry(format!("{} {}", a.kind.icon(), a.kind.label())).or_default() += 1;
            }
        }

        if !kind_counts.is_empty() {
            spans.push(Span::styled(" │ ", Style::default().fg(ELITE_DIM)));

            let mut badges: Vec<String> = kind_counts
                .iter()
                .map(|(label, count)| {
                    if *count > 1 {
                        format!("{}×{}", label, count)
                    } else {
                        label.clone()
                    }
                })
                .collect();
            badges.sort();

            spans.push(Span::styled(
                badges.join("  "),
                Style::default().fg(COLOR_ANOMALY),
            ));
        }
    }

    let header = Line::from(spans);

    let widget = Paragraph::new(header).style(Style::default().bg(BG_DARK));
    frame.render_widget(widget, area);
}

// ── Column Settings Overlay ──────────────────────────────────────

fn draw_settings_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 45u16.min(area.width.saturating_sub(4));
    let height = 12u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    let check = |enabled: bool| if enabled { "[x]" } else { "[ ]" };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {} Atmosphere  ", check(app.column_settings.show_atmosphere)), Style::default().fg(ELITE_ORANGE)),
            Span::styled("(Key: a)", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} Gravity     ", check(app.column_settings.show_gravity)), Style::default().fg(ELITE_ORANGE)),
            Span::styled("(Key: g)", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} Temp(K)     ", check(app.column_settings.show_temperature)), Style::default().fg(ELITE_ORANGE)),
            Span::styled("(Key: t)", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} Discoverer  ", check(app.column_settings.show_discoverer)), Style::default().fg(ELITE_ORANGE)),
            Span::styled("(Key: d)", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "   a/g/t/d: Toggle │ s/Esc/Enter: Close",
            Style::default().fg(ELITE_DIM),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Column Settings ")
        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK));

    let settings = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(Clear, popup);
    frame.render_widget(settings, popup);
}

// ── Status bar ───────────────────────────────────────────────────

/// Status bar at the bottom with keybinding hints.
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        match app.active_tab {
            Tab::Bodies => {
                "q: quit │ Tab/1/2: switch │ ←→/a/d: sub-tabs │ Ctrl+R: reset trip │ ?: help".to_string()
            }
            Tab::History => "q: quit │ Tab/1/2: switch │ ←→/a/d: sub-tabs │ Ctrl+R: reset trip │ ?: help".to_string(),
        }
    };

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {status}"), Style::default().fg(ELITE_DIM)),
    ]))
    .style(Style::default().bg(BG_DARK));

    frame.render_widget(bar, area);
}

// ── Help overlay ─────────────────────────────────────────────────

/// Centered help overlay with all keybindings.
fn draw_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let help_width = 48u16.min(area.width.saturating_sub(4));
    let help_height = 15u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;
    let popup = Rect::new(x, y, help_width, help_height);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  q / Esc      ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Quit", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  Tab / 1 2 3  ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Switch view tab", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓        ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Navigate bodies", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  a / d / ← →  ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Switch Codex sub-tab", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  s            ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Toggle Column Settings", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  i            ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Toggle Inspector overlay", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+R       ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Reset trip stats", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Toggle this help", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ◆ ", Style::default().fg(COLOR_FIRST)),
            Span::styled("First discovery  ", Style::default().fg(ELITE_DIM)),
            Span::styled("◇ ", Style::default().fg(COLOR_FIRST)),
            Span::styled("First mapping", Style::default().fg(ELITE_DIM)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "         Press any key to close",
            Style::default().fg(ELITE_DIM),
        )),
    ];

    let help = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keybindings ")
                .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
        )
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

// ── Helpers ──────────────────────────────────────────────────────

/// Format a tab title with active indicator and number key hint.
pub fn tab_title(name: &str, tab: Tab, active: Tab) -> String {
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
pub fn body_type_color(bt: BodyType) -> Color {
    match bt {
        BodyType::Star => COLOR_STAR,
        BodyType::Planet => COLOR_PLANET,
        BodyType::Moon => COLOR_MOON,
        BodyType::BeltCluster => COLOR_BELT,
        BodyType::Unknown => ELITE_DIM,
    }
}

/// Format a body type for display, using detailed planet class or star type when available.
pub fn format_body_type(body: &crate::model::Body) -> String {
    match body.body_type {
        BodyType::Star => {
            if let Some(ref st) = body.star_type {
                format!("Star ({})", st)
            } else {
                "Star".into()
            }
        }
        BodyType::Planet | BodyType::Moon => {
            if let Some(ref pc) = body.planet_class {
                short_planet_class(pc)
            } else if body.body_type == BodyType::Moon {
                "Moon".into()
            } else {
                "Planet".into()
            }
        }
        BodyType::BeltCluster => "Belt".into(),
        BodyType::Unknown => "?".into(),
    }
}

/// Map verbose journal planet class strings to short TUI labels.
fn short_planet_class(pc: &str) -> String {
    match pc.to_lowercase().as_str() {
        "earthlike body" | "earth-like body" => "Earth-like",
        "water world" => "Water World",
        "ammonia world" => "Ammonia",
        "high metal content body" => "HMC",
        "metal rich body" => "Metal Rich",
        "rocky body" => "Rocky",
        "rocky ice body" => "Rocky Ice",
        "icy body" => "Icy",
        "sudarsky class i gas giant" => "Gas Giant I",
        "sudarsky class ii gas giant" => "Gas Giant II",
        "sudarsky class iii gas giant" => "Gas Giant III",
        "sudarsky class iv gas giant" => "Gas Giant IV",
        "sudarsky class v gas giant" => "Gas Giant V",
        "gas giant with water based life" => "GG Water Life",
        "gas giant with ammonia based life" => "GG Amm. Life",
        "helium rich gas giant" => "He-Rich GG",
        "helium gas giant" => "Helium GG",
        "water giant" => "Water Giant",
        "water giant with life" => "Water Giant+",
        _ => return pc.to_string(),
    }.into()
}

/// Map journal atmosphere strings to compact chemical formulas.
pub fn format_atmosphere(raw: &str) -> String {
    // Normalize CamelCase enum Debug output (e.g. "HotThickCarbonDioxide")
    // into spaced words ("Hot Thick Carbon Dioxide") before matching.
    let mut spaced = String::with_capacity(raw.len() + 8);
    for (i, ch) in raw.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            spaced.push(' ');
        }
        spaced.push(ch);
    }
    let norm = spaced.trim();

    // Strip prefix modifiers (Hot, Thin, Thick)
    let stripped = norm
        .trim_start_matches("Hot ")
        .trim_start_matches("Thin ")
        .trim_start_matches("Thick ");

    // Strip suffix modifier (Rich)
    let is_rich = stripped.ends_with(" Rich");
    let core = if is_rich {
        stripped.trim_end_matches(" Rich")
    } else {
        stripped
    };

    let formula = match core.to_lowercase().as_str() {
        "carbon dioxide" | "carbondioxide" => "CO\u{2082}",
        "sulfur dioxide" | "sulphur dioxide" | "sulfurdioxide" | "sulphurdioxide" => "SO\u{2082}",
        "water" => "H\u{2082}O",
        "ammonia" => "NH\u{2083}",
        "nitrogen" => "N\u{2082}",
        "oxygen" => "O\u{2082}",
        "methane" => "CH\u{2084}",
        "argon" => "Ar",
        "helium" => "He",
        "hydrogen" => "H\u{2082}",
        "neon" => "Ne",
        "silicate vapour" | "silicatevapour" => "SiO\u{2082}",
        "metallic vapour" | "metallicvapour" => "Metal",
        "carbon dioxide atmosphere" | "carbondioxideatmosphere" => "CO\u{2082}",
        _ => return norm.to_string(),
    };

    // Re-add prefix if present
    let prefix = if norm.starts_with("Hot Thick ") {
        "Thick "
    } else if norm.starts_with("Hot Thin ") || norm.starts_with("Thin ") {
        ""
    } else if norm.starts_with("Thick ") {
        "Thick "
    } else if norm.starts_with("Hot ") {
        "Hot "
    } else {
        ""
    };

    let suffix = if is_rich { " Rich" } else { "" };

    format!("{}{}{}", prefix, formula, suffix)
}

/// Format volcanism enum Debug output for human-readable display.
/// Input is the lowercased Debug string (e.g. "silicatevapourgeysers", "majorwatermagma").
/// Output is title-cased with spaces (e.g. "Silicate Vapour Geysers", "Major Water Magma").
pub fn format_volcanism(raw: &str) -> String {
    if raw.is_empty() {
        return "No Volcanism".into();
    }
    // Known volcanism words to split on (ordered longest-first to avoid partial matches)
    let words = [
        "metallic", "silicate", "vapour", "geysers", "magma",
        "water", "carbon", "dioxide", "nitrogen", "ammonia",
        "methane", "iron", "rocky", "major", "minor",
    ];
    let mut remaining = raw.to_lowercase();
    let mut parts: Vec<String> = Vec::new();
    while !remaining.is_empty() {
        let mut matched = false;
        for word in &words {
            if remaining.starts_with(word) {
                let mut chars = word.chars();
                let titled: String = chars.next().unwrap().to_uppercase().to_string() + &chars.as_str();
                parts.push(titled);
                remaining = remaining[word.len()..].to_string();
                matched = true;
                break;
            }
        }
        if !matched {
            // Unknown word — push remaining as-is with title case
            let mut chars = remaining.chars();
            let titled: String = chars.next().unwrap().to_uppercase().to_string() + &chars.as_str();
            parts.push(titled);
            break;
        }
    }
    parts.join(" ")
}

/// Format the value display for a body.
/// Shows mapped_value for DSS'd bodies, FSS value otherwise.
pub fn format_body_value(b: &crate::model::Body) -> String {
    if b.scan_state >= ScanState::DSSMapped && b.mapped_value > 0 {
        format_credits(b.mapped_value)
    } else if b.calculated_value > 0 {
        format_credits(b.calculated_value)
    } else {
        "—".into()
    }
}

/// Format first discovery / first mapping indicators.
pub fn format_first_indicators(b: &crate::model::Body) -> String {
    let mut indicators = String::new();
    if !b.was_discovered {
        indicators.push('◆'); // First discovery
    }
    if !b.was_mapped && b.body_type != BodyType::Star && b.body_type != BodyType::BeltCluster {
        indicators.push('◇'); // First mapping opportunity
    }
    indicators
}

/// Get the minimum colonial separation distance (in meters) for an exobiology genus.
pub fn min_separation_for_genus(genus: &str) -> Option<u32> {
    match genus.to_lowercase().as_str() {
        "aleoida" => Some(150),
        "bacterium" => Some(500),
        "cactoida" => Some(300),
        "clypeus" => Some(150),
        "concha" => Some(150),
        "electricae" => Some(1000),
        "fonticulua" => Some(500),
        "frutexa" => Some(150),
        "fumerola" => Some(100),
        "fungoida" => Some(300),
        "osseus" => Some(800),
        "recepta" => Some(150),
        "stratum" => Some(500),
        "tubus" => Some(800),
        "tussock" => Some(200),
        _ => None,
    }
}

/// Extract base star class from subclass/luminosity string (e.g. "F" from "F9 VAB", "DA" from "DA2").
pub fn get_main_class(subtype: &str) -> String {
    let mut prefix = String::new();
    for c in subtype.chars() {
        if c.is_ascii_digit() || c == ' ' {
            break;
        }
        prefix.push(c);
    }
    if prefix.is_empty() {
        subtype.to_string()
    } else {
        prefix
    }
}

/// Format credits with thousands separators.
pub fn format_credits(value: u64) -> String {
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

/// Format a floating-point number with thousands separators, no decimals.
pub fn format_number(value: f64) -> String {
    format_credits(value.round() as u64)
}

/// Calculate the Great-Circle distance in meters between two planetary coordinates using the Haversine formula.
pub fn calculate_haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64, radius: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + r_lat1.cos() * r_lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    radius * c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::CodexTab;
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
    fn header_renders_system_name_and_region() {
        let mut app = App::new();
        app.system = System::new("Sagittarius A*".into(), 123);
        app.system.region = Some("Galactic Centre".into());

        let output = render_to_string(&app, 120, 10);
        assert!(
            output.contains("Sagittarius A*"),
            "Header should contain system name.\nOutput:\n{output}"
        );
        assert!(
            output.contains("Galactic Centre"),
            "Header should show region.\nOutput:\n{output}"
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
    fn stellar_codex_renders_hierarchical_tree() {
        let mut app = App::new();
        app.active_tab = Tab::History;
        app.active_codex_tab = CodexTab::Stellar;
        
        // Add some subtypes
        app.trip.stellar_codex.insert("F9 VAB".to_string(), 40);
        app.trip.stellar_codex.insert("F1 VA".to_string(), 20);
        app.trip.stellar_codex.insert("F2".to_string(), 6);
        app.trip.stellar_codex.insert("TTS".to_string(), 7);

        let output = render_to_string(&app, 80, 20);
        
        // Main classes should be displayed with sum counts
        assert!(output.contains("F"), "Stellar Codex should show main class F");
        assert!(output.contains("66"), "Stellar Codex should sum F visits (40+20+6=66)");
        
        // Subtypes should be displayed in numeric subclass order (F1, F2, F9)
        assert!(output.contains("├─ F1 VA"), "Should render child F1 VA");
        assert!(output.contains("20"), "Should show F1 VA count");
        assert!(output.contains("├─ F2"), "Should render child F2");
        assert!(output.contains("6"), "Should show F2 count");
        assert!(output.contains("└─ F9 VAB"), "Should render last child F9 VAB");
        assert!(output.contains("40"), "Should show F9 VAB count");

        // TTS should be rendered as main class but not duplicated as child because subtype == main_class
        assert!(output.contains("TTS"), "Should show main class TTS");
        assert!(output.contains("7"), "Should show TTS count");
        assert!(!output.contains("└─ TTS") && !output.contains("├─ TTS"), "Should not render redundant single child TTS");
    }

    #[test]
    fn draw_inspector_exobiology_progress_and_collapsed_credits() {
        let mut app = App::new();
        app.system = System::new("Test System".into(), 4997497796);
        
        let mut planet = Body::new(1, "Test System 1".into());
        planet.short_name = "1".into();
        planet.body_type = BodyType::Planet;
        planet.scan_state = ScanState::DSSMapped;
        planet.distance_ls = Some(123.4);
        planet.bio_signals = 3;
        planet.landable = true;
        planet.planet_class = Some("Rocky body".to_string());
        planet.planet_class_enum = Some(ed_journals::galaxy::PlanetClass::RockyBody);
        planet.atmosphere = Some("Thin Carbon dioxide".to_string());
        planet.gravity = Some(2.5); // forces fallback
        planet.temperature = Some(900.0);
        planet.bio_genuses = vec!["Aleoida".to_string()];
        app.bodies.insert(1, planet);

        app.rebuild_display_order();
        app.selected_body_index = 0; // selection points to planet 1
        app.show_inspector = true;

        let output_unscanned = render_to_string(&app, 180, 25);
        assert!(!output_unscanned.contains("Base:"), "Should not show base credits line anymore");

        // Case 2: Progress 1/3 scanned -> shows progress with coordinates and real-time distance
        app.trip.organic_progress.insert("4997497796_1_Aleoida Arcus - Grey".to_string(), 1);
        let loc = crate::model::trip::OrganicSampleLocation {
            latitude: -10.2345,
            longitude: 140.5678,
            heading: Some(45.0),
        };
        app.trip.organic_locations.insert("4997497796_1_Aleoida Arcus".to_string(), vec![loc]);
        app.last_latitude = Some(-10.2340);
        app.last_longitude = Some(140.5670);
        if let Some(body) = app.bodies.get_mut(&1) {
            body.radius = Some(1000000.0); // 1000 km planet
        }
        let output_progress1 = render_to_string(&app, 180, 25);
        assert!(output_progress1.contains("[Scanned 1/3 | 150m]"), "Should show progress scanned 1/3 | 150m");
        assert!(output_progress1.contains("Location [1/3]:"), "Should show Location [1/3] tree row");
        assert!(output_progress1.contains("-10.2345°, 140.5678°"), "Should show sample coordinates");
        assert!(output_progress1.contains("m)"), "Should show real-time Haversine distance");

        // Case 3: Completed -> shows completed
        app.trip.organic_scans.insert("4997497796_1".to_string(), vec!["Aleoida Arcus - Grey".to_string()]);
        let output_completed = render_to_string(&app, 180, 25);
        assert!(output_completed.contains("[Completed]"), "Should show progress completed");
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

        let output = render_to_string(&app, 180, 12);
        assert!(
            output.contains("Star"),
            "Should show Star body type.\nOutput:\n{output}"
        );
        assert!(
            output.contains("Planet"),
            "Should show Planet body type.\nOutput:\n{output}"
        );
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
    fn header_shows_anomaly_badges() {
        let mut app = App::new();
        app.system = System::new("Anomaly System".into(), 42);

        // Add a retrograde orbit anomaly
        use crate::model::{Anomaly, AnomalyKind};
        app.anomalies.insert(1, vec![Anomaly {
            body_id: 1,
            kind: AnomalyKind::RetrogradeOrbit,
            description: "test".into(),
        }]);

        let output = render_to_string(&app, 120, 10);
        assert!(
            output.contains("Retrograde Orbit"),
            "Header should show anomaly badge.\nOutput:\n{output}"
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
            output.contains("quit") && output.contains("sub-tabs"),
            "Status bar should show keybinding hints.\nOutput:\n{output}"
        );
    }
}
