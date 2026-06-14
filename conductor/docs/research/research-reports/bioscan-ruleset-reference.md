# BioScan Prediction Ruleset Reference

> Date: 2026-06-10
> Source: EDMC-BioScan plugin (installed locally) + ExploData genus.py

## 1. Architecture Overview

### File Layout

**Rulesets:** `EDMC-BioScan/bio_scan/bio_data/rulesets/` — 19 Python files, each exports a `catalog` dict
**Color Mappings:** `ExploData/explo_data/bio_data/genus.py` — THE color/variant data for ALL genera
**Prediction Engine:** `EDMC-BioScan/load.py` — `value_estimate()` function (lines 387-842)
**Star/Body Checks:** `bio_scan/body_data/util.py` — `star_check()` and `body_check()` helpers

### Data Structure Pattern

```python
catalog = {
    '$Codex_Ent_<GenusCode>_Name;': {         # Genus key
        '$Codex_Ent_<SpeciesCode>_Name;': {    # Species key
            'name': 'Human-readable Name',
            'value': 1234567,
            'rulesets': [                        # OR-ed rule sets
                {                               # AND-ed conditions
                    'atmosphere': ['CarbonDioxide'],
                    'body_type': ['Rocky body'],
                    'min_gravity': 0.04,
                    'max_gravity': 0.276,
                    'volcanism': 'None',
                    'regions': ['orion-cygnus']
                },
            ]
        }
    }
}
```

**Semantics:** Species appears if ANY ruleset passes. Within a ruleset, ALL conditions must match (OR of ANDs).

### All Possible Rule Keys

| Key | Type | Description |
|-----|------|-------------|
| `atmosphere` | `list[str]` or `'Any'` | Atmosphere type(s) |
| `atmosphere_component` | `dict[str, float]` | Min % of specific gas (Recepta only) |
| `body_type` | `list[str]` | Body types |
| `min_gravity` / `max_gravity` | `float` | Surface gravity in G |
| `min_temperature` / `max_temperature` | `float` | Surface temp in Kelvin |
| `min_pressure` / `max_pressure` | `float` | Surface pressure in atm |
| `max_orbital_period` | `float` | Max orbital period in seconds (Tubers only) |
| `volcanism` | `list[str]` / `'Any'` / `'None'` | Volcanism constraint |
| `regions` | `list[str]` | Galaxy region keys (prefix `!` inverts) |
| `star` | `list` / `str` | System star type |
| `parent_star` | `list[str]` | Body's direct stellar parent |
| `main_star` | same | Primary star only |
| `guardian` | `bool` | Near Guardian nebula |
| `tuber` | `list[str]` / `'Any'` | Named tuber zone |
| `nebula` | `'all'` / `'large'` | Near a nebula |
| `bodies` | `list[str]` | Sister body types in system |
| `distance` | `float` | Min distance from arrival (ls) |
| `system` | `str` | Exact system name match |

### Unit Conversions

- Gravity: `gravity_G = journal_value / 9.797759`
- Pressure: `pressure_atm = journal_value / 101231.656250`

---

## 2. Color Variant System

### Per-Genus Colors (simple star-based)

```python
'colors': { 'star': { 'B': 'Yellow', 'A': 'Green', ... } }
```

Used by: Aleoida, Cactoida, Clypeus, Fonticulua, Frutexa, Tubus, Tussock

### Per-Species Colors (mixed star/element)

```python
'colors': { 'species': {
    '$Codex_Ent_Bacterial_01_Name;': { 'star': { ... } },
    '$Codex_Ent_Bacterial_02_Name;': { 'element': { ... } },
}}
```

Used by: Bacterium, Concha, Electricae, Fumerola, Fungoida, Osseus, Recepta, Stratum

### No Color Variants

Brain Tree, Anemone, Bark Mound, Crystalline Shards, Sinuous Tubers, Amphora Plant, Radicoida

---

## 3. Star Class → Color Mapping Tables

### Aleoida (genus-level)

| B | A | F | K | M | L | T | TTS | D | W | Y | N |
|---|---|---|---|---|---|---|-----|---|---|---|---|
| Yellow | Green | Teal | Turquoise | Emerald | Lime | Sage | Mauve | Indigo | Grey | Amethyst | Ocher |

