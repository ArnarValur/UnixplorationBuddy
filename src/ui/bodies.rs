use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

use crate::app::{App, Tab};
use super::{ELITE_ORANGE, ELITE_DIM, BG_DARK, HIGHLIGHT_BG, body_type_color, format_body_type, format_body_value, format_atmosphere, tab_title};

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
                    let body_type_str = format_body_type(b);

                    // Atmosphere (chemical formula)
                    let atmo = b
                        .atmosphere
                        .as_deref()
                        .map(|a| format_atmosphere(a))
                        .unwrap_or_else(|| "—".into());

                    // Gravity
                    let gravity_str = b.gravity.map(|g| format!("{:.2} G", g)).unwrap_or_else(|| "—".into());

                    // Temp
                    let temp_str = b.temperature.map(|t| format!("{:.0} K", t)).unwrap_or_else(|| "—".into());

                    // Discovered flag
                    let discoverer_str = if b.was_discovered { "✓" } else { "—" };

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

                    cells.extend(vec![dist, scan, value, bio, geo]);

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

    header_cells.extend(vec!["Dist(Ls)", "Scan", "Value(cr)", "Bio", "Geo"]);
    widths.extend(vec![
        Constraint::Length(9),
        Constraint::Length(4),
        Constraint::Length(11),
        Constraint::Length(4),
        Constraint::Length(4),
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
