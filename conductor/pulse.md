# Pulse — Current Project State

**Last Updated:** 2026-05-27 17:42
**Session Focus:** Solved Flatpak sandbox TUI watcher freeze and implemented exobiology hybrid prediction fallback.

## 🚀 Active Tracks
_None._

## ✅ Recently Completed
- **Sandboxed Flatpak Watcher Fix (2026-05-27):** Replaced inotify-based LiveLogDirReader with a highly robust filesystem-polling log tailer, fully resolving TUI lockups inside sandboxed Flatpak container environments on Linux.
- **Hybrid Exobiology Predictor Fallback (2026-05-27):** Implemented SAA genus extraction and a relaxed matching fallback (matching strictly on genus, atmosphere, and planet class) to prevent blank exobiology panels when strict Canonn boundaries fail.
- **Spelling Normalization Fix (2026-05-27):** Standardized atmosphere normalization to treat 'sulphur' and 'sulfur' interchangeably, fixing exobiology matching failures on sulfur-dioxide planets throughout the galaxy.
- **Phase 2 — Navigation & Exobiology (2026-05-27):** Real-time target-syncing, exobiology predictions, trip codex logs, and route sync with EDSM streaming completely implemented and verified.
- **EDSM Dependency Added (2026-05-27):** Integrated stable `ureq = "3.3.0"` as the lightweight, synchronous API driver, compiling and verifying successfully.
- **Phase 2 Specs (2026-05-27):** Complete exobiology prediction, navigation, trip logbook, and target-syncing specifications defined and committed via a successful domain grill session.

## ⚠️ Blockers
_None._

## 🧠 Session Memory
- *2026-05-27* — Normalized 'sulphur' and 'sulfur' atmosphere spelling to prevent database matching failures on sulfur-dioxide planets. _(operational)_
- *2026-05-27* — Implemented hybrid exobiology prediction fallback using DSS/SAA genus extraction when strict boundaries fail. _(operational)_
- *2026-05-27* — Switched from inotify LiveLogDirReader to a filesystem-polling watcher loop to bypass Flatpak sandbox boundaries. _(operational)_
- **2026-05-27 (Phase 2 Resolution):** Sorted raw codex entries before mapping to Ratatui Row widgets to avoid private `cells` borrowing constraints. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Target-syncing will monitor the `Destination` object inside `Status.json` (System, Body, Name) to automatically focus and highlight the player's targeted body in the TUI. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Plotted route waypoints will be tracked by watching the `NavRoute.json` file in the journal folder. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Route EDSM API enrichment will be fetched asynchronously on a background worker thread using synchronous `ureq` (v3.3.0) to prevent TUI rendering locks. _(operational)_
- **2026-05-27 (Phase 2 Grill):** Exobiology predictions will match FSS `Scan` physical parameters against Canonn distribution boundaries, only compiling and checking if FSS discovery scan detects bio signals > 0 to keep the display clean. _(operational)_

## 📋 Next Session Suggestions
- **Automatic Tab Transitions:** Implement automatic TUI tab switching to the Route tab when the ship begins an FSD jump (detecting `StartJump` / `FSDTarget` events), and automatically hop back to the Bodies tab when arrival/landing in the destination system is complete.
- **Further Exobiology Panel Tweaks:** Refine exobiology panel layout and detailed telemetry displays based on live session feedback.