### Cactoida (genus-level)

| O | A | F | G | M | L | T | Y | TTS | W | D | N |
|---|---|---|---|---|---|---|---|-----|---|---|---|
| Grey | Green | Yellow | Teal | Amethyst | Mauve | Orange | Ocher | Red | Indigo | Turquoise | Sage |

### Clypeus (genus-level)

| B | A | F | G | K | M | L | Y | D | N |
|---|---|---|---|---|---|---|---|---|---|
| Maroon | Orange | Mauve | Amethyst | Grey | Turquoise | Teal | Green | Lime | Yellow |

### Fonticulua (genus-level)

| O | B | A | F | G | K | M | L | T | TTS | Y | W | D | N | Ae |
|---|---|---|---|---|---|---|---|---|-----|---|---|---|---|---|
| Grey | Lime | Green | Yellow | Teal | Emerald | Amethyst | Mauve | Orange | Red | Ocher | Indigo | Turquoise | Sage | Maroon |

### Frutexa (genus-level)

| O | B | F | G | M | L | TTS | W | D | N |
|---|---|---|---|---|---|-----|---|---|---|
| Yellow | Lime | Green | Emerald | Grey | Teal | Mauve | Orange | Indigo | Red |

### Tubus (genus-level)

| O | B | A | F | G | K | M | L | T | TTS | W | D | N |
|---|---|---|---|---|---|---|---|---|-----|---|---|---|
| Green | Emerald | Indigo | Grey | Red | Maroon | Teal | Turquoise | Mauve | Ocher | Lime | Yellow | Amethyst |

### Tussock (genus-level)

| F | G | K | M | L | T | Y | W | D | N |
|---|---|---|---|---|---|---|---|---|---|
| Yellow | Lime | Green | Emerald | Sage | Teal | Red | Orange | Maroon | Yellow |

### Bacterium Aurasus/Alcyoneum/Cerbrus (per-species, same table)

| O | B | A | F | G | K | M | L | T | Y | TTS | Ae | W | D | N |
|---|---|---|---|---|---|---|---|---|---|-----|---|---|---|---|
| Turquoise | Grey | Yellow | Lime | Emerald | Green | Teal | Sage | Red | Mauve | Maroon | Orange | Amethyst | Ocher | Indigo |

### Concha Aureolas/Labiata (per-species, same table)

| B | A | F | G | K | L | Y | W | D | N |
|---|---|---|---|---|---|---|---|---|---|
| Indigo | Teal | Grey | Turquoise | Red | Orange | Yellow | Lime | Green | Emerald |

### Osseus Fractus/Spiralis/Cornibus/Pellebantus (per-species, same table)

| O | A | F | G | K | T | Y | TTS |
|---|---|---|---|---|---|---|-----|
| Yellow | Lime | Turquoise | Grey | Indigo | Emerald | Maroon | Green |

### Recepta Umbrux (star-based)

| B | A | F | G | K | M | T | Y | TTS | L | Ae | D | N |
|---|---|---|---|---|---|---|---|-----|---|---|---|---|
| Turquoise | Amethyst | Mauve | Orange | Red | Maroon | Teal | Lime | Sage | Ocher | Grey | Yellow | Emerald |

### Stratum (most species, same table)

| F | K | M | L | Y | T | TTS | D | Ae | W |
|---|---|---|---|---|---|-----|---|---|---|
| Emerald | Lime | Green | Turquoise | Indigo | Grey | Amethyst | Mauve | Teal | Red |

### Stratum Araneamus (special)

| B | A | N |
|---|---|---|
| Emerald | Emerald | Emerald |

---

## 4. Material (Element) → Color Mapping Tables

### Element Groups

- **Group 1:** antimony, polonium, ruthenium, technetium, tellurium, yttrium
- **Group 2:** cadmium, mercury, molybdenum, niobium, tungsten, tin

### Bacterium Element Species

