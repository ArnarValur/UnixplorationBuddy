//! POI / Tourist Anomaly Detection Engine
//!
//! Pure computational algorithms that run on body scan data to detect
//! notable orbital configurations, unusual bodies, and points of interest.
//!
//! Ported from EDMC-Canonn's `codex.py` anomaly detection logic.
//! All detectors are pure functions: `fn(body, siblings/system) -> Vec<Anomaly>`.

use std::collections::HashMap;

use super::body::{Body, BodyType};

/// A detected anomaly or point of interest on a body or body pair.
#[derive(Debug, Clone, PartialEq)]
pub struct Anomaly {
    /// Which body this anomaly applies to (by body_id).
    pub body_id: u32,
    /// Classification of the anomaly.
    pub kind: AnomalyKind,
    /// Human-readable description for the TUI.
    pub description: String,
}

/// Classification of detected anomalies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnomalyKind {
    // --- Orbital anomalies ---
    /// Body orbits dangerously close to its parent (apoapsis < 2× combined radii).
    CloseOrbit,
    /// Same as CloseOrbit but the body is also landable.
    CloseOrbitLandable,
    /// Two siblings whose orbits approach within 100km, synodic period < 40 days.
    CloseFlypast,
    /// Two siblings whose orbits actually overlap (approach distance < 0).
    CollisionFlypast,

    // --- Body classification anomalies ---
    /// Bodies sharing the same orbit (matching SMA, eccentricity, inclination, period).
    Trojan,
    /// A star orbiting as a satellite (moon-like position in the hierarchy).
    SatelliteStar,
    /// A body nested 3+ levels deep in the planet chain (moon of moon of moon).
    DeeplyNested { depth: u32 },
}

impl AnomalyKind {
    /// Short icon/badge for TUI display.
    pub fn icon(&self) -> &'static str {
        match self {
            AnomalyKind::CloseOrbit => "⚠️",
            AnomalyKind::CloseOrbitLandable => "🛬",
            AnomalyKind::CloseFlypast => "💫",
            AnomalyKind::CollisionFlypast => "💥",
            AnomalyKind::Trojan => "♊",
            AnomalyKind::SatelliteStar => "🌟",
            AnomalyKind::DeeplyNested { .. } => "🌙",
        }
    }

    /// Short label for TUI display.
    pub fn label(&self) -> &'static str {
        match self {
            AnomalyKind::CloseOrbit => "Close Orbit",
            AnomalyKind::CloseOrbitLandable => "Close Orbit (Landable)",
            AnomalyKind::CloseFlypast => "Close Flypast",
            AnomalyKind::CollisionFlypast => "Collision Course",
            AnomalyKind::Trojan => "Trojan",
            AnomalyKind::SatelliteStar => "Satellite Star",
            AnomalyKind::DeeplyNested { depth } => match depth {
                3 => "Moon³",
                4 => "Moon⁴",
                _ => "Deep Moon",
            },
        }
    }
}

// ============================================================================
// Orbital math helpers
// ============================================================================

/// Convert semi-major axis (meters) and eccentricity to apoapsis in meters.
pub fn apoapsis_m(semi_major_axis_m: f64, eccentricity: f64) -> f64 {
    semi_major_axis_m * (1.0 + eccentricity)
}

/// Convert semi-major axis (meters) and eccentricity to periapsis in meters.
pub fn periapsis_m(semi_major_axis_m: f64, eccentricity: f64) -> f64 {
    semi_major_axis_m * (1.0 - eccentricity)
}

/// Calculate the synodic period of two bodies given their orbital periods (in seconds).
/// Returns `None` if both periods are equal (co-orbital).
pub fn synodic_period(period_a: f64, period_b: f64) -> Option<f64> {
    let diff = (1.0 / period_a - 1.0 / period_b).abs();
    if diff < 1e-15 {
        None // co-orbital
    } else {
        Some(1.0 / diff)
    }
}

