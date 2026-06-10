//! Per-species region constraints for exobiology predictions.
//!
//! Region constraints are per-species (not per-genus or per-variant).
//! They use group name strings from BioScan's regions.py, resolved
//! against the region groups in region_data.rs.
//!
//! Semantics:
//! - `"group_name"` = INCLUDE: body's region must be in this group
//! - `"!group_name"` = EXCLUDE: body's region must NOT be in this group
//! - Multiple includes are OR'd (any match = pass)
//! - Multiple excludes are AND'd (all must pass)
//! - Empty = no region constraint (allowed everywhere)

use crate::model::region::region_in_group;

/// Static lookup: species name prefix → region constraint strings.
/// Entries with `!` prefix are exclusions; others are inclusions.
static SPECIES_REGION_CONSTRAINTS: &[(&str, &[&str])] = &[
    // ── Aleoida ──
    ("Aleoida Spica", &["!orion-cygnus-core", "!sagittarius-carina-core"]),
    ("Aleoida Laminiae", &["orion-cygnus", "sagittarius-carina"]),

    // ── Cactoida ──
    ("Cactoida Cortexum", &["orion-cygnus"]),
    ("Cactoida Lapis", &["sagittarius-carina"]),
    ("Cactoida Pullulanta", &["perseus"]),
    ("Cactoida Peperatis", &["scutum-centaurus"]),

    // ── Frutexa ──
    ("Frutexa Flabellum", &["!scutum-centaurus"]),
    ("Frutexa Acus", &["orion-cygnus"]),
    ("Frutexa Flammasis", &["scutum-centaurus"]),
    ("Frutexa Fera", &["outer"]),

    // ── Fungoida ──
    ("Fungoida Stabitis", &["orion-cygnus"]),
    ("Fungoida Gelata", &["!orion-cygnus-core"]),

    // ── Osseus ──
    ("Osseus Fractus", &["!perseus"]),
    ("Osseus Cornibus", &["perseus"]),
    ("Osseus Pellebantus", &["!perseus"]),

    // ── Stratum ──
    ("Stratum Excutitus", &["orion-cygnus"]),
    ("Stratum Laminamus", &["orion-cygnus"]),
    ("Stratum Limaxus", &["scutum-centaurus-core"]),
    ("Stratum Cucumisis", &["sagittarius-carina"]),
    ("Stratum Frigus", &["perseus-core"]),

    // ── Tubus ──
    ("Tubus Conifer", &["perseus"]),
    ("Tubus Cavas", &["scutum-centaurus"]),
    ("Tubus Compagibus", &["sagittarius-carina"]),

    // ── Tussock ──
    ("Tussock Pennata", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),
    ("Tussock Ventusa", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),
    ("Tussock Ignis", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),
    ("Tussock Cultro", &["orion-cygnus"]),
    ("Tussock Catena", &["scutum-centaurus-core"]),
    ("Tussock Pennatis", &["outer"]),
    ("Tussock Serrati", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),
    ("Tussock Albata", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),
    ("Tussock Propagito", &["scutum-centaurus"]),
    ("Tussock Divisa", &["perseus-core"]),
    ("Tussock Caputus", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),
    ("Tussock Triticum", &["sagittarius-carina-core-9", "perseus-core", "orion-cygnus-core"]),

    // ── Anemone ──
    ("Luteolum Anemone", &["anemone-a"]),
    ("Croceum Anemone", &["anemone-a"]),
    ("Puniceum Anemone", &["anemone-a"]),
    ("Roseum Anemone", &["anemone-a"]),
    ("Blatteum Bioluminescent Anemone", &["anemone-a"]),

    // ── Crystalline Shards ──
    ("Crystalline Shards", &["exterior"]),

    // ── Bark Mound ──
    ("Bark Mound", &["!center"]),

    // ── Amphora Plant ──
    ("Amphora Plant", &["amphora"]),
];

/// Look up region constraints for a species by its variant name.
/// Returns the constraint strings if found, or an empty slice if unconstrained.
fn find_constraints(variant_name: &str) -> &'static [&'static str] {
    for &(species, constraints) in SPECIES_REGION_CONSTRAINTS {
        if variant_name.starts_with(species) {
            return constraints;
        }
    }
    &[] // No constraints = allowed everywhere
}

/// Check if a species variant is allowed in the given galactic region.
///
/// Returns `true` if the variant passes region filtering (or has no constraint).
/// Returns `false` if the variant is excluded from this region.
///
/// When `region_id` is `None` (region unknown), constraints are skipped (permissive).
pub fn check_region(variant_name: &str, region_id: Option<u8>) -> bool {
    let constraints = find_constraints(variant_name);
    if constraints.is_empty() {
        return true; // No constraints
    }

    let region_id = match region_id {
        Some(id) => id,
        None => return true, // Unknown region — don't filter
    };

    // Separate includes and excludes
    let mut has_includes = false;
    let mut any_include_matches = false;

    for &constraint in constraints {
        if let Some(group) = constraint.strip_prefix('!') {
            // EXCLUDE: region must NOT be in this group
            if region_in_group(region_id, group) {
                return false; // Excluded
            }
        } else {
            // INCLUDE: region must be in at least one of these groups
            has_includes = true;
            if region_in_group(region_id, constraint) {
                any_include_matches = true;
            }
        }
    }

    // If there were include rules, at least one must have matched
    if has_includes && !any_include_matches {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconstrained_species_pass_everywhere() {
        // Bacterium has no region constraints
        assert!(check_region("Bacterium Acies - Cyan", Some(1)));
        assert!(check_region("Bacterium Acies - Cyan", Some(42)));
        assert!(check_region("Bacterium Acies - Cyan", None));
    }

    #[test]
    fn tussock_pennata_region_locked() {
        // Tussock Pennata: include sagittarius-carina-core-9, perseus-core, orion-cygnus-core
        // Inner Orion Spur (18) is in orion-cygnus-core → should pass
        assert!(check_region("Tussock Pennata - Green", Some(18)));
        // Outer Arm (27) is NOT in any of those groups → should fail
        assert!(!check_region("Tussock Pennata - Green", Some(27)));
    }

    #[test]
    fn osseus_fractus_excluded_from_perseus() {
        // Osseus Fractus: exclude perseus
        // Perseus Arm (30) is in perseus → should fail
        assert!(!check_region("Osseus Fractus - Grey", Some(30)));
        // Inner Orion Spur (18) is NOT in perseus → should pass
        assert!(check_region("Osseus Fractus - Grey", Some(18)));
    }

    #[test]
    fn cactoida_cortexum_orion_cygnus_only() {
        // Cactoida Cortexum: include orion-cygnus
        // Inner Orion Spur (18) is in orion-cygnus → pass
        assert!(check_region("Cactoida Cortexum - Green", Some(18)));
        // Norma Expanse (10) is NOT in orion-cygnus → fail
        assert!(!check_region("Cactoida Cortexum - Green", Some(10)));
    }

    #[test]
    fn unknown_region_is_permissive() {
        // When region is unknown, don't filter
        assert!(check_region("Tussock Pennata - Green", None));
        assert!(check_region("Osseus Fractus - Grey", None));
    }

    #[test]
    fn aleoida_spica_exclude_both_cores() {
        // Aleoida Spica: exclude orion-cygnus-core AND sagittarius-carina-core
        // Outer Arm (27) is in neither → should pass
        assert!(check_region("Aleoida Spica - Green", Some(27)));
        // Inner Orion Spur (18) is in orion-cygnus-core → should fail
        assert!(!check_region("Aleoida Spica - Green", Some(18)));
    }
}
