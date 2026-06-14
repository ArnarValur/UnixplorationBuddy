# Specification — Phase 3 Stellar Codex Hierarchy

## Overview
Differentiate primary star classes in the Trip Stellar Codex into detailed star types and subclasses (e.g. `F9 VAB` instead of just `F`), grouping them hierarchically under their base class in a tree-list representation within the TUI.

## Functional Requirements
1. **Differentiated Star Scanning:**
   - Ingest `Scan` star events.
   - If `body_id == 0`, format the specific subtype string using `{StarType}{Subclass}{ [Luminosity]}`.
   - For example: `"B"`, `9`, `"Vab"` -> `"B9 VAB"`.
   - If luminosity is empty, format as `{StarType}{Subclass}` (e.g. `"N0"`).
   - Write this string as the key in `trip.stellar_codex` and increment visits.

2. **Hierarchical Codex Presentation:**
   - In the Stellar Codex TUI tab, group all registered `stellar_codex` records by their non-digit base class (e.g. `"F"`, `"K"`, `"DA"`).
   - Calculate total visits for the base class as the sum of all its subtypes.
   - Sort base classes in descending order of total visits.
   - Within each base class, list the detailed subtypes sorted by visits descending.
   - Render base classes normally with total visits.
   - Render subtypes nested/indented using tree guide marks (`  ├─ ` and `  └─ `) and styled using `ELITE_DIM` to present a premium tree structure.
   - Avoid redundant nested rendering if a base class has only a single subtype that is identical to the base class (e.g. `"TTS"`).

## Acceptance Criteria
- Detailed star types like `F9 VAB` are correctly saved and serialized.
- The TUI displays base classes with aggregate visit counts in bold `COLOR_STAR` styling.
- Subtypes are rendered nested, indented, and dimmed (`ELITE_DIM`) below their base class.
- Redundant single children matching their base class (like `"TTS"`) are not duplicated/nested.
- Unit tests verify formatting and rendering correctness.
