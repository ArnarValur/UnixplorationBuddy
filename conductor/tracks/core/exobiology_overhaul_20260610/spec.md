# Spec: Exobiology Prediction Overhaul

## Goal

Upgrade UnixplorationBuddy's exobiology predictions from "all possible variants" to **BioScan-grade accuracy** — narrowing species to exact color variants using star class, surface materials, volcanism, pressure, and galactic region filtering.

## Problem

Current predictions show all possible color variants for a species (e.g., "Bacterium Acies (Aquamarine/Cobalt/Cyan/Lime/Magenta/White)") when competing tools like BioScan and EDEB narrow it to the exact single variant. Root causes:

1. **No galactic region tracking** — species with region locks (Tussock, Cactoida, Aleoida, Stratum, Osseus) are predicted in regions they can't exist
2. **Brown dwarf star classes conflated** — L/T/Y all match as "brown dwarf" instead of individually
3. **No volcanism filtering** — Fumerola requires specific volcanism types; Body struct lacks the field
4. **No pressure filtering** — Data stored but never checked; Body struct lacks pressure field
5. **No surface material tracking** — Material-determinant species (Electricae, Fumerola, some Bacterium) can't be narrowed to specific variants
6. **No star→color / material→color lookup** — Even when we could narrow, we display all variants instead of resolving to a single color

## Research Foundation

Four research reports in `conductor/docs/research/research-reports/`:
- `bio-variant-prediction.md` — Initial analysis
- `canonn-data-analysis.md` — Canonn JSON schema, what we use vs ignore
- `bioscan-ruleset-reference.md` — Complete BioScan ruleset reference (19 genera, all mapping tables)
- `galactic-region-lookup.md` — Region bitmap algorithm (2D RLE from StarPos coordinates)

Reference code: BioScan plugin at `conductor/docs/research/EDMC-BioScan/`, ExploData RegionMap at EDMC install path.

## Scope

### In Scope
- Galactic region lookup from StarPos (offline bitmap)
- Volcanism, pressure, surface materials added to Body struct + journal parsing
- Star class matching fix (differentiate L/T/Y, white dwarf subtypes)
- Star→color and material→color variant resolution tables
- Region group filtering in predictions
- Volcanism filtering in predictions
- Pressure filtering in predictions
- Material-based variant narrowing
- Display single resolved variant color instead of slash-separated list
- Region name displayed in top bar or system header

### Out of Scope
- Guardian nebula proximity checks (Brain Tree, Electricae Radialem)
- Tuber zone calculations (Sinuous Tubers)
- Nebula proximity checks (Bark Mound)
- `atmosphere_component` percentage checks (Recepta edge case)
- Parent star vs main star distinction
- Orbital period checks (Tubers)

## Acceptance Criteria

1. Predictions narrow to **single color variant** for stellar-class determinant species
2. Predictions narrow to **single color variant** for material-determinant species when materials are available
3. Region-locked species are **excluded** when outside their region
4. Fumerola species only predicted on bodies with matching volcanism
5. Galactic region name visible in system header
6. All existing tests pass + new tests for region lookup, variant resolution, volcanism matching
7. No regression in prediction accuracy (only narrowing, never missing valid species)

## Technical Approach

### Variant Determinant Types
| Type | Resolution Method | Genera |
|------|-------------------|--------|
| Stellar (genus-level) | Primary star class → single color via genus table | Aleoida, Cactoida, Clypeus, Fonticulua, Frutexa, Tubus, Tussock |
| Stellar (per-species) | Primary star class → single color via species table | Bacterium (3), Concha (2), Osseus (4), Recepta (1), Stratum (8) |
| Material (per-species) | Body surface material → single color | Electricae (2), Fumerola (4), Fungoida (4), Bacterium (10), Concha (2), Osseus (2), Recepta (2) |

### Region Lookup
- 2D RLE bitmap lookup from StarPos (x,z) coordinates
- Y ignored — regions defined on galactic plane
- Constants: `x0=-49985, z0=-24105, scale=83/4096`
- 42 regions, 19 named groups for bio filtering

### Unit Conversions
- Gravity: `journal_value / 9.797759` → G
- Pressure: `journal_value / 101231.656250` → atm
