//! Extreme body anomaly detectors.
//!
//! Detects bodies with unusual physical or orbital characteristics:
//! - Fast rotators (rotation period < 1 hour)
//! - Retrograde orbits (inclination > 90°)
//! - High eccentricity (e > 0.9)
//! - Extreme axial tilt (> 90°)

use std::collections::HashMap;

use super::anomaly::{Anomaly, AnomalyKind};
use super::body::Body;

// ============================================================================
// Detection functions
// ============================================================================

/// Detect bodies with extremely short rotation periods (< 1 hour).
pub fn detect_fast_rotators(bodies: &HashMap<u32, Body>, results: &mut HashMap<u32, Vec<Anomaly>>) {
    for body in bodies.values() {
        let rot = match body.rotational_period {
            Some(v) => v,
            None => continue,
        };

        if rot.abs() < 3600.0 {
            let minutes = rot.abs() / 60.0;
            let desc = format!(
                "{} rotates in {:.1} minutes",
                body.short_name, minutes
            );
            results
                .entry(body.body_id)
                .or_default()
                .push(Anomaly {
                    body_id: body.body_id,
                    kind: AnomalyKind::FastRotator,
                    description: desc,
                });
        }
    }
}

/// Detect bodies with retrograde orbits (inclination > 90°).
/// Inclination > 160° is extreme retrograde.
pub fn detect_retrograde_orbits(
    bodies: &HashMap<u32, Body>,
    results: &mut HashMap<u32, Vec<Anomaly>>,
) {
    for body in bodies.values() {
        let inc = match body.inclination {
            Some(v) => v,
            None => continue,
        };

        if inc.abs() > 90.0 {
            let qualifier = if inc.abs() > 160.0 {
                "extreme retrograde"
            } else {
                "retrograde"
            };
            let desc = format!(
                "{} has {} orbit ({:.1}°)",
                body.short_name, qualifier, inc.abs()
            );
            results
                .entry(body.body_id)
                .or_default()
                .push(Anomaly {
                    body_id: body.body_id,
                    kind: AnomalyKind::RetrogradeOrbit,
                    description: desc,
                });
        }
    }
}

/// Detect bodies with very elongated orbits (eccentricity > 0.9).
pub fn detect_high_eccentricity(
    bodies: &HashMap<u32, Body>,
    results: &mut HashMap<u32, Vec<Anomaly>>,
) {
    for body in bodies.values() {
        let ecc = match body.eccentricity {
            Some(v) => v,
            None => continue,
        };

        if ecc > 0.9 {
            let desc = format!(
                "{} has extremely eccentric orbit (e={:.4})",
                body.short_name, ecc
            );
            results
                .entry(body.body_id)
                .or_default()
                .push(Anomaly {
                    body_id: body.body_id,
                    kind: AnomalyKind::HighEccentricity,
                    description: desc,
                });
        }
    }
}

/// Detect bodies with extreme axial tilt (> 90° in degrees).
/// The `axial_tilt` field is in radians.
pub fn detect_extreme_tilt(
    bodies: &HashMap<u32, Body>,
    results: &mut HashMap<u32, Vec<Anomaly>>,
) {
    for body in bodies.values() {
        let axial_tilt = match body.axial_tilt {
            Some(v) => v,
            None => continue,
        };

        let tilt_deg = axial_tilt.to_degrees().abs();

        if tilt_deg > 90.0 {
            let desc = format!(
                "{} has extreme axial tilt ({:.1}°)",
                body.short_name, tilt_deg
            );
            results
                .entry(body.body_id)
                .or_default()
                .push(Anomaly {
                    body_id: body.body_id,
                    kind: AnomalyKind::ExtremeTilt,
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

    // --- Fast rotator detection ---

    #[test]
    fn test_fast_rotator_detected() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        body.rotational_period = Some(1800.0); // 30 minutes
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_fast_rotators(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert_eq!(results[&1].len(), 1);
        assert_eq!(results[&1][0].kind, AnomalyKind::FastRotator);
        assert!(results[&1][0].description.contains("30.0 minutes"));
    }

    #[test]
    fn test_normal_rotation_not_flagged() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        body.rotational_period = Some(86400.0); // 24 hours
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_fast_rotators(&bodies, &mut results);

        assert!(results.is_empty());
    }

    // --- Retrograde orbit detection ---

    #[test]
    fn test_retrograde_orbit_detected() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        body.inclination = Some(150.0);
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_retrograde_orbits(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert_eq!(results[&1].len(), 1);
        assert_eq!(results[&1][0].kind, AnomalyKind::RetrogradeOrbit);
        assert!(results[&1][0].description.contains("retrograde"));
    }

    #[test]
    fn test_prograde_orbit_not_flagged() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        body.inclination = Some(5.0);
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_retrograde_orbits(&bodies, &mut results);

        assert!(results.is_empty());
    }

    // --- High eccentricity detection ---

    #[test]
    fn test_high_eccentricity_detected() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        body.eccentricity = Some(0.95);
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_high_eccentricity(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert_eq!(results[&1].len(), 1);
        assert_eq!(results[&1][0].kind, AnomalyKind::HighEccentricity);
        assert!(results[&1][0].description.contains("e=0.9500"));
    }

    #[test]
    fn test_normal_eccentricity_not_flagged() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        body.eccentricity = Some(0.1);
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_high_eccentricity(&bodies, &mut results);

        assert!(results.is_empty());
    }

    // --- Extreme tilt detection ---

    #[test]
    fn test_extreme_tilt_detected() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        // 120° in radians
        body.axial_tilt = Some(120.0_f64.to_radians());
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_extreme_tilt(&bodies, &mut results);

        assert!(results.contains_key(&1));
        assert_eq!(results[&1].len(), 1);
        assert_eq!(results[&1][0].kind, AnomalyKind::ExtremeTilt);
        assert!(results[&1][0].description.contains("120.0°"));
    }

    #[test]
    fn test_normal_tilt_not_flagged() {
        let mut bodies = HashMap::new();
        let mut body = make_body(1, "A 1");
        // 23° in radians
        body.axial_tilt = Some(23.0_f64.to_radians());
        bodies.insert(1, body);

        let mut results = HashMap::new();
        detect_extreme_tilt(&bodies, &mut results);

        assert!(results.is_empty());
    }
}
