//! Jumponium / Green System Detection
//!
//! Detects whether a system contains all the raw materials needed for
//! FSD boost synthesis across its landable bodies.
//!
//! Material requirements per grade:
//! - **Basic**    (25%): Carbon, Vanadium, Germanium
//! - **Standard** (50%): + Cadmium, Niobium
//! - **Premium** (100%): + Arsenic, Yttrium, Polonium

use std::collections::HashMap;

use super::body::Body;

// ---------------------------------------------------------------------------
// Material constants
// ---------------------------------------------------------------------------

const BASIC_MATERIALS: &[&str] = &["carbon", "vanadium", "germanium"];

const STANDARD_MATERIALS: &[&str] = &[
    "carbon", "vanadium", "germanium",
    "cadmium", "niobium",
];

const PREMIUM_MATERIALS: &[&str] = &[
    "carbon", "vanadium", "germanium",
    "cadmium", "niobium",
    "arsenic", "yttrium", "polonium",
];

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// FSD boost injection grade achievable from system materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JumponiumGrade {
    Basic,
    Standard,
    Premium,
}

impl JumponiumGrade {
    /// Human-readable label with boost percentage.
    pub fn label(&self) -> &'static str {
        match self {
            JumponiumGrade::Basic    => "Basic FSD Boost (25%)",
            JumponiumGrade::Standard => "Standard FSD Boost (50%)",
            JumponiumGrade::Premium  => "Premium FSD Boost (100%)",
        }
    }

    /// Short icon for TUI display.
    pub fn icon(&self) -> &'static str {
        match self {
            JumponiumGrade::Basic    => "🟢",
            JumponiumGrade::Standard => "🟡",
            JumponiumGrade::Premium  => "⭐",
        }
    }
}

/// Result of jumponium analysis for a system.
#[derive(Debug, Clone, PartialEq)]
pub struct JumponiumResult {
    /// Highest achievable FSD boost grade.
    pub grade: JumponiumGrade,
    /// Which materials are available and on which body_ids they can be found.
    /// Sorted alphabetically by material name for deterministic output.
    pub material_sources: Vec<(String, Vec<u32>)>,
}

// ---------------------------------------------------------------------------
// Detection functions
// ---------------------------------------------------------------------------

/// Returns which jumponium-relevant materials a single body has, with percentages.
///
/// Only checks `surface_materials` — non-landable bodies are included since
/// the journal may list materials even if the body isn't strictly landable
/// (caller can filter on `body.landable` if desired).
pub fn body_jumponium_materials(body: &Body) -> Vec<(&'static str, f64)> {
    let mut found = Vec::new();
    for (mat_name, pct) in &body.surface_materials {
        let lower = mat_name.to_lowercase();
        for &jumpo_mat in PREMIUM_MATERIALS {
            if lower == jumpo_mat {
                found.push((jumpo_mat, *pct));
                break;
            }
        }
    }
    found
}

