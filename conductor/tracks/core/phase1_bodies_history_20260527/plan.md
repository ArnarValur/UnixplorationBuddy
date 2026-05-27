# Implementation Plan: Phase 1 — Bodies + History

> **Track:** `phase1_bodies_history_20260527`
> **Workflow:** Light (plan → execute → verify)
> **Key dependency:** `ed-journals = "0.12.4"` (beta — pin version)

---

## Phase 1: Project Bootstrap

- [x] Task: Initialize Rust project `342e5c3`
    - [x] `cargo init` in repo root
    - [x] Add dependencies to `Cargo.toml`: `ratatui`, `crossterm`, `ed-journals`, `serde`, `serde_json`, `chrono`, `dirs`
    - [x] Verify `cargo check` passes
    - [x] Update `.gitignore` for Rust (`/target/`, etc.)

- [x] Task: Define core data model `342e5c3`
    - [x] `System` struct: name, system_address, body count (discovered/total), total value
    - [x] `Body` struct: body_id, name, body_type, atmosphere, distance_ls, scan_state, mass, terraformability, bio_signals, geo_signals, calculated_value, was_discovered, was_mapped
    - [x] `ScanState` enum: Unseen, Honked, FSSScanned, DSSMapped
    - [x] `BodyHierarchy` tree structure: parent→child relationships derived from naming convention
    - [x] `Trip` struct: systems_visited, bodies_scanned_fss, bodies_mapped_dss, first_discoveries, first_mappings, bio_detected, bio_analysed, total_value

- [x] Task: Set up application skeleton `342e5c3`
    - [x] Main loop: terminal init → event loop → cleanup
    - [x] `App` struct holding System, Bodies, Trip, active tab
    - [x] Basic Ratatui setup with crossterm backend
    - [x] Graceful exit on `q` / `Ctrl+C`

---

## Phase 2: Journal Ingestion

- [x] Task: Journal directory discovery `40dd102`
    - [x] CLI argument `--journal-path <dir>` for explicit path
    - [x] Default path: Steam Proton journal directory
    - [x] Validate directory exists and contains journal files
    - [x] Clear error message if path not found

- [x] Task: Full session replay `40dd102`
    - [x] Use `ed-journals` `LogDir` + `LogFile` + blocking reader to parse journal log files
    - [x] Parse all journal files in chronological order (oldest first)
    - [x] Process relevant events: `FSDJump`, `Location`, `Scan`, `FSSDiscoveryScan`, `FSSBodySignals`, `SAAScanComplete`, `SAASignalsFound`
    - [x] Built own state layer on top of raw events (ed-journals state module not used — too opaque for our needs)
    - [x] Populate System and Body state from replayed events

- [x] Task: Live journal watcher `40dd102`
    - [x] Use `ed-journals` `LiveLogDirReader` for real-time event delivery
    - [x] Handle journal file rotation (LiveLogDirReader handles this internally)
    - [x] Feed new events into same `process_event` pipeline as replay
    - [x] Trigger TUI re-render on state change (mpsc channel + 250ms poll)

---

## Phase 2.5: Foundation Hardening `75b7ace`

- [x] Task: Separate replay from trip accumulation
    - [x] Add `track_trip: bool` parameter to `process_event()`
    - [x] Guard all trip mutations behind `if track_trip`
    - [x] Replay passes `false` (state only), live passes `true`
    - [x] Trip starts at 0 on launch (persistence in Phase 5)

- [x] Task: process_event() test suite (22 tests)
    - [x] Real journal JSON fixtures parsed via serde_json
    - [x] All 7 event handlers tested (FSDJump, Location, FSSDiscoveryScan, Scan, FSSBodySignals, SAAScanComplete, SAASignalsFound)
    - [x] State transition tests (scan upgrade, system reset, body creation)
    - [x] Trip gating tests (replay vs live mode)
    - [x] Edge cases (unknown bodies, duplicate scans, same-system Location)

- [x] Task: TUI rendering tests (4 tests)
    - [x] Ratatui TestBackend pattern established
    - [x] Header renders system name + body count
    - [x] Empty state placeholder
    - [x] History tab renders trip stats
    - [x] Bodies tab renders body types + scan state icons

---

## Phase 3: Body Hierarchy & Value Calculation

- [x] Task: Naming convention parser for Body Hierarchy `c21d7bd`
    - [x] Parse Elite's body naming scheme to extract hierarchy
    - [x] Handle star designations (A, B, C, ABC, CDE)
    - [x] Handle planet numbering (1, 2, 3...)
    - [x] Handle moon lettering (a, b, c...) and sub-moons
    - [x] Handle belt clusters as root-level entries
    - [x] Fallback to flat list for unparseable names
    - [x] Test with real journal data (19 tests)
    - [x] Integrate: hierarchy sorts by sort_key instead of body_id

