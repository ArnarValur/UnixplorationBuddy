use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::model::BodyType;
use crate::model::biology::{colors, predictor};
use super::{
    ELITE_ORANGE, ELITE_DIM, BG_DARK, COLOR_STAR, COLOR_BIO, COLOR_FIRST, COLOR_VALUE_HIGH, COLOR_ANOMALY,
    format_body_type, format_credits, format_number, format_volcanism, min_separation_for_genus, calculate_haversine_distance,
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

    // Render outer block, extract inner area for content layout
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Telemetry: {} ", body.short_name))
        .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let is_planetary = body.body_type == BodyType::Planet || body.body_type == BodyType::Moon;

    // ── Header (full-width: Class + Landable) ──
    let binding = format_body_type(body);
    let mut header_lines = vec![
        Line::from(vec![
            Span::styled("Class: ", Style::default().fg(ELITE_DIM)),
            Span::styled(
                body.planet_class.as_deref().unwrap_or(binding.as_str()),
                Style::default().fg(ELITE_ORANGE).add_modifier(Modifier::BOLD)
            ),
        ]),
    ];

    if is_planetary {
        header_lines.push(Line::from(vec![
            Span::styled("Landable: ", Style::default().fg(ELITE_DIM)),
            Span::styled(if body.landable { "YES 🚀" } else { "NO" }, Style::default().fg(if body.landable { COLOR_STAR } else { ELITE_DIM })),
        ]));
    }

    // ── Physical Properties (left column when materials present) ──
    let mut phys_lines: Vec<Line> = Vec::new();
    // ── Materials (right column) ──
    let mut mat_lines: Vec<Line> = Vec::new();
    // ── Everything below the side-by-side section ──
    let mut rest_lines: Vec<Line> = Vec::new();

    if is_planetary {
        // ── Physical ──
        phys_lines.push(Line::from(Span::styled("── Physical ──", Style::default().fg(ELITE_DIM))));

        if let Some(mass) = body.mass {
            phys_lines.push(Line::from(vec![
                Span::styled("Mass: ", Style::default().fg(ELITE_DIM)),
                Span::styled(format!("{:.4} EM", mass), Style::default().fg(ELITE_ORANGE)),
            ]));
        }
        if let Some(radius) = body.radius {
            phys_lines.push(Line::from(vec![
                Span::styled("Rad:  ", Style::default().fg(ELITE_DIM)),
                Span::styled(format!("{} km", format_number(radius / 1000.0)), Style::default().fg(ELITE_ORANGE)),
            ]));
        }
        if let Some(g) = body.gravity {
            phys_lines.push(Line::from(vec![
                Span::styled("Grav: ", Style::default().fg(ELITE_DIM)),
                Span::styled(format!("{:.2} G", g), Style::default().fg(ELITE_ORANGE)),
            ]));
        }
        if let Some(t) = body.temperature {
            phys_lines.push(Line::from(vec![
                Span::styled("Temp: ", Style::default().fg(ELITE_DIM)),
                Span::styled(format!("{:.0}K ({:.0}°C)", t, t - 273.15), Style::default().fg(ELITE_ORANGE)),
            ]));
        }
        if let Some(pressure) = body.pressure_atm {
            phys_lines.push(Line::from(vec![
                Span::styled("Pres: ", Style::default().fg(ELITE_DIM)),
                Span::styled(format!("{:.2} atm", pressure), Style::default().fg(ELITE_ORANGE)),
            ]));
        }

        let atmo_text = body.atmosphere.as_deref().unwrap_or("No Atmosphere");
        phys_lines.push(Line::from(vec![
            Span::styled("Atmo: ", Style::default().fg(ELITE_DIM)),
            Span::styled(atmo_text, Style::default().fg(ELITE_ORANGE)),
        ]));

        if !body.atmosphere_composition.is_empty() {
            let mut atmo = body.atmosphere_composition.clone();
            atmo.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let parts: Vec<String> = atmo.iter()
                .map(|(name, pct)| format!("{} {:.1}%", atmo_formula(name), pct))
                .collect();
            phys_lines.push(Line::from(vec![
                Span::styled("      ", Style::default()),
                Span::styled(parts.join("  "), Style::default().fg(COLOR_FIRST)),
            ]));
        }

        let volc_text = body.volcanism.as_deref().map(format_volcanism).unwrap_or_else(|| "No Volcanism".into());
        phys_lines.push(Line::from(vec![
            Span::styled("Volc: ", Style::default().fg(ELITE_DIM)),
            Span::styled(volc_text, Style::default().fg(ELITE_ORANGE)),
        ]));

        // ── Materials ──
        if !body.surface_materials.is_empty() {
            mat_lines.push(Line::from(Span::styled("── Materials ──", Style::default().fg(ELITE_DIM))));

            let mut mats = body.surface_materials.clone();
            mats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            for chunk in mats.chunks(2) {
                let mut spans = Vec::new();
                for (i, (name, pct)) in chunk.iter().enumerate() {
                    if i > 0 { spans.push(Span::styled("  ", Style::default())); }
                    spans.push(Span::styled(
                        format!("{:>2}", material_symbol(name)),
                        Style::default().fg(COLOR_FIRST).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        format!(" {:<5}", format!("{:.1}%", pct)),
                        Style::default().fg(ELITE_ORANGE),
                    ));
                }
                mat_lines.push(Line::from(spans));
            }
        }

        // ── Orbital (full-width below) ──
        rest_lines.push(Line::from(Span::styled("── Orbital ──", Style::default().fg(ELITE_DIM))));

        let mut orbital_items: Vec<(&str, String)> = Vec::new();
        if let Some(period) = body.orbital_period {
            orbital_items.push(("Orb", format!("{:.1} D", period / 86400.0)));
        }
        if let Some(sma) = body.semi_major_axis {
            orbital_items.push(("SMA", format!("{:.2} AU", sma / 149_597_870_700.0)));
        }
        if let Some(ecc) = body.eccentricity {
            orbital_items.push(("Ecc", format!("{:.4}", ecc)));
        }
        if let Some(inc) = body.inclination {
            orbital_items.push(("Inc", format!("{:.2}°", inc)));
        }
        if let Some(peri) = body.periapsis {
            orbital_items.push(("Per", format!("{:.2}°", peri)));
        }
        if let Some(tilt) = body.axial_tilt {
            orbital_items.push(("Tlt", format!("{:.2}°", tilt.to_degrees())));
        }

        for chunk in orbital_items.chunks(2) {
            let mut spans = Vec::new();
            spans.push(Span::styled(format!("{}: ", chunk[0].0), Style::default().fg(ELITE_DIM)));
            spans.push(Span::styled(format!("{:<10}", chunk[0].1), Style::default().fg(ELITE_ORANGE)));
            if let Some(second) = chunk.get(1) {
                spans.push(Span::styled(format!(" {}: ", second.0), Style::default().fg(ELITE_DIM)));
                spans.push(Span::styled(second.1.clone(), Style::default().fg(ELITE_ORANGE)));
            }
            rest_lines.push(Line::from(spans));
        }

        if let Some(rot) = body.rotational_period {
            let days = rot / 86400.0;
            let tidal = if body.tidal_lock { " (Locked)" } else { "" };
            rest_lines.push(Line::from(vec![
                Span::styled("Rot: ", Style::default().fg(ELITE_DIM)),
                Span::styled(format!("{:.1} D", days), Style::default().fg(ELITE_ORANGE)),
                Span::styled(tidal, Style::default().fg(ELITE_DIM)),
            ]));
        }
    }


    // ── Anomaly / POI Section ──
    if let Some(anomalies) = app.anomalies.get(&body_id) {
        rest_lines.push(Line::from(""));
        rest_lines.push(Line::from(Span::styled("── ANOMALIES / POI ──", Style::default().fg(COLOR_ANOMALY))));
        for anomaly in anomalies {
            rest_lines.push(Line::from(vec![
                Span::styled(format!(" {} ", anomaly.kind.icon()), Style::default().fg(COLOR_ANOMALY)),
                Span::styled(anomaly.kind.label(), Style::default().fg(COLOR_ANOMALY).add_modifier(Modifier::BOLD)),
            ]));
            rest_lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(&anomaly.description, Style::default().fg(ELITE_DIM)),
            ]));
        }
    }

    // ── Exobiology Predictions ──
    rest_lines.push(Line::from(""));
    rest_lines.push(Line::from(Span::styled("── EXOBIOLOGY PREDICTIONS ──", Style::default().fg(ELITE_DIM))));
    if body.bio_signals > 0 && body.landable {
        let primary_star_id = app.system.primary_star_id.unwrap_or(0);
        let primary_star = app.bodies.get(&primary_star_id).and_then(|b| b.star_class_enum.as_ref());
        let predictions = predictor::predict_species(body, primary_star, app.system.region_id);

        if predictions.is_empty() {
            rest_lines.push(Line::from(" No matching species boundaries"));
        } else {
            rest_lines.push(Line::from(format!(" Signals: {} detected", body.bio_signals)));
            rest_lines.push(Line::from(""));
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

            // Resolve color variants using star class and materials
            let star_debug = primary_star.map(|s| format!("{:?}", s)).unwrap_or_default();
            for g in grouped.iter_mut() {
                let method = colors::color_method(&g.base_name);
                match method {
                    colors::ColorMethod::Star => {
                        if let Some(color) = colors::resolve_star_color(&g.base_name, &star_debug) {
                            g.variants = vec![color.to_string()];
                            // Fix active variant to use resolved color (base-name progress
                            // key can match the wrong dataset variant)
                            if g.active_variant.is_some() {
                                g.active_variant = Some(format!("{} - {}", g.base_name, color));
                            }
                        }
                    }
                    colors::ColorMethod::Element => {
                        if let Some(color) = colors::resolve_element_color(&g.base_name, &body.surface_materials) {
                            g.variants = vec![color.to_string()];
                            if g.active_variant.is_some() {
                                g.active_variant = Some(format!("{} - {}", g.base_name, color));
                            }
                        } else if !body.surface_materials.is_empty() {
                            // Materials present but no match — leave variants as-is
                        } else {
                            g.variants = vec!["⚗ scan needed".to_string()];
                        }
                    }
                    colors::ColorMethod::None => {
                        g.variants.clear(); // No color variants
                    }
                }
            }

            // Sort grouped predictions alphabetically by base name
            grouped.sort_by(|a, b| a.base_name.cmp(&b.base_name));

            let mut total_estimated: u64 = 0;
            let mut total_earned: u64 = 0;
            let species_count = grouped.len();
            let mut completed_count: usize = 0;

            for g in grouped {
                let first_discovery_value = g.reward * 5;
                total_estimated += first_discovery_value;

                if g.active_scanned || g.active_progress == 3 {
                    completed_count += 1;
                    total_earned += first_discovery_value;
                    rest_lines.push(Line::from(vec![
                        Span::styled(format!(" R ▸ {} ", g.active_variant.as_deref().unwrap_or(&g.base_name)), Style::default().fg(COLOR_BIO).add_modifier(Modifier::BOLD)),
                        Span::styled("[Completed]", Style::default().fg(COLOR_BIO)),
                    ]));
                } else if g.active_progress > 0 {
                    let sep_str = min_separation_for_genus(&g.genus)
                        .map(|d| format!(" | {}m", d))
                        .unwrap_or_default();
                    rest_lines.push(Line::from(vec![
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

                            rest_lines.push(Line::from(vec![
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
                    rest_lines.push(Line::from(vec![
                        Span::styled(format!(" ▸ {}{} ", g.base_name, variants_str), Style::default().fg(ELITE_ORANGE)),
                        Span::styled(format!(": {} cr (First)", format_credits(first_discovery_value)), Style::default().fg(COLOR_FIRST)),
                        sep_span,
                    ]));
                }
            }

            // Bio value summary
            if species_count > 0 {
                rest_lines.push(Line::from(Span::styled(" ─────────────────────────────────", Style::default().fg(ELITE_DIM))));
                if completed_count == species_count {
                    // All done — celebratory total
                    rest_lines.push(Line::from(vec![
                        Span::styled(" Total earned: ", Style::default().fg(COLOR_BIO).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} cr ✓", format_credits(total_earned)), Style::default().fg(COLOR_BIO).add_modifier(Modifier::BOLD)),
                    ]));
                } else if completed_count > 0 {
                    // Partial progress — earned / estimated
                    rest_lines.push(Line::from(vec![
                        Span::styled(" Earned: ", Style::default().fg(COLOR_FIRST)),
                        Span::styled(format!("{} cr", format_credits(total_earned)), Style::default().fg(COLOR_FIRST).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" / Est: {} cr", format_credits(total_estimated)), Style::default().fg(ELITE_DIM)),
                    ]));
                } else {
                    // Nothing scanned yet — just estimate
                    rest_lines.push(Line::from(vec![
                        Span::styled(" Est. total: ", Style::default().fg(ELITE_DIM)),
                        Span::styled(format!("~{} cr", format_credits(total_estimated)), Style::default().fg(COLOR_FIRST)),
                    ]));
                }
            }
        }
    } else if body.bio_signals > 0 {
        rest_lines.push(Line::from(" Not landable (exobiology locked)"));
    } else {
        rest_lines.push(Line::from(" No bio signals reported"));
    }

    // ══════════════════════════════════════════════════════
    let header_h = header_lines.len() as u16;
    let bg = Style::default().bg(BG_DARK);
    let rest_total = rest_lines.len() as u16;

    if is_planetary && !mat_lines.is_empty() {
        // Side-by-side: Physical (left) | Materials (right)
        let side_h = std::cmp::max(phys_lines.len(), mat_lines.len()) as u16;

        let rows = Layout::vertical([
            Constraint::Length(header_h),
            Constraint::Length(side_h),
            Constraint::Min(0),
        ]).split(inner);

        frame.render_widget(Paragraph::new(header_lines).style(bg), rows[0]);

        let cols = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(rows[1]);

        frame.render_widget(Paragraph::new(phys_lines).style(bg), cols[0]);
        frame.render_widget(Paragraph::new(mat_lines).style(bg), cols[1]);

        let rest_h = rows[2].height;
        let scroll = clamp_scroll(app.inspector_scroll, rest_total, rest_h);
        let rest_para = Paragraph::new(rest_lines).scroll((scroll, 0)).style(bg);
        frame.render_widget(rest_para, rows[2]);

        if rest_total > rest_h {
            render_scroll_hint(frame, rows[2], scroll, rest_total, rest_h);
        }
    } else if is_planetary {
        // No materials — physical full-width, then rest
        let phys_h = phys_lines.len() as u16;

        let rows = Layout::vertical([
            Constraint::Length(header_h),
            Constraint::Length(phys_h),
            Constraint::Min(0),
        ]).split(inner);

        frame.render_widget(Paragraph::new(header_lines).style(bg), rows[0]);
        frame.render_widget(Paragraph::new(phys_lines).style(bg), rows[1]);

        let rest_h = rows[2].height;
        let scroll = clamp_scroll(app.inspector_scroll, rest_total, rest_h);
        let rest_para = Paragraph::new(rest_lines).scroll((scroll, 0)).style(bg);
        frame.render_widget(rest_para, rows[2]);

        if rest_total > rest_h {
            render_scroll_hint(frame, rows[2], scroll, rest_total, rest_h);
        }
    } else {
        // Star / non-planetary — header + rest only
        let rows = Layout::vertical([
            Constraint::Length(header_h),
            Constraint::Min(0),
        ]).split(inner);

        frame.render_widget(Paragraph::new(header_lines).style(bg), rows[0]);

        let rest_h = rows[1].height;
        let scroll = clamp_scroll(app.inspector_scroll, rest_total, rest_h);
        let rest_para = Paragraph::new(rest_lines).scroll((scroll, 0)).style(bg);
        frame.render_widget(rest_para, rows[1]);

        if rest_total > rest_h {
            render_scroll_hint(frame, rows[1], scroll, rest_total, rest_h);
        }
    }
}

/// Clamp scroll offset so it never scrolls past the last line of content.
fn clamp_scroll(scroll: u16, content_lines: u16, viewport_h: u16) -> u16 {
    if content_lines <= viewport_h {
        0
    } else {
        scroll.min(content_lines.saturating_sub(viewport_h))
    }
}

/// Render a dim scroll indicator in the bottom-right corner of the area.
fn render_scroll_hint(frame: &mut Frame, area: Rect, scroll: u16, total: u16, viewport_h: u16) {
    let max_scroll = total.saturating_sub(viewport_h);
    let indicator = if scroll == 0 {
        " ▼ PgDn "
    } else if scroll >= max_scroll {
        " ▲ PgUp "
    } else {
        " ▲▼ Pg "
    };
    let hint_w = indicator.len() as u16;
    if area.width >= hint_w && area.height > 0 {
        let hint_area = Rect::new(
            area.x + area.width - hint_w,
            area.y + area.height - 1,
            hint_w,
            1,
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(indicator, Style::default().fg(ELITE_DIM).bg(BG_DARK)))),
            hint_area,
        );
    }
}

/// Map Elite Dangerous material names to chemical element symbols.
fn material_symbol(name: &str) -> &str {
    match name.to_lowercase().as_str() {
        "iron" => "Fe",
        "nickel" => "Ni",
        "sulphur" => "S",
        "carbon" => "C",
        "phosphorus" => "P",
        "manganese" => "Mn",
        "zinc" => "Zn",
        "chromium" => "Cr",
        "germanium" => "Ge",
        "vanadium" => "V",
        "niobium" => "Nb",
        "molybdenum" => "Mo",
        "tin" => "Sn",
        "tungsten" => "W",
        "arsenic" => "As",
        "selenium" => "Se",
        "cadmium" => "Cd",
        "tellurium" => "Te",
        "ruthenium" => "Ru",
        "technetium" => "Tc",
        "mercury" => "Hg",
        "polonium" => "Po",
        "yttrium" => "Y",
        "zirconium" => "Zr",
        "antimony" => "Sb",
        "lead" => "Pb",
        "boron" => "B",
        "rhenium" => "Re",
        _ => "??",
    }
}

/// Map atmosphere element names to chemical formulas.
fn atmo_formula(name: &str) -> &str {
    match name {
        "Hydrogen" => "H₂",
        "Helium" => "He",
        "Water" => "H₂O",
        "Oxygen" => "O₂",
        "CarbonDioxide" => "CO₂",
        "SulphurDioxide" => "SO₂",
        "Ammonia" => "NH₃",
        "Methane" => "CH₄",
        "Nitrogen" => "N₂",
        "Neon" => "Ne",
        "Argon" => "Ar",
        "Silicates" => "SiO₂",
        "Iron" => "Fe",
        _ => name,
    }
}
