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

### Phase 2 — Enrichment

- First discoverer status (EDSM API)
- EDSM discoverer name (EDSM API)
- Gravity column
- Full header: completion badge, non-body signals, current activity
- Sortable table columns (by distance, value, etc.)
- Route view

### Phase 3+ — Future

- Biologicals module (dedicated bio grill — Canonn data integration, species prediction engine)
- EDASTRO / Spansh integration
- Configurable color themes
- Export functionality

## Out of Scope

- Windows/macOS support (Linux-first, may come later)
- Overlay mode (no transparency/always-on-top — it's a terminal app on a second monitor)
- EDMC plugin mode (standalone only)
- Real-time multiplayer / wing data
- Route planning (use Spansh/EDDB for that)

## Open Questions

- Canonn data format: `.md` + `.json` files are gathered in `conductor/canonn-data/`. Need to define the parsed schema and build-time processing pipeline during bio module grill.
- EDSM API rate limits and auth: investigate before Phase 2 implementation.
- Journal file location: varies by platform and Steam/native install. Need auto-detection or config.
