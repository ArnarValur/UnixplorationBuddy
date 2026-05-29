use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::{App, Tab};
use super::{ELITE_ORANGE, ELITE_DIM, BG_DARK, COLOR_STAR, HIGHLIGHT_BG, format_credits, tab_title};

pub fn draw_route(frame: &mut Frame, app: &App, area: Rect) {
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