/// Analyse all bodies in a system and determine the highest achievable
/// jumponium grade.
///
/// Returns `None` if not even Basic grade is achievable (missing at least one
/// of Carbon, Vanadium, or Germanium across all landable bodies).
pub fn detect_jumponium(bodies: &HashMap<u32, Body>) -> Option<JumponiumResult> {
    // Collect: material_name → Vec<body_id>
    let mut material_map: HashMap<&'static str, Vec<u32>> = HashMap::new();

    for body in bodies.values() {
        if !body.landable {
            continue;
        }
        for (mat, _pct) in body_jumponium_materials(body) {
            material_map
                .entry(mat)
                .or_default()
                .push(body.body_id);
        }
    }

    // Deduplicate and sort body_ids within each material for determinism.
    for ids in material_map.values_mut() {
        ids.sort_unstable();
        ids.dedup();
    }

    // Determine highest grade.
    let has_all = |mats: &[&str]| -> bool {
        mats.iter().all(|m| material_map.contains_key(m))
    };

    let grade = if has_all(PREMIUM_MATERIALS) {
        JumponiumGrade::Premium
    } else if has_all(STANDARD_MATERIALS) {
        JumponiumGrade::Standard
    } else if has_all(BASIC_MATERIALS) {
        JumponiumGrade::Basic
    } else {
        return None;
    };

    // Build sorted material_sources — only include the materials relevant to
    // the achieved grade.
    let relevant = match grade {
        JumponiumGrade::Basic    => BASIC_MATERIALS,
        JumponiumGrade::Standard => STANDARD_MATERIALS,
        JumponiumGrade::Premium  => PREMIUM_MATERIALS,
    };

    let mut material_sources: Vec<(String, Vec<u32>)> = relevant
        .iter()
        .filter_map(|&mat| {
            material_map
                .get(mat)
                .map(|ids| (mat.to_string(), ids.clone()))
        })
        .collect();

    material_sources.sort_by(|a, b| a.0.cmp(&b.0));

    Some(JumponiumResult {
        grade,
        material_sources,
    })
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal landable body with the given materials.
    fn make_landable(id: u32, name: &str, materials: &[(&str, f64)]) -> Body {
        let mut b = Body::new(id, name.into());
        b.short_name = name.into();
        b.landable = true;
        b.surface_materials = materials
            .iter()
            .map(|(m, p)| (m.to_string(), *p))
            .collect();
        b
    }

    /// Helper: create a non-landable body.
    fn make_non_landable(id: u32, name: &str, materials: &[(&str, f64)]) -> Body {
        let mut b = make_landable(id, name, materials);
        b.landable = false;
        b
    }

    // --- Grade detection tests ---

    #[test]
    fn test_premium_jumponium() {
        let mut bodies = HashMap::new();

        let b1 = make_landable(1, "A 1", &[
            ("carbon", 18.5), ("vanadium", 4.2), ("germanium", 3.8),
            ("cadmium", 1.5), ("niobium", 0.9),
        ]);
        let b2 = make_landable(2, "A 2", &[
            ("arsenic", 1.2), ("yttrium", 0.8), ("polonium", 0.4),
        ]);

        bodies.insert(1, b1);
        bodies.insert(2, b2);

        let result = detect_jumponium(&bodies).expect("should detect premium");
        assert_eq!(result.grade, JumponiumGrade::Premium);
        assert_eq!(result.material_sources.len(), 8);
    }

    #[test]
    fn test_standard_jumponium() {
        let mut bodies = HashMap::new();

        let b1 = make_landable(1, "A 1", &[
            ("carbon", 18.5), ("vanadium", 4.2), ("germanium", 3.8),
            ("cadmium", 1.5), ("niobium", 0.9),
        ]);

        bodies.insert(1, b1);

        let result = detect_jumponium(&bodies).expect("should detect standard");
        assert_eq!(result.grade, JumponiumGrade::Standard);
        assert_eq!(result.material_sources.len(), 5);
    }

    #[test]
    fn test_basic_jumponium() {
        let mut bodies = HashMap::new();

        let b1 = make_landable(1, "A 1", &[
            ("carbon", 18.5), ("vanadium", 4.2), ("germanium", 3.8),
        ]);

        bodies.insert(1, b1);

        let result = detect_jumponium(&bodies).expect("should detect basic");
        assert_eq!(result.grade, JumponiumGrade::Basic);
        assert_eq!(result.material_sources.len(), 3);
    }

    #[test]
    fn test_no_jumponium() {
        let mut bodies = HashMap::new();

        // Has vanadium and germanium but NOT carbon → can't even do Basic.
        let b1 = make_landable(1, "A 1", &[
            ("vanadium", 4.2), ("germanium", 3.8), ("iron", 20.0),
        ]);

        bodies.insert(1, b1);

        assert!(detect_jumponium(&bodies).is_none());
    }

    #[test]
    fn test_materials_across_multiple_bodies() {
        let mut bodies = HashMap::new();

        let b1 = make_landable(1, "A 1", &[("carbon", 15.0)]);
        let b2 = make_landable(2, "A 2", &[("vanadium", 3.0), ("carbon", 10.0)]);
        let b3 = make_landable(3, "A 3", &[("germanium", 5.0)]);

        bodies.insert(1, b1);
        bodies.insert(2, b2);
        bodies.insert(3, b3);

        let result = detect_jumponium(&bodies).expect("should detect basic across 3 bodies");
        assert_eq!(result.grade, JumponiumGrade::Basic);

        // Carbon should list body 1 AND 2.
        let carbon_entry = result
            .material_sources
            .iter()
            .find(|(m, _)| m == "carbon")
            .expect("carbon should be present");
        assert_eq!(carbon_entry.1, vec![1, 2]);

        // Germanium only on body 3.
        let germanium_entry = result
            .material_sources
            .iter()
            .find(|(m, _)| m == "germanium")
            .expect("germanium should be present");
        assert_eq!(germanium_entry.1, vec![3]);
    }

    #[test]
    fn test_body_jumponium_materials() {
        let body = make_landable(1, "A 1", &[
            ("carbon", 18.5),
            ("iron", 20.0),
            ("vanadium", 4.2),
            ("nickel", 15.0),
            ("polonium", 0.4),
        ]);

        let mats = body_jumponium_materials(&body);

        // Should pick out carbon, vanadium, polonium — skip iron, nickel.
        assert_eq!(mats.len(), 3);

        let names: Vec<&str> = mats.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"carbon"));
        assert!(names.contains(&"vanadium"));
        assert!(names.contains(&"polonium"));
        assert!(!names.contains(&"iron"));
    }

    #[test]
    fn test_non_landable_bodies_ignored() {
        let mut bodies = HashMap::new();

        // This body has all materials but is NOT landable.
        let b1 = make_non_landable(1, "Gas Giant", &[
            ("carbon", 18.5), ("vanadium", 4.2), ("germanium", 3.8),
        ]);

        bodies.insert(1, b1);

        assert!(detect_jumponium(&bodies).is_none());
    }

    #[test]
    fn test_empty_system() {
        let bodies = HashMap::new();
        assert!(detect_jumponium(&bodies).is_none());
    }

    #[test]
    fn test_grade_ordering() {
        assert!(JumponiumGrade::Basic < JumponiumGrade::Standard);
        assert!(JumponiumGrade::Standard < JumponiumGrade::Premium);
    }
}
