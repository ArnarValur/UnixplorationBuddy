//! Variant color resolution from star class and surface materials.
//!
//! Elite Dangerous exobiology species have color variants determined by either:
//! 1. **Star class** of the primary star (genus-level or per-species)
//! 2. **Surface material** (element) of the body (per-species)
//!
//! Source: ExploData genus.py + BioScan rulesets

/// Star class key for color lookup — a simplified version of the full star class.
/// Maps from the `StarClass` enum to a short key used in color tables.
pub fn star_key(star_class: &str) -> &str {
    let s = star_class.trim();
    // Map Debug format of StarClass to lookup key
    match s {
        "O" => "O",
        "B" | "BBlueWhiteSuperGiant" => "B",
        "A" | "ABlueWhiteSuperGiant" => "A",
        "F" | "FWhiteSuperGiant" => "F",
        "G" | "GWhiteSuperGiant" => "G",
        "K" | "KOrangeGiant" => "K",
        "M" | "MRedGiant" | "MRedSuperGiant" => "M",
        "L" => "L",
        "T" => "T",
        "Y" => "Y",
        "TTS" => "TTS",
        "N" => "N",
        "H" | "SupermassiveBlackHole" => "H",
        // White dwarfs → "D"
        "D" | "DA" | "DAB" | "DAO" | "DAZ" | "DAV" | "DB" | "DBZ" | "DBV" |
        "DO" | "DOV" | "DQ" | "DC" | "DCV" | "DX" => "D",
        // Wolf-Rayet → "W"
        "W" | "WN" | "WNC" | "WC" | "WO" => "W",
        // Carbon → "C"
        "CS" | "C" | "CN" | "CJ" | "CH" | "CHd" => "C",
        // Herbig Ae/Be
        "Ae" | "Be" | "AeBe" => "Ae",
        // S/MS
        "S" => "S",
        "MS" => "MS",
        _ => s,
    }
}

/// Element symbol from material name (case-insensitive).
/// Returns the periodic table symbol used in color lookup tables.
pub fn element_symbol(material: &str) -> Option<&'static str> {
    match material.to_lowercase().as_str() {
        "antimony" => Some("Sb"),
        "polonium" => Some("Po"),
        "ruthenium" => Some("Ru"),
        "technetium" => Some("Tc"),
        "tellurium" => Some("Te"),
        "yttrium" => Some("Y"),
        "cadmium" => Some("Cd"),
        "mercury" => Some("Hg"),
        "molybdenum" => Some("Mo"),
        "niobium" => Some("Nb"),
        "tungsten" => Some("W"),
        "tin" => Some("Sn"),
        _ => None,
    }
}

// ─── Genus-Level Star → Color Tables ───

static ALEOIDA_COLORS: &[(&str, &str)] = &[
    ("B", "Yellow"), ("A", "Green"), ("F", "Teal"), ("K", "Turquoise"),
    ("M", "Emerald"), ("L", "Lime"), ("T", "Sage"), ("TTS", "Mauve"),
    ("D", "Indigo"), ("W", "Grey"), ("Y", "Amethyst"), ("N", "Ocher"),
];

static CACTOIDA_COLORS: &[(&str, &str)] = &[
    ("O", "Grey"), ("A", "Green"), ("F", "Yellow"), ("G", "Teal"),
    ("M", "Amethyst"), ("L", "Mauve"), ("T", "Orange"), ("Y", "Ocher"),
    ("TTS", "Red"), ("W", "Indigo"), ("D", "Turquoise"), ("N", "Sage"),
];

static CLYPEUS_COLORS: &[(&str, &str)] = &[
    ("B", "Maroon"), ("A", "Orange"), ("F", "Mauve"), ("G", "Amethyst"),
    ("K", "Grey"), ("M", "Turquoise"), ("L", "Teal"), ("Y", "Green"),
    ("D", "Lime"), ("N", "Yellow"),
];

