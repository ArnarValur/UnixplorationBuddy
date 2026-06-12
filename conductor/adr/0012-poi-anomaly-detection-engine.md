# POI Anomaly Detection Engine

> **Recorded:** 2026-06-12 12:22
> **Status:** accepted

Ported EDMC-Canonn's `codex.py` anomaly detection algorithms to a pure Rust detection engine (`src/model/anomaly.rs`, `anomaly_extreme.rs`, `anomaly_jumponium.rs`). 11 per-body detectors run on every Scan event with zero external dependencies — all orbital math, classification, and extreme body checks computed from journal data alone. Results cached in `App.anomalies` HashMap and displayed in both the body table (POI column) and inspector panel (ANOMALIES/POI section). Chose pure-function detector architecture (`fn(bodies, results)`) over trait-based extensibility for simplicity and testability.

## Consequences

- Each new detector is a standalone function added to `detect_anomalies()` — trivial to extend.
- 19+ unit tests validate detection thresholds independently of TUI rendering.
- No API calls, no network — works fully offline in deep space.
