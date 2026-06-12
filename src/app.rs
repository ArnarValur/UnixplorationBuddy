//! Top-level application state for UnixplorationBuddy.

use std::collections::HashMap;

use crate::model::{Body, BodyHierarchy, System, Trip, NavRoute, Anomaly};

/// Which tab is currently active in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Bodies,
    History,
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
    Route,
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
    /// Selected row index in the stellar codex panel.
    pub selected_stellar_index: usize,
    /// Selected row index in the planetary codex panel.
    pub selected_planetary_index: usize,
    /// Whether the left panel is focused in the Stellar&Planetary codex view.
    pub codex_focus_left: bool,
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
    /// Sub-tab inside the Bodies view (System Map table vs Route).
    pub bodies_subtab: BodiesSubTab,
    /// Full star class string of the current system's primary star (e.g. "B9 VAB").
    /// Set on primary star Scan, cleared on FSDJump. Used to highlight the active row in the Stellar Codex.
    pub current_primary_star_class: Option<String>,
    /// Cache of visited systems' bodies for back-navigation.
    /// Keyed by system_address → (System, bodies HashMap).
    pub visited_systems: HashMap<u64, (System, HashMap<u32, Body>)>,
    /// Detected anomalies/POIs for bodies in the current system.
    /// Recomputed after each Scan event.
    pub anomalies: HashMap<u32, Vec<Anomaly>>,
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
            selected_stellar_index: 0,
            selected_planetary_index: 0,
            codex_focus_left: true,
            column_settings: ColumnSettings::default(),
            show_settings: false,
            show_inspector: false,
            last_latitude: None,
            last_longitude: None,
            last_heading: None,
            bodies_subtab: BodiesSubTab::default(),
            current_primary_star_class: None,
            visited_systems: HashMap::new(),
            anomalies: HashMap::new(),
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
            Tab::History => Tab::Bodies,
        };
    }

    /// Cycle to the next codex sub-tab in the History view.
    pub fn next_codex_tab(&mut self) {
        self.active_codex_tab = match self.active_codex_tab {
            CodexTab::Overview => CodexTab::Stellar,
            CodexTab::Stellar => CodexTab::Overview,
        };
    }

    /// Cycle to the previous codex sub-tab in the History view.
    pub fn prev_codex_tab(&mut self) {
        self.active_codex_tab = match self.active_codex_tab {
            CodexTab::Overview => CodexTab::Stellar,
            CodexTab::Stellar => CodexTab::Overview,
        };
    }

    /// Calculate the maximum number of rows in the biological codex.
    pub fn max_bio_codex_rows(&self) -> usize {
        self.trip.biological_codex.len()
    }

    /// Calculate the maximum number of rows in the stellar codex panel.
    /// Accounts for both primary and companion star classes.
    pub fn max_stellar_rows(&self) -> usize {
        let mut groups: HashMap<String, std::collections::HashSet<String>> = HashMap::new();

        // Collect subtypes from both primary and companion codices
        for subtype in self.trip.stellar_codex.keys().chain(self.trip.companion_stellar_codex.keys()) {
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
            groups.entry(main_class).or_default().insert(subtype.clone());
        }

        let mut stellar_rows = 0;
        for (main_class, subtypes) in groups {
            stellar_rows += 1; // main class row
            let has_redundant_single_child = subtypes.len() == 1 && subtypes.contains(&main_class);
            if !has_redundant_single_child {
                stellar_rows += subtypes.len();
            }
        }
        stellar_rows
    }

    /// Calculate the maximum number of rows in the planetary codex panel.
    /// Counts category headers + planet class rows + sub-attribute rows.
    pub fn max_planetary_rows(&self) -> usize {
        // Aggregate unique planet classes and their sub-attribute flags
        let mut class_flags: HashMap<String, (bool, bool, bool, bool, bool)> = HashMap::new();
        for key in self.trip.planetary_codex.keys() {
            let parts: Vec<&str> = key.split('|').collect();
            let planet_class = parts[0].to_string();
            let entry = class_flags.entry(planet_class).or_insert((false, false, false, false, false));
            if parts.contains(&"R") { entry.0 = true; }
            if parts.contains(&"T") { entry.1 = true; }
            if parts.contains(&"L") { entry.2 = true; }
            if parts.contains(&"B") { entry.3 = true; }
            if parts.contains(&"C") { entry.4 = true; }
        }

        let mut categories = std::collections::HashSet::new();
        let mut rows = 0usize;
        for (planet_class, (has_r, has_t, has_l, has_b, has_c)) in &class_flags {
            categories.insert(get_planet_category(planet_class));
            rows += 1; // planet class row
            // sub-attribute rows
            if *has_r { rows += 1; }
            if *has_t { rows += 1; }
            if *has_l { rows += 1; }
            if *has_b { rows += 1; }
            if *has_c { rows += 1; }
        }
        rows += categories.len(); // category header rows
        rows
    }

    /// Move selection down in the active codex.
    pub fn select_next_codex_row(&mut self) {
        match self.active_codex_tab {
            CodexTab::Overview => {
                let max = self.max_bio_codex_rows();
                if max > 0 {
                    self.selected_stellar_index = (self.selected_stellar_index + 1) % max;
                }
            }
            CodexTab::Stellar => {
                if self.codex_focus_left {
                    let max_s = self.max_stellar_rows();
                    if max_s > 0 {
                        self.selected_stellar_index = (self.selected_stellar_index + 1) % max_s;
                    }
                } else {
                    let max_p = self.max_planetary_rows();
                    if max_p > 0 {
                        self.selected_planetary_index = (self.selected_planetary_index + 1) % max_p;
                    }
                }
            }
        }
    }

    /// Move selection up in the active codex.
    pub fn select_previous_codex_row(&mut self) {
        match self.active_codex_tab {
            CodexTab::Overview => {
                let max = self.max_bio_codex_rows();
                if max > 0 {
                    self.selected_stellar_index = if self.selected_stellar_index == 0 {
                        max - 1
                    } else {
                        self.selected_stellar_index - 1
                    };
                }
            }
            CodexTab::Stellar => {
                if self.codex_focus_left {
                    let max_s = self.max_stellar_rows();
                    if max_s > 0 {
                        self.selected_stellar_index = if self.selected_stellar_index == 0 {
                            max_s - 1
                        } else {
                            self.selected_stellar_index - 1
                        };
                    }
                } else {
                    let max_p = self.max_planetary_rows();
                    if max_p > 0 {
                        self.selected_planetary_index = if self.selected_planetary_index == 0 {
                            max_p - 1
                        } else {
                            self.selected_planetary_index - 1
                        };
                    }
                }
            }
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
        assert_eq!(app.max_stellar_rows(), 0);

        // Populate mock stellar codex
        app.trip.stellar_codex.insert("F9 VAB".to_string(), 10);
        app.trip.stellar_codex.insert("F1 VA".to_string(), 5);
        app.trip.stellar_codex.insert("K".to_string(), 3);

        // F group has main F + 2 subtypes = 3 rows.
        // K group has main K (redundant single child) = 1 row.
        // Total stellar rows = 4.
        assert_eq!(app.max_stellar_rows(), 4);

        assert_eq!(app.selected_stellar_index, 0);
        app.select_next_codex_row();
        assert_eq!(app.selected_stellar_index, 1);
        app.select_previous_codex_row();
        assert_eq!(app.selected_stellar_index, 0);
        app.select_previous_codex_row();
        assert_eq!(app.selected_stellar_index, 3);
    }

    #[test]
    fn test_planetary_codex_rows_and_categories() {
        assert_eq!(get_planet_category("Earth-like World"), "Rare Worlds");
        assert_eq!(get_planet_category("High metal content body"), "Terrestrial Worlds");
        assert_eq!(get_planet_category("Sudarsky Class III gas giant"), "Gas Giants");

        let mut app = App::new();
        app.active_codex_tab = CodexTab::Stellar;
        assert_eq!(app.max_planetary_rows(), 0);

        // Ingest mock planetary codex data with sub-attribute flags encoded in keys
        app.trip.planetary_codex.insert("High metal content body|L".to_string(), 3);
        app.trip.planetary_codex.insert("High metal content body|L|T|R".to_string(), 1);
        app.trip.planetary_codex.insert("Earth-like World|L".to_string(), 2);
        app.trip.planetary_codex.insert("Rocky body|L".to_string(), 5);

        // Categories: "Rare Worlds", "Terrestrial Worlds" = 2 header rows
        // HMC: 1 class row + Ringed(1) + Terraformable(1) + Landable(1) = 4 rows
        // Earth-like: 1 class row + Landable(1) = 2 rows
        // Rocky body: 1 class row + Landable(1) = 2 rows
        // Total = 2 + 4 + 2 + 2 = 10
        assert_eq!(app.max_planetary_rows(), 10);
    }

    #[test]
    fn test_bodies_subtab_defaults() {
        let app = App::new();
        assert_eq!(app.bodies_subtab, BodiesSubTab::Table);
    }
}