static FONTICULUA_COLORS: &[(&str, &str)] = &[
    ("O", "Grey"), ("B", "Lime"), ("A", "Green"), ("F", "Yellow"),
    ("G", "Teal"), ("K", "Emerald"), ("M", "Amethyst"), ("L", "Mauve"),
    ("T", "Orange"), ("TTS", "Red"), ("Y", "Ocher"), ("W", "Indigo"),
    ("D", "Turquoise"), ("N", "Sage"), ("Ae", "Maroon"),
];

static FRUTEXA_COLORS: &[(&str, &str)] = &[
    ("O", "Yellow"), ("B", "Lime"), ("F", "Green"), ("G", "Emerald"),
    ("M", "Grey"), ("L", "Teal"), ("TTS", "Mauve"), ("W", "Orange"),
    ("D", "Indigo"), ("N", "Red"),
];

static TUBUS_COLORS: &[(&str, &str)] = &[
    ("O", "Green"), ("B", "Emerald"), ("A", "Indigo"), ("F", "Grey"),
    ("G", "Red"), ("K", "Maroon"), ("M", "Teal"), ("L", "Turquoise"),
    ("T", "Mauve"), ("TTS", "Ocher"), ("W", "Lime"), ("D", "Yellow"),
    ("N", "Amethyst"),
];

static TUSSOCK_COLORS: &[(&str, &str)] = &[
    ("F", "Yellow"), ("G", "Lime"), ("K", "Green"), ("M", "Emerald"),
    ("L", "Sage"), ("T", "Teal"), ("Y", "Red"), ("W", "Orange"),
    ("D", "Maroon"), ("N", "Yellow"),
];

// ─── Per-Species Star → Color Tables ───

/// Bacterium Aurasus (01), Alcyoneum (06), Cerbrus (12) share this table
static BACTERIUM_STAR_COLORS: &[(&str, &str)] = &[
    ("O", "Turquoise"), ("B", "Grey"), ("A", "Yellow"), ("F", "Lime"),
    ("G", "Emerald"), ("K", "Green"), ("M", "Teal"), ("L", "Sage"),
    ("T", "Red"), ("Y", "Mauve"), ("TTS", "Maroon"), ("Ae", "Orange"),
    ("W", "Amethyst"), ("D", "Ocher"), ("N", "Indigo"),
];

/// Concha Aureolas (02) and Labiata (03)
static CONCHA_STAR_COLORS: &[(&str, &str)] = &[
    ("B", "Indigo"), ("A", "Teal"), ("F", "Grey"), ("G", "Turquoise"),
    ("K", "Red"), ("L", "Orange"), ("Y", "Yellow"), ("W", "Lime"),
    ("D", "Green"), ("N", "Emerald"),
];

/// Osseus Fractus (01), Spiralis (03), Cornibus (05), Pellebantus (06)
static OSSEUS_STAR_COLORS: &[(&str, &str)] = &[
    ("O", "Yellow"), ("A", "Lime"), ("F", "Turquoise"), ("G", "Grey"),
    ("K", "Indigo"), ("T", "Emerald"), ("Y", "Maroon"), ("TTS", "Green"),
];

/// Recepta Umbrux (01)
static RECEPTA_STAR_COLORS: &[(&str, &str)] = &[
    ("B", "Turquoise"), ("A", "Amethyst"), ("F", "Mauve"), ("G", "Orange"),
    ("K", "Red"), ("M", "Maroon"), ("T", "Teal"), ("Y", "Lime"),
    ("TTS", "Sage"), ("L", "Ocher"), ("Ae", "Grey"), ("D", "Yellow"),
    ("N", "Emerald"),
];

/// Stratum (most species)
static STRATUM_STAR_COLORS: &[(&str, &str)] = &[
    ("F", "Emerald"), ("K", "Lime"), ("M", "Green"), ("L", "Turquoise"),
    ("Y", "Indigo"), ("T", "Grey"), ("TTS", "Amethyst"), ("D", "Mauve"),
    ("Ae", "Teal"), ("W", "Red"),
];

/// Stratum Araneamus (special — always Emerald)
static STRATUM_ARANEAMUS_COLORS: &[(&str, &str)] = &[
    ("B", "Emerald"), ("A", "Emerald"), ("N", "Emerald"),
];

// ─── Per-Species Element → Color Tables ───

