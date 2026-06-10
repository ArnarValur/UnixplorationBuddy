use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::model::BodyType;
use crate::model::biology::predictor;
use super::{
    ELITE_ORANGE, ELITE_DIM, BG_DARK, COLOR_STAR, COLOR_BIO, COLOR_FIRST, COLOR_VALUE_HIGH,
    format_body_type, format_credits, min_separation_for_genus, calculate_haversine_distance,
};

pub fn draw_inspector(frame: &mut Frame, app: &App, area: Rect) {
    if app.body_display_order.is_empty() {
        return;
    }
    let (body_id, _) = app.body_display_order[app.selected_body_index];
    let body = match app.bodies.get(&body_id) {
        Some(b) => b,
        None => return,
    };

    let binding = format_body_type(body);
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
