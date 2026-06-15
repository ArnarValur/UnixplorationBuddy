# Pulse — Current Project State

**Last Updated:** 2026-06-15 21:22
**Session Focus:** Bodies table visual polish + bio scan genus fallback

## 🚀 Active Tracks

_None active._

## ✅ Recently Completed

- **Bodies Table Visual Polish (2026-06-15):** Tree guides (`├─ └─ │`) for hierarchy nesting, planet-class-aware accent colors (Metal Rich copper, Earth-like green, Water World blue, Ammonia purple, Gas Giant tan), per-cell bio (green) / geo (cyan) / value (green ≥100K) / POI (magenta) color styling, sub-moon depth dimming, dash removal for empty cells, anomaly bold removal. Extended `display_order` to `Vec<(u32, u32, bool)>` with `is_last_sibling` across 9 files. Added `body_display_color()` function. 165/165 tests.

- **Bio Scan Genus Fallback (2026-06-15):** Fixed exobiology progress not showing when predicted species name differs from journal species name (e.g., predictor: "Prasinum Bioluminescent Anemone", journal: "Luteolum Anemone"). Added genus-level fallback in inspector that scans `organic_progress` keys by body prefix and matches via static dataset genus lookup. Covers both in-progress (1/3, 2/3) and completed (3/3) scans. 165/165 tests.

- **Codex Bio Signal Fix + Variant Sub-Rows (2026-06-14):** Fixed critical bulk-move bug in codex key upgrade logic — `remove()` moved entire planet class count into `|B` bucket when one body got bio signals. Changed to single-decrement across all 3 upgrade paths (FSSBodySignals, SAASignalsFound, ScanOrganic). Also fixed FSSBodySignals→Scan race condition where `is_new_body` guard failed because FSSBodySignals always arrives first. Added expanded codex view showing individual key variant sub-rows (e.g., `🚀 Landable: 49`, `🚀🌿 Landable · Bio: 4`) instead of aggregated attribute totals. Patched live trip.json by recounting from journals. 165/165 tests. ADR-0015.

- **Fleet Carrier Docking + Ship Documentation (2026-06-13):** Found carrier on 3rd attempt. Sold 1.3B bio data (all first discoveries: Stratum Tectonicas Emerald, Bacterium Vesicula Red, Electricae Pluma Red, Fumerola Carbosis Cobalt) + 20 pages cartographics (~400M after 25% tariff). Balance 5.0B → 5.396B CR. Explorer Rank: Elite III. Created ship loadout doc (conductor/docs/ship-eaglehawk.md). Trip reset for clean codex. 165/165 tests.

- **Planetary Codex Toggle + Fleet Carrier Run (2026-06-12):** Added `v` key toggle for compact/expanded codex views. Expanded shows nested sub-attribute rows with tree guides. Carrier was gone on arrival (Fojea HG-X D1-1). Found 12-star system in Vulcan Gate. 165/165 tests.

## ⚠️ Blockers

_None._

## 🧠 Session Memory

- _2026-06-15_ — **Moon³ system discovered** — Byoo Euhm ZP-P e5-4 has sub-moons (AB 3 b a, AB 3 e a) orbiting moons of gas giant AB 3. AB 3 b a has a "close orbit" anomaly (apoapsis 4444 km vs 5643 km combined radii — negative surface gap). _(elite)_
- _2026-06-15_ — **Tree guide rendering** — Uses `ancestor_is_last` stack pattern to track vertical continuation lines (`│`) across depth levels. Prefix built per-row during iteration. _(operational)_
- _2026-06-15_ — **Per-cell ratatui styling** — Switched from `Row::new(Vec<String>)` to `Row::new(Vec<Cell>)` to enable independent fg color per column. Name cell uses `Line::from(vec![Span, Span])` for dim prefix + colored name. _(operational)_
- _2026-06-15_ — **Bio prediction mismatch pattern** — Journal `Genus_Localised` field can contain the species name rather than the genus (e.g., "Luteolum Anemone" instead of "Anemone"). Progress key lookup needs genus-level fallback via dataset. _(operational)_

## 📋 Next Session Suggestions

- **Enhancement backlog review:** Review `conductor/docs/enhancement-backlog.md` — items #1 (planet subtype), #10 (highlight current star class), #13 (bio scan total value summary) are quick wins.
- **NavRoute mass code scanner:** Parse route waypoints, flag `g`/`h` mass code systems (deferred).
- **FSDTarget event handling:** Parse `RemainingJumpsInRoute` for live jump counter.
- **Status.json fuel parsing:** Add `Fuel.FuelMain` to Status struct for real-time fuel display.
- **Inspector jumponium drill-down:** Show which body to land on for each material.
- **README Rewrite:** User to write README in human words.
