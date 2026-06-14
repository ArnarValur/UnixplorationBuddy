# State Snapshot Persistence for Instant Cold Start

> **Recorded:** 2026-06-10 11:35
> **Status:** accepted

On graceful exit (`q`), the app serializes the current system, bodies, display order, and journal replay position to `~/.local/share/unixploration-buddy/state.json`. On startup, this snapshot is loaded first for immediate display, then only new journal events (since the saved position) are replayed. This eliminates the 5000+ event full replay on every cold start, making restarts feel seamless.

Body, System, ScanState, and BodyType structs now derive `serde::Serialize`/`serde::Deserialize` to support this. The `ed_journals` crate types (`PlanetClass`, `StarClass`) already had serde support.
