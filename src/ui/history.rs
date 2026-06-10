use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

use crate::app::{App, Tab, CodexTab};
use super::{
    ELITE_ORANGE, ELITE_DIM, BG_DARK, COLOR_STAR, COLOR_PLANET, COLOR_VALUE_HIGH, COLOR_BIO, COLOR_FIRST,
    HIGHLIGHT_BG, HIGH_VALUE_THRESHOLD, format_credits, tab_title, get_main_class,
};

pub fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
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

            let mut state_bio = TableState::default().with_selected(Some(app.selected_stellar_index));
            frame.render_stateful_widget(table_bio, split_chunks[1], &mut state_bio);

            if total_rows_bio > (split_chunks[1].height as usize).saturating_sub(4) {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                let mut scrollbar_state = ScrollbarState::new(total_rows_bio).position(app.selected_stellar_index);
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
                        let is_last_entry = i == len - 1;
                        let prefix = if is_last_entry { "  └─ " } else { "  ├─ " };

                        // Collect sub-attributes for indented child rows
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

                        // Sub-attribute rows (indented under the planet type)
                        let connector = if is_last_entry { "     " } else { "  │  " };
                        let mut sub_attrs: Vec<(&str, u32)> = Vec::new();
                        if entry.ringed_count > 0 {
                            sub_attrs.push(("Ringed", entry.ringed_count));
                        }
                        if entry.terraformable_count > 0 {
                            sub_attrs.push(("Terraformable", entry.terraformable_count));
                        }
                        if entry.landable_count > 0 {
                            sub_attrs.push(("Landable", entry.landable_count));
                        }
                        if entry.life_count > 0 {
                            sub_attrs.push(("Has Life", entry.life_count));
                        }

                        let sub_len = sub_attrs.len();
                        for (j, (label, count)) in sub_attrs.iter().enumerate() {
                            let sub_prefix = if j == sub_len - 1 { "└─ " } else { "├─ " };
                            planetary_rows.push(
                                Row::new(vec![
                                    format!("{}  {}{}", connector, sub_prefix, label),
                                    count.to_string(),
                                ])
                                .style(Style::default().fg(ELITE_DIM))
                            );
                        }
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

            let selected_stellar = if total_stellar > 0 { Some(app.selected_stellar_index % total_stellar) } else { None };
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

            let selected_planetary = if total_planetary > 0 { Some(app.selected_planetary_index % total_planetary) } else { None };
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
