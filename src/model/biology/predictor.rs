//! Exobiology species predictor engine matching body conditions to static Canonn boundaries.

use crate::model::Body;
use crate::model::biology::dataset::{SpeciesVariant, DATASET};
use ed_journals::galaxy::StarClass;

/// Normalizes atmosphere strings down to a basic lowercase alphanumeric string.
/// e.g. `"Thin Carbon dioxide"` -> `"carbondioxide"`, `"CarbonDioxide"` -> `"carbondioxide"`.
pub fn normalize_atmosphere(atmo: &str) -> String {
    let mut normalized = String::new();
    for c in atmo.chars() {
        if c.is_alphanumeric() {
            normalized.push(c.to_ascii_lowercase());
        }
    }
    // Strip common modifiers to match enums with research text
    normalized = normalized
        .replace("thin", "")
        .replace("thick", "")
        .replace("atmosphere", "")
        .replace("sulphur", "sulfur");
        
    if normalized.is_empty() || normalized == "none" || normalized == "no" {
        "noatmosphere".to_string()
    } else {
        normalized
    }
}

/// Normalizes planet class strings down to a basic lowercase alphanumeric string.
/// e.g. `"High metal content world"` -> `"highmetalcontent"`, `"Rocky body"` -> `"rocky"`.
pub fn normalize_planet_class(pc: &str) -> String {
    let mut normalized = String::new();
    for c in pc.chars() {
        if c.is_alphanumeric() {
            normalized.push(c.to_ascii_lowercase());
        }
    }
    normalized
        .replace("body", "")
        .replace("world", "")
}

/// Robustly checks if the system's primary star class matches a Canonn boundary star description.
pub fn match_star_class(canonn_star: &str, system_star: &StarClass) -> bool {
    let norm_canonn = canonn_star.to_lowercase();

    match system_star {
        // Main sequence — match "X (" pattern to avoid false positives
        StarClass::O if norm_canonn.contains("o (") || norm_canonn == "o-type" => true,
        StarClass::B if norm_canonn.starts_with("b (") && !norm_canonn.contains("super giant") => true,
        StarClass::A if norm_canonn.starts_with("a (") && !norm_canonn.contains("super giant") => true,
        StarClass::F if norm_canonn.starts_with("f (") && !norm_canonn.contains("super giant") => true,
        StarClass::G if norm_canonn.starts_with("g (") && !norm_canonn.contains("super giant") => true,
        StarClass::K if norm_canonn.starts_with("k (") && !norm_canonn.contains("giant") => true,
        StarClass::M if norm_canonn.starts_with("m (") && !norm_canonn.contains("giant") => true,

        // Neutron / Black Hole
        StarClass::N if norm_canonn.contains("neutron") => true,
        StarClass::H | StarClass::SupermassiveBlackHole if norm_canonn.contains("black hole") => true,

        // Brown dwarfs — differentiate L, T, Y individually
        StarClass::L if norm_canonn.starts_with("l (") => true,
        StarClass::T if norm_canonn.starts_with("t (") && !norm_canonn.contains("t tauri") => true,
        StarClass::Y if norm_canonn.starts_with("y (") => true,

        // T Tauri
        StarClass::TTS if norm_canonn.contains("t tauri") => true,

        // Herbig Ae/Be
        StarClass::Ae | StarClass::Be | StarClass::AeBe if norm_canonn.contains("herbig") => true,

        // White Dwarfs — match specific subtypes from dataset strings like "White Dwarf (DA) Star"
        StarClass::D | StarClass::DA | StarClass::DAB | StarClass::DAO |
        StarClass::DAZ | StarClass::DAV | StarClass::DB | StarClass::DBZ |
        StarClass::DBV | StarClass::DO | StarClass::DOV | StarClass::DQ |
        StarClass::DC | StarClass::DCV | StarClass::DX => {
            if !norm_canonn.contains("white dwarf") {
                return false;
            }
            // Extract subtype from dataset string: "White Dwarf (DA) Star" -> "DA"
            if let Some(start) = norm_canonn.find('(') {
                if let Some(end) = norm_canonn.find(')') {
                    let dataset_subtype = &canonn_star[start + 1..end]; // preserve original case
                    let system_subtype = format!("{:?}", system_star);
                    return dataset_subtype == system_subtype;
                }
            }
            // Bare "White Dwarf" with no subtype — match any
            true
        }

        // Supergiants — explicit matches
        StarClass::ABlueWhiteSuperGiant if norm_canonn.contains("a (") && norm_canonn.contains("super giant") => true,
        StarClass::BBlueWhiteSuperGiant if norm_canonn.contains("b (") && norm_canonn.contains("super giant") => true,
        StarClass::FWhiteSuperGiant if norm_canonn.contains("f (") && norm_canonn.contains("super giant") => true,
        StarClass::GWhiteSuperGiant if norm_canonn.contains("g (") && norm_canonn.contains("super giant") => true,
        StarClass::MRedSuperGiant if norm_canonn.contains("m (") && norm_canonn.contains("super giant") => true,

        // Giants
        StarClass::MRedGiant if norm_canonn.contains("m (") && norm_canonn.contains("giant") && !norm_canonn.contains("super") => true,
        StarClass::KOrangeGiant if norm_canonn.contains("k (") && norm_canonn.contains("giant") => true,

        // Wolf-Rayet variants
        StarClass::W | StarClass::WN | StarClass::WNC | StarClass::WC | StarClass::WO
            if norm_canonn.contains("wolf-rayet") => true,

        // Carbon star variants
        StarClass::CS | StarClass::C | StarClass::CN | StarClass::CJ |
        StarClass::CH | StarClass::CHd if norm_canonn.contains("carbon") || norm_canonn.contains("cn ") || norm_canonn.contains("cj ") => true,

        // S-type / MS-type
        StarClass::S if norm_canonn.contains("s-type") => true,
        StarClass::MS if norm_canonn.contains("ms-type") => true,

        _ => false,
    }
}