| Species | Sb | Po | Ru | Tc | Te | Y |
|---------|----|----|----|----|----|----|
| Nebulus (02) | Magenta | Gold | Orange | Cyan | Green | Cobalt |
| Acies (04) | Cyan | Magenta | Cobalt | Lime | White | Aquamarine |
| Vesicula (05) | Cyan | Orange | Mulberry | Gold | Red | Lime |
| Informem (08) | Red | Lime | Gold | Aquamarine | Yellow | Cobalt |
| Volu (09) | Red | Aquamarine | Cobalt | Lime | Cyan | Gold |
| Bullaris (10) | Cobalt | Yellow | Aquamarine | Gold | Lime | Red |

| Species | Cd | Hg | Mo | Nb | W | Sn |
|---------|----|----|----|----|---|----|
| Scopulum (03) | White | Peach | Lime | Red | Aquamarine | Mulberry |
| Tela (07) | Gold | Orange | Yellow | Magenta | Green | Cobalt |
| Omentum (11) | Lime | White | Aquamarine | Peach | Blue | Red |
| Verrata (13) | Peach | Red | White | Mulberry | Lime | Blue |

### Electricae (both element-based, Group 1)

| Species | Sb | Po | Ru | Tc | Te | Y |
|---------|----|----|----|----|----|----|
| Pluma (01) | Cobalt | Cyan | Blue | Magenta | Red | Mulberry |
| Radialem (02) | Cyan | Cobalt | Blue | Aquamarine | Magenta | Green |

### Fumerola (all element-based, Group 2)

| Species | Cd | Hg | Mo | Nb | W | Sn |
|---------|----|----|----|----|---|----|
| Carbosis (01) | Orange | Magenta | Gold | Cobalt | Yellow | Cyan |
| Extremus (02) | Aquamarine | Lime | Blue | White | Mulberry | Peach |
| Nitris (03) | White | Peach | Lime | Red | Aquamarine | Mulberry |
| Aquatis (04) | Green | Yellow | Cyan | Gold | Cobalt | Orange |

### Fungoida (all element-based)

| Species | Group | Key1 | Key2 | Key3 | Key4 | Key5 | Key6 |
|---------|-------|------|------|------|------|------|------|
| Setisis (01) | 1 | Sb:Peach | Po:White | Ru:Gold | Tc:Lime | Te:Yellow | Y:Orange |
| Stabitis (02) | 2 | Cd:Blue | Hg:Green | Mo:Magenta | Nb:White | W:Peach | Sn:Orange |
| Bullarum (03) | 1 | Sb:Red | Po:Mulberry | Ru:Magenta | Tc:Peach | Te:Gold | Y:Orange |
| Gelata (04) | 2 | Cd:Cyan | Hg:Lime | Mo:Mulberry | Nb:Green | W:Orange | Sn:Red |

### Concha Element Species

| Species | Group | Colors |
|---------|-------|--------|
| Renibus (01) | 2 | Cd:Red, Hg:Mulberry, Mo:Peach, Nb:Blue, W:White, Sn:Aquamarine |
| Biconcavis (04) | 1 | Sb:Peach, Po:Red, Ru:Orange, Tc:White, Te:Yellow, Y:Gold |

### Osseus Element Species

| Species | Group | Colors |
|---------|-------|--------|
| Discus (02) | 2 | Cd:White, Hg:Lime, Mo:Peach, Nb:Aquamarine, W:Red, Sn:Blue |
| Pumice (04) | 1 | Sb:White, Po:Peach, Ru:Gold, Tc:Lime, Te:Green, Y:Yellow |

### Recepta Element Species

| Species | Group | Colors |
|---------|-------|--------|
| Deltahedronix (02) | 2 | Cd:Lime, Hg:Cyan, Mo:Gold, Nb:Mulberry, W:Red, Sn:Orange |
| Conditivus (03) | 1 | Sb:Lime, Po:White, Ru:Yellow, Tc:Aquamarine, Te:Cyan, Y:Green |

---

## 5. Region Map & Restrictions

### Region Keys → Region IDs

