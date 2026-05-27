# Pulse — Current Project State

**Last Updated:** 2026-05-27 10:52
**Session Focus:** Phase 3 — naming parser, value calculation, GitHub repo setup

## 🚀 Active Tracks
- **phase1_bodies_history_20260527** — Phase 1 (Bodies + History). Status: `in_progress`. Phases 1–2.5 complete, Phase 3 in progress. Phases 4–6 remain.

## ✅ Recently Completed
- **Phase 3 (partial):** Naming convention parser (19 tests), value calculation (14 tests), hierarchy sort by naming convention, value integration into process_event. Test suite 31→65.
- **Phase 2.5 Foundation Hardening** (`75b7ace`): Separated replay from trip accumulation, 22 process_event tests, 4 TUI rendering tests. Test suite 5→31.
- **Phase 2 Journal Ingestion** (`40dd102`): journal directory discovery, full session replay, live watcher thread, event processing for 7 event types, body table UI
- **Phase 1 Bootstrap** (`342e5c3`): cargo init, data model, Ratatui TUI skeleton
- **GitHub repo connected:** ArnarValur/UnixplorationBuddy, history cleaned (260MB codex purged), README rewritten

## ⚠️ Blockers
_None._

## 🧠 Session Memory
- **2026-05-27 (Phase 3 session):** Valuation module uses ed-journals `StarClass`/`PlanetClass` enums directly — avoids string-matching bugs from Display format differences (e.g. "Earth-like Body" vs "Earthlike body"). _(operational)_
- **2026-05-27 (Phase 3 session):** Odyssey/4.0+ mapping bonus always applied — game is on v4.0+, no need for a toggle. _(operational)_
- **2026-05-27 (Phase 3 session):** `conductor/canonn-data/*.json.gz` gitignored — 260MB file was blocking GitHub push. File stays local, not in repo. _(operational)_
- **2026-05-27 (Phase 3 session):** User confirmed live testing available — ED runs on Ultra at 30% GPU on PlutoII. Can ask to launch TUI + game for integration testing anytime. _(operational)_
- **2026-05-27 (Phase 2.5 session):** Trip semantics: trip starts on TUI launch, replay doesn't accumulate. Implemented via `track_trip: bool` on `process_event()`. _(operational)_
- **2026-05-27 (Phase 2.5 session):** Test fixtures: real journal JSON parsed via `serde_json::from_str::<LogEvent>()`. TUI tests use `render_to_string()` via Ratatui TestBackend. _(operational)_

## 📋 Next Session Suggestions
- **Finish Phase 3:** Wire SAAScanComplete efficiency tracking (probes_used <= efficiency_target), update plan.md
- **Phase 4:** TUI rendering polish — scrollable body table, column widths, color-coded body types, value display
- **Phase 5:** Trip persistence (JSON to XDG data dir)
- **Phase 6:** Integration testing with real game — user launches ED + TUI for live testing
