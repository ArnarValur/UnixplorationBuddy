use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Tab};
use super::{ELITE_ORANGE, ELITE_DIM, BG_DARK, body_type_color, tab_title};

// Kepler eccentric anomaly solver using Newton-Raphson method
pub fn solve_kepler(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let mut eccentric_anomaly = mean_anomaly;
    for _ in 0..5 {
        let diff = eccentric_anomaly - eccentricity * eccentric_anomaly.sin() - mean_anomaly;
        let deriv = 1.0 - eccentricity * eccentric_anomaly.cos();
        if deriv.abs() < 1e-9 {
            break;
        }
        eccentric_anomaly -= diff / deriv;
    }
    eccentric_anomaly
}

// 3D Isometric View Camera Projection (pitch: 30°, yaw: 45°)
pub fn project_3d_to_2d(x: f64, y: f64, z: f64) -> (f64, f64) {
    let yaw = 45.0 * std::f64::consts::PI / 180.0;
    let pitch = 30.0 * std::f64::consts::PI / 180.0;
    
    let x_screen = x * yaw.cos() - y * yaw.sin();
    let y_screen = (x * yaw.sin() + y * yaw.cos()) * pitch.cos() - z * pitch.sin();
    (x_screen, y_screen)
}

// Draw Orrery Map canvas using high-res Braille sub-pixel symbols
pub fn draw_orrery(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::canvas::{Canvas, Line};
    
    if app.body_display_order.is_empty() {
        let content = Paragraph::new(" No bodies discovered yet")
            .style(Style::default().fg(ELITE_DIM).bg(BG_DARK))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(tab_title("Orrery Map", Tab::Bodies, app.active_tab))
                    .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
            );
        frame.render_widget(content, area);
        return;
    }

    let primary_star_id = app.system.primary_star_id.unwrap_or(0);
    let selected_body_id = app.body_display_order
        .get(app.selected_body_index)
        .map(|&(id, _)| id)
        .unwrap_or(primary_star_id);

    // --- Dynamic Stable Normalization ---
    // 1. Find max semi-major axis for all planet/star bodies (excluding Moons/BeltClusters)
    let mut max_planet_a: f64 = 1.0;
    for &(body_id, _, _) in &app.body_display_order {
        if body_id == primary_star_id {
            continue;
        }
        if let Some(b) = app.bodies.get(&body_id) {
            if b.body_type != crate::model::BodyType::Moon && b.body_type != crate::model::BodyType::BeltCluster {
                let a = b.semi_major_axis.unwrap_or_else(|| {
                    b.distance_ls.map(|d| d * 2.99792e8).unwrap_or((body_id as f64) * 5e9)
                });
                max_planet_a = max_planet_a.max(a);
            }
        }
    }
    
    let u_max = (1.0 + max_planet_a / 1e9).ln();
    let c_planet = if u_max > 0.0 { 26.0 / u_max } else { 15.0 };

    // 2. Find max semi-major axis for moons of each planet
    let mut max_moon_a: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    for &(body_id, _, _) in &app.body_display_order {
        if let Some(b) = app.bodies.get(&body_id) {
            if let Some(pid) = b.parent_id {
                if b.body_type == crate::model::BodyType::Moon {
                    let a = b.semi_major_axis.unwrap_or_else(|| {
                        b.distance_ls.map(|d| d * 2.99792e8).unwrap_or((body_id as f64) * 1e8)
                    });
                    let entry = max_moon_a.entry(pid).or_insert(1.0);
                    *entry = entry.max(a);
                }
            }
        }
    }

    let mut c_moons: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    for (&pid, &max_a) in &max_moon_a {
        let w_max = (1.0 + max_a / 1e7).ln();
        let c_m = if w_max > 0.0 { 4.5 / w_max } else { 3.5 };
        c_moons.insert(pid, c_m);
    }

    // 3. Resolve absolute 3D scaled coordinates hierarchically in one sequential pass
    let mut abs_scaled_positions: std::collections::HashMap<u32, (f64, f64, f64)> = std::collections::HashMap::new();
    abs_scaled_positions.insert(primary_star_id, (0.0, 0.0, 0.0));

    for &(body_id, _, _) in &app.body_display_order {
        if body_id == primary_star_id {
            continue;
        }
        if let Some(b) = app.bodies.get(&body_id) {
            let parent_pos = b.parent_id
                .and_then(|pid| abs_scaled_positions.get(&pid).copied())
                .unwrap_or((0.0, 0.0, 0.0));

            let relative_pos = if let (Some(a), Some(e), Some(incl), Some(peri), Some(period)) = (
                b.semi_major_axis,
                b.eccentricity,
                b.inclination,
                b.periapsis,
                b.orbital_period,
            ) {
                let mean_anom_epoch = b.mean_anomaly.unwrap_or(0.0);
                let node = b.ascending_node.unwrap_or(0.0);

                let n = 2.0 * std::f64::consts::PI / period;
                let m = (mean_anom_epoch * std::f64::consts::PI / 180.0) + n * app.sim_time;

                let eccentric_anomaly = solve_kepler(m, e);

                let x_plane = a * (eccentric_anomaly.cos() - e);
                let y_plane = a * (1.0 - e * e).sqrt() * eccentric_anomaly.sin();

                let i_rad = incl * std::f64::consts::PI / 180.0;
                let w_rad = peri * std::f64::consts::PI / 180.0;
                let node_rad = node * std::f64::consts::PI / 180.0;

                let cos_w = w_rad.cos();
                let sin_w = w_rad.sin();
                let cos_node = node_rad.cos();
                let sin_node = node_rad.sin();
                let cos_i = i_rad.cos();
                let sin_i = i_rad.sin();

                let x_3d = x_plane * (cos_w * cos_node - sin_w * sin_node * cos_i) - y_plane * (sin_w * cos_node + cos_w * sin_node * cos_i);
                let y_3d = x_plane * (cos_w * sin_node + sin_w * cos_node * cos_i) - y_plane * (sin_w * sin_node - cos_w * cos_node * cos_i);
                let z_3d = x_plane * (sin_w * sin_i) + y_plane * (cos_w * sin_i);

                (x_3d, y_3d, z_3d)
            } else {
                let r = b.distance_ls.map(|d| d * 2.99792e8).unwrap_or((body_id as f64) * 5e9);
                let speed = 2.0 * std::f64::consts::PI / (r.powf(1.5) * 1e-6).max(100.0);
                let angle = speed * app.sim_time;
                let x_3d = r * angle.cos();
                let y_3d = r * angle.sin();
                let z_3d = 0.0;
                (x_3d, y_3d, z_3d)
            };

            let r_mag = (relative_pos.0.powi(2) + relative_pos.1.powi(2) + relative_pos.2.powi(2)).sqrt();
            let is_moon = b.body_type == crate::model::BodyType::Moon;
            
            let r_scaled = if is_moon {
                let c_m = b.parent_id.and_then(|pid| c_moons.get(&pid).copied()).unwrap_or(3.5);
                c_m * (1.0 + r_mag / 1e7).ln()
            } else {
                c_planet * (1.0 + r_mag / 1e9).ln()
            };

            let scaled_rel = if r_mag > 1.0 {
                (
                    (r_scaled / r_mag) * relative_pos.0,
                    (r_scaled / r_mag) * relative_pos.1,
                    (r_scaled / r_mag) * relative_pos.2,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            let abs_pos = (
                parent_pos.0 + scaled_rel.0,
                parent_pos.1 + scaled_rel.1,
                parent_pos.2 + scaled_rel.2,
            );

            abs_scaled_positions.insert(body_id, abs_pos);
        }
    }

    // 4. Resolve orbital path lines for all scanned bodies
    let mut orbits = Vec::new();
    for &(body_id, _, _) in &app.body_display_order {
        if body_id == primary_star_id {
            continue;
        }
        if let Some(b) = app.bodies.get(&body_id) {
            let parent_pos = b.parent_id
                .and_then(|pid| abs_scaled_positions.get(&pid).copied())
                .unwrap_or((0.0, 0.0, 0.0));

            let is_moon = b.body_type == crate::model::BodyType::Moon;
            let scale_factor = if is_moon {
                b.parent_id.and_then(|pid| c_moons.get(&pid).copied()).unwrap_or(3.5)
            } else {
                c_planet
            };
            let r0 = if is_moon { 1e7 } else { 1e9 };

            let mut points = Vec::new();
            let steps = 32;

            for step in 0..=steps {
                let step_angle = (step as f64) * 2.0 * std::f64::consts::PI / (steps as f64);
                
                let rel_pos = if let (Some(a), Some(e), Some(incl), Some(peri)) = (
                    b.semi_major_axis,
                    b.eccentricity,
                    b.inclination,
                    b.periapsis,
                ) {
                    let node = b.ascending_node.unwrap_or(0.0);

                    let x_plane = a * (step_angle.cos() - e);
                    let y_plane = a * (1.0 - e * e).sqrt() * step_angle.sin();

                    let i_rad = incl * std::f64::consts::PI / 180.0;
                    let w_rad = peri * std::f64::consts::PI / 180.0;
                    let node_rad = node * std::f64::consts::PI / 180.0;

                    let cos_w = w_rad.cos();
                    let sin_w = w_rad.sin();
                    let cos_node = node_rad.cos();
                    let sin_node = node_rad.sin();
                    let cos_i = i_rad.cos();
                    let sin_i = i_rad.sin();

                    let x_3d = x_plane * (cos_w * cos_node - sin_w * sin_node * cos_i) - y_plane * (sin_w * cos_node + cos_w * sin_node * cos_i);
                    let y_3d = x_plane * (cos_w * sin_node + sin_w * cos_node * cos_i) - y_plane * (sin_w * sin_node - cos_w * cos_node * cos_i);
                    let z_3d = x_plane * (sin_w * sin_i) + y_plane * (cos_w * sin_i);

                    (x_3d, y_3d, z_3d)
                } else {
                    let r = b.distance_ls.map(|d| d * 2.99792e8).unwrap_or((body_id as f64) * 5e9);
                    let x_3d = r * step_angle.cos();
                    let y_3d = r * step_angle.sin();
                    let z_3d = 0.0;
                    (x_3d, y_3d, z_3d)
                };

                let r_mag = (rel_pos.0.powi(2) + rel_pos.1.powi(2) + rel_pos.2.powi(2)).sqrt();
                let r_scaled = scale_factor * (1.0 + r_mag / r0).ln();
                let scaled_rel = if r_mag > 1.0 {
                    (
                        (r_scaled / r_mag) * rel_pos.0,
                        (r_scaled / r_mag) * rel_pos.1,
                        (r_scaled / r_mag) * rel_pos.2,
                    )
                } else {
                    (0.0, 0.0, 0.0)
                };

                let abs_pt = (
                    parent_pos.0 + scaled_rel.0,
                    parent_pos.1 + scaled_rel.1,
                    parent_pos.2 + scaled_rel.2,
                );

                let (xs, ys) = project_3d_to_2d(abs_pt.0, abs_pt.1, abs_pt.2);
                points.push((xs, ys));
            }
            orbits.push((body_id, points));
        }
    }

    let speed_text = format!("Speed: {:.3}x", app.sim_speed);

    // Pre-calculate owned drawable body components to capture by value into static closure
    struct DrawableBody {
        xs: f64,
        ys: f64,
        symbol: &'static str,
        short_name: String,
        body_color: Color,
        label_style: Style,
    }

    let mut drawables = Vec::new();
    for &(body_id, _, _) in &app.body_display_order {
        if body_id == primary_star_id {
            continue;
        }
        if let Some(b) = app.bodies.get(&body_id) {
            if let Some(&pos) = abs_scaled_positions.get(&body_id) {
                let (xs, ys) = project_3d_to_2d(pos.0, pos.1, pos.2);
                let is_selected = body_id == selected_body_id;
                
                let symbol = if b.body_type == crate::model::BodyType::Moon {
                    "•"
                } else {
                    "●"
                };

                let body_color = if is_selected {
                    Color::Yellow
                } else {
                    body_type_color(b.body_type)
                };

                let label_color = if is_selected {
                    Color::White
                } else {
                    Color::Rgb(160, 160, 160)
                };

                let label_style = if is_selected {
                    Style::default().fg(label_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(label_color)
                };

                drawables.push(DrawableBody {
                    xs,
                    ys,
                    symbol,
                    short_name: b.short_name.clone(),
                    body_color,
                    label_style,
                });
            }
        }
    }

    let active_tab = app.active_tab;
    let canvas_max_x = 45.0;
    let canvas_max_y = 35.0;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_title("Orrery Map", Tab::Bodies, active_tab))
                .style(Style::default().fg(ELITE_ORANGE).bg(BG_DARK)),
        )
        .x_bounds([-canvas_max_x, canvas_max_x])
        .y_bounds([-canvas_max_y, canvas_max_y])
        .marker(ratatui::symbols::Marker::Braille)
        .paint(move |ctx| {
            // 1. Draw orbits
            for &(body_id, ref pts) in &orbits {
                let is_selected = body_id == selected_body_id;
                let orbit_color = if is_selected {
                    Color::Yellow
                } else {
                    ELITE_DIM
                };

                for i in 0..pts.len() - 1 {
                    ctx.draw(&Line {
                        x1: pts[i].0,
                        y1: pts[i].1,
                        x2: pts[i+1].0,
                        y2: pts[i+1].1,
                        color: orbit_color,
                    });
                }
            }

            // 2. Draw Center Primary Star
            let (star_x, star_y) = project_3d_to_2d(0.0, 0.0, 0.0);
            ctx.print(
                star_x - 0.5,
                star_y - 0.25,
                Span::styled("★", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            );

            // 3. Draw Bodies (Planets and Moons) and Labels
            for b in &drawables {
                ctx.print(
                    b.xs - 0.4,
                    b.ys - 0.25,
                    Span::styled(b.symbol, Style::default().fg(b.body_color).add_modifier(Modifier::BOLD)),
                );

                ctx.print(
                    b.xs + 0.8,
                    b.ys - 0.25,
                    Span::styled(b.short_name.clone(), b.label_style),
                );
            }

            // 4. Render simulation speed multiplier overlay inside canvas
            ctx.print(
                canvas_max_x - (speed_text.len() as f64) * 0.9 - 1.0,
                canvas_max_y - 2.0,
                Span::styled(speed_text.clone(), Style::default().fg(ELITE_DIM)),
            );
        });

    frame.render_widget(canvas, area);
}
