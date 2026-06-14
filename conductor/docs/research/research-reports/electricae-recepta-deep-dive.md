# Electricae & Recepta — Deep Dive Research Report

> **Author:** Hermes (research subagent) for CMDR ADDINATOR
> **Date:** 2026-06-13
> **Sources:** Canonn JSON/MD dataset, BioScan ruleset reference, Canonn.science, community research

---

## Quick Reference

| Species | Body Type | Atmosphere | Temp (K) | Gravity | Value (CR) | Determinant | Rarity |
|---------|-----------|------------|----------|---------|------------|-------------|--------|
| **Electricae Pluma** | Icy (100%) | Thin Argon/Neon | 20–150 | 0.03–0.28G | 6,284,600 | Material (G1) | 27% of genus |
| **Electricae Radialem** | Icy (100%) | Thin Argon | 30–149 | 0.03–0.27G | 6,284,600 | Material (G1) | 73% of genus |
| **Recepta Umbrux** | Icy (46%), Rocky (37%) | Thin SO₂ (87%) / CO₂ (13%) | 132–273 | 0.04–0.28G | 12,934,900 | Stellar Class | 52% of genus |
| **Recepta Conditivus** | Icy (72%), Rocky (15%) | Thin SO₂ (96%) / CO₂ (4%) | 132–273 | 0.04–0.28G | 14,313,700 | Material (G1) | 30% of genus |
| **Recepta Deltahedronix** | Rocky (80%), Icy (11%) | Thin SO₂ (75%) / CO₂ (25%) | 132–272 | 0.04–0.28G | 16,202,800 | Material (G2) | 18% of genus |

---

## Electricae

### Genus Overview

Superconductive organisms found **exclusively on extremely cold ice worlds** near frozen lakes. The visible tips protrude from the ice near fissures where it is thinnest. The bulk extends below the surface into subsurface melt, potentially for **several kilometres**.

They use thermal circulation of surrounding fluid to drive an electrochemical process — hence the limitation to noble gas atmospheres (Argon/Neon). The surface structure creates a point of electrical potential difference, producing bioluminescent displays.

- **Min Colonial Separation:** 1,000m *(largest of all Odyssey genera!)*
- **Region Restrictions:** None — found galaxy-wide
- **Reproduction:** Presumably occurs below the surface by unidentified process

---

### Electricae Pluma (27% of genus)

> *Extends a tip of four connected loops above the ice, each covered with brightly luminous fronds.*

**Vista Genomics:** 6,284,600 CR

| Condition | Detail |
|-----------|--------|
| **Body Type** | Icy (100%) |
| **Atmosphere** | Thin Argon (94%), Thin Argon-rich (5%), Thin Neon-rich (1%) |
| **Temperature** | Min 20.01K · Avg 80.35K · Mode 56.36K · Max 149.67K |
| **Pressure** | Min 0.000988 atm · Max 0.0987 atm |
| **Gravity** | Min 0.03G · Avg 0.19G · Mode 0.23G · Max 0.28G |
| **Volcanism** | Mostly none (94%). Tolerates Water Geysers, CO₂ Geysers, Nitrogen Magma |
| **⭐ Star Requirement** | **A-class, White Dwarf, Neutron Star, Black Hole, or Herbig Ae/Be** |
| **Nebula** | Not required |

**Variant Colors (Material → Color):**

| Antimony | Polonium | Ruthenium | Technetium | Tellurium | Yttrium |
|----------|----------|-----------|------------|-----------|---------|
| Cobalt | Cyan | Blue | Magenta | **Red** ✓ | Mulberry |

> ✓ = CMDR ADDINATOR's first discovery variant

**Key insight:** Pluma is the *rarer* Electricae (27%) but *easier to hunt* because you just need the right star class — no nebula dependency. Target A/WD/Neutron systems and scan their icy moons.

---

### Electricae Radialem (73% of genus)

> *Protrudes bioluminescent stalks that radiate out in all directions. Evidence of uncalibrated link with proximity of nebulae.*

**Vista Genomics:** 6,284,600 CR

| Condition | Detail |
|-----------|--------|
| **Body Type** | Icy (100%) |
| **Atmosphere** | Thin Argon (96%), Thin Argon-rich (3%), Thin Neon-rich (<1%) |
| **Temperature** | Min 29.72K · Avg 70.38K · Mode 54.71K · Max 149.24K |
| **Pressure** | Min 0.000987 atm · Max 0.098 atm |
| **Gravity** | Min 0.03G · Avg 0.15G · Mode **0.05G** · Max 0.27G |
| **Volcanism** | Mostly none (96%). Tolerates Water Geysers, CO₂ Geysers, Nitrogen Magma |
| **⭐ Star Requirement** | **Any star class** — M dwarfs dominate (60%), K (21%), F/G also common |
| **⚠️ Nebula** | **REQUIRED — must be within ~100–150 LY of any nebula** (incl. planetary nebulae) |

