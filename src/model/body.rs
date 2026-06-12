use ed_journals::galaxy::{PlanetClass, StarClass};

/// Discovery progression of a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum ScanState {
    Unseen,
    #[allow(dead_code)]
    Honked,
    FSSScanned,
    DSSMapped,
}

impl ScanState {
    /// Icon character for display in the TUI.
    pub fn icon(&self) -> &'static str {
        match self {
            ScanState::Unseen => "○",
            ScanState::Honked => "◐",
            ScanState::FSSScanned => "●",
            ScanState::DSSMapped => "★",
        }
    }
}

impl Default for ScanState {
    fn default() -> Self {
        ScanState::Unseen
    }
}

/// Classification of a celestial object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BodyType {
    Star,
    Planet,
    Moon,
    BeltCluster,
    Unknown,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::Unknown
    }
}

/// A celestial object within a system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub body_id: u32,
    #[allow(dead_code)]
    pub name: String,
    /// Display name — just the suffix, e.g. "A 1 a".
    pub short_name: String,
    pub body_type: BodyType,
    pub atmosphere: Option<String>,
    pub distance_ls: Option<f64>,
    pub scan_state: ScanState,
    /// Earth masses for planets, solar masses for stars.
    pub mass: Option<f64>,
    pub terraformable: bool,
    pub bio_signals: u32,
    pub geo_signals: u32,
    pub calculated_value: u64,
    /// Credits from DSS mapping (0 if unmapped).
    pub mapped_value: u64,
    /// Already discovered by another commander.
    pub was_discovered: bool,
    /// Already mapped by another commander.
    pub was_mapped: bool,
    /// Parent body_id for hierarchy.
    pub parent_id: Option<u32>,
    /// Sortable key derived from body naming convention (see `model::naming`).
    pub sort_key: String,
    /// Raw planet class string from journal (e.g., "Earthlike body").
    pub planet_class: Option<String>,
    /// Raw star type string from journal (e.g., "B", "N", "DA").
    pub star_type: Option<String>,
    /// Typed planet class for value recalculation (avoids Display format mismatch).
    pub planet_class_enum: Option<PlanetClass>,
    /// Typed star class for value recalculation.
    pub star_class_enum: Option<StarClass>,
    /// Whether DSS mapping used optimal probes (efficiency bonus).
    pub probes_efficient: bool,
    /// Surface gravity (in m/s^2 or Gs).
    pub gravity: Option<f64>,
    /// Surface temperature (in Kelvin).
    pub temperature: Option<f64>,
    /// Planetary radius (in meters).
    pub radius: Option<f64>,
    /// Capable of planetary landing.
    pub landable: bool,
    /// Has planetary rings.
    pub ringed: bool,
    /// Biological genuses reported by SAA scan.
    pub bio_genuses: Vec<String>,
    /// Volcanism description string (e.g., "major silicate vapour geysers").
    pub volcanism: Option<String>,
    /// Surface pressure in atmospheres.
    pub pressure_atm: Option<f64>,
    /// Surface materials and their percentages.
    pub surface_materials: Vec<(String, f64)>,
    /// Atmosphere composition and percentages (e.g., Hydrogen 73.7%, Helium 26.2%).
    pub atmosphere_composition: Vec<(String, f64)>,
    // Keplerian orbital elements for TUI Orrery simulation
    pub semi_major_axis: Option<f64>,
    pub eccentricity: Option<f64>,
    pub inclination: Option<f64>,
    pub periapsis: Option<f64>,
    pub ascending_node: Option<f64>,
    pub orbital_period: Option<f64>,
    pub mean_anomaly: Option<f64>,
    /// Rotational period in seconds.
    pub rotational_period: Option<f64>,
    /// Axial tilt in radians.
    pub axial_tilt: Option<f64>,
    /// Whether the body is tidally locked.
    pub tidal_lock: bool,
}

impl Body {
    /// Create a new body with sensible defaults.
    /// Only `body_id` and `name` are required up front.
    pub fn new(body_id: u32, name: String) -> Self {
        Self {
            body_id,
            short_name: name.clone(),
            name,
            body_type: BodyType::default(),
            atmosphere: None,
            distance_ls: None,
            scan_state: ScanState::default(),
            mass: None,
            terraformable: false,
            bio_signals: 0,
            geo_signals: 0,
            calculated_value: 0,
            mapped_value: 0,
            was_discovered: false,
            was_mapped: false,
            parent_id: None,
            sort_key: String::new(),
            planet_class: None,
            star_type: None,
            planet_class_enum: None,
            star_class_enum: None,
            probes_efficient: false,
            gravity: None,
            temperature: None,
            radius: None,
            landable: false,
            ringed: false,
            bio_genuses: Vec::new(),
            volcanism: None,
            pressure_atm: None,
            surface_materials: Vec::new(),
            atmosphere_composition: Vec::new(),
            semi_major_axis: None,
            eccentricity: None,
            inclination: None,
            periapsis: None,
            ascending_node: None,
            orbital_period: None,
            mean_anomaly: None,
            rotational_period: None,
            axial_tilt: None,
            tidal_lock: false,
        }
    }
}
