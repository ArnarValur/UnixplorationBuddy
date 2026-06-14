# Specification — Phase 2 Exploration Enrichment & Navigation

## Overview
This track delivers the exobiology prediction engine, real-time cockpit target syncing, trip codex registries, and plotted route navigation with background EDSM streaming.

## Functional Requirements
1. **Current System Split-Pane:**
   - Implement a horizontal 70/30 layout under standard viewports (width >= 110 characters).
   - Left Pane renders the hierarchical system bodies table.
   - Right Pane renders the selected body's exobiology predictions, EDSM discoverer, and physical telemetry (gravity, temp, atmo type for landables).
   - Collapses to a 100% full-width tree view on small viewports with a manual `i` toggle override.
2. **Target Sync:**
   - Parse `Status.json`'s destination body ID in real-time during live loops to automatically set the TUI selected index.
3. **Trip Codex Registry:**
   - Add Stellar (primary star class tally), Planetary (scanned planet class tally), and Biological (exobiology completed scans tally) Codex sub-tabs.
4. **Biologist Module (Offline Predictor):**
   - Match planet class, atmosphere, gravity, temp, and star class against a build-time strongly-typed static Canonn dataset module (`src/model/biology/dataset.rs`).
   - Only compute and display predictions if bio signals count > 0.
5. **Route Navigation:**
   - Monitor `NavRoute.json` to extract planned system names and star classes.
   - Flag scoopable stars using a green fuel pump icon `⛽`.
   - Stream EDSM value, discoverer, and planetary stats (valuable, terraformable, landable counts) asynchronously via `ureq` on a sequential worker thread with 200ms sleep debouncing and session caching.

## Acceptance Criteria
- TUI dynamically collapses Right Pane when width < 110 columns.
- Cockpit target changes auto-scroll and select the correct body in the table.
- Trip sub-views toggle smoothly via Arrow keys and update correctly when jumps/scans occur.
- Exobiology predictions show VG credit values and Canonn probability match boundaries.
- Plotted routes in Galaxy Map render automatically with `⛽` markers and load EDSM data in the background without frame drops.