/// Checks if a single species variant is physically capable of spawning on the given body.
pub fn match_variant(variant: &SpeciesVariant, body: &Body, primary_star: Option<&StarClass>) -> bool {
    // 1. Must be landable
    if !body.landable {
        return false;
    }

    // 2. Planet class matching
    if let Some(ref pc) = body.planet_class {
        let norm_pc = normalize_planet_class(pc);
        let mut matches_body = variant.bodies.is_empty();
        for b in variant.bodies {
            if normalize_planet_class(b) == norm_pc {
                matches_body = true;
                break;
            }
        }
        if !matches_body {
            return false;
        }
    }

    // 3. Atmosphere matching
    let norm_atmo = body.atmosphere.as_deref().map(normalize_atmosphere).unwrap_or_else(|| "noatmosphere".to_string());
    let mut matches_atmo = variant.atmosphere_types.is_empty();
    for a in variant.atmosphere_types {
        let norm_rule = normalize_atmosphere(a);
        if norm_rule == norm_atmo || (norm_rule == "noatmosphere" && norm_atmo == "none") {
            matches_atmo = true;
            break;
        }
    }
    if !matches_atmo {
        return false;
    }

    // 4. Gravity matching (in Gs)
    if let Some(g) = body.gravity {
        if g < variant.min_g || g > variant.max_g {
            return false;
        }
    }

    // 5. Temperature matching (in Kelvin)
    if let Some(t) = body.temperature {
        if t < variant.min_t || t > variant.max_t {
            return false;
        }
    }

    // 6. Primary Star class matching
    if let Some(star) = primary_star {
        let mut matches_star = variant.primary_stars.is_empty();
        for s in variant.primary_stars {
            if match_star_class(s, star) {
                matches_star = true;
                break;
            }
        }
        if !matches_star {
            return false;
        }
    }

    // 7. Volcanism filtering
    //    Empty volcanism array = no constraint.
    //    ["No volcanism"] = body must have no volcanism.
    //    Other values = body must have volcanism matching at least one entry (substring).
    if !variant.volcanism.is_empty() {
        let has_no_volcanism_rule = variant.volcanism.iter().any(|v| {
            let lv = v.to_lowercase();
            lv == "no volcanism" || lv == "none"
        });
        let has_volcanism_rules: Vec<&str> = variant.volcanism.iter()
            .filter(|v| {
                let lv = v.to_lowercase();
                lv != "no volcanism" && lv != "none"
            })
            .copied()
            .collect();

        match &body.volcanism {
            None => {
                // Body has no volcanism — only valid if "No volcanism" is in the list
                if !has_no_volcanism_rule {
                    return false;
                }
            }
            Some(body_volc) => {
                // Body HAS volcanism — check if it matches one of the non-"No volcanism" entries
                if has_volcanism_rules.is_empty() {
                    // Dataset only lists "No volcanism" but body has volcanism → fail
                    return false;
                }
                let body_volc_lower = body_volc.to_lowercase();
                let matches_any = has_volcanism_rules.iter().any(|rule| {
                    let rule_lower = rule.to_lowercase();
                    // Substring match: "minor metallic magma" matches body "minor metallic magma volcanism"
                    body_volc_lower.contains(&rule_lower) || rule_lower.contains(&body_volc_lower)
                });
                if !matches_any && !has_no_volcanism_rule {
                    return false;
                }
            }
        }
    }

    // 8. Pressure filtering (in atm)
    if variant.min_p > 0.0 || variant.max_p < f64::MAX {
        if let Some(p) = body.pressure_atm {
            if p < variant.min_p || p > variant.max_p {
                return false;
            }
        }
    }

    true
}

