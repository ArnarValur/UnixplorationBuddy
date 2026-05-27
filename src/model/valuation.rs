//! Exploration value calculation — ported from Pioneer's body_calc.py.
//!
//! Source: <https://forums.frontier.co.uk/threads/exploration-value-formulae.232000/>
//! Reference implementation: EDMC-Pioneer by Silarn
//!
//! Calculates FSS scan value, DSS mapped value, and honk value for stars and planets.
//! All values account for first discovery / first mapping bonuses.
//! Odyssey/4.0+ mapping bonus is applied by default (game is on v4.0+).

use ed_journals::galaxy::{PlanetClass, StarClass};

/// Computed exploration values for a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodyValue {
    /// Credits from FSS scan.
    pub fss_value: u64,
    /// Credits from DSS mapping (with Odyssey bonus). 0 for stars.
    pub mapped_value: u64,
}

/// Base value constant for a star class.
fn star_base_value(star_class: &StarClass) -> f64 {
    match star_class {
        // Neutron stars and black holes
        StarClass::N | StarClass::H => 22_628.0,
        // Supermassive black hole
        StarClass::SupermassiveBlackHole => 33.5678,
        // White dwarfs (all D* variants)
        StarClass::D
        | StarClass::DA
        | StarClass::DAB
        | StarClass::DAO
        | StarClass::DAZ
        | StarClass::DAV
        | StarClass::DB
        | StarClass::DBZ
        | StarClass::DBV
        | StarClass::DO
        | StarClass::DOV
        | StarClass::DQ
        | StarClass::DC
        | StarClass::DCV
        | StarClass::DX => 14_057.0,
        // Everything else: main sequence, giants, Wolf-Rayet, carbon, etc.
        _ => 1_200.0,
    }
}

/// Base value and terraform bonus for a planet class.
///
/// Returns `(base_value, terraform_bonus)`.
fn planet_base_values(planet_class: &PlanetClass, terraformable: bool) -> (f64, f64) {
    match planet_class {
        PlanetClass::MetalRichBody => (21_790.0, 0.0),
        PlanetClass::AmmoniaWorld => (96_932.0, 0.0),
        PlanetClass::SudarskyClassIGasGiant => (1_656.0, 0.0),
        PlanetClass::SudarskyClassIIGasGiant | PlanetClass::HighMetalContentBody => {
            let tf = if terraformable { 100_677.0 } else { 0.0 };
            (9_654.0, tf)
        }
        PlanetClass::WaterWorld => {
            let tf = if terraformable { 116_295.0 } else { 0.0 };
            (64_831.0, tf)
        }
        PlanetClass::EarthlikeBody => {
            // Natural ELWs always get terraform bonus;
            // already-terraformed ELWs do not.
            let tf = if !terraformable { 116_295.0 } else { 0.0 };
            (64_831.0, tf)
        }
        _ => {
            // Rocky, Icy, Rocky Ice, gas giants III-V, etc.
            let tf = if terraformable { 93_328.0 } else { 0.0 };
            (300.0, tf)
        }
    }
}

const Q: f64 = 0.56591828;
const MASS_EXPONENT: f64 = 0.2;
const MIN_VALUE: f64 = 500.0;

/// Calculate exploration value for a **star**.
pub fn calculate_star_value(
    star_class: &StarClass,
    stellar_mass: f64,
    first_discovery: bool,
) -> BodyValue {
    let k = star_base_value(star_class);
    let mass = stellar_mass.max(1.0);

    let mut value = k + (mass * k / 66.25);

    if first_discovery {
        value *= 2.6;
    }

    BodyValue {
        fss_value: value.round() as u64,
        mapped_value: 0, // Stars can't be mapped
    }
}

