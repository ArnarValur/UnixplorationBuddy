# Pulse — Current Project State

**Last Updated:** 2026-06-14 08:56
**Session Focus:** Fix planetary codex bio signal counting + add expanded variant sub-rows

## 🚀 Active Tracks

_None active._

## ✅ Recently Completed

- **Codex Bio Signal Fix + Variant Sub-Rows (2026-06-14):** Fixed critical bulk-move bug in codex key upgrade logic — `remove()` moved entire planet class count into `|B` bucket when one body got bio signals. Changed to single-decrement across all 3 upgrade paths (FSSBodySignals, SAASignalsFound, ScanOrganic). Also fixed FSSBodySignals→Scan race condition where `is_new_body` guard failed because FSSBodySignals always arrives first. Added expanded codex view showing individual key variant sub-rows (e.g., `🚀 Landable: 49`, `🚀🌿 Landable · Bio: 4`) instead of aggregated attribute totals. Patched live trip.json by recounting from journals. 165/165 tests. ADR-0015.

- **Fleet Carrier Docking + Ship Documentation (2026-06-13):** Found carrier on 3rd attempt. Sold 1.3B bio data (all first discoveries: Stratum Tectonicas Emerald, Bacterium Vesicula Red, Electricae Pluma Red, Fumerola Carbosis Cobalt) + 20 pages cartographics (~400M after 25% tariff). Balance 5.0B → 5.396B CR. Explorer Rank: Elite III. Created ship loadout doc (conductor/docs/ship-eaglehawk.md). Trip reset for clean codex. 165/165 tests.

- **Planetary Codex Toggle + Fleet Carrier Run (2026-06-12):** Added `v` key toggle for compact/expanded codex views. Expanded shows nested sub-attribute rows with tree guides. Carrier was gone on arrival (Fojea HG-X D1-1). Found 12-star system in Vulcan Gate. 165/165 tests.

- **Planetary Codex Polish + Route Enrichment (2026-06-12):** Condensed planetary codex layout (sub-rows → inline colorized badges, ~60% fewer rows). Alphabetical sort. Route tab: StarPos distance calculation, remaining distance, progress counter in title, star class coloring (scoopable/non-scoopable), visited system dimming, non-scoopable streak fuel warnings, keyboard/mouse scroll support. 165/165 tests.

- **Inspector Scrolling + EDSM Cleanup (2026-06-12):** Added PgUp/PgDn scrolling to inspector sidebar (3 lines/step). Scroll clamped to content bounds, auto-resets on body change. Dim scroll indicator (▲▼) when content overflows. Removed dead EDSM Telemetry section (always 0 cr). 165/165 tests. Commits `e0b3f0a`, `454d7f0`.

## ⚠️ Blockers

_None._

## 🧠 Session Memory

- _2026-06-14_ — **Codex bulk-move bug** — `remove()` on old key moved ALL counts for a planet class when upgrading one body. Root cause: FSSBodySignals always precedes Scan, so the race condition guard (`is_new_body`) failed silently, and the upgrade code moved totals instead of singles. _(operational)_
- _2026-06-14_ — **Expanded codex variant sub-rows** — Changed expanded view from aggregated attribute totals to individual key variants with emoji labels and counts in the Scans column. _(operational)_
- _2026-06-14_ — **Cargo build cache miss** — `cargo build --release` didn't pick up source changes made during a chained `cargo test && cargo build --release` pipeline. Required `touch` on source files to force recompile. _(operational)_
- _2026-06-13_ — **NavRoute star type limitation** — NavRoute.json only provides base star class letter (M, G, F). Full spectral type (M5 VA) only available from Scan events for visited systems. Game data limitation, parked. _(elite)_
- _2026-06-13_ — **Fleet carrier found** — Third carrier search successful. First two had departed. 25% tariff accepted — better than 230+ jumps back to the Bubble from coordinates (-18920, -11, 6825). _(elite)_
- _2026-06-13_ — **Elite III acquired** — Explorer rank milestone after selling accumulated exploration data. _(elite)_
- _2026-06-13_ — **Ship loadout documented** — Mandalay EagleHawk (AVJ-93) full module list captured in `conductor/docs/ship-eaglehawk.md`. 85.76 ly unladen jump range. _(operational)_

## 📋 Next Session Suggestions

- **Verify variant sub-rows rendering:** Restart UnixplorationBuddy and confirm the expanded codex shows individual key variants (Plain, Landable, Landable·Bio, etc.) not aggregated totals. The last build needed a `touch` force-recompile.
- **NavRoute mass code scanner:** Parse route waypoints, flag `g`/`h` mass code systems (deferred).
- **FSDTarget event handling:** Parse `RemainingJumpsInRoute` for live jump counter.
- **Status.json fuel parsing:** Add `Fuel.FuelMain` to Status struct for real-time fuel display.
- **Inspector jumponium drill-down:** Show which body to land on for each material.
- **README Rewrite:** User to write README in human words.
