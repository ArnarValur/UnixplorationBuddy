# Pulse — Current Project State

**Last Updated:** 2026-05-29 19:56
**Session Focus:** Expanded right TUI pane to 40%, sorted predicted exobiology species alphabetically, and integrated minimum colonial separation distances.

## 🚀 Active Tracks

_None._

## ✅ Recently Completed

- **Exobiology Telemetry & Layout Refinements (2026-05-29):** Expanded the TUI right-pane inspector to 40% (shrinking Name column footprint on left), mapped all 15 Odyssey genera to their minimum colonial separation distances, sorted predicted species alphabetically (A-Z) for quick pilot lookup, and formatted active sampling progress to show `[Scanned i/3 | Dm]` and predictions as `[Dm]`. Verified with passing unit tests. All 97 tests pass green.
- **Hierarchical Planetary Codex & Badges (2026-05-27):** Overhauled `src/model/body.rs`, `src/app.rs`, `src/journal.rs`, and `src/ui/mod.rs` to structure scanned worlds into three distinct category groups. Implemented backward-compatible key encoding (`PlanetClass|L|T|R|B`) to track landables, terraformables, rings, and life-bearing counts trip-wide in `trip.json` with zero database migrations. Added green leaf `🌿` life badges, DNA/coordinates, and dynamic category sum rows. Verified with robust unit tests. All 97 tests pass green.
- **Dynamic Primary Star Discovery (2026-05-27):** Overhauled `src/model/system.rs` and `src/journal.rs` to extract the arrival star's `body_id` dynamically from journal `FSDJump` and `Location` events. Replaced all hardcoded `body_id == 0` checks in the `Stellar Codex` and `Exobiology Predictor` with dynamic lookups. Fully resolves missing codex counts and exobiology predictor failures in multi-star binary/trinary systems.
- **Transient Live Notification Footer (2026-05-27):** Implemented live, dynamic status notifications in the TUI footer upon FSD jump, FSS scan, and DSS mapping completions. Configured crossterm input polling to instantly dismiss transient status messages and restore the default keybindings bar upon any key press, resolving stale startup replay footers.
- **Legacy Codex In-Memory Auto-Pruner (2026-05-27):** Integrated a robust upgrader inside the `load` sequence of `src/persistence.rs` to automatically identify the presence of legacy flat star classes (like "K" or "F") in `stellar_codex`, cleanly purging all non-authentic/mock star classes and resetting the codex to a 100% authentic, subtype-only slate. Successfully resolves TUI save-on-exit overwrite loops.

- **Stateful Codex Scrolling & Highlight (2026-05-27):** Refactored the raw Stellar, Planetary, and Biological Codex tables inside the History sub-views to use stateful TableState, selection row highlighting, and right-pane scrollbars. Integrated Up/Down keyboard navigation directly into the History tab for seamless codex scrolling. All 96 tests pass green.
- **Exobiology Tree-List Refinement (2026-05-27):** Refined the nested tree structure under active exobiology species in the inspector panel to show Location [i/3] labels instead of generic indices. Verified all coordinate rendering and Haversine distance computations via robust unit tests.
- **Exobiology Replay & Grouping Refinements (2026-05-27):** Enabled exobiology progress state and scan history reconstruction during startup replay mode by running `ScanOrganic` regardless of track_trip status. Grouped predictions color variants by base species name to compress row listings. Implemented exobiology genus constraint: dynamically filters out all other species of a genus once a specific species is actively scanned, saving immense terminal rows and visual clutter. All 95 tests pass green.
- **Exobiology UI Refinements (2026-05-27):** Implemented exobiology sampling progress tracking for all genetic stages (Log = 1/3, Sample = 2/3, Analyse = 3/3 / Completed) with dynamic TUI updates, compacted the layout to a single premium line per predicted species variant (eliminating the redundant `Base` value row), added robust key lookups falling back to base species names, and validated all exobiology features with unit tests.
- **Automatic Tab Transitions (2026-05-27):** Implemented automatic TUI tab switching to the Route tab when starting a live `Hyperspace` jump (if a route is plotted), and automatically returning to the Bodies tab when arriving in the destination system, protected against replay mode.
- **Stellar Codex Hierarchy (2026-05-27):** Parsed specific star types (`{StarType}{Subclass} {Luminosity}` like `F9 VAB` or `K5 VA`) on star `Scan` events, and rendered them using a beautiful, premium tree-guide structure (` ├─ ` / ` └─ `) under their main class inside the Stellar Codex TUI tab, summing base visits automatically.
- **Sandboxed Flatpak Watcher Fix (2026-05-27):** Replaced inotify-based LiveLogDirReader with a highly robust filesystem-polling log tailer, fully resolving TUI lockups inside sandboxed Flatpak container environments on Linux.
- **Hybrid Exobiology Predictor Fallback (2026-05-27):** Implemented SAA genus extraction and a relaxed matching fallback (matching strictly on genus, atmosphere, and planet class) to prevent blank exobiology panels when strict Canonn boundaries fail.
- **Spelling Normalization Fix (2026-05-27):** Standardized atmosphere normalization to treat 'sulphur' and 'sulfur' interchangeably, fixing exobiology matching failures on sulfur-dioxide planets throughout the galaxy.
- **Phase 2 — Navigation & Exobiology (2026-05-27):** Real-time target-syncing, exobiology predictions, trip codex logs, and route sync with EDSM streaming completely implemented and verified.
- **EDSM Dependency Added (2026-05-27):** Integrated stable `ureq = "3.3.0"` as the lightweight, synchronous API driver, compiling and verifying successfully.
- **Phase 2 Specs (2026-05-27):** Complete exobiology prediction, navigation, trip logbook, and target-syncing specifications defined and committed via a successful domain grill session.

