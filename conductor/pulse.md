# Pulse — Current Project State

**Last Updated:** 2026-06-12 17:04
**Session Focus:** Inspector telemetry sidebar layout rework

## 🚀 Active Tracks

_None active._

## ✅ Recently Completed

- **Inspector Sidebar Layout Rework (2026-06-12):** Restructured inspector from single monolithic Paragraph to multi-section Layout. Physical properties (left column) + Materials (right column) side-by-side using Ratatui horizontal Layout split. Orbital/EDSM/Anomalies/Exobiology full-width below. 165/165 tests.

- **POI Anomaly Detection + TUI Integration (2026-06-12):** Built anomaly detection engine from EDMC-Canonn codex.py. 11 detectors (close orbit, flypast, collision, trojan, satellite star, moon³, fast rotator, retrograde orbit, high eccentricity, extreme tilt). Jumponium Green System detection (Basic/Standard/Premium). Header bar repurposed as POI ticker. Inspector panel shows anomaly details. 165/165 tests. Commits up to `f8db7f8`.

- **/conductor-elite workflow + EDMC-Canonn research (2026-06-12):** Created `/conductor-elite` workflow. Cloned `canonn-science/EDMC-Canonn` for reference. Brainstormed topbar rework, POI integration, tourist anomaly detection. Commit `4fbc7b1`.

- **FSD Charging Route Tab Switch + Display Fixes (2026-06-11):** Status.json `Flags` bit 17 detection for early route tab switching. Element color resolver, atmosphere/volcanism fixes, diamond column removal. 129/129 tests.

- **Inspector Planetary Telemetry Enrichment (2026-06-10):** Full planetary stats in inspector. 3 new Body fields. 129/129 tests.

- **Stellar Tracking UI + Bio Signals Fix (2026-06-10):** Primary star highlighting, companion tracking, Bio Signals rename. 129/129 tests.

## ⚠️ Blockers

_None._

## 🧠 Session Memory

- _2026-06-12_ — **POI anomaly detection engine** — Ported 5 orbital anomaly detectors from EDMC-Canonn codex.py to pure Rust. Used subagent parallelism (2 builders) for Batch 2: 4 extreme body detectors + jumponium. All wired into `detect_anomalies()` and TUI (POI column + inspector). _(architectural, ADR-0012)_
- _2026-06-12_ — **Jumponium as system-level indicator** — Separate from per-body anomalies. Stored in `App.jumponium`, displayed as header badge. Basic/Standard/Premium grades with icons. _(architectural, ADR-0013)_
- _2026-06-12_ — **Header bar repurposed** — Removed body count + credits. Now shows system name, region, jumponium badge, anomaly ticker (icon×count). Aligns with exploration-first UX. _(architectural, ADR-0014)_
- _2026-06-12_ — **Neighborhood black hole scanning** — Investigated EDSM/Spansh APIs for nearby POI discovery. APIs are blind in deep unexplored space. Mass code heuristic (PG name → star type) noted as future track idea. No API can control in-game galaxy map. _(operational)_
- _2026-06-12_ — **NavRoute mass code scanner idea** — Parse plotted route system names, flag mass code `g`/`h` systems as likely black holes/neutrons. Zero API calls, pure name math. Noted in hunting checklist future ideas. _(operational)_
- _2026-06-12_ — **E2E validation: Extreme Tilt** — First anomaly confirmed in-game at `Flya Eom ES-A d1-1` body 1 (HMC, 164° axial tilt). Header POI ticker, inspector ANOMALIES/POI section both rendering correctly. _(operational)_
- _2026-06-12_ — **Nearest carrier/station search research** _(elite)_ — Inara API has no search endpoint (push-only). Spansh `POST /api/stations/search` with `type: Fleet Carrier` + `reference_system` works but needs known system. Inara web scrape (`GET /elite/nearest-stations/?formbrief=1&ps1=SYSTEM&pi16=39`) returns server-rendered HTML with carrier IDs, systems, distances — parseable, no auth. Both fail in deep unexplored space (unknown systems). Thought experiment only, no track planned. _(operational)_
- _2026-06-12_ — **EDDN upload feasibility** _(elite)_ — Standalone apps CAN upload to EDDN (HTTP POST + zlib to `eddn.edcd.io:4430/upload/`). No registration needed. UB already has journal parsing + `ureq`. Would need schema transform, PII stripping, `gameversion`/`gamebuild` from Fileheader. EDSM also accepts direct uploads (`api-journal-v1` with API key). ~300-500 lines Rust total. Noted as future possibility. _(operational)_
- _2026-06-12_ — **Inspector sidebar side-by-side layout** — Restructured from single Paragraph to Layout-based rendering. Physical props left column, Materials right column. Block rendered separately, inner area split with Layout::vertical + Layout::horizontal. Bodies without materials fall back to full-width physical. Stars skip side-by-side entirely. _(operational)_

## 📋 Next Session Suggestions

- **E2E anomaly testing:** Jump to systems and verify detection triggers in the TUI.
- **NavRoute mass code scanner:** Parse route waypoints, flag `g`/`h` mass code systems.
- **Inspector jumponium drill-down:** Show which body to land on for each material.
- **Ring analysis detectors:** Need ring data on Body struct first, then large/small/dense ring detection.
- **Star inspector enrichment:** Luminosity, stellar mass, age, absolute magnitude.
- **README Rewrite:** User to write README in human words.
