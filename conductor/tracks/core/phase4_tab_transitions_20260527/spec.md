# Specification — Phase 4 Automatic Tab Transitions

## Overview
Implement automatic TUI tab transitions that improve commander situational awareness during flight. The TUI should automatically switch to the Route view when starting a hyperspace jump to another system (if a route is plotted) and return to the Bodies view when arriving in the new system.

## Functional Requirements
1. **Jump Start Transition:**
   - Ingest the `StartJump` journal event inside `process_event()`.
   - If the jump is a `Hyperspace` jump (targeting another system):
     - Check if there is a plotted route in `app.plotted_route` with at least one waypoint.
     - If true, transition `app.active_tab` to `Tab::Route`.
   - Ensure this transition is only triggered in live mode (`track_trip == true`) to prevent flashing tabs during startup journal replays.

2. **System Arrival Transition:**
   - When a system transition is completed via the `FSDJump` event:
     - Automatically transition `app.active_tab` back to `Tab::Bodies` (only in live mode `track_trip == true`).
     - This ensures the commander sees the newly entered system's FSS discovery progression immediately upon arrival.

## Acceptance Criteria
- Starting a hyperspace jump with an active navigation route automatically focuses the Route tab in live mode.
- Completing the jump and entering the destination system automatically returns the focus to the Bodies tab in live mode.
- Replaying old journal logs during startup does not trigger tab switches.
- Unit tests verify the tab transition logic.