/// Calculate exploration value for a **planet or moon**.
///
/// `probes_efficient` should be `true` when `probes_used <= efficiency_target`
/// from `SAAScanComplete`, which grants a 1.25x mapping bonus.
pub fn calculate_planet_value(
    planet_class: &PlanetClass,
    earth_masses: f64,
    terraformable: bool,
    first_discovery: bool,
    first_mapping: bool,
    probes_efficient: bool,
) -> BodyValue {
    let (base, tf_bonus) = planet_base_values(planet_class, terraformable);
    let k = base + tf_bonus;
    let mass = earth_masses.max(1.0);

    // FSS scan value
    let scan_value = (k + k * Q * mass.powf(MASS_EXPONENT)).max(MIN_VALUE);

    // Mapping multiplier
    let mapping_multiplier = match (first_discovery, first_mapping) {
        (true, true) => 3.699622554,
        (false, true) => 8.0956,
        _ => 10.0 / 3.0,
    };

    let mut mapped_value = scan_value * mapping_multiplier;

    // Odyssey/4.0+ bonus (always applied — game is on v4.0+)
    let odyssey_bonus = (mapped_value * 0.3).max(555.0);
    mapped_value += odyssey_bonus;
    mapped_value = mapped_value.max(MIN_VALUE);

    // Efficiency bonus: 1.25x when using optimal probes
    if probes_efficient {
        mapped_value *= 1.25;
    }

    // First discovery multiplier (applied after all other calculations)
    let fd_mult = if first_discovery { 2.6 } else { 1.0 };

    BodyValue {
        fss_value: (scan_value * fd_mult).round() as u64,
        mapped_value: (mapped_value * fd_mult).round() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Star values
    // ---------------------------------------------------------------

    #[test]
    fn main_sequence_star_value() {
        let v = calculate_star_value(&StarClass::K, 1.0, false);
        // k=1200, value = 1200 + (1.0 * 1200 / 66.25) = 1218
        assert_eq!(v.fss_value, 1218);
        assert_eq!(v.mapped_value, 0);
    }

    #[test]
    fn main_sequence_star_first_discovery() {
        let v = calculate_star_value(&StarClass::K, 1.0, true);
        // 1218 * 2.6 = 3167
        assert_eq!(v.fss_value, 3167);
    }

    #[test]
    fn neutron_star_value() {
        let v = calculate_star_value(&StarClass::N, 1.0, false);
        // k=22628, value = 22628 + 341.56 = 22970
        assert_eq!(v.fss_value, 22970);
    }

    #[test]
    fn white_dwarf_value() {
        let v = calculate_star_value(&StarClass::DA, 0.5, false);
        // mass clamped to 1.0, k=14057
        assert_eq!(v.fss_value, 14269);
    }

    #[test]
    fn high_mass_star() {
        let v = calculate_star_value(&StarClass::B, 3.18, false);
        assert_eq!(v.fss_value, 1258);
    }

    // ---------------------------------------------------------------
    // Planet values
    // ---------------------------------------------------------------

    #[test]
    fn earthlike_body_value() {
        let v = calculate_planet_value(
            &PlanetClass::EarthlikeBody, 1.0, false, true, true, true,
        );
        assert!(v.fss_value > 700_000, "ELW first disc FSS should be >700k, got {}", v.fss_value);
        assert!(v.mapped_value > 4_000_000, "ELW mapped should be >4M, got {}", v.mapped_value);
    }

    #[test]
    fn icy_body_value() {
        let v = calculate_planet_value(
            &PlanetClass::IcyBody, 0.1, false, false, false, false,
        );
        assert_eq!(v.fss_value, 500);
    }

    #[test]
    fn ammonia_world_value() {
        let v = calculate_planet_value(
            &PlanetClass::AmmoniaWorld, 1.0, false, false, false, false,
        );
        assert_eq!(v.fss_value, 151_788);
    }

    #[test]
    fn water_world_terraformable_first_disc() {
        let v = calculate_planet_value(
            &PlanetClass::WaterWorld, 1.0, true, true, false, false,
        );
        assert_eq!(v.fss_value, 737_434);
    }

    // ---------------------------------------------------------------
    // Modifiers
    // ---------------------------------------------------------------

    #[test]
    fn first_discovery_applies_2_6x() {
        let no_fd = calculate_planet_value(
            &PlanetClass::MetalRichBody, 1.0, false, false, false, false,
        );
        let fd = calculate_planet_value(
            &PlanetClass::MetalRichBody, 1.0, false, true, false, false,
        );
        let expected = (no_fd.fss_value as f64 * 2.6).round() as u64;
        let diff = (fd.fss_value as i64 - expected as i64).abs();
        assert!(diff <= 1, "~2.6x: expected {expected}, got {}", fd.fss_value);
    }

    #[test]
    fn mapping_increases_value() {
        let unmapped = calculate_planet_value(
            &PlanetClass::HighMetalContentBody, 1.0, true, false, false, false,
        );
        let mapped = calculate_planet_value(
            &PlanetClass::HighMetalContentBody, 1.0, true, false, true, false,
        );
        assert!(mapped.mapped_value > unmapped.fss_value * 5);
    }

    #[test]
    fn efficiency_bonus_adds_25_percent() {
        let no_eff = calculate_planet_value(
            &PlanetClass::WaterWorld, 1.0, false, false, true, false,
        );
        let eff = calculate_planet_value(
            &PlanetClass::WaterWorld, 1.0, false, false, true, true,
        );
        let expected = (no_eff.mapped_value as f64 * 1.25).round() as u64;
        let diff = (eff.mapped_value as i64 - expected as i64).abs();
        assert!(diff <= 1, "~1.25x: expected {expected}, got {}", eff.mapped_value);
    }

    #[test]
    fn higher_mass_increases_value() {
        let light = calculate_planet_value(
            &PlanetClass::RockyBody, 0.01, false, false, false, false,
        );
        let heavy = calculate_planet_value(
            &PlanetClass::RockyBody, 10.0, false, false, false, false,
        );
        assert!(heavy.fss_value > light.fss_value);
    }

    // ---------------------------------------------------------------
    // Cross-validation with Pioneer reference values
    // ---------------------------------------------------------------

    #[test]
    fn gas_giant_class_i_low_value() {
        let v = calculate_planet_value(
            &PlanetClass::SudarskyClassIGasGiant, 100.0, false, false, false, false,
        );
        // Gas giants have low scan value but are common
        assert!(v.fss_value > 3_000, "Gas giant should be >3k, got {}", v.fss_value);
        assert!(v.fss_value < 10_000, "Gas giant should be <10k, got {}", v.fss_value);
    }

    #[test]
    fn rocky_terraformable_is_valuable() {
        let v = calculate_planet_value(
            &PlanetClass::RockyBody, 0.5, true, true, true, true,
        );
        // Terraformable rocky body with full bonuses should be significant
        assert!(v.mapped_value > 1_000_000, "TF rocky mapped should be >1M, got {}", v.mapped_value);
    }
}