/// Calculate the closest approach distance between two co-planar elliptical orbits.
/// Uses periapsis of outer orbit minus apoapsis of inner orbit.
/// Returns the minimum distance in meters (can be negative if orbits overlap).
pub fn closest_approach_m(
    sma_a: f64, ecc_a: f64,
    sma_b: f64, ecc_b: f64,
) -> f64 {
    let apo_a = apoapsis_m(sma_a, ecc_a);
    let peri_b = periapsis_m(sma_b, ecc_b);
    let apo_b = apoapsis_m(sma_b, ecc_b);
    let peri_a = periapsis_m(sma_a, ecc_a);

    let (inner_apo, outer_peri) = if sma_a < sma_b {
        (apo_a, peri_b)
    } else {
        (apo_b, peri_a)
    };

    outer_peri - inner_apo
}

// ============================================================================
// Detection functions
// ============================================================================

/// Run all anomaly detectors on the current system's bodies.
/// Returns a map of body_id → Vec<Anomaly> for bodies with detected anomalies.
pub fn detect_anomalies(bodies: &HashMap<u32, Body>) -> HashMap<u32, Vec<Anomaly>> {
    let mut results: HashMap<u32, Vec<Anomaly>> = HashMap::new();

    // Orbital anomalies
    detect_close_orbits(bodies, &mut results);
    detect_flypasts(bodies, &mut results);

    // Body classification anomalies
    detect_trojans(bodies, &mut results);
    detect_satellite_stars(bodies, &mut results);
    detect_deeply_nested(bodies, &mut results);

    results
}

// ============================================================================
// Orbital anomaly detectors (from Builder A)
// ============================================================================

/// Detect bodies in dangerously close orbits to their parent.
/// A close orbit is defined as: apoapsis < 2 × (body_radius + parent_radius).
pub fn detect_close_orbits(bodies: &HashMap<u32, Body>, results: &mut HashMap<u32, Vec<Anomaly>>) {
    for body in bodies.values() {
        let parent_id = match body.parent_id {
            Some(pid) => pid,
            None => continue,
        };

        let sma = match body.semi_major_axis {
            Some(v) => v,
            None => continue,
        };

        let ecc = match body.eccentricity {
            Some(v) => v,
            None => continue,
        };

        let body_radius = match body.radius {
            Some(v) => v,
            None => continue,
        };

        let parent = match bodies.get(&parent_id) {
            Some(p) => p,
            None => continue,
        };

        let parent_radius = parent.radius.unwrap_or(0.0);

        let apo = apoapsis_m(sma, ecc);
        let combined = 2.0 * (body_radius + parent_radius);

        if apo < combined {
            let kind = if body.landable {
                AnomalyKind::CloseOrbitLandable
            } else {
                AnomalyKind::CloseOrbit
            };

            let desc = format!(
                "{} orbits dangerously close — apoapsis {:.0} km vs {:.0} km combined radii threshold",
                body.short_name,
                apo / 1000.0,
                combined / 1000.0,
            );

            results
                .entry(body.body_id)
                .or_default()
                .push(Anomaly {
                    body_id: body.body_id,
                    kind,
                    description: desc,
                });
        }
    }
}

