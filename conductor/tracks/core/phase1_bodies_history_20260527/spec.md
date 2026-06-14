# Track Spec: Phase 1 — Bodies + History

> **Track:** `phase1_bodies_history_20260527`
> **Type:** feature
> **Domain:** core
> **PRD Anchor:** Phase 1 — Bodies + History

---

## Overview

Deliver the foundational UnixplorationBuddy experience: a Rust/Ratatui TUI that reads Elite Dangerous journal files, reconstructs the current System's Body Hierarchy, calculates Body Values inline, and presents two tab-based views — **Bodies** (hierarchical body table) and **History** (persistent Trip statistics) — with a slim System Header.

This is the first runnable version of UnixplorationBuddy. It must feel complete enough to be useful on a second monitor during an exploration session.

---

## Functional Requirements

### Journal Ingestion (ADR-0002)

- On startup, discover the journal directory (default: Steam Proton path, configurable via CLI arg or env var)
- Perform **full session replay**: parse all journal files from the current game session to reconstruct System and Body state
- Detect current session boundary via `Fileheader` / `LoadGame` Journal Events
- After replay, watch the active journal file for new Journal Events in real-time (file watcher)
- When a new journal file appears (game restart), switch to watching it
- Relevant Journal Events: `FSDJump`, `Location`, `Scan`, `FSSDiscoveryScan`, `FSSBodySignals`, `SAAScanComplete`, `SAASignalsFound`, `SellExplorationData`, `MultiSellExplorationData`

### Bodies View

- **Hierarchical body table** displaying all Bodies in the current System
- **Columns:** Name, Type, Atmosphere, Distance (Ls), Scan State icons, Body Value (cr), Bio signal count, Geo signal count
- **Body Hierarchy** derived via naming convention parser: parse Elite's naming scheme (e.g., "Swoilz AA-A d1 A 1 a" → star A → planet 1 → moon a)
  - Stars at root level
  - Planets indented under their parent star
  - Moons indented under their parent planet
  - Belt clusters handled as special root-level entries
- **Scan State icons:** Visual indicators for unseen / honked / FSS scanned / DSS mapped
- **Body Value:** Inline calculated on each `Scan` / `SAAScanComplete` event arrival (ADR-0003). Uses `ed-journals` crate's `exploration` module, validated against Pioneer's known values
- **Bio/Geo signal counts** from `FSSBodySignals` / `SAASignalsFound` events
- **Expandable rows:** Press Enter on a body with Bio Signals to reveal species predictions (Canonn data) with Vista Genomics Prices — *Phase 1 displays signal counts only; species prediction engine is Phase 3 scope*
- **Auto-update** in real-time as Journal Events arrive

### History View

- **Trip statistics** for the current persistent Trip:
  - Systems visited (count)
  - Bodies scanned (FSS + DSS counts)
  - First discoveries (count)
  - First mappings (count)
  - Bio signals detected / analysed (counts)
  - Estimated total Body Value accumulated
- **Persistent Trip:** Trip state saved to a local JSON file. Survives app restarts. User manually resets via keybinding (e.g., when selling data at a station)
- Trip file location: `~/.local/share/unixploration-buddy/trip.json` (XDG compliant)

### System Header

- **Slim single-line header** at top of both tabs
- Displays: System name, discovered body count (e.g., "20 of 25"), total estimated System value
- Updates on `FSDJump` (new system) and as Bodies are scanned

### TUI Layout

- **Tab-based navigation:** Bodies tab and History tab, full-screen each
- Switch tabs via keyboard shortcut (e.g., Tab key or 1/2 number keys)
- System Header persists across both tabs (top bar)
- Elite orange-on-black aesthetic (Ratatui styled with 24-bit color)
- Quit via `q` or `Ctrl+C`

---

## Non-Functional Requirements

- **Startup time:** < 2 seconds including full session replay (typical session = ~50 journal files, ~10k events)
- **Render latency:** < 50ms from Journal Event to display update
- **Memory:** < 50 MB RSS for a system with 100 bodies
- **Binary size:** Single static binary, reasonable size (< 20 MB release build)
- **Terminal compatibility:** Must work in kitty, alacritty, gnome-terminal (the three terminals in project-context.md debugging protocol)

---

## Acceptance Criteria

1. `cargo build --release` produces a single binary
2. Running the binary with `--journal-path <dir>` parses journal files and displays the Bodies table
3. Bodies are correctly hierarchically nested (star → planet → moon) via naming convention
4. Body Values match Pioneer's calculations for known test systems (within 1% rounding tolerance)
5. Tab switching between Bodies and History works via keyboard
6. Trip statistics accumulate correctly across journal replay
7. Trip persists to disk and survives app restart
8. Manual trip reset keybinding clears accumulated stats
9. Real-time updates work: scanning a body in-game updates the TUI within 1 second
10. App exits cleanly on `q` / `Ctrl+C`

---

## Edge Cases & Constraints

- **Empty system:** Display "No bodies discovered" in the Bodies table
- **Honk-only bodies:** Bodies known from `FSSDiscoveryScan` have minimal data (name, body count) — display with "—" for missing columns
- **Name parsing edge cases:** Procedurally generated names, systems with barycenters, binary star naming (A/B/C). Must handle gracefully — fallback to flat list if hierarchy can't be derived
- **Journal path not found:** Clear error message with expected path and how to configure
- **Multiple game sessions in journals:** Must correctly identify the *current* session boundary
- **Large systems (100+ bodies):** Table must scroll, not truncate
- **Trip file corruption:** If `trip.json` is malformed, start a fresh trip with a warning

---

## Dependencies

- `ed_journals` crate — journal file reading, event parsing, file watching
- `ratatui` + `crossterm` — TUI rendering
- No external tracks — this is the first track

---

## Out of Scope (explicitly NOT Phase 1)

- Species prediction / Canonn data integration (Phase 3 — PRD)
- EDSM enrichment / first discoverer status (Phase 2 — PRD)
- Gravity column (Phase 2 — PRD)
- Sortable columns (Phase 2 — PRD)
- Route view (Phase 2 — PRD)
- Configurable themes (Phase 3+ — PRD)
- Expandable bio species rows with predictions (Phase 3 — bodies show signal *counts* only in Phase 1)
- Windows/macOS support (PRD out of scope)