// Element Group 1: Sb, Po, Ru, Tc, Te, Y
// Element Group 2: Cd, Hg, Mo, Nb, W, Sn

/// Bacterium Nebulus (02) — Group 1
static BACTERIUM_NEBULUS_ELEM: &[(&str, &str)] = &[
    ("Sb", "Magenta"), ("Po", "Gold"), ("Ru", "Orange"), ("Tc", "Cyan"),
    ("Te", "Green"), ("Y", "Cobalt"),
];
/// Bacterium Scopulum (03) — Group 2
static BACTERIUM_SCOPULUM_ELEM: &[(&str, &str)] = &[
    ("Cd", "White"), ("Hg", "Peach"), ("Mo", "Lime"), ("Nb", "Red"),
    ("W", "Aquamarine"), ("Sn", "Mulberry"),
];
/// Bacterium Acies (04) — Group 1
static BACTERIUM_ACIES_ELEM: &[(&str, &str)] = &[
    ("Sb", "Cyan"), ("Po", "Magenta"), ("Ru", "Cobalt"), ("Tc", "Lime"),
    ("Te", "White"), ("Y", "Aquamarine"),
];
/// Bacterium Vesicula (05) — Group 1
static BACTERIUM_VESICULA_ELEM: &[(&str, &str)] = &[
    ("Sb", "Cyan"), ("Po", "Orange"), ("Ru", "Mulberry"), ("Tc", "Gold"),
    ("Te", "Red"), ("Y", "Lime"),
];
/// Bacterium Tela (07) — Group 2
static BACTERIUM_TELA_ELEM: &[(&str, &str)] = &[
    ("Cd", "Gold"), ("Hg", "Orange"), ("Mo", "Yellow"), ("Nb", "Magenta"),
    ("W", "Green"), ("Sn", "Cobalt"),
];
/// Bacterium Informem (08) — Group 1
static BACTERIUM_INFORMEM_ELEM: &[(&str, &str)] = &[
    ("Sb", "Red"), ("Po", "Lime"), ("Ru", "Gold"), ("Tc", "Aquamarine"),
    ("Te", "Yellow"), ("Y", "Cobalt"),
];
/// Bacterium Volu (09) — Group 1
static BACTERIUM_VOLU_ELEM: &[(&str, &str)] = &[
    ("Sb", "Red"), ("Po", "Aquamarine"), ("Ru", "Cobalt"), ("Tc", "Lime"),
    ("Te", "Cyan"), ("Y", "Gold"),
];
/// Bacterium Bullaris (10) — Group 1
static BACTERIUM_BULLARIS_ELEM: &[(&str, &str)] = &[
    ("Sb", "Cobalt"), ("Po", "Yellow"), ("Ru", "Aquamarine"), ("Tc", "Gold"),
    ("Te", "Lime"), ("Y", "Red"),
];
/// Bacterium Omentum (11) — Group 2
static BACTERIUM_OMENTUM_ELEM: &[(&str, &str)] = &[
    ("Cd", "Lime"), ("Hg", "White"), ("Mo", "Aquamarine"), ("Nb", "Peach"),
    ("W", "Blue"), ("Sn", "Red"),
];
/// Bacterium Verrata (13) — Group 2
static BACTERIUM_VERRATA_ELEM: &[(&str, &str)] = &[
    ("Cd", "Peach"), ("Hg", "Red"), ("Mo", "White"), ("Nb", "Mulberry"),
    ("W", "Lime"), ("Sn", "Blue"),
];

/// Electricae Pluma (01) — Group 1
static ELECTRICAE_PLUMA_ELEM: &[(&str, &str)] = &[
    ("Sb", "Cobalt"), ("Po", "Cyan"), ("Ru", "Blue"), ("Tc", "Magenta"),
    ("Te", "Red"), ("Y", "Mulberry"),
];
/// Electricae Radialem (02) — Group 1
static ELECTRICAE_RADIALEM_ELEM: &[(&str, &str)] = &[
    ("Sb", "Cyan"), ("Po", "Cobalt"), ("Ru", "Blue"), ("Tc", "Aquamarine"),
    ("Te", "Magenta"), ("Y", "Green"),
];