## ⚠️ Blockers

_None._

## 🧠 Session Memory

- _2026-05-29_ — Expanded right pane to 40%, sorted exobio predictions alphabetically, and integrated genus separation distances. _(operational)_
- _2026-05-27_ — Overhauled Planetary Codex to use categories and status badges (🚀, 🌍, 🪐, 🌿) with composite key encoding. _(operational)_
- _2026-05-27_ — Overhauled primary star index detection to dynamically resolve arrival star's BodyID from jump/location events, fixing Stellar Codex and Biologist. _(operational)_
- _2026-05-27_ — Set transient status notifications for FSDJump, Scan, and SAAScanComplete. _(operational)_
- _2026-05-27_ — Configured any keyboard press to instantly clear status messages in TUI. _(operational)_
- _2026-05-27_ — Implemented automatic startup legacy codex pruner in persistence layer. _(operational)_

- _2026-05-27_ — Refactored Codex views to use stateful state and scrollbars. _(operational)_
- _2026-05-27_ — Wired Up/Down arrow keys to scroll active Codex list when History tab is selected. _(operational)_
- _2026-05-27_ — Injected mock star subclass data into user's trip.json for immediate visualization. _(operational)_
- _2026-05-27_ — Unified exobiology sample locations label to show Location [i/3] format. _(operational)_
- _2026-05-27_ — Explained the real-time Status.json parsing and Haversine distance mechanism. _(operational)_
- _2026-05-27_ — Reconstructed active exobiology progress and completed scans during startup journal replay. _(operational)_
- _2026-05-27_ — Grouped predicted exobiology color variants in TUI right-pane to condense rows. _(operational)_
- _2026-05-27_ — Filtered out redundant species of the same genus once a specific species is actively scanned, honoring Elite Dangerous exobiology rules. _(operational)_
- _2026-05-27_ — Indexed exobiology organic progress under specific variant localized names in addition to species and genus names. _(operational)_
- _2026-05-27_ — Resolved exobiology predictions panel key mismatch by implementing a base species name fallback lookup. _(operational)_
- _2026-05-27_ — Compacted exobiology predictions UI layout in the inspector to a single premium line (name and first discovery value) to save 50% vertical space. _(operational)_
- _2026-05-27_ — Ingested StartJump event matching the Hyperspace struct variant to automate tab navigation. _(operational)_
- _2026-05-27_ — Handled live-mode protection inside StartJump and FSDJump to ensure tab transitions only occur during live sessions. _(operational)_
- _2026-05-27_ — Grouped and summed primary star types by their base non-digit class letters, sorting base classes and subtypes descending by visits. _(operational)_
- _2026-05-27_ — Formatted star subtypes using subclass numbers and capitalized luminosity codes from scan events to match in-game Star Types perfectly. _(operational)_
- _2026-05-27_ — Normalized 'sulphur' and 'sulfur' atmosphere spelling to prevent database matching failures on sulfur-dioxide planets. _(operational)_
- _2026-05-27_ — Implemented hybrid exobiology prediction fallback using DSS/SAA genus extraction when strict boundaries fail. _(operational)_
- _2026-05-27_ — Switched from inotify LiveLogDirReader to a filesystem-polling watcher loop to bypass Flatpak sandbox boundaries. _(operational)_
- **2026-05-27 (Phase 2 Resolution):** Sorted raw codex entries before mapping to Ratatui Row widgets to avoid private `cells` borrowing constraints. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Target-syncing will monitor the `Destination` object inside `Status.json` (System, Body, Name) to automatically focus and highlight the player's targeted body in the TUI. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Plotted route waypoints will be tracked by watching the `NavRoute.json` file in the journal folder. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Route EDSM API enrichment will be fetched asynchronously on a background worker thread using synchronous `ureq` (v3.3.0) to prevent TUI rendering locks. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Exobiology predictions will match FSS `Scan` physical parameters against Canonn distribution boundaries, only compiling and checking if FSS discovery scan detects bio signals > 0 to keep the display clean. _(operational)_

## 📋 Next Session Suggestions

- **Planetary Codex Verification:** Commander manually explores a few systems to verify FSS scans, DSS mapping, and life-bearing planet count updates in the Planetary Codex.
- **Phase 6 — Exobiology Coordinate Tracking:** Implement physical exobiology coordinate tracking mapping to exobiology sampler stages in TUI.