/// Evaluates body telemetry against the static dataset and returns all matching exobiology species.
pub fn predict_species(body: &Body, primary_star: Option<&StarClass>) -> Vec<SpeciesVariant> {
    // Only predict if body actually has biological signals reported
    if body.bio_signals == 0 || !body.landable {
        return Vec::new();
    }

    let mut matches: Vec<SpeciesVariant> = DATASET
        .iter()
        .filter(|v| match_variant(v, body, primary_star))
        .cloned()
        .collect();

    // If we have definitive genus information from SAA scan, filter/fallback based on it
    if !body.bio_genuses.is_empty() {
        matches.retain(|v| {
            body.bio_genuses
                .iter()
                .any(|g| g.eq_ignore_ascii_case(v.genus))
        });

        // If strict filtering returned no matches for the known genus, fall back to relaxed matching
        // (atmosphere and planet class matching, ignoring gravity, temperature, and primary star)
        if matches.is_empty() {
            let norm_pc = body.planet_class.as_deref().map(normalize_planet_class).unwrap_or_default();
            let norm_atmo = body.atmosphere.as_deref().map(normalize_atmosphere).unwrap_or_else(|| "noatmosphere".to_string());

            matches = DATASET
                .iter()
                .filter(|v| {
                    // 1. Genus must match one of the reported genuses
                    let genus_matches = body.bio_genuses.iter().any(|g| g.eq_ignore_ascii_case(v.genus));
                    if !genus_matches {
                        return false;
                    }

                    // 2. Planet class matches (if defined in dataset variant)
                    let planet_class_matches = v.bodies.is_empty() || v.bodies.iter().any(|b| normalize_planet_class(b) == norm_pc);
                    if !planet_class_matches {
                        return false;
                    }

                    // 3. Atmosphere matches (if defined in dataset variant)
                    let atmo_matches = v.atmosphere_types.is_empty() || v.atmosphere_types.iter().any(|a| {
                        let norm_rule = normalize_atmosphere(a);
                        norm_rule == norm_atmo || (norm_rule == "noatmosphere" && norm_atmo == "none")
                    });
                    if !atmo_matches {
                        return false;
                    }

                    true
                })
                .cloned()
                .collect();
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BodyType;

    #[test]
    fn test_normalize_atmosphere() {
        assert_eq!(normalize_atmosphere("Thin Carbon dioxide"), "carbondioxide");
        assert_eq!(normalize_atmosphere("CarbonDioxide"), "carbondioxide");
        assert_eq!(normalize_atmosphere("Thin Sulphur dioxide"), "sulfurdioxide");
        assert_eq!(normalize_atmosphere("No atmosphere"), "noatmosphere");
        assert_eq!(normalize_atmosphere("None"), "noatmosphere");
    }

    #[test]
    fn test_normalize_planet_class() {
        assert_eq!(normalize_planet_class("High metal content world"), "highmetalcontent");
        assert_eq!(normalize_planet_class("Rocky body"), "rocky");
        assert_eq!(normalize_planet_class("Icy body"), "icy");
    }

    #[test]
    fn test_match_star_class() {
        assert!(match_star_class("K (Yellow-Orange) Star", &StarClass::K));
        assert!(match_star_class("M (Red dwarf) Star", &StarClass::M));
        assert!(match_star_class("Neutron Star", &StarClass::N));
        assert!(match_star_class("Black Hole", &StarClass::H));
        assert!(match_star_class("White Dwarf (DA) Star", &StarClass::DA));
    }

    #[test]
    fn test_match_variant_bounds() {
        let mut body = Body::new(1, "Test Body".into());
        body.body_type = BodyType::Planet;
        body.landable = true;
        body.planet_class = Some("Rocky body".to_string());
        body.atmosphere = Some("Thin Carbon dioxide".to_string());
        body.gravity = Some(0.16);
        body.temperature = Some(178.0);
        body.bio_signals = 1;

        let variant = SpeciesVariant {
            name: "Aleoida Arcus - Grey",
            genus: "Aleoida",
            reward: 7252500,
            atmosphere_types: &["Thin Carbon dioxide"],
            bodies: &["Rocky body"],
            primary_stars: &["Wolf-Rayet Star", "M (Red dwarf) Star"],
            min_g: 0.05,
            max_g: 0.25,
            min_t: 150.0,
            max_t: 200.0,
            min_p: 0.0,
            max_p: 1.0,
            volcanism: &["No volcanism"],
        };

        // Standard match should succeed
        assert!(match_variant(&variant, &body, Some(&StarClass::M)));

        // Mismatched star should fail
        assert!(!match_variant(&variant, &body, Some(&StarClass::K)));

        // Gravity out of bounds should fail
        body.gravity = Some(0.30);
        assert!(!match_variant(&variant, &body, Some(&StarClass::M)));
    }

    #[test]
    fn test_relaxed_fallback_when_strict_bounds_fail() {
        let mut body = Body::new(1, "Test Planet".into());
        body.body_type = BodyType::Planet;
        body.landable = true;
        body.planet_class = Some("Rocky body".to_string());
        body.atmosphere = Some("Thin Carbon dioxide".to_string());
        // Gravity and temperature are completely out of range of Aleoida bounds
        body.gravity = Some(2.5);
        body.temperature = Some(900.0);
        body.bio_signals = 1;
        // BUT we know Aleoida is present via SAA scan
        body.bio_genuses = vec!["Aleoida".to_string()];

        // Strict matching would fail completely because of gravity and temperature.
        // But with bio_genuses set, the relaxed fallback should kick in and find Aleoida variants!
        let predictions = predict_species(&body, Some(&StarClass::M));
        assert!(!predictions.is_empty(), "Relaxed fallback should predict species even if bounds fail");
        assert_eq!(predictions[0].genus, "Aleoida");
    }

    #[test]
    fn test_brown_dwarf_differentiation() {
        // L brown dwarf should match only "L (Brown dwarf) Star"
        assert!(match_star_class("L (Brown dwarf) Star", &StarClass::L));
        assert!(!match_star_class("T (Brown dwarf) Star", &StarClass::L));
        assert!(!match_star_class("Y (Brown dwarf) Star", &StarClass::L));

        // T brown dwarf should match only "T (Brown dwarf) Star"
        assert!(match_star_class("T (Brown dwarf) Star", &StarClass::T));
        assert!(!match_star_class("L (Brown dwarf) Star", &StarClass::T));
        assert!(!match_star_class("Y (Brown dwarf) Star", &StarClass::T));

        // Y brown dwarf should match only "Y (Brown dwarf) Star"
        assert!(match_star_class("Y (Brown dwarf) Star", &StarClass::Y));
        assert!(!match_star_class("L (Brown dwarf) Star", &StarClass::Y));
        assert!(!match_star_class("T (Brown dwarf) Star", &StarClass::Y));
    }

    #[test]
    fn test_white_dwarf_subtypes() {
        // DA should match "White Dwarf (DA) Star" but not "White Dwarf (DB) Star"
        assert!(match_star_class("White Dwarf (DA) Star", &StarClass::DA));
        assert!(!match_star_class("White Dwarf (DB) Star", &StarClass::DA));
        assert!(!match_star_class("White Dwarf (DC) Star", &StarClass::DA));

        // DB should match "White Dwarf (DB) Star" only
        assert!(match_star_class("White Dwarf (DB) Star", &StarClass::DB));
        assert!(!match_star_class("White Dwarf (DA) Star", &StarClass::DB));

        // DC should match "White Dwarf (DC) Star" only
        assert!(match_star_class("White Dwarf (DC) Star", &StarClass::DC));
        assert!(!match_star_class("White Dwarf (DA) Star", &StarClass::DC));

        // DAZ should match "White Dwarf (DAZ) Star" only
        assert!(match_star_class("White Dwarf (DAZ) Star", &StarClass::DAZ));
        assert!(!match_star_class("White Dwarf (DA) Star", &StarClass::DAZ));
    }

    #[test]
    fn test_supergiant_matching() {
        // A supergiant should match "A (Blue-White super giant) Star"
        assert!(match_star_class("A (Blue-White super giant) Star", &StarClass::ABlueWhiteSuperGiant));
        // But should NOT match regular "A (Blue-White) Star"
        assert!(!match_star_class("A (Blue-White) Star", &StarClass::ABlueWhiteSuperGiant));

        // Regular A should NOT match supergiant
        assert!(!match_star_class("A (Blue-White super giant) Star", &StarClass::A));

        // B supergiant
        assert!(match_star_class("B (Blue-White super giant) Star", &StarClass::BBlueWhiteSuperGiant));
        assert!(!match_star_class("B (Blue-White) Star", &StarClass::BBlueWhiteSuperGiant));

        // M Red giant vs M Red super giant vs M regular
        assert!(match_star_class("M (Red giant) Star", &StarClass::MRedGiant));
        assert!(!match_star_class("M (Red super giant) Star", &StarClass::MRedGiant));
        assert!(match_star_class("M (Red super giant) Star", &StarClass::MRedSuperGiant));
        assert!(!match_star_class("M (Red dwarf) Star", &StarClass::MRedGiant));
    }

    #[test]
    fn test_volcanism_filtering_no_volcanism() {
        let mut body = Body::new(1, "Test".into());
        body.body_type = BodyType::Planet;
        body.landable = true;
        body.planet_class = Some("Rocky body".to_string());
        body.atmosphere = Some("Thin Carbon dioxide".to_string());
        body.gravity = Some(0.16);
        body.temperature = Some(178.0);
        body.volcanism = None; // No volcanism

        let variant = SpeciesVariant {
            name: "Test Species",
            genus: "Aleoida",
            reward: 1000,
            atmosphere_types: &["Thin Carbon dioxide"],
            bodies: &["Rocky body"],
            primary_stars: &[],
            min_g: 0.0, max_g: 10.0,
            min_t: 0.0, max_t: 1000.0,
            min_p: 0.0, max_p: 100.0,
            volcanism: &["No volcanism"],
        };

        // Body with no volcanism should match "No volcanism" rule
        assert!(match_variant(&variant, &body, None));

        // Body WITH volcanism should NOT match "No volcanism" only rule
        body.volcanism = Some("minor metallic magma".to_string());
        assert!(!match_variant(&variant, &body, None));
    }

    #[test]
    fn test_volcanism_filtering_requires_volcanism() {
        let mut body = Body::new(1, "Test".into());
        body.body_type = BodyType::Planet;
        body.landable = true;
        body.planet_class = Some("Rocky body".to_string());
        body.atmosphere = Some("Thin Sulfur dioxide".to_string());
        body.gravity = Some(0.16);
        body.temperature = Some(400.0);

        let variant = SpeciesVariant {
            name: "Fumerola Test",
            genus: "Fumerola",
            reward: 1000,
            atmosphere_types: &["Thin Sulfur dioxide"],
            bodies: &["Rocky body"],
            primary_stars: &[],
            min_g: 0.0, max_g: 10.0,
            min_t: 0.0, max_t: 1000.0,
            min_p: 0.0, max_p: 100.0,
            volcanism: &["Minor Metallic Magma", "Metallic Magma"],
        };

        // No volcanism should fail
        body.volcanism = None;
        assert!(!match_variant(&variant, &body, None));

        // Matching volcanism should pass
        body.volcanism = Some("minor metallic magma".to_string());
        assert!(match_variant(&variant, &body, None));

        // Non-matching volcanism should fail
        body.volcanism = Some("water geysers".to_string());
        assert!(!match_variant(&variant, &body, None));
    }

    #[test]
    fn test_pressure_filtering() {
        let mut body = Body::new(1, "Test".into());
        body.body_type = BodyType::Planet;
        body.landable = true;
        body.planet_class = Some("Rocky body".to_string());
        body.atmosphere = Some("Thin Carbon dioxide".to_string());
        body.gravity = Some(0.16);
        body.temperature = Some(178.0);
        body.volcanism = None;
        body.pressure_atm = Some(0.05); // 0.05 atm

        let variant = SpeciesVariant {
            name: "Test Species",
            genus: "Aleoida",
            reward: 1000,
            atmosphere_types: &["Thin Carbon dioxide"],
            bodies: &["Rocky body"],
            primary_stars: &[],
            min_g: 0.0, max_g: 10.0,
            min_t: 0.0, max_t: 1000.0,
            min_p: 0.01, max_p: 0.10,
            volcanism: &["No volcanism"],
        };

        // Pressure in range should match
        assert!(match_variant(&variant, &body, None));

        // Pressure out of range should fail
        body.pressure_atm = Some(0.20);
        assert!(!match_variant(&variant, &body, None));

        // Pressure below minimum should fail
        body.pressure_atm = Some(0.005);
        assert!(!match_variant(&variant, &body, None));
    }
}