/// Fumerola Carbosis (01) — Group 2
static FUMEROLA_CARBOSIS_ELEM: &[(&str, &str)] = &[
    ("Cd", "Orange"), ("Hg", "Magenta"), ("Mo", "Gold"), ("Nb", "Cobalt"),
    ("W", "Yellow"), ("Sn", "Cyan"),
];
/// Fumerola Extremus (02) — Group 2
static FUMEROLA_EXTREMUS_ELEM: &[(&str, &str)] = &[
    ("Cd", "Aquamarine"), ("Hg", "Lime"), ("Mo", "Blue"), ("Nb", "White"),
    ("W", "Mulberry"), ("Sn", "Peach"),
];
/// Fumerola Nitris (03) — Group 2
static FUMEROLA_NITRIS_ELEM: &[(&str, &str)] = &[
    ("Cd", "White"), ("Hg", "Peach"), ("Mo", "Lime"), ("Nb", "Red"),
    ("W", "Aquamarine"), ("Sn", "Mulberry"),
];
/// Fumerola Aquatis (04) — Group 2
static FUMEROLA_AQUATIS_ELEM: &[(&str, &str)] = &[
    ("Cd", "Green"), ("Hg", "Yellow"), ("Mo", "Cyan"), ("Nb", "Gold"),
    ("W", "Cobalt"), ("Sn", "Orange"),
];

/// Fungoida Setisis (01) — Group 1
static FUNGOIDA_SETISIS_ELEM: &[(&str, &str)] = &[
    ("Sb", "Peach"), ("Po", "White"), ("Ru", "Gold"), ("Tc", "Lime"),
    ("Te", "Yellow"), ("Y", "Orange"),
];
/// Fungoida Stabitis (02) — Group 2
static FUNGOIDA_STABITIS_ELEM: &[(&str, &str)] = &[
    ("Cd", "Blue"), ("Hg", "Green"), ("Mo", "Magenta"), ("Nb", "White"),
    ("W", "Peach"), ("Sn", "Orange"),
];
/// Fungoida Bullarum (03) — Group 1
static FUNGOIDA_BULLARUM_ELEM: &[(&str, &str)] = &[
    ("Sb", "Red"), ("Po", "Mulberry"), ("Ru", "Magenta"), ("Tc", "Peach"),
    ("Te", "Gold"), ("Y", "Orange"),
];
/// Fungoida Gelata (04) — Group 2
static FUNGOIDA_GELATA_ELEM: &[(&str, &str)] = &[
    ("Cd", "Cyan"), ("Hg", "Lime"), ("Mo", "Mulberry"), ("Nb", "Green"),
    ("W", "Orange"), ("Sn", "Red"),
];

/// Concha Renibus (01) — Group 2
static CONCHA_RENIBUS_ELEM: &[(&str, &str)] = &[
    ("Cd", "Red"), ("Hg", "Mulberry"), ("Mo", "Peach"), ("Nb", "Blue"),
    ("W", "White"), ("Sn", "Aquamarine"),
];
/// Concha Biconcavis (04) — Group 1
static CONCHA_BICONCAVIS_ELEM: &[(&str, &str)] = &[
    ("Sb", "Peach"), ("Po", "Red"), ("Ru", "Orange"), ("Tc", "White"),
    ("Te", "Yellow"), ("Y", "Gold"),
];

/// Osseus Discus (02) — Group 2
static OSSEUS_DISCUS_ELEM: &[(&str, &str)] = &[
    ("Cd", "White"), ("Hg", "Lime"), ("Mo", "Peach"), ("Nb", "Aquamarine"),
    ("W", "Red"), ("Sn", "Blue"),
];
/// Osseus Pumice (04) — Group 1
static OSSEUS_PUMICE_ELEM: &[(&str, &str)] = &[
    ("Sb", "White"), ("Po", "Peach"), ("Ru", "Gold"), ("Tc", "Lime"),
    ("Te", "Green"), ("Y", "Yellow"),
];