/// Detect close and collision flypasts between sibling bodies.
/// Close flypast: orbit approach within 100km, synodic period < 40 days.
/// Collision flypast: orbits actually overlap (approach < 0).
pub fn detect_flypasts(bodies: &HashMap<u32, Body>, results: &mut HashMap<u32, Vec<Anomaly>>) {
    // Group bodies by parent_id to find siblings.
    let mut siblings_map: HashMap<u32, Vec<&Body>> = HashMap::new();
    for body in bodies.values() {
        if let Some(pid) = body.parent_id {
            siblings_map.entry(pid).or_default().push(body);
        }
    }

    let forty_days_s = 40.0 * 86400.0;

    for siblings in siblings_map.values() {
        for i in 0..siblings.len() {
            for j in (i + 1)..siblings.len() {
                let a = siblings[i];
                let b = siblings[j];

                let (sma_a, ecc_a, period_a) = match (
                    a.semi_major_axis,
                    a.eccentricity,
                    a.orbital_period,
                ) {
                    (Some(s), Some(e), Some(p)) => (s, e, p),
                    _ => continue,
                };

                let (sma_b, ecc_b, period_b) = match (
                    b.semi_major_axis,
                    b.eccentricity,
                    b.orbital_period,
                ) {
                    (Some(s), Some(e), Some(p)) => (s, e, p),
                    _ => continue,
                };

                let approach = closest_approach_m(sma_a, ecc_a, sma_b, ecc_b);

                if approach < 0.0 {
                    let desc = format!(
                        "{} and {} have overlapping orbits (approach {:.0} km)",
                        a.short_name,
                        b.short_name,
                        approach / 1000.0,
                    );

                    results
                        .entry(a.body_id)
                        .or_default()
                        .push(Anomaly {
                            body_id: a.body_id,
                            kind: AnomalyKind::CollisionFlypast,
                            description: desc.clone(),
                        });

                    results
                        .entry(b.body_id)
                        .or_default()
                        .push(Anomaly {
                            body_id: b.body_id,
                            kind: AnomalyKind::CollisionFlypast,
                            description: desc,
                        });
                } else if approach < 100_000.0 {
                    if let Some(syn) = synodic_period(period_a, period_b) {
                        if syn < forty_days_s {
                            let desc = format!(
                                "{} and {} approach within {:.1} km (synodic period {:.1} days)",
                                a.short_name,
                                b.short_name,
                                approach / 1000.0,
                                syn / 86400.0,
                            );

                            results
                                .entry(a.body_id)
                                .or_default()
                                .push(Anomaly {
                                    body_id: a.body_id,
                                    kind: AnomalyKind::CloseFlypast,
                                    description: desc.clone(),
                                });

                            results
                                .entry(b.body_id)
                                .or_default()
                                .push(Anomaly {
                                    body_id: b.body_id,
                                    kind: AnomalyKind::CloseFlypast,
                                    description: desc,
                                });
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Body classification anomaly detectors (from Builder B)
// ============================================================================

/// Detect trojan bodies sharing the same orbit.
/// Matching: similar SMA, eccentricity, inclination, period — but different argument of periapsis.
pub fn detect_trojans(bodies: &HashMap<u32, Body>, results: &mut HashMap<u32, Vec<Anomaly>>) {
    // Group bodies by parent_id
    let mut siblings: HashMap<u32, Vec<&Body>> = HashMap::new();
    for body in bodies.values() {
        if let Some(pid) = body.parent_id {
            siblings.entry(pid).or_default().push(body);
        }
    }

    for group in siblings.values() {
        for i in 0..group.len() {
            for j in (i + 1)..group.len() {
                let a = group[i];
                let b = group[j];

                let (sma_a, ecc_a, inc_a, per_a, peri_a) = match (
                    a.semi_major_axis,
                    a.eccentricity,
                    a.inclination,
                    a.orbital_period,
                    a.periapsis,
                ) {
                    (Some(s), Some(e), Some(i), Some(p), Some(w)) => (s, e, i, p, w),
                    _ => continue,
                };

                let (sma_b, ecc_b, inc_b, per_b, peri_b) = match (
                    b.semi_major_axis,
                    b.eccentricity,
                    b.inclination,
                    b.orbital_period,
                    b.periapsis,
                ) {
                    (Some(s), Some(e), Some(i), Some(p), Some(w)) => (s, e, i, p, w),
                    _ => continue,
                };

                // SMA ratio within 0.5%
                let sma_max = sma_a.max(sma_b);
                if sma_max <= 0.0 {
                    continue;
                }
                if (sma_a - sma_b).abs() / sma_max >= 0.005 {
                    continue;
                }

                // Eccentricity difference < 0.01
                if (ecc_a - ecc_b).abs() >= 0.01 {
                    continue;
                }

                // Inclination difference < 0.5°
                if (inc_a - inc_b).abs() >= 0.5 {
                    continue;
                }

                // Period ratio within 0.5%
                let per_max = per_a.max(per_b);
                if per_max <= 0.0 {
                    continue;
                }
                if (per_a - per_b).abs() / per_max >= 0.005 {
                    continue;
                }

                // Exclude 180° binaries
                let peri_diff = (peri_a - peri_b).abs() % 360.0;
                if (peri_diff - 180.0).abs() < 5.0 {
                    continue;
                }

                let desc = format!(
                    "{} and {} share the same orbit (trojan pair)",
                    a.short_name, b.short_name
                );

                results.entry(a.body_id).or_default().push(Anomaly {
                    body_id: a.body_id,
                    kind: AnomalyKind::Trojan,
                    description: desc.clone(),
                });
                results.entry(b.body_id).or_default().push(Anomaly {
                    body_id: b.body_id,
                    kind: AnomalyKind::Trojan,
                    description: desc,
                });
            }
        }
    }
}

/// Detect stars orbiting in satellite (moon-like) positions.
pub fn detect_satellite_stars(
    bodies: &HashMap<u32, Body>,
    results: &mut HashMap<u32, Vec<Anomaly>>,
) {
    for body in bodies.values() {
        if body.body_type != BodyType::Star {
            continue;
        }

        let mut is_satellite = false;

        // Check 1: parent is a Planet or Moon
        if let Some(pid) = body.parent_id {
            if let Some(parent) = bodies.get(&pid) {
                if parent.body_type == BodyType::Planet || parent.body_type == BodyType::Moon {
                    is_satellite = true;
                }
            }
        }

        // Check 2: moon-like naming pattern
        if !is_satellite {
            is_satellite = has_moon_like_name(&body.short_name);
        }

        if is_satellite {
            let desc = format!("{} is a star in satellite orbit", body.short_name);
            results.entry(body.body_id).or_default().push(Anomaly {
                body_id: body.body_id,
                kind: AnomalyKind::SatelliteStar,
                description: desc,
            });
        }
    }
}

/// Check if a name has a moon-like pattern: last segment after the last space
/// is a single lowercase letter, and there's at least one digit earlier in the name.
fn has_moon_like_name(name: &str) -> bool {
    let trimmed = name.trim();
    if let Some(space_pos) = trimmed.rfind(' ') {
        let suffix = &trimmed[space_pos + 1..];
        let prefix = &trimmed[..space_pos];

        if suffix.len() == 1 {
            let ch = suffix.as_bytes()[0];
            if ch.is_ascii_lowercase() && prefix.chars().any(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    false
}

/// Detect bodies nested 3+ levels deep in the planet/moon chain.
pub fn detect_deeply_nested(
    bodies: &HashMap<u32, Body>,
    results: &mut HashMap<u32, Vec<Anomaly>>,
) {
    for body in bodies.values() {
        if body.body_type != BodyType::Planet && body.body_type != BodyType::Moon {
            continue;
        }

        let mut depth: u32 = 0;
        let mut current_id = body.parent_id;
        let mut iterations = 0;

        while let Some(pid) = current_id {
            iterations += 1;
            if iterations > 10 {
                break;
            }

            if let Some(parent) = bodies.get(&pid) {
                if parent.body_type == BodyType::Planet || parent.body_type == BodyType::Moon {
                    depth += 1;
                }
                current_id = parent.parent_id;
            } else {
                break;
            }
        }

        if depth >= 3 {
            let depth_desc = match depth {
                3 => "moon of a moon of a moon".to_string(),
                n => format!("nested {} levels deep in planet chain", n),
            };
            let desc = format!("{} is a {}", body.short_name, depth_desc);
            results.entry(body.body_id).or_default().push(Anomaly {
                body_id: body.body_id,
                kind: AnomalyKind::DeeplyNested { depth },
                description: desc,
            });
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a body with minimal defaults.
    fn make_body(id: u32, name: &str) -> Body {
        let mut b = Body::new(id, name.into());
        b.short_name = name.into();
        b
    }

    // --- Orbital math helpers ---

    #[test]
    fn apoapsis_periapsis_basic() {
        let sma = 1_000_000.0;
        let ecc = 0.1;
        assert!((apoapsis_m(sma, ecc) - 1_100_000.0).abs() < 0.01);
        assert!((periapsis_m(sma, ecc) - 900_000.0).abs() < 0.01);
    }

    #[test]
    fn synodic_period_basic() {
        let syn = synodic_period(10.0, 12.0).unwrap();
        assert!((syn - 60.0).abs() < 0.1);
    }

    #[test]
    fn synodic_period_coorbital() {
        assert!(synodic_period(100.0, 100.0).is_none());
    }

    #[test]
    fn closest_approach_non_overlapping() {
        let dist = closest_approach_m(100_000.0, 0.0, 200_000.0, 0.0);
        assert!((dist - 100_000.0).abs() < 0.01);
    }

    #[test]
    fn closest_approach_overlapping() {
        let dist = closest_approach_m(100_000.0, 0.5, 120_000.0, 0.5);
        assert!(dist < 0.0);
    }

    #[test]
    fn empty_bodies_no_anomalies() {
        let bodies = HashMap::new();
        let results = detect_anomalies(&bodies);
        assert!(results.is_empty());
    }

    // --- Close orbit detection ---

    #[test]
    fn test_close_orbit_detected() {
        let mut bodies = HashMap::new();

        let mut parent = make_body(0, "Star");
        parent.radius = Some(100_000.0);
        bodies.insert(0, parent);

        // sma=100km, ecc=0.1 → apo=110km. threshold=2×(50+100)=300km. 110 < 300 → close!
        let mut child = make_body(1, "A 1");
        child.parent_id = Some(0);
        child.semi_major_axis = Some(100_000.0);
        child.eccentricity = Some(0.1);
        child.radius = Some(50_000.0);
        child.landable = false;
        bodies.insert(1, child);

        let mut results = HashMap::new();
        detect_close_orbits(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert_eq!(results[&1].len(), 1);
        assert_eq!(results[&1][0].kind, AnomalyKind::CloseOrbit);
    }

    #[test]
    fn test_close_orbit_not_triggered_normal() {
        let mut bodies = HashMap::new();

        let mut parent = make_body(0, "Star");
        parent.radius = Some(100_000.0);
        bodies.insert(0, parent);

        let mut child = make_body(1, "A 1");
        child.parent_id = Some(0);
        child.semi_major_axis = Some(10_000_000.0);
        child.eccentricity = Some(0.05);
        child.radius = Some(10_000.0);
        bodies.insert(1, child);

        let mut results = HashMap::new();
        detect_close_orbits(&bodies, &mut results);
        assert!(results.is_empty());
    }

    #[test]
    fn test_close_orbit_landable_variant() {
        let mut bodies = HashMap::new();

        let mut parent = make_body(0, "Star");
        parent.radius = Some(100_000.0);
        bodies.insert(0, parent);

        let mut child = make_body(1, "A 1");
        child.parent_id = Some(0);
        child.semi_major_axis = Some(100_000.0);
        child.eccentricity = Some(0.1);
        child.radius = Some(50_000.0);
        child.landable = true;
        bodies.insert(1, child);

        let mut results = HashMap::new();
        detect_close_orbits(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert_eq!(results[&1][0].kind, AnomalyKind::CloseOrbitLandable);
    }

    // --- Flypast detection ---

    #[test]
    fn test_close_flypast_detected() {
        let mut bodies = HashMap::new();

        let parent = make_body(0, "Star");
        bodies.insert(0, parent);

        let mut a = make_body(1, "A 1");
        a.parent_id = Some(0);
        a.semi_major_axis = Some(1_000_000.0);
        a.eccentricity = Some(0.0);
        a.orbital_period = Some(1000.0);
        bodies.insert(1, a);

        // approach = 1050 - 1000 = 50km < 100km, synodic ≈ 0.127 days < 40 days
        let mut b = make_body(2, "A 2");
        b.parent_id = Some(0);
        b.semi_major_axis = Some(1_050_000.0);
        b.eccentricity = Some(0.0);
        b.orbital_period = Some(1100.0);
        bodies.insert(2, b);

        let mut results = HashMap::new();
        detect_flypasts(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert!(results.contains_key(&2));
        assert_eq!(results[&1][0].kind, AnomalyKind::CloseFlypast);
        assert_eq!(results[&2][0].kind, AnomalyKind::CloseFlypast);
    }

    #[test]
    fn test_collision_flypast_detected() {
        let mut bodies = HashMap::new();

        let parent = make_body(0, "Star");
        bodies.insert(0, parent);

        let mut a = make_body(1, "A 1");
        a.parent_id = Some(0);
        a.semi_major_axis = Some(1_000_000.0);
        a.eccentricity = Some(0.5);
        a.orbital_period = Some(1000.0);
        bodies.insert(1, a);

        // inner_apo=1500km, outer_peri=600km → approach=-900km (overlap!)
        let mut b = make_body(2, "A 2");
        b.parent_id = Some(0);
        b.semi_major_axis = Some(1_200_000.0);
        b.eccentricity = Some(0.5);
        b.orbital_period = Some(1200.0);
        bodies.insert(2, b);

        let mut results = HashMap::new();
        detect_flypasts(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert!(results.contains_key(&2));
        assert_eq!(results[&1][0].kind, AnomalyKind::CollisionFlypast);
    }

    #[test]
    fn test_no_flypast_distant_siblings() {
        let mut bodies = HashMap::new();

        let parent = make_body(0, "Star");
        bodies.insert(0, parent);

        let mut a = make_body(1, "A 1");
        a.parent_id = Some(0);
        a.semi_major_axis = Some(1_000_000.0);
        a.eccentricity = Some(0.0);
        a.orbital_period = Some(1000.0);
        bodies.insert(1, a);

        let mut b = make_body(2, "A 2");
        b.parent_id = Some(0);
        b.semi_major_axis = Some(5_000_000.0);
        b.eccentricity = Some(0.0);
        b.orbital_period = Some(5000.0);
        bodies.insert(2, b);

        let mut results = HashMap::new();
        detect_flypasts(&bodies, &mut results);
        assert!(results.is_empty());
    }

    // --- Trojan detection ---

    #[test]
    fn test_trojan_detected() {
        let mut bodies = HashMap::new();

        let mut star = make_body(0, "A");
        star.body_type = BodyType::Star;
        bodies.insert(0, star);

        let mut p1 = make_body(1, "A 1");
        p1.body_type = BodyType::Planet;
        p1.parent_id = Some(0);
        p1.semi_major_axis = Some(1_000_000_000.0);
        p1.eccentricity = Some(0.05);
        p1.inclination = Some(1.0);
        p1.orbital_period = Some(86400.0);
        p1.periapsis = Some(30.0);
        bodies.insert(1, p1);

        let mut p2 = make_body(2, "A 2");
        p2.body_type = BodyType::Planet;
        p2.parent_id = Some(0);
        p2.semi_major_axis = Some(1_000_000_000.0);
        p2.eccentricity = Some(0.05);
        p2.inclination = Some(1.0);
        p2.orbital_period = Some(86400.0);
        p2.periapsis = Some(90.0); // 60° apart — not a binary
        bodies.insert(2, p2);

        let mut results = HashMap::new();
        detect_trojans(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert!(results.contains_key(&2));
        assert_eq!(results[&1][0].kind, AnomalyKind::Trojan);
    }

    #[test]
    fn test_binary_not_trojan() {
        let mut bodies = HashMap::new();

        let mut star = make_body(0, "A");
        star.body_type = BodyType::Star;
        bodies.insert(0, star);

        let mut p1 = make_body(1, "A 1");
        p1.body_type = BodyType::Planet;
        p1.parent_id = Some(0);
        p1.semi_major_axis = Some(500_000_000.0);
        p1.eccentricity = Some(0.02);
        p1.inclination = Some(2.0);
        p1.orbital_period = Some(50000.0);
        p1.periapsis = Some(10.0);
        bodies.insert(1, p1);

        let mut p2 = make_body(2, "A 2");
        p2.body_type = BodyType::Planet;
        p2.parent_id = Some(0);
        p2.semi_major_axis = Some(500_000_000.0);
        p2.eccentricity = Some(0.02);
        p2.inclination = Some(2.0);
        p2.orbital_period = Some(50000.0);
        p2.periapsis = Some(190.0); // 180° offset — binary
        bodies.insert(2, p2);

        let mut results = HashMap::new();
        detect_trojans(&bodies, &mut results);
        assert!(results.is_empty());
    }

    #[test]
    fn test_no_trojan_different_orbits() {
        let mut bodies = HashMap::new();

        let mut star = make_body(0, "A");
        star.body_type = BodyType::Star;
        bodies.insert(0, star);

        let mut p1 = make_body(1, "A 1");
        p1.body_type = BodyType::Planet;
        p1.parent_id = Some(0);
        p1.semi_major_axis = Some(1_000_000_000.0);
        p1.eccentricity = Some(0.05);
        p1.inclination = Some(1.0);
        p1.orbital_period = Some(86400.0);
        p1.periapsis = Some(30.0);
        bodies.insert(1, p1);

        let mut p2 = make_body(2, "A 2");
        p2.body_type = BodyType::Planet;
        p2.parent_id = Some(0);
        p2.semi_major_axis = Some(2_000_000_000.0);
        p2.eccentricity = Some(0.05);
        p2.inclination = Some(1.0);
        p2.orbital_period = Some(172800.0);
        p2.periapsis = Some(30.0);
        bodies.insert(2, p2);

        let mut results = HashMap::new();
        detect_trojans(&bodies, &mut results);
        assert!(results.is_empty());
    }

    // --- Satellite star detection ---

    #[test]
    fn test_satellite_star_detected() {
        let mut bodies = HashMap::new();

        let mut planet = make_body(1, "A 1");
        planet.body_type = BodyType::Planet;
        bodies.insert(1, planet);

        let mut sat_star = make_body(2, "A 1 a");
        sat_star.body_type = BodyType::Star;
        sat_star.parent_id = Some(1);
        bodies.insert(2, sat_star);

        let mut results = HashMap::new();
        detect_satellite_stars(&bodies, &mut results);

        assert!(results.contains_key(&2));
        assert_eq!(results[&2][0].kind, AnomalyKind::SatelliteStar);
    }

    #[test]
    fn test_normal_star_not_satellite() {
        let mut bodies = HashMap::new();

        let mut star = make_body(0, "A");
        star.body_type = BodyType::Star;
        bodies.insert(0, star);

        let mut results = HashMap::new();
        detect_satellite_stars(&bodies, &mut results);
        assert!(results.is_empty());
    }

    // --- Deeply nested detection ---

    #[test]
    fn test_deeply_nested_moon3() {
        let mut bodies = HashMap::new();

        let mut star = make_body(0, "A");
        star.body_type = BodyType::Star;
        bodies.insert(0, star);

        let mut p = make_body(1, "A 1");
        p.body_type = BodyType::Planet;
        p.parent_id = Some(0);
        bodies.insert(1, p);

        let mut m1 = make_body(2, "A 1 a");
        m1.body_type = BodyType::Moon;
        m1.parent_id = Some(1);
        bodies.insert(2, m1);

        let mut m2 = make_body(3, "A 1 a i");
        m2.body_type = BodyType::Moon;
        m2.parent_id = Some(2);
        bodies.insert(3, m2);

        let mut m3 = make_body(4, "A 1 a i x");
        m3.body_type = BodyType::Moon;
        m3.parent_id = Some(3);
        bodies.insert(4, m3);

        let mut results = HashMap::new();
        detect_deeply_nested(&bodies, &mut results);

        assert!(results.contains_key(&4));
        assert_eq!(results[&4][0].kind, AnomalyKind::DeeplyNested { depth: 3 });
    }

    #[test]
    fn test_shallow_moon_not_nested() {
        let mut bodies = HashMap::new();

        let mut star = make_body(0, "A");
        star.body_type = BodyType::Star;
        bodies.insert(0, star);

        let mut p = make_body(1, "A 1");
        p.body_type = BodyType::Planet;
        p.parent_id = Some(0);
        bodies.insert(1, p);

        let mut m = make_body(2, "A 1 a");
        m.body_type = BodyType::Moon;
        m.parent_id = Some(1);
        bodies.insert(2, m);

        let mut results = HashMap::new();
        detect_deeply_nested(&bodies, &mut results);
        assert!(results.is_empty());
    }
}
