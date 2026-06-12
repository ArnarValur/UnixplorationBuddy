# 🔭 Anomaly Hunting Checklist

> CMDR's field guide — find 'em, checkbox 'em.

---

## Batch 1 — Orbital Anomalies (Deployed ✅)

- [ ] ⚠️ **Close Orbit** — Body skimming its parent (apoapsis < 2× radii)
- [ ] 🛬 **Close Orbit (Landable)** — Same but you can land on it!
- [ ] 💫 **Close Flypast** — Two siblings buzzing within 100km
- [ ] 💥 **Collision Course** — Sibling orbits actually overlap
- [ ] ♊ **Trojan** — Two bodies sharing the same orbit
- [ ] 🌟 **Satellite Star** — Star orbiting as a moon
- [ ] 🌙 **Moon³** — Moon of a moon of a moon

## Batch 2 — Extreme Bodies (Building... 🔧)

- [ ] 🌀 **Fast Rotator** — Rotation period under 1 hour
- [ ] 🔄 **Retrograde Orbit** — Inclination > 90° (orbiting backwards)
- [ ] 📐 **High Eccentricity** — Eccentricity > 0.9 (comet-like orbit)
- [ ] 🔀 **Extreme Tilt** — Axial tilt > 90° (Uranus-like)

## Batch 2 — Jumponium (Building... 🔧)

- [ ] 🟢 **Green System (Basic)** — Carbon + Vanadium + Germanium
- [ ] 🟡 **Green System (Standard)** — + Cadmium + Niobium
- [ ] ⭐ **Green System (Premium)** — + Arsenic + Yttrium + Polonium

---

## Tips for Hunting

| Anomaly | Where to look |
|---------|---------------|
| Close Orbit | Tight binary systems, moons of gas giants |
| Flypast/Collision | Systems with many moons at similar distances |
| Trojan | Large systems with 10+ bodies |
| Satellite Star | Multi-star hierarchies (4+ stars) |
| Moon³ | Complex gas giant systems with nested moons |
| Fast Rotator | Small rocky bodies close to stars |
| Retrograde | Captured moons, outer system bodies |
| High Eccentricity | Outer planets, binary star companions |
| Extreme Tilt | Random — Uranus is the poster child |
| Jumponium | Scan ALL landable bodies in a system |

---

## 💡 Future Ideas

- [ ] **NavRoute Mass Code Scanner** — Parse plotted route, flag systems with mass code `g`/`h` (likely black holes/neutrons). Zero API calls, pure name parsing.
- [ ] **Neighborhood Discovery** — APIs (EDSM/Spansh) are blind in deep unexplored space. Need a local approach — possibly mass code heuristics on procedurally generated sector names.
- [ ] **In-game galaxy map** — Can't be controlled externally. Any "go here" suggestion needs to be a TUI display the CMDR reads and manually enters.

> Mark 'em when you find 'em, CMDR. o7