**Variant Colors (Material → Color):**

| Antimony | Polonium | Ruthenium | Technetium | Tellurium | Yttrium |
|----------|----------|-----------|------------|-----------|---------|
| Cyan | Cobalt | Blue | Aquamarine | Magenta | Green |

**Key insight:** More common (73% of Electricae) but harder to find because of the **nebula proximity requirement**. Gravity skews much lower (mode 0.05G) — look for tiny icy moons near nebulae.

---

### Pluma vs Radialem — Decision Matrix

| Factor | Pluma | Radialem |
|--------|-------|----------|
| Star class | A / WD / Neutron / BH / Ae required | Any (M dwarfs dominate) |
| Nebula | ❌ Not required | ✅ **REQUIRED** (~100–150 LY) |
| Gravity mode | 0.23G (heavier bodies) | 0.05G (lighter bodies) |
| Population | 27% (rarer) | 73% (more common) |
| Temp mode | 56K | 55K |
| Hunting strategy | Find exotic stars → scan icy moons | Navigate near nebulae → scan icy moons |

---

## Recepta

### Genus Overview

Extremophiles building **shielding bubbles** from inorganic and hydrocarbon materials, creating an isolated biome with regulated temperature and chemical composition. Growth requires careful melting, regrowing and freezing of the external shell — meaning **larger Recepta are very old**.

Reproduction via **budding** — creates a smaller version that rolls under gravity/wind before deploying a holdfast at its final position.

- **Min Colonial Separation:** 150m *(very short — easy to sample)*
- **Region Restrictions:** None
- **Terrain Preference:** Flat, slightly undulating, or gently sloping
- **Dual Determinant Genus:** Umbrux = Stellar Class; Conditivus & Deltahedronix = Material

---

### Recepta Umbrux (52% of genus — Stellar Class Determinant)

> *Grows a thick latticed structure for protection. Fine translucent membrane stretches between gaps, allowing sunlight for photosynthesis.*

**Vista Genomics:** 12,934,900 CR

| Condition | Detail |
|-----------|--------|
| **Body Type** | Icy (43–46%), Rocky (37%), Rocky Ice (13–17%), HMC (3%) |
| **Atmosphere** | Thin Sulphur Dioxide (87%), Thin Carbon Dioxide (13%) |
| **Temperature** | Min 132K · Avg 170K · Mode 134K · Max 273K |
| **Pressure** | Min 0.000987 atm · Max 0.0986 atm |
| **Gravity** | Min 0.04G · Avg 0.22G · Mode 0.27G · Max 0.28G |
| **Volcanism** | Insignificant — vast majority no volcanism |
| **Star Class** | Any — M dwarfs dominate (~79%) |

**Variant Colors (Stellar Class → Color):**

| B | A | F | G | K | M | L | T | TTS | Ae | Y | D | N |
|---|---|---|---|---|---|---|---|-----|---|---|---|---|
| Turquoise | Amethyst | Mauve | Orange | Red | Maroon | Ocher | Teal | Sage | **Grey** | **Lime** | **Yellow** | Emerald |

> ⚠️ **Ultra-rare variants:** Grey (Ae — only 3 known!), Lime (Y — only 9 known), Yellow (D — only 8 known)

---

### Recepta Conditivus (30% of genus — Material Group 1)

> *Body suspended inside a sphere-shaped translucent membrane filled with chemical-rich fluid. Chemical exchange via membrane and root structure.*

**Vista Genomics:** 14,313,700 CR

| Condition | Detail |
|-----------|--------|
| **Body Type** | **Icy (72%)**, Rocky (15%), Rocky Ice (12%), HMC (1%) |
| **Atmosphere** | Thin Sulphur Dioxide (96%), Thin Carbon Dioxide (4%) |
| **Temperature** | Min 132K · Avg 178K · Mode 134K · Max 273K |
| **Pressure** | Min 0.000987 atm · Max 0.0984 atm |
| **Gravity** | Min 0.04G · Avg 0.23G · Mode 0.23G · Max 0.28G |
| **Volcanism** | Insignificant |

**Variant Colors (Material → Color):**

| Antimony | Polonium | Ruthenium | Technetium | Tellurium | Yttrium |
|----------|----------|-----------|------------|-----------|---------|
| Lime | White | Yellow | Aquamarine | Cyan | Green |

