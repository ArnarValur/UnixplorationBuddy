# Implementation Plan — Phase 2 Navigation & Exobiology

## Phase 1: Research & Setup
- [x] Add `ureq` to `Cargo.toml`.
- [ ] Set up `Status.json` and `NavRoute.json` file watchers.
- [ ] Implement build-time exobiology parser to generate `src/model/biology/dataset.rs` from `conductor/canonn-data/`.

## Phase 2: Core Data Types & Ingestion
- [ ] Expand `Body` model with `gravity`, `temperature`, and `landable` fields.
- [ ] Implement `Status.json` parsing in the live journal watcher.
- [ ] Implement `NavRoute.json` monitoring and parsing.
- [ ] Add `ScanOrganic` ingestion for exobiology completed analyses.

## Phase 3: Background EDSM Thread & Cache
- [ ] Implement background worker thread loop with crossbeam/mpsc message passing.
- [ ] Add `ureq` sequential GET queue with 200ms delay throttling.
- [ ] Implement session memory caching for EDSM system data.

## Phase 4: Biologist Engine
- [ ] Build the exobiology matching logic comparing `Body` conditions to generated Canonn boundaries.
- [ ] Set up the exobiology species Vista Genomics payout registry.

## Phase 5: TUI Layout & Rendering
- [ ] Rebuild the main TUI layout to support split-pane rendering (Left: table, Right: Inspector).
- [ ] Implement dynamic split collapse under 110 terminal columns.
- [ ] Implement the Settings overlay (`s` key) with dynamic column visibility.
- [ ] Implement the Trip Codex sub-views navigation (`h/l` or arrow keys).
- [ ] Implement the Route Exploration tab showing scoopable diagnostics and streamed EDSM badges (`💰`, `🌍`, `🚀`).

## Phase 6: Integration & Verification
- [ ] Wire up real-time target syncing between status logs and active selection.
- [ ] Add integration and unit tests for exobiology matching and route parsing.
- [ ] Run live journal simulation and verify split-pane rendering robustness.
