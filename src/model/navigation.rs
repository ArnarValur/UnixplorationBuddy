//! Navigation models representing real-time cockpit status and plotted routes.

use serde::{Deserialize, Serialize};

/// Targeted destination within Status.json.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Destination {
    #[serde(rename = "System")]
    pub system: String,
    #[serde(rename = "Body")]
    pub body: u32,
    #[serde(rename = "Name")]
    pub name: String,
}

/// Real-time cockpit status snapshot from Status.json.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Status {
    pub timestamp: String,
    pub event: String,
    #[serde(rename = "Destination")]
    pub destination: Option<Destination>,
}

/// Plotted system waypoint within NavRoute.json.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RouteEntry {
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: u64,
    #[serde(rename = "StarPos")]
    pub star_pos: Vec<f64>,
    #[serde(rename = "StarClass")]
    pub star_class: String,
}

/// Plotted navigation route from NavRoute.json.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NavRoute {
    pub timestamp: String,
    pub event: String,
    #[serde(rename = "Route")]
    pub route: Vec<RouteEntry>,
}