**Key insight:** The **icy body specialist** of the Recepta family — 72% on ice. If you're scanning an icy world with Thin SO₂, Conditivus is likely.

---

### Recepta Deltahedronix (18% of genus — Material Group 2)

> *Produces a thick lattice of trunks in a deltahedron shape around the central organism. Captures and focuses geothermal heat for thermosynthesis.*

**Vista Genomics:** 16,202,800 CR *(highest value Recepta!)*

| Condition | Detail |
|-----------|--------|
| **Body Type** | **Rocky (80%)**, Icy (11%), HMC (7%), Rocky Ice (2%) |
| **Atmosphere** | Thin Sulphur Dioxide (75%), Thin Carbon Dioxide (25%) |
| **Temperature** | Min 132K · Avg 152K · Mode 134K · Max 272K |
| **Pressure** | Min 0.000987 atm · Max 0.0984 atm |
| **Gravity** | Min 0.04G · Avg 0.19G · Mode 0.27G · Max 0.28G |
| **Volcanism** | Low numbers with volcanism — tolerance not preference |

**Variant Colors (Material → Color):**

| Cadmium | Mercury | Molybdenum | Niobium | Tin | Tungsten |
|---------|---------|------------|---------|-----|----------|
| Lime | Cyan | Gold | Mulberry | Orange | Red |

**Key insight:** The **rocky body specialist** — 80% on rocky worlds. Rarest Recepta (18%) and most valuable (16.2M CR). Higher CO₂ atmosphere representation (25% vs 4–13% for siblings). Uses a **different material group** (Cadmium/Mercury/Molybdenum/Niobium/Tin/Tungsten) than Conditivus.

---

## Hunting Guide

### Finding Electricae

1. **For Pluma:** Jump to systems with **A-class, White Dwarf, or Neutron** primary stars. Scan icy bodies with Thin Argon atmosphere, gravity ≤0.27G. No nebula needed
2. **For Radialem:** Navigate to within **~100 LY of any nebula**. Scan icy bodies with Thin Argon atmosphere, gravity ≤0.27G, around ANY star class
3. **On surface:** Both species are near **frozen lakes and fissures**. Bioluminescent — **easier to spot on the dark side** of the planet. Cover ground near lake boundaries
4. **Separation warning:** 1,000m between samples — **largest of all biologicals**. Plan your SRV routes accordingly

### Finding Recepta

1. **All species:** Target planets with **Thin Sulphur Dioxide** atmosphere, gravity ≤0.27G, temp 132–270K
2. **Umbrux (most common):** Any star class works. M dwarfs = Maroon (most common variant)
3. **Conditivus:** Same SO₂ conditions. **Prefers icy bodies** (72%). Check surface materials for color prediction
4. **Deltahedronix (rarest, highest value):** Focus on **rocky bodies** with Thin SO₂ — 80% of findings. Higher CO₂ atmosphere representation
5. **Terrain:** **Flat, undulating, or gently sloping** areas. They like to roll after budding
6. **Separation:** Only 150m — very short, quick to sample three

---

## Data Gaps

| Gap | Notes |
|-----|-------|
| Radialem nebula distance cutoff | Community-estimated ~100–150 LY, not officially confirmed by Frontier |
| Umbrux Grey (Ae star) | Only **3 known observations** — extremely rare, needs more data |
| Umbrux Lime (Y dwarf) | Only **9 known observations** |
| Umbrux Yellow (D white dwarf) | Only **8 known observations** |
| Max pressure hard-limit | All observations under ~0.098 atm, but no confirmed hard cap |
| Electricae reproduction | "Presumably occurs below the surface by some unidentified process" — unresearched |

---

## Sources

- `conductor/canonn-data/electricae.md` — Canonn species profile
- `conductor/canonn-data/Electricae.json` — Canonn observation dataset
- `conductor/canonn-data/Recepta.md` — Canonn species profile
- `conductor/canonn-data/Recepta.json` — Canonn observation dataset
- `conductor/docs/research/research-reports/bioscan-ruleset-reference.md` — BioScan prediction rules
- `conductor/docs/research/research-reports/bio-variant-prediction.md` — Variant prediction logic
- `conductor/docs/research/research-reports/canonn-data-analysis.md` — Dataset analysis
- [Canonn.science/codex/electricae](https://canonn.science/codex/electricae/)
- [Canonn.science/codex/recepta](https://canonn.science/codex/recepta/)
- [Canonn Bioforge](https://canonn-science.github.io/bioforge)
- Community research (Reddit, Frontier Forums)
