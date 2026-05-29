# Implementation Plan — System Orrery TUI View

## Phase 1: Model & State Extensions
- [x] Add orbital parameter fields to the `Body` struct in `src/model/body.rs`. (3fd794c)
- [x] Update `process_event` inside `src/journal.rs` to parse orbital parameters from `Scan` events. (3fd794c)
- [x] Define the `BodiesSubTab` enum in `src/app.rs` with `Table` and `Orrery` variants. (3fd794c)
- [x] Add active subtab, simulated time, and speed multiplier state to `App` in `src/app.rs`. (3fd794c)

## Phase 2: Keyboard Inputs & Ticking
- [x] Modify `src/main.rs` input handler: (3fd794c)
  - Toggle `app.bodies_subtab` using `a`/`d` or `Left`/`Right` keys when `app.active_tab == Tab::Bodies`.
  - Accelerate/decelerate simulation speed using `[` and `]`.
- [x] Change cross-term event polling duration to `100ms` in `src/main.rs` to drive the animation. (3fd794c)

## Phase 3: Keplerian Math & Projection
- [x] Implement a Kepler solver (`Eccentric Anomaly` solver) using Newton-Raphson iteration inside `src/ui/mod.rs`. (3fd794c)
- [x] Implement isometric projection (pitch: 30°, yaw: 45°) to map 3D Keplerian coordinates to 2D canvas coordinates. (3fd794c)
- [x] Add robust logarithmic distance compression to handle widely spaced bodies. (3fd794c)

## Phase 4: Orrery Canvas Rendering
- [x] Create `draw_orrery` in `src/ui/mod.rs` using `ratatui::widgets::canvas::Canvas`. (3fd794c)
- [x] Plot orbital path lines and dynamic position markers (star, planets, moons). (3fd794c)
- [x] Render dimmed names/indices next to each body. (3fd794c)
- [x] Render a centered subtab selector at the bottom of the left pane. (3fd794c)

## Phase 5: Verification & Tests
- [x] Add unit tests verifying: (3fd794c)
  - Parsing of new orbital fields from journal logs.
  - Input actions (speed change, subtab toggle).
  - Orbit math solver accuracy.
- [x] Run cargo test and manual validation on PlutoII. (3fd794c)
