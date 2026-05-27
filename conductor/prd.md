# UnixplorationBuddy — Product Requirements

> Created: 2026-05-27
> Status: Draft — first grill session

## Overview

UnixplorationBuddy is a standalone Rust TUI application for Linux commanders playing Elite Dangerous. It monitors journal files in real-time and displays exploration data — body hierarchy, scan states, calculated values, bio signals, and trip statistics — on a second monitor while the player explores.

## In Scope

### Phase 1 — Bodies + History

**Bodies View (core table):**
- Hierarchical body table: stars → planets → moons, ordered by name/orbital hierarchy
- Columns: Name, Type, Atmosphere, Distance (Ls), Scan/Map status icons, Calculated Value, Bio/Geo signal counts
- Expandable rows: press Enter on a body to reveal bio species predictions (Canonn data) with Vista Genomics prices
- Auto-updates in real-time as journal events arrive (FSDJump, Scan, FSSDiscoveryScan, SAAScanComplete, etc.)

**History View (trip statistics):**
- Current exploration trip: estimated total value, systems discovered, bodies scanned, first discoveries, bio signals/analysed
- Aggregated lifetime exploration history

**System Header (slim):**
- System name + discovered body count (e.g., "20 of 25") + total estimated value

**Value Calculation:**
- Self-contained Rust port of community-derived formulas (from Pioneer's body_calc.py)
- No API dependency for base values

**Data Source:**
- Primary: Elite Dangerous journal files read via `ed_journals` crate
- Journal watcher for live updates (second-monitor use case)

### Phase 2 — Exploration Enrichment & Navigation (Approved)

**Current System Exploration (Landing Tab Overhaul & Target Sync):**
- Standardised 70/30 horizontal split-pane:
  - **Left Pane:** Hierarchical Body Tree Table (Name, Type, Distance (Ls), Scan Icon, calculated Value, Bio/Geo counts, first mapping/discovery indicator).
  - **Right Pane:** Telemetry & Inspector Panel displaying the targeted/selected body's physical and exobiological conditions.
- **Target Sync:** Continuously watches `Status.json`'s `Destination` block to auto-focus and highlight the currently targeted body in-game.
- **Modular Column Settings Window:** Modular column rendering system enabling toggling columns (e.g. Gravity, Atmosphere, Temp, EDSM Discoverer) dynamically from a settings overlay (key: `s`).
- **Telemetry Details:** Surface gravity, temperature, and atmosphere type rendering for landable planets.

**Trip Codex (Logbook & Statistics):**
- Four fluid sub-views under the Exploration Trip tab (`Left/Right` arrow keys or `a/d` to toggle):
  - **Overview:** General trip stats (Systems visited, scanned, mapped, total value, total organic value, first discoveries).
  - **Stellar Codex:** Live tally count of every primary star class jumped to (e.g., Star Class G3 V, Neutron, Black Hole).
  - **Planetary Codex:** Live tally count of every planet class scanned (e.g., High metal content body, Earth-like World).
  - **Biological Codex:** Live tally count of every exobiology species successfully scanned/analysed on the trip.

**Biologist Module (Canonn Prediction Engine):**
- **Trigger Condition:** Predictions are evaluated **only** when the FSS discovery scan or DSS mapping reports biological signals count > 0, keeping cockpit HUD telemetry clean.
- **Offline Predictor:** Core prediction engine matching planet physical properties (atmosphere type, body class, surface gravity, surface temperature, primary star class) against the bundled Canonn exobiology distribution dataset (`conductor/canonn-data/`).
- **Vista Genomics Value Integration:** Displays predicted exobiological species, probability levels (High, Medium, Low), exobiology rewards, and exobiology genetic scan tracking via `ScanOrganic` journal events.

**Route Exploration (Plotted Nav Tracking):**
- **Plotted Route Sync:** Watches `NavRoute.json` to ingest plotted route waypoints automatically.
- **Scoopable Star Indicators:** Displays a warning fuel pump icon `⛽` next to scoopable star classes (O, B, A, F, G, K, M) along the path.
- **Asynchronous EDSM Streaming:** Utilizes a lightweight background thread with `ureq` (v3.3.0) to fetch EDSM data (estimated values, system completion status, valuable planets counts, first discoverer CMDR names) asynchronously, updating the TUI dynamically without blocking rendering.
- **Visual Status Row:** Table rendering jump index, system name, scoopable star status, EDSM exploration completion status, EDSM total value, EDSM CMDR discoverer, and visual icons (`💰` = valuable bodies count, `🌍` = terraformables count, `🚀` = landables count).

### Phase 3+ — Future

- EDASTRO / Spansh Route Planning API integration
- Interactive Galaxy Map TUI preview (ASCII/Unicode star clusters)
- Custom color themes & profile exporting

## Out of Scope

- Windows/macOS support (Linux-first, native standard Pop!_OS flatness)
- Overlay mode (no transparency/always-on-top — pure TUI designed for a second monitor)
- EDMC plugin mode (standalone Rust binary only)
- Real-time multiplayer wing data
- Galaxy Route planning algorithms (ingests planned route from game or Spansh files)

## Open Questions

- *Resolved:* Canonn exobiology prediction engine is run entirely offline by embedding the parsed data boundaries.
- *Resolved:* EDSM APIs are queried asynchronously via `ureq` in a background worker thread.
- *Resolved:* Target-syncing is fully realized by reading `Status.json`'s destination block.