- [/] Task: Own value calculation (Pioneer port, replaces ed-journals' incomplete version) `a1ffbcb`
    - [x] Port Pioneer's body_calc.py formulas to Rust (src/model/valuation.rs)
    - [x] Star values: all star classes, mass scaling, first discovery
    - [x] Planet values: all planet classes, terraform bonus, mass scaling
    - [x] First discovery (2.6x) and first mapping modifiers
    - [x] Odyssey/4.0+ mapping bonus (always on)
    - [x] Efficiency bonus (1.25x when probes_used <= efficiency_target)
    - [x] 14 value calculation tests
    - [x] Integration: calculated_value + mapped_value set on Scan events
    - [x] Wire SAAScanComplete efficiency tracking (probes_efficient flag) `0004381`
    - [x] Total system value aggregation from individual Body Values `0004381`

---

## Phase 4: TUI Rendering `bf5dc44`

- [x] Task: System Header widget
    - [x] Slim single-line top bar
    - [x] Display: system name, "N of M bodies", total system value
    - [x] Elite orange-on-black styling
    - [x] Updates on FSDJump and body scan events

- [x] Task: Bodies View tab
    - [x] Hierarchical table with indentation for parent→child Bodies
    - [x] Columns: Name, Type, Atmosphere, Distance (Ls), Scan State icons, Value (cr), Bio count, Geo count
    - [x] Scan State rendered as icons/symbols (e.g., ○ unseen, ◐ honked, ● FSS, ★ DSS)
    - [x] Scrollable table for large systems (StatefulWidget + Scrollbar)
    - [x] Highlight selected row (keyboard navigation: ↑/↓)
    - [x] Color-coded body types (stars=yellow, planets=blue, moons=grey)
    - [x] "No bodies discovered" placeholder for empty systems
    - [x] First discovery/mapping indicators (◆/◇)
    - [x] Value column shows mapped_value for DSS'd bodies

- [x] Task: History View tab
    - [x] Trip statistics display: systems, bodies FSS'd, bodies DSS'd, first discoveries, first mappings, bio counts, total value
    - [x] Clean layout with labeled stat rows
    - [x] Same color scheme as Bodies View
    - [x] First discovery/mapping stat highlights

- [x] Task: Tab navigation
    - [x] Tab key or 1/2 to switch between Bodies and History
    - [x] Visual tab indicator (▸ prefix + number key hints)
    - [x] System Header persists across both tabs
    - [x] Context-aware status bar keybindings per tab

---

## Phase 5: Trip Persistence

- [/] Task: Trip file I/O
    - [ ] Serialize Trip to JSON at `~/.local/share/unixploration-buddy/trip.json`
    - [ ] Create directory if it doesn't exist (XDG compliant via `dirs` crate)
    - [ ] Load Trip on startup (if file exists)
    - [ ] Save Trip on each state change (debounced — at most once per second)
    - [ ] Handle malformed trip file gracefully (fresh trip + warning)

- [ ] Task: Manual trip reset
    - [ ] Keybinding (e.g., `r` with confirmation prompt) to reset Trip stats
    - [ ] Clear all counters, save empty trip to disk
    - [ ] Display confirmation in status bar

- [ ] Task: Trip accumulation from journal events
    - [ ] Increment systems_visited on `FSDJump`
    - [ ] Increment bodies_scanned on `Scan` (FSS) / `SAAScanComplete` (DSS)
    - [ ] Track first discoveries via `Scan` event `was_discovered` field
    - [ ] Track first mappings via `SAAScanComplete` `was_mapped` field
    - [ ] Track bio signals from `FSSBodySignals` / `SAASignalsFound`
    - [ ] Accumulate total value from Body Value calculations

---

## Phase 6: Integration & Polish

- [ ] Task: End-to-end integration testing
    - [ ] Test with real journal files from player's journal directory
    - [ ] Verify body hierarchy for 3+ known systems
    - [ ] Verify value calculations match Pioneer for 3+ known systems
    - [ ] Test tab switching, scrolling, keyboard navigation
    - [ ] Test startup with no journal files (error handling)
    - [ ] Test startup with corrupted trip file

- [ ] Task: Terminal compatibility verification
    - [ ] Test in kitty
    - [ ] Test in alacritty
    - [ ] Test in gnome-terminal
    - [ ] Fix any rendering issues (color fallback, unicode support)

- [ ] Task: Polish & UX
    - [ ] Loading indicator during session replay
    - [ ] Status bar with current activity (watching journal, last event time)
    - [ ] Keybinding help (press `?` for help overlay or footer hint)
    - [ ] README.md with installation and usage instructions
