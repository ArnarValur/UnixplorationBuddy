//! TUI rendering for UnixplorationBuddy.
//!
//! Elite orange-on-black aesthetic with color-coded body types,
//! scrollable table, and clear visual hierarchy.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

use crate::app::{App, Tab, CodexTab};
use crate::model::{BodyType, ScanState};
use crate::model::biology::predictor;

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
        Tab::Route => draw_route(frame, app, chunks[1]),
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

    // Split-Pane check
    let show_inspect = area.width >= 110 || app.show_inspector;
    let main_chunks = if show_inspect {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    let table_area = main_chunks[0];

    // Build table rows from display order
    let rows: Vec<Row> = app
        .body_display_order
        .iter()
        .enumerate()
        .map(|(i, &(body_id, depth))| {
            let body = app.bodies.get(&body_id);
            let indent = "  ".repeat(depth as usize);

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

                    // Gravity
                    let gravity_str = b.gravity.map(|g| format!("{:.2} G", g)).unwrap_or_else(|| "—".into());

                    // Temp
                    let temp_str = b.temperature.map(|t| format!("{:.0} K", t)).unwrap_or_else(|| "—".into());

                    // EDSM discoverer
                    let discoverer_str = if b.was_discovered { "CMDR" } else { "—" };

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

                    let mut cells = vec![name, body_type_str];

                    if app.column_settings.show_atmosphere {
                        cells.push(atmo);
                    }
                    if app.column_settings.show_gravity {
                        cells.push(gravity_str);
                    }
                    if app.column_settings.show_temperature {
                        cells.push(temp_str);
                    }
                    if app.column_settings.show_discoverer {
                        cells.push(discoverer_str.to_string());
                    }

                    cells.extend(vec![dist, scan, value, bio, geo, first]);

                    Row::new(cells).style(row_style)
                }
                None => {
                    let style = if i == app.selected_body_index {
                        Style::default().fg(ELITE_DIM).bg(HIGHLIGHT_BG)
                    } else {
                        Style::default().fg(ELITE_DIM)
                    };

                    let mut cells = vec![format!("{}?", indent), "?".into()];

                    if app.column_settings.show_atmosphere {
                        cells.push("—".into());
                    }
                    if app.column_settings.show_gravity {
                        cells.push("—".into());
                    }
                    if app.column_settings.show_temperature {
                        cells.push("—".into());
                    }
                    if app.column_settings.show_discoverer {
                        cells.push("—".into());
                    }

                    cells.extend(vec![
                        "—".into(),
                        "—".into(),
                        "○".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "".into(),
                    ]);

                    Row::new(cells).style(style)
                }
            }
        })
        .collect();

    let mut header_cells = vec!["Name", "Type"];
    let mut widths = vec![Constraint::Min(16), Constraint::Length(8)];

    if app.column_settings.show_atmosphere {
        header_cells.push("Atmosphere");
        widths.push(Constraint::Length(14));
    }
    if app.column_settings.show_gravity {
        header_cells.push("Gravity");
        widths.push(Constraint::Length(8));
    }
    if app.column_settings.show_temperature {
        header_cells.push("Temp(K)");
        widths.push(Constraint::Length(8));
    }
    if app.column_settings.show_discoverer {
        header_cells.push("Discoverer");
        widths.push(Constraint::Length(14));
    }

    header_cells.extend(vec!["Dist(Ls)", "Scan", "Value(cr)", "Bio", "Geo", ""]);
    widths.extend(vec![
        Constraint::Length(9),
        Constraint::Length(4),
        Constraint::Length(11),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(5),
    ]);

    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(ELITE_ORANGE)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .bottom_margin(0);

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
    frame.render_stateful_widget(table, table_area, &mut table_state);

    // Scrollbar for large systems
    if app.body_display_order.len() > (table_area.height as usize).saturating_sub(4) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(app.body_display_order.len())
            .position(app.selected_body_index);

        frame.render_stateful_widget(
            scrollbar,
            table_area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }

    // Render inspector right pane
    if show_inspect {
        draw_inspector(frame, app, main_chunks[1]);
    }
}

// ── Inspector panel ──────────────────────────────────────────────

