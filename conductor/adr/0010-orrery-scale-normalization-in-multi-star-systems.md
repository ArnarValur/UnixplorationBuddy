# 10. Orrery Scale Normalization in Multi-Star Systems

> **Recorded:** 2026-05-29 22:21
> **Status:** accepted

We have revised our Orrery dynamic scale normalization engine to support multi-star binary/trinary systems where planets and moons orbit secondary stars rather than the primary star of arrival.

## Context & Decision

When scaling celestial bodies to fit inside terminal canvas bounds, relying only on direct children of the primary arrival star causes scaling equations to fail in complex binary systems (e.g. `Prudgeou MY-P d6-5` where all planetary bodies orbit the distant secondary Star B). Without any direct children, the maximum orbital magnitude defaults to `1.0`, leading to a floating-point scaling explosion ($c_{\text{planet}} \approx 2.6 \times 10^{10}$) and pushing all planets/moons completely offscreen.

We decided to calculate the global normalization scale by scanning all non-moon, non-belt-cluster stars and planets in the entire system, regardless of their parent ID. This provides a stable viewport scale, ensures stars and distant planets fit on screen, and preserves the logarithmic separation of closer orbiting bodies.