/// Recepta Deltahedronix (02) — Group 2
static RECEPTA_DELTAHEDRONIX_ELEM: &[(&str, &str)] = &[
    ("Cd", "Lime"), ("Hg", "Cyan"), ("Mo", "Gold"), ("Nb", "Mulberry"),
    ("W", "Red"), ("Sn", "Orange"),
];
/// Recepta Conditivus (03) — Group 1
static RECEPTA_CONDITIVUS_ELEM: &[(&str, &str)] = &[
    ("Sb", "Lime"), ("Po", "White"), ("Ru", "Yellow"), ("Tc", "Aquamarine"),
    ("Te", "Cyan"), ("Y", "Green"),
];

// ─── Lookup Functions ───

/// Look up color in a static table.
fn lookup_color(table: &'static [(&str, &str)], key: &str) -> Option<&'static str> {
    for &(k, v) in table {
        if k == key {
            return Some(v);
        }
    }
    None
}

/// Extract the species code from a variant name.
/// e.g. "Bacterium Nebulus" from "Bacterium Nebulus - Cyan"
fn species_name(variant_name: &str) -> &str {
    variant_name.split(" - ").next().unwrap_or(variant_name)
}

/// Resolve the color variant for a prediction based on the system's primary star class.
///
/// `star_class_debug` should be the `format!("{:?}", star_class)` string (e.g. "K", "DA", "TTS").
/// Returns the color name (e.g. "Emerald", "Lime") or None if star class has no mapping.
pub fn resolve_star_color(variant_name: &str, star_class_debug: &str) -> Option<&'static str> {
    let key = star_key(star_class_debug);
    let species = species_name(variant_name);
    let genus = variant_name.split_whitespace().next().unwrap_or("");

    // Per-species star tables first
    match species {
        "Bacterium Aurasus" | "Bacterium Alcyoneum" | "Bacterium Cerbrus" =>
            return lookup_color(BACTERIUM_STAR_COLORS, key),
        "Concha Aureolas" | "Concha Labiata" =>
            return lookup_color(CONCHA_STAR_COLORS, key),
        "Osseus Fractus" | "Osseus Spiralis" | "Osseus Cornibus" | "Osseus Pellebantus" =>
            return lookup_color(OSSEUS_STAR_COLORS, key),
        "Recepta Umbrux" =>
            return lookup_color(RECEPTA_STAR_COLORS, key),
        "Stratum Araneamus" =>
            return lookup_color(STRATUM_ARANEAMUS_COLORS, key),
        "Stratum Excutitus" | "Stratum Paleas" | "Stratum Laminamus" | "Stratum Limaxus" |
        "Stratum Cucumisis" | "Stratum Tectonicas" | "Stratum Frigus" =>
            return lookup_color(STRATUM_STAR_COLORS, key),
        _ => {}
    }

    // Genus-level star tables
    match genus {
        "Aleoida" => lookup_color(ALEOIDA_COLORS, key),
        "Cactoida" => lookup_color(CACTOIDA_COLORS, key),
        "Clypeus" => lookup_color(CLYPEUS_COLORS, key),
        "Fonticulua" => lookup_color(FONTICULUA_COLORS, key),
        "Frutexa" => lookup_color(FRUTEXA_COLORS, key),
        "Tubus" => lookup_color(TUBUS_COLORS, key),
        "Tussock" => lookup_color(TUSSOCK_COLORS, key),
        _ => None,
    }
}

