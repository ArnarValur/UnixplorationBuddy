# Implementation Plan — Phase 4 Automatic Tab Transitions

## Phase 1: Event Handling
- [ ] Implement `StartJump` event ingestion arm inside `src/journal.rs`.
- [ ] If `StartJump` is a `Hyperspace` jump and a route is plotted (i.e. `app.plotted_route` is present and has waypoints), transition `app.active_tab` to `Tab::Route` (only in live `track_trip == true` mode).
- [ ] If `FSDJump` is processed, transition `app.active_tab` to `Tab::Bodies` (only in live `track_trip == true` mode).

## Phase 2: Testing & Verification
- [ ] Add unit tests for automatic tab transitions under live vs replay modes for `StartJump` and `FSDJump`.
- [ ] Run test suite to verify no regressions.
