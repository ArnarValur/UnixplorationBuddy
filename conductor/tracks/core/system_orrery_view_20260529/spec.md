# Specification — System Orrery TUI View

## Overview
Implement a real-time, animated 3D Keplerian Orrery System Map inside the TUI's **Bodies** tab to render a visual representation of system planets/moons revolving dynamically in a time-lapse simulation. This is a purely aesthetic, premium "candy-addition" that makes the TUI feel extremely responsive and alive.

---

## Functional Requirements

### 1. Bodies Subtab Integration
- Add two subtabs to the **Bodies** tab:
  1. `[1] System Map` (Default hierarchical table)
  2. `[2] Orrery Map` (Interactive animated canvas)
- Allow switching between these subtabs using `a`/`d` or `Left`/`Right` arrow keys when the `Bodies` tab is active.
- When `Orrery Map` is active, the left pane (60% width) is replaced by the Orrery 3D Canvas, while the right pane (40% width) remains the active Telemetry Inspector.
- Synchronize selection: The body selected in the Orrery corresponds to the one highlighted in the inspector.

### 2. Ratatui Canvas Rendering
- Use `ratatui::widgets::canvas::Canvas` to render high-resolution paths and symbols using Braille sub-pixel rendering.
- **Symbol System:**
  - Primary Star: Rendered as a bright yellow/orange `★` at the center `(0, 0)`.
  - Planets: Rendered as `●` in color-coded categories matching their class.
  - Moons: Rendered as `•` orbiting their parent planet.
- **Labels:** Render short names (e.g. `1`, `2 a`) in dimmed text (`Style::default().fg(Color::DarkGray)`) adjacent to the bodies.
- **Orbits:** Draw faint orbital ellipses for all scanned bodies using dotted/braille lines.

### 3. Keplerian Orbital Mechanics
- Parse and store orbital parameters from FSS `Scan` events:
  - `semi_major_axis` ($a$, in meters)
  - `eccentricity` ($e$)
  - `orbital_inclination` ($i$, in degrees)
  - `periapsis` ($\omega$, in degrees)
  - `orbital_period` ($T$, in seconds)
  - `mean_anomaly` ($M_0$, in degrees/radians)
- Solve Kepler's Equation for Eccentric Anomaly $E$ at any given simulated time $t$:
  $$M = M_0 + \frac{2\pi}{T} (t - t_0)$$
  $$M = E - e \sin E$$
- Compute 3D position in the orbital plane, then project onto a fixed isometric viewpoint:
  - Pitch: $30^\circ$, Yaw: $45^\circ$.
  - Transform:
    $$x_{\text{screen}} = x' \cos(\text{yaw}) - y' \sin(\text{yaw})$$
    $$y_{\text{screen}} = (x' \sin(\text{yaw}) + y' \cos(\text{yaw})) \sin(\text{pitch})$$

### 4. Adaptive Logarithmic Scaling
- Actual semi-major axes span orders of magnitude ($10^8$ m to $10^{12}$ m).
- Apply a logarithmic compression function to the semi-major axis to fit all bodies comfortably on the canvas:
  $$r_{\text{scaled}} = C \cdot \ln(1.0 + \frac{a}{a_{\text{scale}}})$$
- Dynamically scale the coordinate boundaries of the Canvas to fit the furthest discovered planet, preventing bodies from flying off-screen.

### 5. Time & Animation Controls
- Run the TUI main loop with a faster `100ms` update rate when in the Orrery subtab to enable smooth frame-by-frame rendering.
- Keep track of an in-app simulated epoch time that increments on every frame.
- Start with an auto-scaled logarithmic time multiplier so orbits revolve dynamically at startup.
- Add manual controls to accelerate/decelerate the simulation speed:
  - `[`: Decrease speed (halve speed)
  - `]`: Increase speed (double speed)

---

## Acceptance Criteria
- Pressing `a` / `d` inside the `Bodies` tab toggles between the Table and the Orrery.
- Elliptical orbits are plotted for all planets, with moons shown correctly orbiting their parent planets.
- Simulated planetary speed respects Kepler's Third Law (inner planets orbit faster).
- The animation is smooth and can be sped up or slowed down using `[` and `]`.
- All unit tests build and pass cleanly.
