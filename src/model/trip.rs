use serde::{Deserialize, Serialize};

/// Position telemetry of a taken exobiology genetic sample.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrganicSampleLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub heading: Option<f64>,
}

/// Accumulated exploration statistics for a trip.
/// Persisted to disk as JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Trip {
    pub systems_visited: u32,
    pub bodies_scanned_fss: u32,
    pub bodies_mapped_dss: u32,
    pub first_discoveries: u32,
    pub first_mappings: u32,
    pub bio_detected: u32,
    pub bio_analysed: u32,
    pub total_value: u64,
    #[serde(default)]
    pub stellar_codex: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub companion_stellar_codex: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub planetary_codex: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub biological_codex: std::collections::HashMap<String, u32>,
    /// Maps \"systemAddress_bodyId\" to a list of successfully analysed species names on that body.
    #[serde(default)]
    pub organic_scans: std::collections::HashMap<String, Vec<String>>,
    /// Maps \"systemAddress_bodyId_speciesName\" to the current stage (1, 2, or 3).
    #[serde(default)]
    pub organic_progress: std::collections::HashMap<String, u8>,
    /// Maps \"systemAddress_bodyId_speciesName\" to recorded sample locations.
    #[serde(default)]
    pub organic_locations: std::collections::HashMap<String, Vec<OrganicSampleLocation>>,
}

impl Trip {
    /// Reset all statistics to zero.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
