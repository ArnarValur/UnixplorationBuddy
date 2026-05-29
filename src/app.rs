//! Top-level application state for UnixplorationBuddy.

use std::collections::HashMap;

use crate::model::{Body, BodyHierarchy, System, Trip, NavRoute};

/// Which tab is currently active in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Bodies,
    History,
    Route,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Bodies
    }
}

/// Which sub-tab in the Trip view is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexTab {
    Overview, // conjoined Overview + Biological
    Stellar,  // conjoined Stellar + Planetary
}

impl Default for CodexTab {
    fn default() -> Self {
        CodexTab::Overview
    }
}

/// Which sub-tab in the Bodies view is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodiesSubTab {
    Table,
    Orrery,
}

impl Default for BodiesSubTab {
    fn default() -> Self {
        BodiesSubTab::Table
    }
}

/// Dynamic column rendering settings toggled via the settings overlay.
#[derive(Debug, Clone)]
pub struct ColumnSettings {
    pub show_atmosphere: bool,
    pub show_gravity: bool,
    pub show_temperature: bool,
    pub show_discoverer: bool,
}

impl Default for ColumnSettings {
    fn default() -> Self {
        Self {
            show_atmosphere: true,
            show_gravity: true,
            show_temperature: true,
            show_discoverer: true,
        }
    }
}

/// Cached system exploration data fetched from the EDSM API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EdsmSystemData {
    pub name: String,
    pub estimated_value: u64,
    pub estimated_value_mapped: u64,
    pub discoverer: Option<String>,
    pub valuable_bodies: usize,
    pub terraformable_bodies: usize,
    pub landable_bodies: usize,
}

/// Top-level application state.
pub struct App {
    /// Current star system.
    pub system: System,
    /// All known bodies in the current system, keyed by body_id.
    pub bodies: HashMap<u32, Body>,
    /// Cumulative trip statistics.
    pub trip: Trip,
    /// The currently visible tab.
    pub active_tab: Tab,
    /// Set to true to exit the main loop.
    pub should_quit: bool,
    /// Index into `body_display_order` for the selected row.
    pub selected_body_index: usize,
    /// Cached display order: `(body_id, depth)` pairs derived from the body hierarchy.
    pub body_display_order: Vec<(u32, u32)>,
    /// Status message displayed in the footer bar.
    pub status_message: Option<String>,
    /// Whether the help overlay is currently visible.
    pub show_help: bool,
    /// Current targeted body ID from Status.json (Target Sync).
    pub targeted_body_id: Option<u32>,
    /// Plotted route waypoints from NavRoute.json.
    pub plotted_route: Option<NavRoute>,
    /// EDSM systems data cache.
    pub edsm_cache: HashMap<String, EdsmSystemData>,
    /// Active sub-tab in Trip tab.
    pub active_codex_tab: CodexTab,
    /// Selected row index in the active codex tab.
    pub selected_codex_index: usize,
    /// Modular columns visibility toggles.
    pub column_settings: ColumnSettings,
    /// Whether the Settings overlay is currently visible.
    pub show_settings: bool,
    /// Force display Right Pane Inspector even on small viewports.
    pub show_inspector: bool,
    /// Real-time planetary latitude from Status.json.
    pub last_latitude: Option<f64>,
    /// Real-time planetary longitude from Status.json.
    pub last_longitude: Option<f64>,
    /// Real-time planetary heading from Status.json.
    pub last_heading: Option<f64>,
    /// Sub-tab inside the Bodies view (System Map table vs Orrery canvas).
    pub bodies_subtab: BodiesSubTab,
    /// Simulated time value driving the Keplerian Orrery.
    pub sim_time: f64,
    /// Simulated time progression speed multiplier.
    pub sim_speed: f64,
}