/// Resolve the color variant based on surface materials (elements).
///
/// `materials` should be a list of (material_name, percentage) tuples
/// from the Body's surface_materials field.
/// Returns the color name or None if no element mapping exists for this species.
pub fn resolve_element_color(variant_name: &str, materials: &[(String, f64)]) -> Option<&'static str> {
    let species = species_name(variant_name);

    let table: &[(&str, &str)] = match species {
        // Bacterium element species
        "Bacterium Nebulus" => BACTERIUM_NEBULUS_ELEM,
        "Bacterium Scopulum" => BACTERIUM_SCOPULUM_ELEM,
        "Bacterium Acies" => BACTERIUM_ACIES_ELEM,
        "Bacterium Vesicula" => BACTERIUM_VESICULA_ELEM,
        "Bacterium Tela" => BACTERIUM_TELA_ELEM,
        "Bacterium Informem" => BACTERIUM_INFORMEM_ELEM,
        "Bacterium Volu" => BACTERIUM_VOLU_ELEM,
        "Bacterium Bullaris" => BACTERIUM_BULLARIS_ELEM,
        "Bacterium Omentum" => BACTERIUM_OMENTUM_ELEM,
        "Bacterium Verrata" => BACTERIUM_VERRATA_ELEM,
        // Electricae
        "Electricae Pluma" => ELECTRICAE_PLUMA_ELEM,
        "Electricae Radialem" => ELECTRICAE_RADIALEM_ELEM,
        // Fumerola
        "Fumerola Carbosis" => FUMEROLA_CARBOSIS_ELEM,
        "Fumerola Extremus" => FUMEROLA_EXTREMUS_ELEM,
        "Fumerola Nitris" => FUMEROLA_NITRIS_ELEM,
        "Fumerola Aquatis" => FUMEROLA_AQUATIS_ELEM,
        // Fungoida
        "Fungoida Setisis" => FUNGOIDA_SETISIS_ELEM,
        "Fungoida Stabitis" => FUNGOIDA_STABITIS_ELEM,
        "Fungoida Bullarum" => FUNGOIDA_BULLARUM_ELEM,
        "Fungoida Gelata" => FUNGOIDA_GELATA_ELEM,
        // Concha (element species)
        "Concha Renibus" => CONCHA_RENIBUS_ELEM,
        "Concha Biconcavis" => CONCHA_BICONCAVIS_ELEM,
        // Osseus (element species)
        "Osseus Discus" => OSSEUS_DISCUS_ELEM,
        "Osseus Pumice" => OSSEUS_PUMICE_ELEM,
        // Recepta (element species)
        "Recepta Deltahedronix" => RECEPTA_DELTAHEDRONIX_ELEM,
        "Recepta Conditivus" => RECEPTA_CONDITIVUS_ELEM,
        _ => return None,
    };

    // Find the first material on the body that has a color mapping
    for (mat, _pct) in materials {
        if let Some(sym) = element_symbol(mat) {
            if let Some(color) = lookup_color(table, sym) {
                return Some(color);
            }
        }
    }

    None
}

/// Determine the color determination method for a species.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMethod {
    /// Color determined by primary star class
    Star,
    /// Color determined by surface material (element)
    Element,
    /// No color variants (e.g. Anemone, Brain Tree, Bark Mound)
    None,
}