```
orion-cygnus:            [1, 4, 7, 8, 16, 17, 18, 35]
orion-cygnus-1:          [4, 7, 8, 16, 17, 18, 35]
orion-cygnus-core:       [7, 8, 16, 17, 18, 35]
sagittarius-carina:      [1, 4, 9, 18, 19, 20, 21, 22, 23, 40]
sagittarius-carina-core: [9, 18, 19, 20, 21, 22, 23, 40]
sagittarius-carina-core-9: [18, 19, 20, 21, 22, 23, 40]
scutum-centaurus:        [1, 4, 9, 10, 11, 12, 24, 25, 26, 42, 28]
scutum-centaurus-core:   [9, 10, 11, 12, 24, 25, 26, 42, 28]
outer:                   [1, 2, 5, 6, 13, 14, 27, 29, 31, 41, 37]
perseus:                 [1, 3, 7, 15, 30, 32, 33, 34, 36, 38, 39]
perseus-core:            [3, 7, 15, 30, 32, 33, 34, 36, 38, 39]
exterior:                [14, 21-29, 31, 34, 36-42]
center:                  [1, 2, 3]
```

### Region-Locked Species (key examples)

| Species | Region |
|---------|--------|
| Aleoida Spica | outer, perseus, scutum-centaurus |
| Aleoida Laminiae | orion-cygnus, sagittarius-carina |
| Cactoida Cortexum | orion-cygnus |
| Cactoida Lapis | sagittarius-carina |
| Cactoida Pullulanta | perseus |
| Cactoida Peperatis | scutum-centaurus |
| Tussock (7 species) | sagittarius-carina-core-9, perseus-core, orion-cygnus-core |
| Stratum Excutitus | orion-cygnus |
| Stratum Limaxus | scutum-centaurus-core |
| Tubus Conifer | perseus |
| Tubus Cavas | scutum-centaurus |
| Fungoida Stabitis | orion-cygnus |
| Osseus Fractus/Pellebantus | !perseus |
| Osseus Cornibus | perseus |

---

## 6. star_check() Logic

```
'A' → matches 'A' or 'A_BlueWhiteSuperGiant'
'B' → matches 'B' or 'B_BlueWhiteSuperGiant'
'F' → matches 'F' or 'F_WhiteSuperGiant'
'G' → matches 'G' or 'G_WhiteSuperGiant'
'K' → matches 'K' or 'K_OrangeGiant'
'M' → matches 'M', 'M_RedGiant', 'M_RedSuperGiant'
'D', 'C', 'W' → prefix match (DA, DB, DC all match 'D')
Everything else → exact match
```

---

## 7. Volcanism Matching Semantics

- `'None'` → body must have NO volcanism
- `'Any'` → body must HAVE some volcanism
- `['carbon', 'methane']` → substring match (`str.find()`)
- `['=rocky magma']` → exact match (prefix `=`)
- `['!nitrogen']` → NOT match (must have volcanism, but not this)

---

## 8. Porting Notes for Rust

### Suggested Rust Types

```rust
struct Ruleset {
    atmosphere: Option<AtmosphereConstraint>,
    atmosphere_component: Option<HashMap<String, f64>>,
    body_type: Option<Vec<String>>,
    min_gravity: Option<f64>,
    max_gravity: Option<f64>,
    min_temperature: Option<f64>,
    max_temperature: Option<f64>,
    min_pressure: Option<f64>,
    max_pressure: Option<f64>,
    volcanism: Option<VolcanismRule>,
    regions: Option<Vec<RegionConstraint>>,
    star: Option<Vec<StarMatch>>,
    parent_star: Option<Vec<String>>,
    main_star: Option<Vec<StarMatch>>,
}

enum VolcanismRule { Any, None, Match(Vec<VolcMatch>) }
enum VolcMatch { Contains(String), Exact(String), Not(String) }
enum RegionConstraint { Include(String), Exclude(String) }
enum AtmosphereConstraint { Any, Types(Vec<String>) }
```

### 25 Known Color Names

Amethyst, Aquamarine, Blue, Cobalt, Cyan, Emerald, Gold, Green, Grey, Indigo, Lime, Magenta, Maroon, Mauve, Mulberry, Ocher, Orange, Peach, Red, Sage, Teal, Turquoise, White, Yellow
