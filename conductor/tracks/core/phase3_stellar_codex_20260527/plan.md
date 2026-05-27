# Implementation Plan — Phase 3 Stellar Codex Hierarchy

## Phase 1: Ingestion Refinement
- [x] Parse specific star subtypes by combining `StarType`, `Subclass`, and `Luminosity` on `Scan` events in `src/journal.rs`. (3db929a)
- [x] Save the specific subtype string as the key in `trip.stellar_codex`. (3db929a)

## Phase 2: UI Grouping & Tree Rendering
- [x] Implement `get_main_class` helper in `src/ui/mod.rs` to extract non-digit prefixes (like `F` from `F9 VAB`). (b2a6ffb)
- [x] Construct the hierarchical groups in the TUI renderer, summing total visits for main classes and nesting detailed subtypes. (b2a6ffb)
- [x] Render the main class row with total count, followed by indented child rows using tree guide guides (`  ├─ ` and `  └─ `). (b2a6ffb)
- [x] Sort main classes by total visits descending, and subtypes within a class by individual visits descending. (b2a6ffb)

## Phase 3: Integration & Tests
- [x] Verify compilation and test suite correctness. (5cb2990)
- [x] Add unit tests for specific star class formatting and hierarchical Codex rendering. (5cb2990)
- [x] Perform integration verification using a mock/real journal session replay. (5cb2990)