/// Get the color determination method for a given variant/species.
pub fn color_method(variant_name: &str) -> ColorMethod {
    let species = species_name(variant_name);
    let genus = variant_name.split_whitespace().next().unwrap_or("");

    // Check element-based species first
    match species {
        "Bacterium Nebulus" | "Bacterium Scopulum" | "Bacterium Acies" |
        "Bacterium Vesicula" | "Bacterium Tela" | "Bacterium Informem" |
        "Bacterium Volu" | "Bacterium Bullaris" | "Bacterium Omentum" |
        "Bacterium Verrata" |
        "Electricae Pluma" | "Electricae Radialem" |
        "Fumerola Carbosis" | "Fumerola Extremus" | "Fumerola Nitris" |
        "Fumerola Aquatis" |
        "Fungoida Setisis" | "Fungoida Stabitis" | "Fungoida Bullarum" |
        "Fungoida Gelata" |
        "Concha Renibus" | "Concha Biconcavis" |
        "Osseus Discus" | "Osseus Pumice" |
        "Recepta Deltahedronix" | "Recepta Conditivus" => ColorMethod::Element,

        // Star-based per-species
        "Bacterium Aurasus" | "Bacterium Alcyoneum" | "Bacterium Cerbrus" |
        "Concha Aureolas" | "Concha Labiata" |
        "Osseus Fractus" | "Osseus Spiralis" | "Osseus Cornibus" | "Osseus Pellebantus" |
        "Recepta Umbrux" |
        "Stratum Araneamus" | "Stratum Excutitus" | "Stratum Paleas" |
        "Stratum Laminamus" | "Stratum Limaxus" | "Stratum Cucumisis" |
        "Stratum Tectonicas" | "Stratum Frigus" => ColorMethod::Star,

        _ => match genus {
            // Genus-level star-based
            "Aleoida" | "Cactoida" | "Clypeus" | "Fonticulua" | "Frutexa" |
            "Tubus" | "Tussock" => ColorMethod::Star,
            // No color variants
            _ => ColorMethod::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aleoida_star_colors() {
        assert_eq!(resolve_star_color("Aleoida Arcus", "K"), Some("Turquoise"));
        assert_eq!(resolve_star_color("Aleoida Arcus", "M"), Some("Emerald"));
        assert_eq!(resolve_star_color("Aleoida Arcus", "N"), Some("Ocher"));
        assert_eq!(resolve_star_color("Aleoida Arcus", "DA"), Some("Indigo")); // DA → D
    }

    #[test]
    fn tussock_star_colors() {
        assert_eq!(resolve_star_color("Tussock Pennata", "K"), Some("Green"));
        assert_eq!(resolve_star_color("Tussock Pennata", "M"), Some("Emerald"));
        assert_eq!(resolve_star_color("Tussock Pennata", "Y"), Some("Red"));
    }

    #[test]
    fn bacterium_star_vs_element_species() {
        // Aurasus uses star colors
        assert_eq!(color_method("Bacterium Aurasus"), ColorMethod::Star);
        assert_eq!(resolve_star_color("Bacterium Aurasus", "K"), Some("Green"));

        // Nebulus uses element colors
        assert_eq!(color_method("Bacterium Nebulus"), ColorMethod::Element);
        let mats: Vec<(String, f64)> = vec![("antimony".to_string(), 1.0), ("iron".to_string(), 5.0)];
        assert_eq!(resolve_element_color("Bacterium Nebulus", &mats), Some("Magenta"));
    }

    #[test]
    fn fungoida_element_colors() {
        let mats: Vec<(String, f64)> = vec![("cadmium".to_string(), 1.0)];
        assert_eq!(resolve_element_color("Fungoida Gelata", &mats), Some("Cyan"));
        assert_eq!(resolve_element_color("Fungoida Stabitis", &mats), Some("Blue"));
    }

    #[test]
    fn fumerola_element_colors() {
        let mats: Vec<(String, f64)> = vec![("tungsten".to_string(), 1.0)];
        assert_eq!(resolve_element_color("Fumerola Carbosis", &mats), Some("Yellow"));
        assert_eq!(resolve_element_color("Fumerola Aquatis", &mats), Some("Cobalt"));
    }

    #[test]
    fn no_color_species() {
        assert_eq!(color_method("Brain Tree"), ColorMethod::None);
        assert_eq!(color_method("Bark Mound"), ColorMethod::None);
        assert_eq!(color_method("Luteolum Anemone"), ColorMethod::None);
        assert_eq!(color_method("Crystalline Shards"), ColorMethod::None);
    }

    #[test]
    fn star_key_mapping() {
        assert_eq!(star_key("DA"), "D");
        assert_eq!(star_key("DB"), "D");
        assert_eq!(star_key("WN"), "W");
        assert_eq!(star_key("KOrangeGiant"), "K");
        assert_eq!(star_key("MRedGiant"), "M");
        assert_eq!(star_key("ABlueWhiteSuperGiant"), "A");
        assert_eq!(star_key("TTS"), "TTS");
    }

    #[test]
    fn stratum_araneamus_always_emerald() {
        assert_eq!(resolve_star_color("Stratum Araneamus", "B"), Some("Emerald"));
        assert_eq!(resolve_star_color("Stratum Araneamus", "A"), Some("Emerald"));
        assert_eq!(resolve_star_color("Stratum Araneamus", "N"), Some("Emerald"));
    }

    #[test]
    fn element_symbol_lookup() {
        assert_eq!(element_symbol("antimony"), Some("Sb"));
        assert_eq!(element_symbol("Antimony"), Some("Sb"));
        assert_eq!(element_symbol("tungsten"), Some("W"));
        assert_eq!(element_symbol("iron"), None); // Not a color-determining element
    }
}
