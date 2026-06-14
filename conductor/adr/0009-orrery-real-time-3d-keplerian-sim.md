# 9. Real-Time TUI 3D Keplerian Orrery Visualization

> **Recorded:** 2026-05-29 22:20
> **Status:** accepted

We have introduced a real-time, animated 3D Keplerian Orrery map inside the TUI's **Bodies** tab. It serves as a visual playground alongside the standard hierarchical System Map table.

## Context & Decision

Rendering complex astrophysical mechanics in a pure text-based interface requires balancing CPU efficiency, resolution limits, and math stability:
1. **Graphics Engine**: We chose the Ratatui `Canvas` widget equipped with high-resolution Braille sub-pixel rendering.
2. **Astrodynamics**: We integrated numerical Newton-Raphson solvers to converge Kepler's transcendental eccentric anomaly equation:
   $$M = E - e \sin E$$
   and compute precise Cartesian relative coordinate matrices dynamically under time-lapse progression.
3. **Projection**: We adopted a fixed isometric camera projection ($30^\circ$ pitch, $45^\circ$ yaw) to establish a 3D sense of depth on a 2D terminal plane.
4. **Logarithmic Scaling**: Linear scaling squashes inner systems into a single character. We applied logarithmic compression:
   $$r_{\text{scaled}} = C \cdot \ln(1 + \frac{r}{r_0})$$
   to ensure planets and moons remain visually distinct regardless of extreme orbital magnitudes.