impl App {
    /// Create a new `App` with empty defaults.
    pub fn new() -> Self {
        Self {
            system: System::default(),
            bodies: HashMap::new(),
            trip: Trip::default(),
            active_tab: Tab::default(),
            should_quit: false,
            selected_body_index: 0,
            body_display_order: Vec::new(),
            status_message: None,
            show_help: false,
            targeted_body_id: None,
            plotted_route: None,
            edsm_cache: HashMap::new(),
            active_codex_tab: CodexTab::default(),
            selected_codex_index: 0,
            column_settings: ColumnSettings::default(),
            show_settings: false,
            show_inspector: false,
            last_latitude: None,
            last_longitude: None,
            last_heading: None,
            bodies_subtab: BodiesSubTab::default(),
            sim_time: 0.0,
            sim_speed: 1.0,
        }
    }

    /// Signal the application to exit.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Cycle to the next tab.
    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Bodies => Tab::History,
            Tab::History => Tab::Route,
            Tab::Route => Tab::Bodies,
        };
    }

    /// Cycle to the next codex sub-tab in the History view.
    pub fn next_codex_tab(&mut self) {
        self.selected_codex_index = 0;
        self.active_codex_tab = match self.active_codex_tab {
            CodexTab::Overview => CodexTab::Stellar,
            CodexTab::Stellar => CodexTab::Overview,
        };
    }

    /// Cycle to the previous codex sub-tab in the History view.
    pub fn prev_codex_tab(&mut self) {
        self.selected_codex_index = 0;
        self.active_codex_tab = match self.active_codex_tab {
            CodexTab::Overview => CodexTab::Stellar,
            CodexTab::Stellar => CodexTab::Overview,
        };
    }

    /// Calculate the maximum number of rows in the active codex tab.
    pub fn max_codex_rows(&self) -> usize {
        match self.active_codex_tab {
            CodexTab::Overview => self.trip.biological_codex.len(),
            CodexTab::Stellar => {
                let mut groups: HashMap<String, Vec<String>> = HashMap::new();
                for subtype in self.trip.stellar_codex.keys() {
                    let mut main_class = String::new();
                    for c in subtype.chars() {
                        if c.is_ascii_digit() || c == ' ' {
                            break;
                        }
                        main_class.push(c);
                    }
                    if main_class.is_empty() {
                        main_class = subtype.clone();
                    }
                    groups.entry(main_class).or_insert_with(Vec::new).push(subtype.clone());
                }
                
                let mut stellar_rows = 0;
                for (main_class, subtypes) in groups {
                    stellar_rows += 1; // main class row
                    let has_redundant_single_child = subtypes.len() == 1 && subtypes[0] == main_class;
                    if !has_redundant_single_child {
                        stellar_rows += subtypes.len();
                    }
                }

                let mut unique_classes = std::collections::HashSet::new();
                for key in self.trip.planetary_codex.keys() {
                    let parts: Vec<&str> = key.split('|').collect();
                    unique_classes.insert(parts[0].to_string());
                }

                let mut categories = std::collections::HashSet::new();
                for planet_class in &unique_classes {
                    categories.insert(get_planet_category(planet_class));
                }

                let planetary_rows = unique_classes.len() + categories.len();
                std::cmp::max(stellar_rows, planetary_rows)
            }
        }
    }

    /// Move selection down in the active codex.
    pub fn select_next_codex_row(&mut self) {
        let max_rows = self.max_codex_rows();
        if max_rows > 0 {
            self.selected_codex_index = (self.selected_codex_index + 1) % max_rows;
        }
    }

    /// Move selection up in the active codex.
    pub fn select_previous_codex_row(&mut self) {
        let max_rows = self.max_codex_rows();
        if max_rows > 0 {
            self.selected_codex_index = if self.selected_codex_index == 0 {
                max_rows - 1
            } else {
                self.selected_codex_index - 1
            };
        }
    }

    /// Move selection down in the body list.
    pub fn select_next_body(&mut self) {
        if !self.body_display_order.is_empty() {
            self.selected_body_index =
                (self.selected_body_index + 1) % self.body_display_order.len();
        }
    }

    /// Move selection up in the body list.
    pub fn select_previous_body(&mut self) {
        if !self.body_display_order.is_empty() {
            self.selected_body_index = self
                .selected_body_index
                .checked_sub(1)
                .unwrap_or(self.body_display_order.len() - 1);
        }
    }

    /// Rebuild `body_display_order` from the current bodies using [`BodyHierarchy`].
    pub fn rebuild_display_order(&mut self) {
        let bodies: Vec<Body> = self.bodies.values().cloned().collect();
        let hierarchy = BodyHierarchy::build(&bodies);
        self.body_display_order = hierarchy.display_order();

        // Target Sync: Auto-focus the targeted body if it exists in the hierarchy
        if let Some(body_id) = self.targeted_body_id {
            if let Some(pos) = self.body_display_order.iter().position(|&(id, _)| id == body_id) {
                self.selected_body_index = pos;
                return;
            }
        }

        // Clamp selection to valid range.
        if self.body_display_order.is_empty() {
            self.selected_body_index = 0;
        } else if self.selected_body_index >= self.body_display_order.len() {
            self.selected_body_index = self.body_display_order.len() - 1;
        }
    }
}

