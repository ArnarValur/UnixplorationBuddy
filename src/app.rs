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
    Overview,
    Stellar,
    Planetary,
    Biological,
}

impl Default for CodexTab {
    fn default() -> Self {
        CodexTab::Overview
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
    /// Modular columns visibility toggles.
    pub column_settings: ColumnSettings,
    /// Whether the Settings overlay is currently visible.
    pub show_settings: bool,
    /// Force display Right Pane Inspector even on small viewports.
    pub show_inspector: bool,
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
            column_settings: ColumnSettings::default(),
            show_settings: false,
            show_inspector: false,
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
        self.active_codex_tab = match self.active_codex_tab {
            CodexTab::Overview => CodexTab::Stellar,
            CodexTab::Stellar => CodexTab::Planetary,
            CodexTab::Planetary => CodexTab::Biological,
            CodexTab::Biological => CodexTab::Overview,
        };
    }

    /// Cycle to the previous codex sub-tab in the History view.
    pub fn prev_codex_tab(&mut self) {
        self.active_codex_tab = match self.active_codex_tab {
            CodexTab::Overview => CodexTab::Biological,
            CodexTab::Stellar => CodexTab::Overview,
            CodexTab::Planetary => CodexTab::Stellar,
            CodexTab::Biological => CodexTab::Planetary,
        };
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
        assert_eq!(app.active_codex_tab, CodexTab::Planetary);

        app.next_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Biological);

        app.next_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Overview);

        app.prev_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Biological);

        app.prev_codex_tab();
        assert_eq!(app.active_codex_tab, CodexTab::Planetary);
    }
}
