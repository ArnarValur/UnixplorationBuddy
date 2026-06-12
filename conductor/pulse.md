# Pulse — Current Project State

**Last Updated:** 2026-06-12 22:23
**Session Focus:** Planetary Codex polish + Route Exploration enrichment

## 🚀 Active Tracks

_None active._

## ✅ Recently Completed

- **Planetary Codex Polish + Route Enrichment (2026-06-12):** Condensed planetary codex layout (sub-rows → inline colorized badges, ~60% fewer rows). Alphabetical sort. Route tab: StarPos distance calculation, remaining distance, progress counter in title, star class coloring (scoopable/non-scoopable), visited system dimming, non-scoopable streak fuel warnings, keyboard/mouse scroll support. 165/165 tests.

- **Inspector Scrolling + EDSM Cleanup (2026-06-12):** Added PgUp/PgDn scrolling to inspector sidebar (3 lines/step). Scroll clamped to content bounds, auto-resets on body change. Dim scroll indicator (▲▼) when content overflows. Removed dead EDSM Telemetry section (always 0 cr). 165/165 tests. Commits `e0b3f0a`, `454d7f0`.

- **Inspector Sidebar Layout Rework (2026-06-12):** Restructured inspector from single monolithic Paragraph to multi-section Layout. Physical properties (left column) + Materials (right column) side-by-side using Ratatui horizontal Layout split. Orbital/Anomalies/Exobiology full-width below. 165/165 tests.

- **POI Anomaly Detection + TUI Integration (2026-06-12):** Built anomaly detection engine from EDMC-Canonn codex.py. 11 detectors (close orbit, flypast, collision, trojan, satellite star, moon³, fast rotator, retrograde orbit, high eccentricity, extreme tilt). Jumponium Green System detection (Basic/Standard/Premium). Header bar repurposed as POI ticker. Inspector panel shows anomaly details. 165/165 tests. Commits up to `f8db7f8`.

- **/conductor-elite workflow + EDMC-Canonn research (2026-06-12):** Created `/conductor-elite` workflow. Cloned `canonn-science/EDMC-Canonn` for reference. Brainstormed topbar rework, POI integration, tourist anomaly detection. Commit `4fbc7b1`.

## ⚠️ Blockers

_None._

## 🧠 Session Memory

- _2026-06-12_ — **Condensed planetary codex** — Eliminated sub-attribute rows (Ringed/Landable/Terraformable/Bio Signals/Confirmed Life). Merged as inline badges with per-badge Span coloring (🪐/🚀 dim, 🌍 green, 🌿 teal, ✅ bold teal). Scan-count brightness gradient on planet names. ~60% vertical space saved. _(operational)_
- _2026-06-12_ — **Route StarPos distance calculation** — Euclidean 3D from NavRoute.json coordinates. Per-jump distance + suffix-sum remaining distance. Total route distance in title. No API calls — pure math from existing data. _(operational)_
- _2026-06-12_ — **Non-scoopable streak detection** — Flags 3+ consecutive non-KGBFOAM stars with ⚠️ fuel warning in Notes column. Pre-computed per-frame as boolean vec. _(operational)_
- _2026-06-12_ — **Route scroll state** — `selected_route_index: usize` in App. Auto-snaps to current system on FSD charging, NavRoute load/update. Manual via Up/Down keys and mouse scroll when Route sub-tab active. _(operational)_
- _2026-06-12_ — **Route visual enrichment** — Visited systems dimmed (ELITE_DIM). Non-scoopable stars in red. Current system highlighted. EDSM "0" → "—". Progress in title `[40/90 · 2,847 ly · 50 left]`. 🏁 flag on destination. _(operational)_

## 📋 Next Session Suggestions

- **Trip reset at fleet carrier:** Sell cartographics + bio data, Ctrl+R to zero trip counters for clean codex.
- **NavRoute mass code scanner:** Parse route waypoints, flag `g`/`h` mass code systems (deferred this session).
- **FSDTarget event handling:** Parse `RemainingJumpsInRoute` for live jump counter.
- **Status.json fuel parsing:** Add `Fuel.FuelMain` to Status struct for real-time fuel display.
- **Inspector jumponium drill-down:** Show which body to land on for each material.
- **README Rewrite:** User to write README in human words.
