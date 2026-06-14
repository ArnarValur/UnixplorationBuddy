# Enhancement Backlog

> Filed: 2026-05-30
> Status: Parked — revisit tonight or tomorrow
> Source: User notes from public release audit session

---

## 1. Planet Subtype in Type Column

**Problem:** The Type column shows generic "Planet" for all planetary bodies. Should show the actual planet class — Ice Planet, Gas Giant, High Metal Content, etc.

**Screenshots:** User attached a screenshot showing all bodies listed as "Planet" in a system with 9 planets.

**Preliminary proposal:**
- The journal `Scan` event already provides `PlanetClass` (e.g., `"Sudarsky class I gas giant"`, `"Icy body"`, `"High metal content body"`).
- The `Body` model likely already stores this data — it's used in the Planetary Codex.
- Map `PlanetClass` to short display labels for the Type column:
  - `"Sudarsky class I gas giant"` → `"Gas Giant I"`
  - `"Icy body"` → `"Icy World"`
  - `"High metal content body"` → `"HMC World"`
  - `"Earth-like body"` → `"Earth-like"`
  - etc.
- Same approach for stars: show spectral class (G2V, M3V) instead of just "Star".
- Column width may need to expand slightly — test with longest label.

**Effort:** Small — display-only change in the bodies table renderer.

---

## 2. Dead Keybindings in Footer (`s`, `i`)

**Problem:** The bottom keybindings bar shows `s: settings` and `i: toggle inspector` but they appear to do nothing (or are context-dependent and confusing).

**Preliminary proposal:**
- Audit which keybindings are actually wired in `main.rs` key handler.
- Either: wire them properly, or remove them from the footer if they're only active in certain contexts.
- Consider showing context-sensitive keybindings (only show what works on the current tab).

**Effort:** Small — audit + either fix bindings or update footer rendering.

---

## 3. Park the Orrery Map

**Problem:** The Orrery Map subtab was an experiment. User wants it hidden from the UI.

