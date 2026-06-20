use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

use crate::app::{App, Tab};
use super::{ELITE_ORANGE, ELITE_DIM, BG_DARK, HIGHLIGHT_BG, COLOR_ANOMALY, COLOR_BIO, COLOR_GEO, COLOR_VALUE_HIGH, HIGH_VALUE_THRESHOLD, body_display_color, format_body_type, format_body_value, format_atmosphere, tab_title};

/// Bodies tab — hierarchical, scrollable body table with color-coded types.
pub fn draw_bodies(frame: &mut Frame, app: &App, area: Rect) {
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

    // Split the left content pane vertically to reserve a 1-line subtab selection bar at the bottom
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Content (hierarchical table OR 3D Orrery canvas)
            Constraint::Length(1),   // Centered subtab selector
        ])
        .split(main_chunks[0]);

    let content_area = left_layout[0];
    let selector_area = left_layout[1];

    // Center-aligned subtab bar at the bottom
    let (tab1_prefix, tab2_prefix) = match app.bodies_subtab {
        crate::app::BodiesSubTab::Table => ("● ", "○ "),
        crate::app::BodiesSubTab::Route => ("○ ", "● "),
    };

    let selector_spans = vec![
        Span::styled(format!("{}{}", tab1_prefix, "System Map"), Style::default().fg(if app.bodies_subtab == crate::app::BodiesSubTab::Table { ELITE_ORANGE } else { ELITE_DIM })),
        Span::styled("   │   ", Style::default().fg(ELITE_DIM)),
        Span::styled(format!("{}{}", tab2_prefix, "Route"), Style::default().fg(if app.bodies_subtab == crate::app::BodiesSubTab::Route { ELITE_ORANGE } else { ELITE_DIM })),
    ];
    let selector_para = Paragraph::new(Line::from(selector_spans))
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().bg(BG_DARK));
    frame.render_widget(selector_para, selector_area);

    if app.bodies_subtab == crate::app::BodiesSubTab::Route {
        super::route::draw_route(frame, app, content_area);
        
        // Render inspector right pane if split-screen is active
        if show_inspect {
            super::inspector::draw_inspector(frame, app, main_chunks[1]);
        }
        return;
    }

    let table_area = content_area;

    // ── Tree guide state ────────────────────────────────────────
    // Track which ancestor depths have continuation lines (│).
    // ancestor_is_last[d] = true means the body at depth d was the last sibling,
    // so no vertical continuation line is drawn for that column.
    let mut ancestor_is_last: Vec<bool> = Vec::new();

    // Build table rows from display order
    let rows: Vec<Row> = app
        .body_display_order
        .iter()
        .enumerate()
        .map(|(i, &(body_id, depth, is_last))| {
            // Update tree guide stack
            ancestor_is_last.truncate(depth as usize);
            ancestor_is_last.push(is_last);

            let body = app.bodies.get(&body_id);

            // Build tree guide prefix
            let mut prefix = String::new();
            if depth > 0 {
                for d in 1..depth as usize {
                    if ancestor_is_last[d] {
                        prefix.push_str("   ");
                    } else {
                        prefix.push_str("│  ");
                    }
                }
                if is_last {
                    prefix.push_str("└─ ");
                } else {
                    prefix.push_str("├─ ");
                }
            }

            match body {
                Some(b) => {
                    let type_color = body_display_color(b, depth);
                    let is_selected = i == app.selected_body_index;

                    // Name cell with tree guide prefix (dim) + body name (colored)
                    let name_cell = Cell::from(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(ELITE_DIM)),
                        Span::styled(b.short_name.clone(), Style::default().fg(type_color)),
                    ]));

                    // Body type label
                    let body_type_str = format_body_type(b);

                    // Atmosphere (chemical formula)
                    let atmo = b
                        .atmosphere
                        .as_deref()
                        .map(|a| format_atmosphere(a))
                        .unwrap_or_default();

                    // Gravity
                    let gravity_str = b.gravity.map(|g| format!("{:.2} G", g)).unwrap_or_default();

                    // Temp
                    let temp_str = b.temperature.map(|t| format!("{:.0} K", t)).unwrap_or_default();

                    // Discovered flag
                    let discoverer_str = if b.was_discovered { "✓" } else { "" };

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
                        .unwrap_or_default();

                    // Scan state icon
                    let scan = b.scan_state.icon().to_string();

                    // Value display — show mapped_value for DSS'd bodies
                    let value_str = format_body_value(b);
                    let value_num = if b.scan_state >= crate::model::ScanState::DSSMapped && b.mapped_value > 0 {
                        b.mapped_value
                    } else {
                        b.calculated_value
                    };
                    let value_color = if value_num >= HIGH_VALUE_THRESHOLD {
                        COLOR_VALUE_HIGH
                    } else {
                        type_color
                    };

                    // Bio/Geo signal counts — colored per-cell
                    let bio_cell = if b.bio_signals > 0 {
                        Cell::from(b.bio_signals.to_string()).style(Style::default().fg(COLOR_BIO))
                    } else {
                        Cell::from("")
                    };
                    let geo_cell = if b.geo_signals > 0 {
                        Cell::from(b.geo_signals.to_string()).style(Style::default().fg(COLOR_GEO))
                    } else {
                        Cell::from("")
                    };

                    // Anomaly / POI badges
                    let poi_cell = if let Some(anomalies) = app.anomalies.get(&body_id) {
                        let badges = anomalies.iter().map(|a| a.kind.icon()).collect::<Vec<_>>().join("");
                        Cell::from(badges).style(Style::default().fg(COLOR_ANOMALY))
                    } else {
                        Cell::from("")
                    };

                    let row_bg = if is_selected {
                        Some(HIGHLIGHT_BG)
                    } else {
                        None
                    };
                    let base_style = if let Some(bg) = row_bg {
                        Style::default().fg(type_color).bg(bg)
                    } else {
                        Style::default().fg(type_color)
                    };

                    let mut cells: Vec<Cell> = vec![
                        name_cell,
                        Cell::from(body_type_str),
                    ];

                    if app.column_settings.show_atmosphere {
                        cells.push(Cell::from(atmo));
                    }
                    if app.column_settings.show_gravity {
                        cells.push(Cell::from(gravity_str));
                    }
                    if app.column_settings.show_temperature {
                        cells.push(Cell::from(temp_str));
                    }
                    if app.column_settings.show_discoverer {
                        cells.push(Cell::from(discoverer_str.to_string()));
                    }

                    cells.extend(vec![
                        Cell::from(dist),
                        Cell::from(scan),
                        Cell::from(value_str).style(Style::default().fg(value_color)),
                        bio_cell,
                        geo_cell,
                    ]);

                    // POI column last
                    cells.push(poi_cell);

                    Row::new(cells).style(base_style)
                }
                None => {
                    let style = if i == app.selected_body_index {
                        Style::default().fg(ELITE_DIM).bg(HIGHLIGHT_BG)
                    } else {
                        Style::default().fg(ELITE_DIM)
                    };

                    let name_cell = Cell::from(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(ELITE_DIM)),
                        Span::styled("?", Style::default().fg(ELITE_DIM)),
                    ]));

                    let mut cells: Vec<Cell> = vec![name_cell, Cell::from("?")];

                    if app.column_settings.show_atmosphere {
                        cells.push(Cell::from(""));
                    }
                    if app.column_settings.show_gravity {
                        cells.push(Cell::from(""));
                    }
                    if app.column_settings.show_temperature {
                        cells.push(Cell::from(""));
                    }
                    if app.column_settings.show_discoverer {
                        cells.push(Cell::from(""));
                    }

                    cells.extend(vec![
                        Cell::from(""),
                        Cell::from("○"),
                        Cell::from(""),
                        Cell::from(""),
                        Cell::from(""),
                        Cell::from(""),
                    ]);

                    Row::new(cells).style(style)
                }
            }
        })
        .collect();

    let mut header_cells = vec!["Name", "Type"];
    let mut widths = vec![Constraint::Min(16), Constraint::Length(14)];

    if app.column_settings.show_atmosphere {
        header_cells.push("Atmo");
        widths.push(Constraint::Length(10));
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
        header_cells.push("Disc");
        widths.push(Constraint::Length(6));
    }

    header_cells.extend(vec!["Dist(Ls)", "Scan", "Value(cr)", "Bio", "Geo", "POI"]);
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
        super::inspector::draw_inspector(frame, app, main_chunks[1]);
    }
}
