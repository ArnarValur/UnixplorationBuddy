# Plan: Exobiology Prediction Overhaul

## Phase 1 — Foundation: Body Struct & Journal Parsing

> Add missing fields to Body and parse them from journal events.

- [x] **1.1** Add `volcanism: Option<String>` to Body struct
- [x] **1.2** Add `pressure: Option<f64>` to Body struct (store in atm, convert from Pa)
- [x] **1.3** Add `surface_materials: Vec<(String, f64)>` to Body struct
- [x] **1.4** Parse `Volcanism` field from Scan journal events into Body
- [x] **1.5** Parse `SurfacePressure` from Scan events, convert Pa→atm (`/ 101325.0`)
- [x] **1.6** Parse `Materials` array from Scan events into Body
- [x] **1.7** Add `Serialize`/`Deserialize` for new fields (state persistence compat)
- [x] **1.8** Tests: verify volcanism, pressure, materials parsed from mock journal events

## Phase 2 — Galactic Region Lookup [checkpoint: 58a3b7f]

> Implement offline region determination from StarPos coordinates.

- [x] **2.1** Create `src/model/region.rs` module
- [x] **2.2** Port the 42 region names as static array
- [x] **2.3** Port the RLE bitmap data from ExploData's `RegionMapData.py` → Rust static data
- [x] **2.4** Implement `find_region(x: f64, z: f64) -> Option<(u8, &'static str)>` with RLE walk
- [x] **2.5** Add `region: Option<(u8, String)>` to App (or System) struct
- [x] **2.6** Compute region on `FSDJump`/`Location` events using StarPos
- [x] **2.7** Display region name in system header bar
- [x] **2.8** Port BioScan's 19 region groups (orion-cygnus, perseus, outer, etc.) as lookup
- [x] **2.9** Tests: known coordinates → expected region IDs, edge cases, Sol → Galactic Centre

## Phase 3 — Prediction Engine Upgrades [checkpoint: 0fd0b0a]

> Fix star matching, add volcanism/pressure/region filtering.

- [x] **3.1** Fix `match_star_class()` — differentiate L vs T vs Y brown dwarfs individually
- [x] **3.2** Fix white dwarf subtypes (DA/DB/DC match 'D' prefix)
- [x] **3.3** Fix supergiant variants (A_BlueWhiteSuperGiant matches 'A', etc.)
- [x] **3.4** Add volcanism filtering to `match_variant()` — check body volcanism against variant's volcanism list (None/Any/substring match)
- [x] **3.5** Add pressure filtering to `match_variant()` — check body pressure against min_p/max_p
- [x] **3.6** Add region filtering — check current region ID against variant's allowed regions
- [x] **3.7** Add region data to SpeciesVariant struct (parse from Canonn JSON `regions` field)
- [x] **3.8** Add volcanism constraint semantics (None = no volcanism, Any = must have, list = substring match)
- [x] **3.9** Tests: brown dwarf L matches only L variants, region-locked species excluded outside region, volcanism filtering

## Phase 4 — Variant Color Resolution [checkpoint: 4d2af12]

> Resolve predictions to single color variants instead of showing all.

- [x] **4.1** Create `src/model/biology/colors.rs` — star→color and material→color lookup tables
- [x] **4.2** Encode genus-level star tables (Aleoida, Cactoida, Clypeus, Fonticulua, Frutexa, Tubus, Tussock)
- [x] **4.3** Encode per-species star tables (Bacterium 01/06/12, Concha 02/03, Osseus 01/03/05/06, Recepta 01, Stratum)
- [x] **4.4** Encode per-species material tables (all Bacterium element species, Electricae, Fumerola, Fungoida, Concha, Osseus, Recepta)
- [x] **4.5** Implement `resolve_variant_color(genus, species, star_class, materials) -> Option<String>`
- [x] **4.6** Update inspector display: show single "Species — Color" instead of "Species (Color1/Color2/...)"
- [x] **4.7** Fallback: if materials unavailable for material-determinant species, show "(material needed)" or similar
- [x] **4.8** Tests: known star + genus → expected color, known material + species → expected color

## Phase 5 — Integration & Polish [checkpoint: 4d2af12]

> Wire everything together and verify end-to-end.

- [x] **5.1** End-to-end test: mock body with full data → single predicted variant with color
- [x] **5.2** Verify state persistence handles new Body fields (backward compat with existing state.json)
- [x] **5.3** Update dataset generation if needed (ensure Canonn JSON volcanism/regions/materials are in dataset)
- [x] **5.4** Run full test suite — all existing + new tests green (129/129)
- [x] **5.5** Build release binary and live test against current system
