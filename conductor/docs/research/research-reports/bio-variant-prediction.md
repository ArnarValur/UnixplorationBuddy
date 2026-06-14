# Bio Variant Prediction Research

> Date: 2026-06-10
> Source: EDMC-Pioneer reverse-engineering + Canonn data analysis

## Key Discovery

Pioneer does **NOT** predict bio species — that's the **BioScan** plugin (separate repo: `github.com/Silarn/EDMC-BioScan`). Our predictor already filters correctly by atmosphere/body/gravity/temp/star, but shows too many colors because of two issues.

## Two Types of Variant Determinants

| Determinant | How it works | Affected Genera |
|---|---|---|
| **Stellar Class** | Primary star class → exactly ONE color | Aleoida, Cactoida, Clypeus, Fonticulua, Frutexa, Stratum, Tubus, Tussock |
| **Material** | Surface material present on body → specific color | Electricae, Fumerola |
| **Dual (Both)** | Some species stellar, some material | Bacterium, Concha, Osseus, Recepta, Fungoida |

**For stellar-class determinants, each star class maps to EXACTLY ONE color.** If you know the primary star, you know the exact color.

## Why We Show Multiple Colors

1. **Brown dwarf conflation:** `match_star_class()` line 59 treats L/T/Y all as "brown dwarf" — matches ALL brown dwarf variants instead of just the right one
2. **White dwarf subtypes:** DA/DB/DC etc. not differentiated
3. **Material species:** Bacterium Acies etc. need surface material data we don't track at all

## Fix Levels

### Level 1 — Fix star class matching (easy, big impact)
- Differentiate L vs T vs Y brown dwarfs individually in `match_star_class()`
- Same for white dwarf subtypes
- **Eliminates most multi-color predictions for stellar-class genera**

### Level 2 — Hardcoded Star→Color mapping table (medium)
- Encode deterministic mapping directly per genus instead of relying on Canonn histogram data
- Example: Aleoida: B→Yellow, A→Green, F→Teal, K→Turquoise, M→Emerald, L→Lime, T→Sage, TTS→Mauve, Y→Amethyst, W→Grey, D→Indigo, N→Ocher
- Each Canonn genus doc has the exact table

### Level 3 — Material tracking (larger)
- Add `materials: Vec<(String, f64)>` to `Body` struct
- Parse `Materials` array from journal `Scan` events
- Build material→color lookup tables for material-determinant species

## Relevant Files

- Predictor: `src/model/biology/predictor.rs` (fix `match_star_class` at line 46-73)
- Display: `src/ui/inspector.rs` (grouping at line 103-143)
- Body: `src/model/body.rs` (needs materials field for Level 3)
- Dataset: `src/model/biology/dataset.rs` (generated from Canonn JSON)
- Canonn docs: `conductor/canonn-data/*.md` (variant mapping tables)