/// Helper to classify a planet class into a premium category.
pub fn get_planet_category(planet_class: &str) -> &'static str {
    let lower = planet_class.to_lowercase();
    if lower.contains("earth-like") || lower.contains("ammonia world") || lower.contains("water world") {
        "Rare Worlds"
    } else if lower.contains("gas giant") || lower.contains("water giant") {
        "Gas Giants"
    } else {
        "Terrestrial Worlds"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_tab_cycling() {
        let mut app = App::new();
        assert_eq!(app.active_codex_tab, CodexTab::Overview);

        app.next_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Stellar);

        app.next_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Overview);

        app.prev_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Stellar);

        app.prev_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Overview);
    }

    #[test]
    fn test_codex_row_selection() {
        let mut app = App::new();
        app.active_codex_tab = CodexTab::Stellar;
        assert_eq!(app.max_codex_rows(), 0);

        // Populate mock stellar codex
        app.trip.stellar_codex.insert("F9 VAB".to_string(), 10);
        app.trip.stellar_codex.insert("F1 VA".to_string(), 5);
        app.trip.stellar_codex.insert("K".to_string(), 3);

        // F group has main F + 2 subtypes = 3 rows.
        // K group has main K (redundant single child) = 1 row.
        // Total rows = 4.
        assert_eq!(app.max_codex_rows(), 4);

        assert_eq!(app.selected_codex_index, 0);
        app.select_next_codex_row();
        assert_eq!(app.selected_codex_index, 1);
        app.select_previous_codex_row();
        assert_eq!(app.selected_codex_index, 0);
        app.select_previous_codex_row();
        assert_eq!(app.selected_codex_index, 3);
    }

    #[test]
    fn test_planetary_codex_rows_and_categories() {
        assert_eq!(get_planet_category("Earth-like World"), "Rare Worlds");
        assert_eq!(get_planet_category("High metal content body"), "Terrestrial Worlds");
        assert_eq!(get_planet_category("Sudarsky Class III gas giant"), "Gas Giants");

        let mut app = App::new();
        app.active_codex_tab = CodexTab::Stellar;
        assert_eq!(app.max_codex_rows(), 0);

        // Ingest mock planetary codex data with sub-attribute flags encoded in keys
        app.trip.planetary_codex.insert("High metal content body|L".to_string(), 3);
        app.trip.planetary_codex.insert("High metal content body|L|T|R".to_string(), 1);
        app.trip.planetary_codex.insert("Earth-like World|L".to_string(), 2);
        app.trip.planetary_codex.insert("Rocky body|L".to_string(), 5);

        // Active categories: "Rare Worlds", "Terrestrial Worlds" (2 categories)
        // Unique planet classes: "High metal content body", "Earth-like World", "Rocky body" (3 classes)
        // Total rows = 2 + 3 = 5
        assert_eq!(app.max_codex_rows(), 5);
    }

    #[test]
    fn test_orrery_subtab_defaults() {
        let app = App::new();
        assert_eq!(app.bodies_subtab, BodiesSubTab::Table);
        assert_eq!(app.sim_time, 0.0);
        assert_eq!(app.sim_speed, 1.0);
    }
}
