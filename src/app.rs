//! Top-level application state for UnixplorationBuddy.

use std::collections::HashMap;

use crate::model::{Body, BodyHierarchy, System, Trip};

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

        // Clamp selection to valid range.
        if self.body_display_order.is_empty() {
            self.selected_body_index = 0;
        } else if self.selected_body_index >= self.body_display_order.len() {
            self.selected_body_index = self.body_display_order.len() - 1;
        }
    }
}
