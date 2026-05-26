/// Discovery progression of a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanState {
    Unseen,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone)]
pub struct Body {
    pub body_id: u32,
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
    /// Already discovered by another commander.
    pub was_discovered: bool,
    /// Already mapped by another commander.
    pub was_mapped: bool,
    /// Parent body_id for hierarchy.
    pub parent_id: Option<u32>,
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
            was_discovered: false,
            was_mapped: false,
            parent_id: None,
        }
    }
}