fn draw_inspector(frame: &mut Frame, app: &App, area: Rect) {
    if app.body_display_order.is_empty() {
        return;
    }
    let (body_id, _) = app.body_display_order[app.selected_body_index];
    let body = match app.bodies.get(&body_id) {
        Some(b) => b,
        None => return,
    };

    let binding = format_body_type(body.body_type);
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Class: ", Style::default().fg(ELITE_DIM)),
            Span::styled(
                body.planet_class.as_deref().unwrap_or(binding.as_str()),
                Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)
            ),
        ]),
    ];

    if body.body_type == BodyType::Planet || body.body_type == BodyType::Moon {
        lines.push(Line::from(vec![
            Span::styled("Landable: ", Style::default().fg(ELITE_DIM)),
            Span::styled(if body.landable { "YES 🚀" } else { "NO" }, Style::default().fg(if body.landable { COLOR_STAR } else { ELITE_DIM })),
        ]));
        
        let gravity_text = body.gravity.map(|g| format!("{:.2} G", g)).unwrap_or_else(|| "—".into());
        lines.push(Line::from(vec![
            Span::styled("Gravity:  ", Style::default().fg(ELITE_DIM)),
            Span::styled(gravity_text, Style::default().fg(ELITE_ORANGE)),
        ]));

        let temp_text = body.temperature.map(|t| format!("{:.0} K ({:.0}°C)", t, t - 273.15)).unwrap_or_else(|| "—".into());
        lines.push(Line::from(vec![
            Span::styled("Temp:     ", Style::default().fg(ELITE_DIM)),
            Span::styled(temp_text, Style::default().fg(ELITE_ORANGE)),
        ]));

        let atmo_text = body.atmosphere.as_deref().unwrap_or("None");
        lines.push(Line::from(vec![
            Span::styled("Atmo:     ", Style::default().fg(ELITE_DIM)),
            Span::styled(atmo_text, Style::default().fg(ELITE_ORANGE)),
        ]));
    }

    if let Some(cache) = app.edsm_cache.get(&app.system.name) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("── EDSM TELEMETRY ──", Style::default().fg(ELITE_DIM))));
        if let Some(ref cmdr) = cache.discoverer {
            lines.push(Line::from(vec![
                Span::styled("CMDR:     ", Style::default().fg(ELITE_DIM)),
                Span::styled(cmdr, Style::default().fg(COLOR_STAR)),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("Value:    ", Style::default().fg(ELITE_DIM)),
            Span::styled(format!("{} cr", format_credits(cache.estimated_value_mapped)), Style::default().fg(COLOR_VALUE_HIGH)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("── EXOBIOLOGY PREDICTIONS ──", Style::default().fg(ELITE_DIM))));
    if body.bio_signals > 0 && body.landable {
        let primary_star_id = app.system.primary_star_id.unwrap_or(0);
        let primary_star = app.bodies.get(&primary_star_id).and_then(|b| b.star_class_enum.as_ref());
        let predictions = predictor::predict_species(body, primary_star);

        if predictions.is_empty() {
            lines.push(Line::from(" No matching species boundaries"));
        } else {
            lines.push(Line::from(format!(" Signals: {} detected", body.bio_signals)));
            lines.push(Line::from(""));
            let scan_key = format!("{}_{}", app.system.system_address, body.body_id);
            let organic_scans = app.trip.organic_scans.get(&scan_key);

            struct GroupedPrediction {
                base_name: String,
                genus: String,
                reward: u64,
                variants: Vec<String>,
                active_variant: Option<String>,
                active_progress: u8,
                active_scanned: bool,
            }

            let mut grouped: Vec<GroupedPrediction> = Vec::new();

            for variant in predictions {
                let parts: Vec<&str> = variant.name.split(" - ").collect();
                let base_name = parts[0].to_string();
                let color = parts.get(1).map(|s| s.to_string()).unwrap_or_default();

                let progress_key_full = format!("{}_{}_{}", app.system.system_address, body.body_id, variant.name);
                let progress_val_full = app.trip.organic_progress.get(&progress_key_full).cloned().unwrap_or(0);

                let progress_key_base = format!("{}_{}_{}", app.system.system_address, body.body_id, base_name);
                let progress_val_base = app.trip.organic_progress.get(&progress_key_base).cloned().unwrap_or(0);

                let progress_val = std::cmp::max(progress_val_full, progress_val_base);

                let has_scanned = organic_scans.map(|s| {
                    s.contains(&variant.name.to_string()) || s.contains(&base_name)
                }).unwrap_or(false);

                if let Some(g) = grouped.iter_mut().find(|g| g.base_name == base_name) {
                    if !color.is_empty() && !g.variants.contains(&color) {
                        g.variants.push(color);
                    }
                    if progress_val > g.active_progress {
                        g.active_progress = progress_val;
                        g.active_variant = Some(variant.name.to_string());
                    }
                    if has_scanned {
                        g.active_scanned = true;
                        g.active_variant = Some(variant.name.to_string());
                    }
                } else {
                    grouped.push(GroupedPrediction {
                        base_name: base_name.clone(),
                        genus: variant.genus.to_string(),
                        reward: variant.reward,
                        variants: if color.is_empty() { vec![] } else { vec![color] },
                        active_variant: if progress_val > 0 || has_scanned { Some(variant.name.to_string()) } else { None },
                        active_progress: progress_val,
                        active_scanned: has_scanned,
                    });
                }
            }

            // Exobiology rule: A planet can never have more than one species of the same Genus.
            // If the player has active progress or has scanned any species of a genus, filter out all other species of that genus.
            let active_genuses: Vec<String> = grouped.iter()
                .filter(|g| g.active_progress > 0 || g.active_scanned)
                .map(|g| g.genus.clone())
                .collect();

            if !active_genuses.is_empty() {
                grouped.retain(|g| {
                    if active_genuses.contains(&g.genus) {
                        g.active_progress > 0 || g.active_scanned
                    } else {
                        true
                    }
                });
            }

            // Sort grouped predictions alphabetically by base name
            grouped.sort_by(|a, b| a.base_name.cmp(&b.base_name));

            for g in grouped {
                if g.active_scanned || g.active_progress == 3 {
                    lines.push(Line::from(vec![
                        Span::styled(format!(" R ▸ {} ", g.active_variant.as_deref().unwrap_or(&g.base_name)), Style::default().fg(COLOR_BIO).add_modifier(Modifier::BOLD)),
                        Span::styled("[Completed]", Style::default().fg(COLOR_BIO)),
                    ]));
                } else if g.active_progress > 0 {
                    let sep_str = min_separation_for_genus(&g.genus)
                        .map(|d| format!(" | {}m", d))
                        .unwrap_or_default();
                    lines.push(Line::from(vec![
                        Span::styled(format!(" R ▸ {} ", g.active_variant.as_deref().unwrap_or(&g.base_name)), Style::default().fg(COLOR_BIO).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("[Scanned {}/3{}]", g.active_progress, sep_str), Style::default().fg(COLOR_FIRST).add_modifier(Modifier::BOLD)),
                    ]));

                    // Render tracked sample locations under the active species
                    let location_key = format!("{}_{}_{}", app.system.system_address, body.body_id, g.base_name);
                    if let Some(locs) = app.trip.organic_locations.get(&location_key) {
                        for (i, loc) in locs.iter().enumerate() {
                            let is_last = i == locs.len() - 1;
                            let prefix = if is_last { "   └─ " } else { "   ├─ " };

                            let dist_str = if let (Some(cur_lat), Some(cur_lon), Some(rad)) = (app.last_latitude, app.last_longitude, body.radius) {
                                let dist_m = calculate_haversine_distance(loc.latitude, loc.longitude, cur_lat, cur_lon, rad);
                                if dist_m >= 1000.0 {
                                    format!(" ({:.2} km)", dist_m / 1000.0)
                                } else {
                                    format!(" ({:.0} m)", dist_m)
                                }
                            } else {
                                "".to_string()
                            };

                            lines.push(Line::from(vec![
                                Span::styled(prefix, Style::default().fg(ELITE_DIM)),
                                Span::styled(format!("Location [{}/3]: ", i + 1), Style::default().fg(ELITE_DIM)),
                                Span::styled(format!("{:.4}°, {:.4}°", loc.latitude, loc.longitude), Style::default().fg(COLOR_VALUE_HIGH)),
                                Span::styled(dist_str, Style::default().fg(COLOR_FIRST).add_modifier(Modifier::BOLD)),
                            ]));
                        }
                    }
                } else {
                    let variants_str = if g.variants.is_empty() {
                        "".to_string()
                    } else {
                        format!(" ({})", g.variants.join("/"))
                    };
                    let sep_span = min_separation_for_genus(&g.genus)
                        .map(|d| Span::styled(format!(" [{}m]", d), Style::default().fg(ELITE_DIM)))
                        .unwrap_or_else(|| Span::raw(""));
                    lines.push(Line::from(vec![
                        Span::styled(format!(" ▸ {}{} ", g.base_name, variants_str), Style::default().fg(ELITE_ORANGE)),
                        Span::styled(format!(": {} cr (First)", format_credits(g.reward * 5)), Style::default().fg(COLOR_FIRST)),
                        sep_span,
                    ]));
                }
            }
        }
    } else if body.bio_signals > 0 {
        lines.push(Line::from(" Not landable (exobiology locked)"));
    } else {
        lines.push(Line::from(" No bio signals reported"));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Telemetry: {} ", body.short_name))
        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(paragraph, area);
}

// ── Plotted NavRoute tab ─────────────────────────────────────────

fn draw_route(frame: &mut Frame, app: &App, area: Rect) {
    let route = match &app.plotted_route {
        Some(r) if !r.route.is_empty() => r,
        _ => {
            let content = Paragraph::new(" No plotted navigation route\n Waypoints will sync in real-time when plotted in-game")
                .style(Style::default().fg(ELITE_DIM).bg(BG_DARK))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(tab_title("Route", Tab::Route, app.active_tab))
                        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
                );
            frame.render_widget(content, area);
            return;
        }
    };

    let rows: Vec<Row> = route.route.iter().enumerate().map(|(i, entry)| {
        let scoopable = matches!(entry.star_class.chars().next(), Some('O') | Some('B') | Some('A') | Some('F') | Some('G') | Some('K') | Some('M'));
        let scoop_str = if scoopable { "⛽" } else { "—" };

        let cache = app.edsm_cache.get(&entry.star_system);
        
        let value_str = cache.map(|c| format_credits(c.estimated_value_mapped)).unwrap_or_else(|| "—".into());
        let discoverer_str = cache.and_then(|c| c.discoverer.as_deref()).unwrap_or("—");

        // Badges
        let mut badges = String::new();
        if let Some(c) = cache {
            if c.valuable_bodies > 0 {
                badges.push_str(&format!("💰x{} ", c.valuable_bodies));
            }
            if c.terraformable_bodies > 0 {
                badges.push_str(&format!("🌍x{} ", c.terraformable_bodies));
            }
            if c.landable_bodies > 0 {
                badges.push_str(&format!("🚀x{} ", c.landable_bodies));
            }
        }

        let is_current = entry.star_system == app.system.name;
        let style = if is_current {
            Style::default().fg(COLOR_STAR).bg(HIGHLIGHT_BG)
        } else {
            Style::default().fg(ELITE_ORANGE)
        };

        Row::new(vec![
            format!(" #{}", i + 1),
            entry.star_system.clone(),
            format!("{} {}", entry.star_class, scoop_str),
            value_str,
            discoverer_str.to_string(),
            badges,
        ])
        .style(style)
    }).collect();

    let header = Row::new(vec![
        " Jump", "Star System", "Star Class", "EDSM Value(cr)", "EDSM Discoverer", "Badges",
    ])
    .style(
        Style::default()
            .fg(ELITE_ORANGE)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    );

    let widths = [
        Constraint::Length(7),
        Constraint::Min(18),
        Constraint::Length(12),
        Constraint::Length(15),
        Constraint::Length(16),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title("Route Exploration", Tab::Route, app.active_tab))
                .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
        )
        .style(Style::default().bg(BG_DARK));

    frame.render_widget(table, area);
}

// ── History / Trip Codex tab ─────────────────────────────────────

fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),   // Codex view content
            Constraint::Length(1), // Sub-tabs header at the bottom
        ])
        .split(area);

    let content_area = chunks[0];
    let trip = &app.trip;

    match app.active_codex_tab {
        CodexTab::Overview => {
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
            let table_stats = Table::new(rows, widths)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(tab_title("Trip Statistics", Tab::History, app.active_tab))
                        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
                )
                .style(Style::default().bg(BG_DARK));

            let mut entries: Vec<(&String, &u32)> = trip.biological_codex.iter().collect();
            entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
            let rows_bio: Vec<Row> = entries.iter().map(|(species, count)| {
                Row::new(vec![
                    (*species).clone(),
                    count.to_string(),
                ]).style(Style::default().fg(COLOR_BIO))
            }).collect();

            let header_bio = Row::new(vec!["Species Name", "Analyses Completed"])
                .style(Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

            let total_rows_bio = rows_bio.len();
            let widths_bio = [Constraint::Length(35), Constraint::Min(10)];
            let table_bio = Table::new(rows_bio, widths_bio)
                .header(header_bio)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(tab_title("Biological Codex", Tab::History, app.active_tab))
                        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
                )
                .row_highlight_style(Style::default().bg(HIGHLIGHT_BG))
                .style(Style::default().bg(BG_DARK));

            let split_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_area);

            frame.render_widget(table_stats, split_chunks[0]);

            let mut state_bio = TableState::default().with_selected(Some(app.selected_codex_index));
            frame.render_stateful_widget(table_bio, split_chunks[1], &mut state_bio);

            if total_rows_bio > (split_chunks[1].height as usize).saturating_sub(4) {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                let mut scrollbar_state = ScrollbarState::new(total_rows_bio).position(app.selected_codex_index);
                frame.render_stateful_widget(
                    scrollbar,
                    split_chunks[1].inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
                    &mut scrollbar_state,
                );
            }
        }
        CodexTab::Stellar => {
            struct MainClassGroup {
                main_class: String,
                total_visits: u32,
                subtypes: Vec<(String, u32)>,
            }

            let mut groups: std::collections::HashMap<String, MainClassGroup> = std::collections::HashMap::new();

            for (subtype, count) in &trip.stellar_codex {
                let main_class = get_main_class(subtype);
                let group = groups.entry(main_class.clone()).or_insert_with(|| MainClassGroup {
                    main_class: main_class.clone(),
                    total_visits: 0,
                    subtypes: Vec::new(),
                });
                group.total_visits += count;
                group.subtypes.push((subtype.clone(), *count));
            }

            let mut group_list: Vec<MainClassGroup> = groups.into_values().collect();
            group_list.sort_by(|a, b| b.total_visits.cmp(&a.total_visits).then_with(|| a.main_class.cmp(&b.main_class)));

            for group in &mut group_list {
                group.subtypes.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            }

            let mut stellar_rows = Vec::new();

            for group in group_list {
                stellar_rows.push(
                    Row::new(vec![
                        group.main_class.clone(),
                        group.total_visits.to_string(),
                    ])
                    .style(Style::default().fg(COLOR_STAR).add_modifier(Modifier::BOLD))
                );

                let has_redundant_single_child = group.subtypes.len() == 1 && group.subtypes[0].0 == group.main_class;
                if !has_redundant_single_child {
                    let len = group.subtypes.len();
                    for (i, (subtype, count)) in group.subtypes.iter().enumerate() {
                        let is_last = i == len - 1;
                        let prefix = if is_last { "  └─ " } else { "  ├─ " };
                        stellar_rows.push(
                            Row::new(vec![
                                format!("{}{}", prefix, subtype),
                                count.to_string(),
                            ])
                            .style(Style::default().fg(ELITE_DIM))
                        );
                    }
                }
            }

            // Group our entries and aggregate sub-attributes for Planetary Codex
            struct PlanetCodexGrouped {
                planet_class: String,
                total_scans: u32,
                landable_count: u32,
                terraformable_count: u32,
                ringed_count: u32,
                life_count: u32,
            }

            let mut grouped: std::collections::HashMap<String, PlanetCodexGrouped> = std::collections::HashMap::new();
            for (key, count) in &trip.planetary_codex {
                let parts: Vec<&str> = key.split('|').collect();
                let planet_class = parts[0].to_string();
                let is_landable = parts.contains(&"L");
                let is_terraformable = parts.contains(&"T");
                let has_rings = parts.contains(&"R");
                let has_life = parts.contains(&"B");

                let entry = grouped.entry(planet_class.clone()).or_insert_with(|| PlanetCodexGrouped {
                    planet_class: planet_class.clone(),
                    total_scans: 0,
                    landable_count: 0,
                    terraformable_count: 0,
                    ringed_count: 0,
                    life_count: 0,
                });

                entry.total_scans += count;
                if is_landable {
                    entry.landable_count += count;
                }
                if is_terraformable {
                    entry.terraformable_count += count;
                }
                if has_rings {
                    entry.ringed_count += count;
                }
                if has_life {
                    entry.life_count += count;
                }
            }

            let mut rare_list = Vec::new();
            let mut terrestrial_list = Vec::new();
            let mut gas_list = Vec::new();

            for entry in grouped.into_values() {
                let cat = crate::app::get_planet_category(&entry.planet_class);
                if cat == "Rare Worlds" {
                    rare_list.push(entry);
                } else if cat == "Gas Giants" {
                    gas_list.push(entry);
                } else {
                    terrestrial_list.push(entry);
                }
            }

            rare_list.sort_by(|a, b| b.total_scans.cmp(&a.total_scans).then_with(|| a.planet_class.cmp(&b.planet_class)));
            terrestrial_list.sort_by(|a, b| b.total_scans.cmp(&a.total_scans).then_with(|| a.planet_class.cmp(&b.planet_class)));
            gas_list.sort_by(|a, b| b.total_scans.cmp(&a.total_scans).then_with(|| a.planet_class.cmp(&b.planet_class)));

            let mut planetary_rows = Vec::new();

            let categories = [
                ("Rare Worlds", rare_list, COLOR_VALUE_HIGH),
                ("Terrestrial Worlds", terrestrial_list, COLOR_PLANET),
                ("Gas Giants", gas_list, COLOR_STAR),
            ];

            for (cat_name, list, cat_color) in categories {
                if !list.is_empty() {
                    let cat_total: u32 = list.iter().map(|e| e.total_scans).sum();
                    planetary_rows.push(
                        Row::new(vec![
                            cat_name.to_string(),
                            cat_total.to_string(),
                        ])
                        .style(Style::default().fg(cat_color).add_modifier(Modifier::BOLD))
                    );

                    let len = list.len();
                    for (i, entry) in list.iter().enumerate() {
                        let is_last = i == len - 1;
                        let prefix = if is_last { "  └─ " } else { "  ├─ " };

                        let mut badges = Vec::new();
                        if entry.landable_count > 0 {
                            badges.push(format!("🚀x{}", entry.landable_count));
                        }
                        if entry.terraformable_count > 0 {
                            badges.push(format!("🌍x{}", entry.terraformable_count));
                        }
                        if entry.ringed_count > 0 {
                            badges.push(format!("🪐x{}", entry.ringed_count));
                        }
                        if entry.life_count > 0 {
                            badges.push(format!("🌿x{}", entry.life_count));
                        }

                        let badges_str = if badges.is_empty() {
                            "".to_string()
                        } else {
                            format!("  ({})", badges.join(" │ "))
                        };

                        planetary_rows.push(
                            Row::new(vec![
                                format!("{}{}{}", prefix, entry.planet_class, badges_str),
                                entry.total_scans.to_string(),
                            ])
                            .style(Style::default().fg(ELITE_DIM))
                        );
                    }
                }
            }

            let total_stellar = stellar_rows.len();
            let total_planetary = planetary_rows.len();

            let split_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(content_area);

            // Draw Stellar Codex on Left
            let header_stellar = Row::new(vec!["Primary Star Class", "Visits"])
                .style(Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

            let table_stellar = Table::new(stellar_rows, [Constraint::Length(30), Constraint::Min(10)])
                .header(header_stellar)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(tab_title("Stellar Codex", Tab::History, app.active_tab))
                        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
                )
                .row_highlight_style(Style::default().bg(HIGHLIGHT_BG))
                .style(Style::default().bg(BG_DARK));

            let selected_stellar = if total_stellar > 0 { Some(app.selected_codex_index % total_stellar) } else { None };
            let mut state_stellar = TableState::default().with_selected(selected_stellar);
            frame.render_stateful_widget(table_stellar, split_chunks[0], &mut state_stellar);

            if total_stellar > (split_chunks[0].height as usize).saturating_sub(4) {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                let mut scrollbar_state = ScrollbarState::new(total_stellar).position(selected_stellar.unwrap_or(0));
                frame.render_stateful_widget(
                    scrollbar,
                    split_chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
                    &mut scrollbar_state,
                );
            }

            // Draw Planetary Codex on Right
            let header_planetary = Row::new(vec!["Planet Class Hierarchy", "Scans"])
                .style(Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

            let table_planetary = Table::new(planetary_rows, [Constraint::Length(42), Constraint::Min(10)])
                .header(header_planetary)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(tab_title("Planetary Codex", Tab::History, app.active_tab))
                        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
                )
                .row_highlight_style(Style::default().bg(HIGHLIGHT_BG))
                .style(Style::default().bg(BG_DARK));

            let selected_planetary = if total_planetary > 0 { Some(app.selected_codex_index % total_planetary) } else { None };
            let mut state_planetary = TableState::default().with_selected(selected_planetary);
            frame.render_stateful_widget(table_planetary, split_chunks[1], &mut state_planetary);

            if total_planetary > (split_chunks[1].height as usize).saturating_sub(4) {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                let mut scrollbar_state = ScrollbarState::new(total_planetary).position(selected_planetary.unwrap_or(0));
                frame.render_stateful_widget(
                    scrollbar,
                    split_chunks[1].inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
                    &mut scrollbar_state,
                );
            }
        }
    }

    // Draw Codex conjoined sub-tabs at the bottom
    let sub_tabs = Line::from(vec![
        Span::styled(" Overview & Biology ", Style::default().fg(if app.active_codex_tab == CodexTab::Overview { COLOR_STAR } else { ELITE_DIM }).add_modifier(if app.active_codex_tab == CodexTab::Overview { Modifier::UNDERLINED | Modifier::BOLD } else { Modifier::empty() })),
        Span::styled(" │ ", Style::default().fg(ELITE_DIM)),
        Span::styled(" Stellar & Planetary ", Style::default().fg(if app.active_codex_tab == CodexTab::Stellar { COLOR_STAR } else { ELITE_DIM }).add_modifier(if app.active_codex_tab == CodexTab::Stellar { Modifier::UNDERLINED | Modifier::BOLD } else { Modifier::empty() })),
    ]);
    frame.render_widget(Paragraph::new(sub_tabs).style(Style::default().bg(BG_DARK)).alignment(ratatui::layout::Alignment::Center), chunks[1]);
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
            Tab::Bodies => "q: quit │ Tab/1/2/3: switch │ ↑↓: navigate │ s: settings │ i: toggle inspector │ ?: help".to_string(),
            Tab::History => "q: quit │ Tab/1/2/3: switch │ ←→/a/d: sub-tabs │ Ctrl+R: reset trip │ ?: help".to_string(),
            Tab::Route => "q: quit │ Tab/1/2/3: switch │ ?: help".to_string(),
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
fn tab_title(name: &str, tab: Tab, active: Tab) -> String {
    let num = match tab {
        Tab::Bodies => "1",
        Tab::History => "2",
        Tab::Route => "3",
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

/// Get the minimum colonial separation distance (in meters) for an exobiology genus.
fn min_separation_for_genus(genus: &str) -> Option<u32> {
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
fn get_main_class(subtype: &str) -> String {
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

/// Calculate the Great-Circle distance in meters between two planetary coordinates using the Haversine formula.
fn calculate_haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64, radius: f64) -> f64 {
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
        
        // Subtypes should be displayed with tree lines
        assert!(output.contains("├─ F9 VAB"), "Should render child F9 VAB");
        assert!(output.contains("40"), "Should show F9 VAB count");
        assert!(output.contains("├─ F1 VA"), "Should render child F1 VA");
        assert!(output.contains("20"), "Should show F1 VA count");
        assert!(output.contains("└─ F2"), "Should render last child F2");
        assert!(output.contains("6"), "Should show F2 count");

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
