# Galactic Region Determination — Research Report

> Date: 2026-06-10
> Source: ExploData RegionMap.py, BioScan regions.py, elite-api-docs journal events

## TL;DR

Region is determined **purely from StarPos coordinates (x,y,z)** using a **2D run-length encoded bitmap lookup** (projecting onto X-Z plane). Y is ignored. No API needed. ~235KB lookup table. 100% offline.

---

## The Algorithm

```python
x0 = -49985
z0 = -24105

def findRegion(x, y, z):
    px = int((x - x0) * 83 / 4096)    # Galactic X → pixel X
    pz = int((z - z0) * 83 / 4096)    # Galactic Z → pixel Z
    # Y coordinate is IGNORED — 2D projection

    row = regionmap[pz]               # RLE-encoded row for this Z
    rx = 0
    for rl, pv in row:                # Walk run-length pairs
        if px < rx + rl:
            break
        rx += rl
    return (pv, regions[pv]) if pv != 0 else None
```

- Grid resolution: ~49.35 ly per pixel
- Bitmap: ~2048×2050 pixels
- Origin: `x0=-49985, z0=-24105`

## 42 Regions (IDs 1-42)

```
 1: Galactic Centre           2: Empyrean Straits        3: Ryker's Hope
 4: Odin's Hold               5: Norma Arm               6: Arcadian Stream
 7: Izanami                   8: Inner Orion-Perseus      9: Inner Scutum-Centaurus
10: Norma Expanse            11: Trojan Belt             12: The Veils
13: Newton's Vault           14: The Conduit             15: Outer Orion-Perseus
16: Orion-Cygnus Arm         17: Temple                  18: Inner Orion Spur
19: Hawking's Gap            20: Dryman's Point          21: Sagittarius-Carina Arm
22: Mare Somnia              23: Acheron                 24: Formorian Frontier
25: Hieronymus Delta         26: Outer Scutum-Centaurus  27: Outer Arm
28: Aquila's Halo            29: Errant Marches          30: Perseus Arm
31: Formidine Rift           32: Vulcan Gate             33: Elysian Shore
34: Sanguineous Rim          35: Outer Orion Spur        36: Achilles's Altar
37: Xibalba                  38: Lyra's Song             39: Tenebrae
40: The Abyss                41: Kepler's Crest          42: The Void
```

## Journal Events

| Event | Has Region? | Has StarPos? |
|-------|------------|-------------|
| `FSDJump` | ❌ | ✅ `StarPos: [x,y,z]` |
| `Location` | ❌ | ✅ `StarPos: [x,y,z]` |
| `CodexEntry` | ✅ `$Codex_RegionName_N;` | ❌ |
| `Scan` | ❌ | ❌ |

`CodexEntry` only fires on new discoveries — not usable as primary source.

## Rust Implementation Plan

1. Store `regions` name array (42 entries)
2. Port RLE bitmap data (~235KB Python → ~150KB Rust/binary)
3. Implement `find_region(x: f64, _y: f64, z: f64) -> Option<(u8, &'static str)>`
4. Store `region_map` groupings from BioScan's `regions.py` (19 named groups → sets of region IDs)

**Data source file:** `ExploData/explo_data/RegionMapData.py`

## Region Groups (for bio filtering)

```
orion-cygnus:            [1, 4, 7, 8, 16, 17, 18, 35]
sagittarius-carina:      [1, 4, 9, 18, 19, 20, 21, 22, 23, 40]
scutum-centaurus:        [1, 4, 9, 10, 11, 12, 24, 25, 26, 42, 28]
outer:                   [1, 2, 5, 6, 13, 14, 27, 29, 31, 41, 37]
perseus:                 [1, 3, 7, 15, 30, 32, 33, 34, 36, 38, 39]
exterior:                [14, 21-29, 31, 34, 36-42]
center:                  [1, 2, 3]
```

(Plus sub-variants like `orion-cygnus-core`, `perseus-core`, etc.)
