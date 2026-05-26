# Journal files as primary data source

> **Recorded:** 2026-05-27 00:10
> **Status:** accepted

Elite Dangerous writes all game events as JSON records to player journal files (`Journal.YYYY-MM-DDThhmmss.log`). UnixplorationBuddy reads these files directly using the `ed_journals` Rust crate instead of depending on EDMC or ExploData as intermediaries.

The journal contains everything needed for Phase 1: system jumps, body scans, FSS discoveries, DSS mappings, bio signal detections, and scan completion. EDSM, Spansh, and EDASTRO are relegated to supplemental HTTP APIs for enrichment data (discoverer names, visitation status) in Phase 2+.

This eliminates the EDMC runtime dependency entirely and makes the app self-contained — it only needs access to the journal directory.

## Consequences

- Must handle journal file discovery (path varies by Steam/native/Proton install)
- Must implement a file watcher for live updates (inotify on Linux)
- Must parse journal JSON events (handled by `ed_journals` crate)
- No dependency on EDMC running simultaneously
