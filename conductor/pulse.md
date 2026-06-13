# Pulse — Current Project State

**Last Updated:** 2026-06-13 09:42
**Session Focus:** Fleet carrier docking + data sell + ship documentation

## 🚀 Active Tracks

_None active._

## ✅ Recently Completed

- **Fleet Carrier Docking + Ship Documentation (2026-06-13):** Found carrier on 3rd attempt. Sold 1.3B bio data (all first discoveries: Stratum Tectonicas Emerald, Bacterium Vesicula Red, Electricae Pluma Red, Fumerola Carbosis Cobalt) + 20 pages cartographics (~400M after 25% tariff). Balance 5.0B → 5.396B CR. Explorer Rank: Elite III. Created ship loadout doc (conductor/docs/ship-eaglehawk.md). Trip reset for clean codex. 165/165 tests.

- **Planetary Codex Toggle + Fleet Carrier Run (2026-06-12):** Added `v` key toggle for compact/expanded codex views. Expanded shows nested sub-attribute rows with tree guides. Carrier was gone on arrival (Fojea HG-X D1-1). Found 12-star system in Vulcan Gate. 165/165 tests.

- **Planetary Codex Polish + Route Enrichment (2026-06-12):** Condensed planetary codex layout (sub-rows → inline colorized badges, ~60% fewer rows). Alphabetical sort. Route tab: StarPos distance calculation, remaining distance, progress counter in title, star class coloring (scoopable/non-scoopable), visited system dimming, non-scoopable streak fuel warnings, keyboard/mouse scroll support. 165/165 tests.

- **Inspector Scrolling + EDSM Cleanup (2026-06-12):** Added PgUp/PgDn scrolling to inspector sidebar (3 lines/step). Scroll clamped to content bounds, auto-resets on body change. Dim scroll indicator (▲▼) when content overflows. Removed dead EDSM Telemetry section (always 0 cr). 165/165 tests. Commits `e0b3f0a`, `454d7f0`.

- **Inspector Sidebar Layout Rework (2026-06-12):** Restructured inspector from single monolithic Paragraph to multi-section Layout. Physical properties (left column) + Materials (right column) side-by-side using Ratatui horizontal Layout split. Orbital/Anomalies/Exobiology full-width below. 165/165 tests.

## ⚠️ Blockers

_None._

## 🧠 Session Memory

- _2026-06-13_ — **NavRoute star type limitation** — NavRoute.json only provides base star class letter (M, G, F). Full spectral type (M5 VA) only available from Scan events for visited systems. Game data limitation, parked. _(elite)_
- _2026-06-13_ — **Fleet carrier found** — Third carrier search successful. First two had departed. 25% tariff accepted — better than 230+ jumps back to the Bubble from coordinates (-18920, -11, 6825). _(elite)_
- _2026-06-13_ — **Elite III acquired** — Explorer rank milestone after selling accumulated exploration data. _(elite)_
- _2026-06-13_ — **Ship loadout documented** — Mandalay EagleHawk (AVJ-93) full module list captured in `conductor/docs/ship-eaglehawk.md`. 85.76 ly unladen jump range. _(operational)_
- _2026-06-12_ — **Condensed planetary codex** — Eliminated sub-attribute rows (Ringed/Landable/Terraformable/Bio Signals/Confirmed Life). Merged as inline badges with per-badge Span coloring (🪐/🚀 dim, 🌍 green, 🌿 teal, ✅ bold teal). Scan-count brightness gradient on planet names. ~60% vertical space saved. _(operational)_
- _2026-06-12_ — **Route StarPos distance calculation** — Euclidean 3D from NavRoute.json coordinates. Per-jump distance + suffix-sum remaining distance. Total route distance in title. No API calls — pure math from existing data. _(operational)_

## 📋 Next Session Suggestions

- **NavRoute mass code scanner:** Parse route waypoints, flag `g`/`h` mass code systems (deferred).
- **FSDTarget event handling:** Parse `RemainingJumpsInRoute` for live jump counter.
- **Status.json fuel parsing:** Add `Fuel.FuelMain` to Status struct for real-time fuel display.
- **Inspector jumponium drill-down:** Show which body to land on for each material.
- **README Rewrite:** User to write README in human words.
