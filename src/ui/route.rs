use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

use crate::app::{App, Tab};
use super::{ELITE_ORANGE, ELITE_DIM, BG_DARK, COLOR_STAR, HIGHLIGHT_BG, format_credits, tab_title};

/// Non-scoopable star color — muted red.
const COLOR_NON_SCOOPABLE: Color = Color::Rgb(200, 70, 50);

/// Check if a star class is fuel-scoopable (KGBFOAM).
fn is_scoopable(star_class: &str) -> bool {
    matches!(star_class.chars().next(), Some('O') | Some('B') | Some('A') | Some('F') | Some('G') | Some('K') | Some('M'))
}

pub fn draw_route(frame: &mut Frame, app: &App, area: Rect) {
    let route = match &app.plotted_route {
        Some(r) if !r.route.is_empty() => r,
        _ => {
            let content = Paragraph::new(" No plotted navigation route\n Waypoints will sync in real-time when plotted in-game")
                .style(Style::default().fg(ELITE_DIM).bg(BG_DARK))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(tab_title("Route", Tab::Bodies, app.active_tab))
                        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
                );
            frame.render_widget(content, area);
            return;
        }
    };

    let total_jumps = route.route.len();

    // Find current system index for scrolling and progress
    let current_idx = route.route.iter().position(|e| e.star_system == app.system.name);

    // Pre-compute jump distances from StarPos coordinates (Euclidean 3D)
    let mut jump_distances: Vec<f64> = Vec::with_capacity(total_jumps);
    jump_distances.push(0.0); // First waypoint — no prior jump
    for i in 1..total_jumps {
        let prev = &route.route[i - 1].star_pos;
        let curr = &route.route[i].star_pos;
        if prev.len() == 3 && curr.len() == 3 {
            let dx = curr[0] - prev[0];
            let dy = curr[1] - prev[1];
            let dz = curr[2] - prev[2];
            jump_distances.push((dx * dx + dy * dy + dz * dz).sqrt());
        } else {
            jump_distances.push(0.0);
        }
    }

    // Pre-compute remaining distances (suffix sum from each position to destination)
    let mut remaining_distances: Vec<f64> = vec![0.0; total_jumps];
    for i in (0..total_jumps.saturating_sub(1)).rev() {
        remaining_distances[i] = jump_distances[i + 1] + remaining_distances[i + 1];
    }
    let total_distance = remaining_distances[0];

    // Detect non-scoopable streaks (3+ consecutive) for fuel warnings
    let mut fuel_warnings: Vec<bool> = vec![false; total_jumps];
    {
        let mut streak_start = 0usize;
        let mut in_streak = false;
        for i in 0..total_jumps {
            if !is_scoopable(&route.route[i].star_class) {
                if !in_streak {
                    streak_start = i;
                    in_streak = true;
                }
            } else {
                if in_streak && (i - streak_start) >= 3 {
                    for j in streak_start..i {
                        fuel_warnings[j] = true;
                    }
                }
                in_streak = false;
            }
        }
        if in_streak && (total_jumps - streak_start) >= 3 {
            for j in streak_start..total_jumps {
                fuel_warnings[j] = true;
            }
        }
    }

    // Build rows with per-cell styling
    let rows: Vec<Row> = route.route.iter().enumerate().map(|(i, entry)| {
        let scoopable = is_scoopable(&entry.star_class);
        let is_current = current_idx == Some(i);
        let is_visited = current_idx.map_or(false, |ci| i < ci);

        // Row base style
        let row_style = if is_current {
            Style::default().fg(COLOR_STAR)
        } else if is_visited {
            Style::default().fg(ELITE_DIM)
        } else {
            Style::default().fg(ELITE_ORANGE)
        };

        // Star class: red for non-scoopable (unless visited/current)
        let star_style = if is_current {
            row_style
        } else if !scoopable && !is_visited {
            Style::default().fg(COLOR_NON_SCOOPABLE)
        } else {
            row_style
        };

        let scoop_str = if scoopable { "⛽" } else { "—" };

        // Jump distance
        let dist_str = if i == 0 {
            "—".to_string()
        } else {
            format!("{:.1}", jump_distances[i])
        };

        // Remaining distance
        let remaining_str = if remaining_distances[i] > 0.0 {
            format_credits(remaining_distances[i].round() as u64)
        } else {
            "🏁".to_string()
        };
        let remaining_style = if is_current {
            Style::default().fg(COLOR_STAR).add_modifier(Modifier::BOLD)
        } else {
            row_style
        };

        // EDSM data
        let cache = app.edsm_cache.get(&entry.star_system);
        let value_str = cache.map(|c| {
            if c.estimated_value_mapped > 0 { format_credits(c.estimated_value_mapped) } else { "—".into() }
        }).unwrap_or_else(|| "—".into());

        // Notes: discoverer + EDSM badges + fuel warnings
        let mut notes = String::new();
        if let Some(c) = cache {
            if let Some(ref disc) = c.discoverer {
                notes.push_str(disc);
                notes.push(' ');
            }
            if c.valuable_bodies > 0 {
                notes.push_str(&format!("💰{} ", c.valuable_bodies));
            }
            if c.terraformable_bodies > 0 {
                notes.push_str(&format!("🌍{} ", c.terraformable_bodies));
            }
        }
        if fuel_warnings[i] {
            notes.push_str("⚠️ fuel");
        }
        let notes_style = if fuel_warnings[i] && !is_visited {
            Style::default().fg(COLOR_NON_SCOOPABLE)
        } else {
            row_style
        };

        Row::new(vec![
            Cell::from(Span::styled(format!(" #{}", i + 1), row_style)),
            Cell::from(Span::styled(entry.star_system.clone(), row_style)),
            Cell::from(Span::styled(format!("{} {}", entry.star_class, scoop_str), star_style)),
            Cell::from(Span::styled(dist_str, row_style)),
            Cell::from(Span::styled(remaining_str, remaining_style)),
            Cell::from(Span::styled(value_str, row_style)),
            Cell::from(Span::styled(notes.trim().to_string(), notes_style)),
        ])
    }).collect();

    // Header
    let header = Row::new(vec![
        " Jump", "Star System", "Star", "Dist(ly)", "Rem.(ly)", "EDSM Val", "Notes",
    ])
    .style(
        Style::default()
            .fg(ELITE_ORANGE)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    );

    let widths = [
        Constraint::Length(6),
        Constraint::Min(16),
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(10),
        Constraint::Min(8),
    ];

    // Title with progress + distance
    let title = if let Some(ci) = current_idx {
        let rem_ly = remaining_distances[ci].round() as u64;
        let rem_jumps = total_jumps - ci - 1;
        format!("Route [{}/{} · {} ly · {} left]", ci + 1, total_jumps, format_credits(rem_ly), rem_jumps)
    } else {
        format!("Route [{} jumps · {} ly]", total_jumps, format_credits(total_distance.round() as u64))
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title(&title, Tab::Bodies, app.active_tab))
                .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
        )
        .row_highlight_style(Style::default().bg(HIGHLIGHT_BG))
        .style(Style::default().bg(BG_DARK));

    let mut state = TableState::default().with_selected(Some(app.selected_route_index));
    frame.render_stateful_widget(table, area, &mut state);

    if total_jumps > (area.height as usize).saturating_sub(4) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        let pos = app.selected_route_index;
        let mut scrollbar_state = ScrollbarState::new(total_jumps).position(pos);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}
