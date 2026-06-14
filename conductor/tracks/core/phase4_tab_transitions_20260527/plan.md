# Implementation Plan — Phase 4 Automatic Tab Transitions

## Phase 1: Event Handling
- [x] Implement `StartJump` event ingestion arm inside `src/journal.rs`. (04d3e21)
- [x] If `StartJump` is a `Hyperspace` jump and a route is plotted (i.e. `app.plotted_route` is present and has waypoints), transition `app.active_tab` to `Tab::Route` (only in live `track_trip == true` mode). (04d3e21)
- [x] If `FSDJump` is processed, transition `app.active_tab` to `Tab::Bodies` (only in live `track_trip == true` mode). (04d3e21)

## Phase 2: Testing & Verification
- [x] Add unit tests for automatic tab transitions under live vs replay modes for `StartJump` and `FSDJump`. (04d3e21)
- [x] Run test suite to verify no regressions. (04d3e21)