**Preliminary proposal:**
- Remove the "Orrery Map" subtab selector from the Bodies tab bottom bar.
- Keep the `orrery.rs` code in the codebase (don't delete) — it may return later.
- Remove or gate the Orrery rendering behind a feature flag or config toggle.
- Clean up any Orrery-specific keybindings (speed controls, etc.) from the active key handler.

**Effort:** Small — UI routing change, no logic deletion needed.

---

## 4. European Keyboard Layout + Left-Side Keybindings

**Problem:** Current keybindings assume US keyboard layout. European/Icelandic layouts have different key positions. All keybindings should be reachable from the left hand (right hand is on mouse/HOTAS).

**Preliminary proposal:**
- Map all interactive keybindings to the left-hand zone: `q`, `w`, `e`, `r`, `a`, `s`, `d`, `f`, `1-5`, `Tab`, `Esc`, `Shift`, `Ctrl`.
- Remove any right-side-only bindings (brackets, etc.).
- The PageUp/PageDown/-/+/= mapping for orrery speed was already layout-aware — apply the same principle to all bindings.
- Consider a keybinding reference that shows physical positions rather than key names.
- **User note:** Not specifically Icelandic — it's an Icelandic layout on a Norwegian/Scandinavian physical keyboard. Symbols like `þ`, `.`, `+`, `'`, `?`, `Æ` are next to Enter and hard to reach with left hand. Just needs left-hand-only shortcuts with no obscure symbols.

**Effort:** Medium — requires remapping and testing across layouts.

---

## 5. "Display Settings" Rename + Mouse-Expandable Sidebar

**Problem:** "Graphic Settings" is misleading for a TUI — should be "Display Settings" or "Column Settings". Also: can the inspector sidebar be mouse-expandable/draggable?

**Preliminary proposal:**
- Rename: straightforward string change.
- Mouse sidebar resize: Ratatui supports mouse events via crossterm's `EnableMouseCapture`. Feasible approach:
  - Track a `sidebar_width_percent` state variable (currently hardcoded at 40%).
  - Detect mouse drag on the divider column.
  - Clamp between 20%–60% range.
  - This is a nice-to-have, not a priority. Could also just use keyboard shortcuts (`[`/`]` or similar) to resize.

**Effort:** Rename = trivial. Mouse resize = medium (mouse event handling, state management).

---

## 6. Trip Stats Enrichment — System Type Breakdown

**Problem:** "Systems Visited: 779" is a flat number. Could be richer with hierarchical sub-counts.

**Preliminary proposal:**
- Under "Systems Visited", add indented sub-rows:
  ```
  Systems Visited           779
    ├─ Single star            412
    ├─ Binary                 287
    ├─ Trinary                 64
    └─ Quaternary+             16
  ```
- Data source: the journal `FSDJump` event contains the system's body count and star positions. Alternatively, count unique star `BodyID`s per system from `Scan` events.
- Store system multiplicity in the trip model (increment counters on FSDJump).
- Similarly for "Biosignals Detected: 365":
  - Clarify: is this counting bodies with bio signals, or total bio signal count across all bodies?
  - Could break down by genus: `Bacterium: 120, Stratum: 85, ...`
- Other enrichment ideas for trip stats:
  - Furthest system from Sol
  - Most valuable system discovered
  - Total distance jumped (Ly)
  - Average system value
  - Bodies mapped vs scanned ratio
  - First discoveries percentage

**Effort:** Medium — model changes to track sub-counts, display changes in history renderer.

---

## 7. Biological Codex — Species Variant Sub-Rows

**Problem:** "Aleoida Coronamus: 7" is a flat count. Would be richer with variant/color sub-rows.

**Preliminary proposal:**
- Under each species entry, show indented variant rows:
  ```
  Aleoida Coronamus              7
    ├─ Amethyst                   3
    ├─ Grey                       2
    └─ Teal                       2
  ```
- Data source: `ScanOrganic` journal events include the full species variant name (e.g., "Aleoida Coronamus - Amethyst").
- The trip model's codex tracking would need to store variant-level counts, not just species-level.
- Matches the existing tree-rendering pattern used in the Stellar Codex (subclass/luminosity indents).

**Effort:** Medium — trip model expansion + codex renderer update. Pattern already exists from stellar codex.

---

## 8. Planet Class Hierarchy — Attribute Sub-Rows

**Problem:** `Water World (🌍x36 │ 🪐x2 │ 🌿x6) 36` has the badges inline but could use vertical expansion.

**Preliminary proposal:**
- Expand each planet class into a tree:
  ```
  Water World                    36
    ├─ With rings                  2
    ├─ With life                   6
    └─ Terraformable              12
  ```
- **User note:** Water Worlds, Earth-likes, and Ammonia Worlds are never landable — don't show a "Landable" row for those classes. Only show attributes that are actually possible for each class.
- This uses vertical space more effectively and makes the badges scannable at a glance.
- The data is already tracked via the composite key encoding (ADR-0008: `PlanetClass|L|T|R|B`).
- Just a rendering change in the planetary codex table.

**Effort:** Small-medium — rendering only, data already available.

---

## 9. Route + Bodies Tab Merge → "System Overview"

**Problem:** Route Exploration tab feels empty. Bodies tab could be renamed "System Overview". Consider merging or enriching.

**User decision:** Merge both into one main tab with **two subtabs** (like the existing Planetary/Biosignal codex pattern):
- Subtab 1: **System Map** — current system bodies (the existing Bodies view)
- Subtab 2: **Route** — plotted navigation route with enriched columns

Switch between subtabs with the existing `a`/`d` or `←`/`→` pattern.

**Route columns to implement:**
  ```
  System Name | Star Class | Distance(Ly) | Scoopable | Status | Value(EDSM) | Valuable | Terraform | Landable | Discoverer
  ```
  - **Star Class:** primary star spectral class from EDSM or journal
  - **Distance:** jump distance in Ly
  - **Scoopable:** ⛽ indicator (already exists)
  - **Status:** EDSM exploration completion (already exists)
  - **Value (EDSM):** estimated system value (already exists)
  - **Valuable:** count of high-value bodies (already exists as 💰)
  - **Terraform:** count of terraformable worlds
  - **Landable:** count of landable bodies
  - **Discoverer:** CMDR name from EDSM (already exists)

- Most of this data is already fetched from EDSM — it's a matter of parsing additional fields from the response and adding columns.

**Effort:** Medium — tab restructure + EDSM response field extraction + column additions.

---

## Priority Grouping (Suggested)

### Quick Wins (single session)
1. Planet subtype in Type column (#1)
2. Dead keybinding audit (#2)
3. Park Orrery Map (#3)
4. Display Settings rename (#5 — rename only)

### Medium Effort (1–2 sessions)
5. Keybinding remapping (#4)
6. Planet Class sub-rows (#8)
7. Bio Codex variant sub-rows (#7)
8. Route tab enrichment (#9)

### Larger Scope
9. Trip stats hierarchical breakdown (#6)
10. Mouse-expandable sidebar (#5 — mouse part)

---

## 10. Stellar Codex — Highlight Current Primary Star Class

> Filed: 2026-06-10
> Source: User idea during live exploration session (screenshot: B8 IIIAB system)

**Problem:** The Stellar Codex lists all primary star classes with visit counts, but when you jump into a system it's hard to spot *which row just incremented*. You have to remember the previous count.

**Preliminary proposal:**
- On FSDJump/Location, resolve the current system's primary star class (already known — it's what we increment).
- In the Stellar Codex renderer, match the current primary star class row and apply a highlight style (e.g. bold + accent color on the visit count, or a `►` marker in the gutter).
- The subclass row (e.g. `B8 IIIAB`) should be highlighted, **and** the parent class summary row (e.g. `B  20`) could get a subtle indicator too.
- Highlight should clear/move on next FSDJump.

**Effort:** Small — rendering-only change. The current star class is already tracked in state.

---

## 11. Stellar Codex — Track Companion Stars (Multi-Star Systems)

> Filed: 2026-06-10
> Source: User idea during live exploration session (screenshot: B8 IIIAB, F8 VB, A9 VI triple system)

**Problem:** The Stellar Codex currently only tracks the *primary* star class per system visit. In multi-star systems (binaries, trinaries, etc.) the companion stars are invisible to the codex. The user wants a full picture of *all* stars encountered.

**Preliminary proposal — two approaches to consider:**

### Option A: Two-column layout (Primary + Companions)
```
Primary Star Class     Primary Visits    Companion Visits
B                      20                8
├─ B0 VZ               10                2
├─ B8 IIIAB            3                 —
...
```
- "Primary Visits" = existing count (unchanged).
- "Companion Visits" = stars seen as secondary/tertiary in a system (from `Scan` events where `BodyID > 0` or the star is not the arrival star).
- Pro: Keeps primary tally clean, adds companion data as a new dimension.
- Con: Adds column width pressure.

### Option B: Separate companion codex subtab
- Keep existing Stellar Codex as-is (primary only).
- Add a third codex subtab: **All Stars** (or "Companion Stars").
- This lists every star class encountered regardless of primary/companion role.
- Pro: No layout changes to existing codex. Clean separation.
- Con: Another subtab to navigate.

### Option C: Single merged view with role indicator
- One list, but each star class row shows `Primary: N / Companion: M / Total: N+M`.
- Pro: Everything in one view.
- Con: Busiest layout.

**Data source:** `Scan` journal events for stars include `StarType` and `DistanceFromArrivalLS`. Stars with `DistanceFromArrivalLS == 0` (or the lowest ID) are primary; the rest are companions. The trip model would need a second counter map (e.g. `companion_stellar_codex: HashMap<String, u32>`).

**Effort:** Medium — model expansion (new counter map) + journal parsing change (track non-primary star scans) + codex renderer update.

---

## Priority Grouping (Suggested)

### Quick Wins (single session)
1. Planet subtype in Type column (#1)
2. Dead keybinding audit (#2)
3. Park Orrery Map (#3)
4. Display Settings rename (#5 — rename only)
5. **Highlight current star class in Stellar Codex (#10)**

---

## 12. 🐛 Planetary Codex — "Has Life" Counts Look Inflated

> Filed: 2026-06-10
> Source: User observation during live session (screenshot: HMC 878 "Has Life" out of 950 total — that's 92%)

**Problem:** The "Has Life" sub-attribute counts in the Planetary Codex look way too high. Example: HMC shows 878/950 with life, Rocky Body 693/744. Life is *not* that common in Elite Dangerous — these numbers are suspect.

**Possible causes to investigate:**
1. **Journal parsing bug** — The `B` flag in the composite key (`PlanetClass|L|T|R|B`) might be triggered by the wrong condition. Is it checking for bio signals (`bio_signals > 0`) when it should check for something else? Bio signals ≠ "has life" in the traditional sense.
2. **Corrupted trip state** — The persisted `planetary_codex` HashMap may have accumulated bad data across sessions or journal replays.
3. **Double-counting** — Bodies might be counted multiple times (e.g., on both `Scan` and `FSSBodySignals` events).
4. **Definition mismatch** — "Has Life" might mean "has biological signals detected" (from `FSSBodySignals`) rather than "has confirmed life" (from `ScanOrganic`). If so, the label is misleading but the data might be correct — bio signals are actually quite common on landable bodies with atmospheres.

**Action:** Audit the `B` flag logic in the journal parser and the planetary codex increment path. Cross-check a few systems manually against EDSM or journal files.

**Effort:** Small investigation — may be a one-line fix or just a label clarification.

---

## Priority Grouping (Suggested)

### 🐛 Investigate
1. **"Has Life" count inflation (#12)** — verify data integrity

### Quick Wins (single session)
2. Planet subtype in Type column (#1)
3. Dead keybinding audit (#2)
4. Park Orrery Map (#3)
5. Display Settings rename (#5 — rename only)
6. **Highlight current star class in Stellar Codex (#10)**

### Medium Effort (1–2 sessions)
7. Keybinding remapping (#4)
8. Planet Class sub-rows (#8)
9. Bio Codex variant sub-rows (#7)
10. Route tab enrichment (#9)
11. **Companion star tracking (#11)**

### Larger Scope
12. Trip stats hierarchical breakdown (#6)
13. Mouse-expandable sidebar (#5 — mouse part)

---

## 13. Exobiology — Total Value Summary on Scan Completion

> Filed: 2026-06-11
> Source: User note during live exploration session (screenshot: Exobiology Predictions panel)

**Problem:** When scanning biological species on a body, the TUI shows individual credit values per species (e.g., `Bacterium Aurasus (Lime) : 5,000,000 cr`). After completing all 3 samples for every predicted species on a body, there's no summary of the combined payout. The user has to mentally add up 5–8 species values to know what the body was worth biologically.

**Preliminary proposal:**
- When all predicted species on a body have reached `[Completed]` status (all 3 genetic samples collected), display a **total value summary line** at the bottom of the Exobiology Predictions panel.
- Format example:
  ```
  — EXOBIOLOGY PREDICTIONS —
  Signals: 8 detected

  R ► Aleoida Coronamus - Teal [Completed]
  ► Bacterium Aurasus (Lime) : 5,000,000 cr (First) [500m]
  ► Bacterium Cerbrus (Lime) : 8,449,000 cr (First) [500m]
  ► Bacterium Tela (Yellow) : 9,745,000 cr (First) [500m]
  ► Bacterium Volu (Gold) : 38,873,500 cr (First) [500m]
  ► Concha Labiata (Grey) : 11,762,000 cr (First) [150m]
  ► Concha Renibus (Peach) : 22,862,000 cr (First) [150m]
  ► Frutexa Fera (Green) : 8,162,500 cr (First) [150m]

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Total: ~105,854,000 cr
  ```
- Could show a running subtotal as species are completed (not just at full completion), e.g. `Earned so far: 52,322,000 cr / ~105,854,000 cr est.`
- The per-species credit values are already known from the prediction engine. Summing them is trivial.
- Consider whether "First Discovery" bonus multiplier should be factored in (currently the values shown already include it when applicable).

**Effort:** Small — sum the displayed values, render one extra line. Data already available.

---

## 14. 🌌 ED-Specialized Small Language Model (Moonshot)

> Filed: 2026-06-11
> Source: User idea during live exploration session

**Idea:** Train a small transformer or RLM base model on vast amounts of Elite Dangerous data — journals, EDSM dumps, Canonn datasets, system/body naming conventions, bio rulesets, lore, etc. Use it as an embedded companion brain in UnixplorationBuddy for fun & useful things.

**Potential use cases:**
- **Smarter bio predictions** — learn patterns the rule engine can't express
- **System name pattern recognition** — predict what body types to expect from naming conventions
- **Route intelligence** — "this region tends to have high ELW density"
- **Lore-aware commentary** — flavor text about regions, phenomena, star types
- **Anomaly detection** — "this system has unusual characteristics"
- **Value estimation** — predict system value before full FSS scan

**Platform:** Saturn (ASUS Ascent GX10 — ARM64, 20-core, 122GB RAM, NVIDIA GB10) would be the training/inference box.

**Data sources to consider:**
- Player journal archives (anonymized)
- EDSM full galaxy dump
- Canonn bio/xeno datasets (already in `conductor/canonn-data/`)
- Spansh dumps
- Elite wiki / lore corpus

**Effort:** Large — research project. Data collection, model selection, training pipeline, inference integration.

---

## Priority Grouping (Updated)

### 🐛 Investigate
1. **"Has Life" count inflation (#12)** — verify data integrity

### Quick Wins (single session)
2. Planet subtype in Type column (#1)
3. Dead keybinding audit (#2)
4. Park Orrery Map (#3)
5. Display Settings rename (#5 — rename only)
6. **Highlight current star class in Stellar Codex (#10)**
7. **Bio scan total value summary (#13)**

### Medium Effort (1–2 sessions)
8. Keybinding remapping (#4)
9. Planet Class sub-rows (#8)
10. Bio Codex variant sub-rows (#7)
11. Route tab enrichment (#9)
12. **Companion star tracking (#11)**

### Larger Scope
13. Trip stats hierarchical breakdown (#6)
14. Mouse-expandable sidebar (#5 — mouse part)
