# Canonn Exobiology Data Analysis

> Date: 2026-06-10
> Source: Canonn bio JSON/MD files + comparison with our predictor

## 1. JSON Schema (Identical Across All Genera)

Confirmed identical schema across **Bacterium.json**, **Aleoida.json**, **Electricae.json**, **Fumerola.json**. Each JSON file is a dict keyed by codex entry ID (e.g., `"2320101"`). Each entry has:

### Top-Level Fields
| Field | Type | Description |
|---|---|---|
| `name` | string | `"Bacterium Aurasus - Turquoise"` — species + variant color |
| `id` / `fdevname` | string | FDev codex reference — **encodes variant determinant** (star class letter or material name) |
| `hud_category` | string | Always `"Biology"` |
| `count` | int | Number of recorded observations |
| `reward` | int | Vista Genomics payout in credits |
| **Body conditions** | | |
| `bodies` | string[] | Planet classes: `["Rocky body", "High metal content world"]` |
| `atmosphereType` | string[] | Full atmosphere strings: `["Thin Carbon dioxide"]` |
| `atmosComposition` | string[] | Gas names: `["Carbon dioxide", "Sulphur dioxide"]` |
| `volcanism` | string[] | `["No volcanism", "Rocky Magma"]` |
| `solidComposition` | string[] | `["Ice", "Metal", "Rock"]` |
| **Star data** | | |
| `primaryStars` | string[] | System primary star classes where found |
| `localStars` | string[] | Nearby/local star classes (companion stars) |
| `systemBodyTypes` | string[] | All body types in systems where found |
| **Boundary ranges** | | |
| `ming`, `maxg` | float | Gravity range (in G) |
| `mint`, `maxt` | float | Temperature range (in K) |
| `minp`, `maxp` | float | Pressure range (in atm) |
| `mind`, `maxd` | float | Distance from Sol range (in ly) |
| **Materials** | | |
| `materials` | string[] | Top body surface materials (by frequency) |
| `regions` | string[] | Galaxy regions where found |

---

## 2. Bacterium Variant Mapping (Dual Determinant)

### Stellar Class Determinant Species (3 species):
- Bacterium Alcyoneum (Ammonia atmo)
- Bacterium Aurasus (CO₂ atmo)
- Bacterium Cerbrus (SO₂/Water atmo)

**Star → Color mapping** (shared across all stellar-class Bacterium):

| Star | O | B | A | F | G | K | M | L | T | TTS | Ae | Y | W | D | N |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Color | Tu | Gy | Ye | Li | Em | Gr | Te | Sg | Re | Mr | Or | Mv | Am | Oc | In |

### Material Determinant Species (10 species):

**Group 1** (Antimony/Polonium/Ruthenium/Technetium/Tellurium/Yttrium):
Acies, Bullaris, Informem, Nebulus, Vesicula, Volu

**Group 2** (Cadmium/Mercury/Molybdenum/Niobium/Tin/Tungsten):
Omentum, Scopulum, Tela, Verrata

> Each species has its OWN unique material→color mapping — not consistent across species.

### Min Colonial Separation: **500m**
### Region Restrictions: **None** — All Bacterium species show "No Preference"

---

## 3. Cross-Genus Summary

| Genus | Det. Type | Colonial Sep | Region-Locked? |
|---|---|---|---|
| **Aleoida** | Stellar | 150m | YES — Laminiae & Spica |
| **Bacterium** | **Dual** | 500m | No |
| **Cactoida** | Stellar | 300m | YES — Every species |
| **Clypeus** | Stellar | — | Speculumi: distance≥2000ls |
| **Concha** | **Dual** | 150m | No |
| **Electricae** | Material | 1000m | No (Radialem needs nebula) |
| **Fonticulua** | Stellar | — | No |
| **Frutexa** | Stellar | — | YES — Acus, Fera, Flammasis, Flabellum |
| **Fumerola** | Material | 100m | No |
| **Fungoida** | Material (all 4) | — | YES — Stabitis=OC, Gelata=!OC-core |
| **Osseus** | **Dual** | 800m | YES — Fractus/Pellebantus=!P, Cornibus=P |
| **Recepta** | Mixed (1 star, 2 mat) | — | No |
| **Stratum** | Stellar | 500m | YES — Many species |
| **Tubus** | Stellar | — | YES — Conifer, Cavas, Compagibus |
| **Tussock** | Stellar | 200m | YES — Most species |

### Consistent Patterns:
1. Star→Color mapping is **genus-wide** — same star class → same color within a genus
2. Material→Color mapping is **per-species**
3. Two material groups: Group 1 (Sb/Po/Ru/Tc/Te/Y) and Group 2 (Cd/Hg/Mo/Nb/Sn/W)
4. Region locking is **per-species**, not per-genus
5. Volcanism matters — especially for Fumerola and some Bacterium material species

---

## 4. What We Use vs. Ignore in Our Predictor

### Currently Checked ✅
- `atmosphere_types` — atmosphere matching
- `bodies` — planet class matching
- `primary_stars` — star class matching
- `min_g`/`max_g` — gravity range
- `min_t`/`max_t` — temperature range

### Stored but NOT Checked ❌
- `min_p`/`max_p` — pressure range (Body lacks pressure field)
- `volcanism` — (Body lacks volcanism field)

### Completely Ignored from Canonn Data
| Field | Impact | Priority |
|---|---|---|
| **`regions`** | 🔴 **P0** — Region locking is HARD constraint for Tussock, Cactoida, Aleoida, Stratum, Osseus |
| **`volcanism`** | 🔴 **P0** — Fumerola REQUIRES specific volcanism types |
| **Pressure** | 🟡 **P1** — Body struct lacks pressure field |
| **`materials`** | 🟡 **P1** — Narrows material-determinant variants to 1 |
| **`atmosComposition`** | 🟢 P2 — Mostly redundant with atmosphereType |
| **`localStars`** | 🟢 P2 — Not hard constraints |

### Missing from Body Struct
- `volcanism: Option<String>` — Journal provides `"Volcanism"` in Scan events
- `pressure: Option<f64>` — Journal provides `"SurfacePressure"`
- `surface_materials: Vec<String>` — Journal provides `"Materials"` array
- `region: Option<String>` — Available from `FSDJump`/`Location` events

---

## 5. Variant Determinant Summary (All Genera)

| Genus | Det. Type | Star Table | Material Group | Species Count |
|---|---|---|---|---|
| Aleoida | Stellar | ✅ | — | 5 species |
| Bacterium | **Dual** | ✅ (3) | Both (10) | 13 species |
| Cactoida | Stellar | ✅ | — | 5 species |
| Clypeus | Stellar | ✅ | — | 3 species |
| Concha | **Dual** | ✅ (2) | Both (2) | 4 species |
| Electricae | Material | — | Group 1 | 2 species |
| Fonticulua | Stellar | ✅ | — | 6 species |
| Frutexa | Stellar | ✅ | — | 7 species |
| Fumerola | Material | — | Group 2 | 4 species |
| Fungoida | Stellar/Mat | ✅ | Both | 4 species |
| Osseus | **Dual** | ✅ (4) | Both (2) | 6 species |
| Recepta | Stellar | ✅ | — | 3 species |
| Stratum | Stellar | ✅ | — | 8 species |
| Tubus | Stellar | ✅ | — | 5 species |
| Tussock | Stellar | ✅ | — | 15 species |

**Material Group 1**: Antimony, Polonium, Ruthenium, Technetium, Tellurium, Yttrium
**Material Group 2**: Cadmium, Mercury, Molybdenum, Niobium, Tin, Tungsten
