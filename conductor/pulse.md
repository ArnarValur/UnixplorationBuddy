# Pulse — Current Project State

**Last Updated:** 2026-05-27 10:27
**Session Focus:** Phase 2.5 Foundation Hardening — replay/trip fix, test suite, TUI testing

## 🚀 Active Tracks
- **phase1_bodies_history_20260527** — Phase 1 (Bodies + History). Status: `in_progress`. Phases 1–2.5 complete. Phase 3–6 remain.

## ✅ Recently Completed
- **Phase 2.5 Foundation Hardening** (`75b7ace`): Separated replay from trip accumulation, 22 process_event tests, 4 TUI rendering tests. Test suite 5→31.
- **Phase 2 Journal Ingestion** (`40dd102`): journal directory discovery, full session replay, live watcher thread, event processing for 7 event types, body table UI
- **Phase 1 Bootstrap** (`342e5c3`): cargo init, Cargo.toml with pinned deps, data model (System, Body, ScanState, BodyHierarchy, Trip), Ratatui TUI skeleton
- Conductor resumed, index reconciled (adr/, docs/ links added)
- **Grill session:** Full tech stack pivot from Python/EDMC-plugin to Rust/Ratatui standalone TUI

## ⚠️ Blockers
_None._

## 🧠 Session Memory
- **2026-05-27 (Phase 2.5 session):** User-driven design correction: trip should NOT accumulate from full journal history. A trip starts when the TUI opens, like Windows Exploration Buddy. Implemented via `track_trip: bool` on `process_event()` — replay passes `false`, live passes `true`. _(operational)_
- **2026-05-27 (Phase 2.5 session):** Test fixture pattern established: embed real journal JSON as `const &str`, parse with `serde_json::from_str::<LogEvent>()`. Constructing ed-journals structs programmatically is impractical (too many nested fields). TUI tests use `render_to_string()` via Ratatui TestBackend. _(operational)_
- **2026-05-27 (Phase 2.5 session):** User confirmed live testing is available — ED runs on Ultra at 30% GPU on the workstation. Can ask user to launch TUI + game for integration testing. _(operational)_
- **2026-05-27 (grill session):** Major pivot — dropped EDMC plugin approach entirely. User drove the direction: Ratatui TUI, journal-first, second-monitor auto-updating app. Gravity column intentionally excluded from Phase 1. Bio species displayed as expandable rows (not tooltips). Canonn bio data ADR deferred — will re-grill when building the biology module. CETI repo (`carsonbfl/CETI`) noted as reference for journal monitoring + EDSM/Spansh/EDASTRO API patterns.
- **2026-05-27 (implementation session):** Built Phase 1 & 2 of the track. Own state layer on top of raw ed-journals events (crate's `state` module too opaque). Full journal replay (all files, not just current session) for complete trip accumulation. Background thread + mpsc channel for live watcher (matches crossterm blocking event loop). ed-journals body_id is `u8`, our model uses `u32` — upcast at boundary. Atmosphere rendered via `AtmosphereType` Debug fmt (no Display impl in crate). 5 unit tests passing, 0 errors, 4 dead-code warnings (expected).

## 📋 Next Session Suggestions
- Phase 3: Body hierarchy naming convention parser + exploration value calculation
- Phase 4: TUI polish — scrollable body table, column widths, color-coded body types
- Phase 5: Trip persistence (JSON to XDG data dir)
- Phase 6: Integration testing with real game session — user can launch ED + TUI for live testing
