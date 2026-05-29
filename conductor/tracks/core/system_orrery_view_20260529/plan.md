# Implementation Plan — System Orrery TUI View

## Phase 1: Model & State Extensions
- [ ] Add orbital parameter fields to the `Body` struct in `src/model/body.rs`.
- [ ] Update `process_event` inside `src/journal.rs` to parse orbital parameters from `Scan` events.
- [ ] Define the `BodiesSubTab` enum in `src/app.rs` with `Table` and `Orrery` variants.
- [ ] Add active subtab, simulated time, and speed multiplier state to `App` in `src/app.rs`.

## Phase 2: Keyboard Inputs & Ticking
- [ ] Modify `src/main.rs` input handler:
  - Toggle `app.bodies_subtab` using `a`/`d` or `Left`/`Right` keys when `app.active_tab == Tab::Bodies`.
  - Accelerate/decelerate simulation speed using `[` and `]`.
- [ ] Change cross-term event polling duration to `100ms` in `src/main.rs` to drive the animation.

## Phase 3: Keplerian Math & Projection
- [ ] Implement a Kepler solver (`Eccentric Anomaly` solver) using Newton-Raphson iteration inside `src/ui/mod.rs` or a math helper.
- [ ] Implement isometric projection (pitch: 30°, yaw: 45°) to map 3D Keplerian coordinates to 2D canvas coordinates.
- [ ] Add robust logarithmic distance compression to handle widely spaced bodies.

## Phase 4: Orrery Canvas Rendering
- [ ] Create `draw_orrery` in `src/ui/mod.rs` using `ratatui::widgets::canvas::Canvas`.
- [ ] Plot orbital path lines and dynamic position markers (star, planets, moons).
- [ ] Render dimmed names/indices next to each body.
- [ ] Render a centered subtab selector at the bottom of the left pane.

## Phase 5: Verification & Tests
- [ ] Add unit tests verifying:
  - Parsing of new orbital fields from journal logs.
  - Input actions (speed change, subtab toggle).
  - Orbit math solver accuracy.
- [ ] Run cargo test and manual validation on PlutoII.
