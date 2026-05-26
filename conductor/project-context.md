<!-- Template: TheOracle v2.1 -->
# Project Context — UnixplorationBuddy

> Single identity + operational document for this project.
>
> **Created by `/conductor-init`; user-edited thereafter — no command writes here.**
> Persona and communication style are handled separately by `/hermes`.
>
> Section order is deliberate: **identity-first (1–3), operational-second (4–8).**
> Agents partial-reading this file should see *what the product is* before *how to behave when working on it*.

---

## 1. Product Definition

- **Name:** UnixplorationBuddy
- **Tagline:** A premium exploration companion for Linux commanders, built as an EDMC plugin.
- **Description:** Reverse-engineers the Windows-only "Elite Dangerous Exploration Buddy" (by Panostrede) as an open-source EDMC plugin for Linux. Replaces the plain-text, vertically-scrolling body list from Pioneer with a rich-formatted, hierarchical body display featuring tabular layout, color-coded scan values, parent→moon grouping, and premium aesthetics. Forked from Silarn/EDMC-Pioneer, which provides the data layer (body scanning, value calculation, EDSM integration, ExploData database). UnixplorationBuddy focuses exclusively on the **display layer** — making that data beautiful and usable.
- **Target Audience:** Linux Elite Dangerous pilots who use EDMC (via Flatpak or native install) and lack a proper exploration companion equivalent to the Windows Exploration Buddy.
- **Key Differentiators:**
  - Only dedicated exploration companion plugin for Linux
  - Native EDMC plugin — no Wine, no workarounds
  - Open-source (community-driven, unlike the Windows original)
  - Rich Tkinter UI using `tk.Text` tags for per-element styling
  - Reuses Pioneer's battle-tested data layer instead of reinventing it

---

## 2. Product Guidelines

- **Brand Voice:** Technical but approachable — speaks the language of the Elite Dangerous pilot community. Open-source ethos: clear documentation, welcoming to contributors, no gatekeeping.
- **UX Principles:**
  - **Information density** — tabular layout like Windows Exploration Buddy; no wasted vertical space
  - **Hierarchy clarity** — parent→moon grouping visible at a glance via indentation and naming convention parsing
  - **EDMC theme integration** — respects EDMC's orange-on-black theme engine; extends it with richer formatting (bold, color-coded values, strikethrough for lost data)
  - **Compact by default** — moons inline/nested under parents, not each on 4-5 lines
- **Accessibility:** Keyboard navigable within EDMC's Tkinter context. Respect system font scaling. High-contrast orange-on-black inherited from EDMC theme.

---

## 3. Tech Stack

- **Languages:** Python 3.x
- **Frameworks:** Tkinter (via EDMC's bundled tk), EDMC Plugin API
- **Databases:** SQLAlchemy (via ExploData dependency — provides the body/system/value database)
- **Deployment Targets:** EDMC plugin directory (local install). Must be Flatpak-compatible (`io.edcd.EDMarketConnector`).
- **Hosting:** GitHub (open-source distribution). Users clone/download into their EDMC plugins folder.

---

## 4. Caution Levels

| Domain                        | Level       | Notes                                                    |
|-------------------------------|-------------|----------------------------------------------------------|
| Display / UI layer            | 🟡 Careful  | Visual impact — test rendering in EDMC                   |
| Pioneer data layer            | 🔴 Critical | Upstream code — changes here must be cherry-pickable     |
| EDMC plugin API surface       | 🔴 Critical | Breaking the API contract = plugin won't load            |
| ExploData / SQLAlchemy models | 🔴 Critical | Shared database — never alter schema without ExploData   |
| Theme / color values          | 🟢 Normal   | Aesthetic only — low risk                                |
| conductor/ files              | 🟡 Careful  | Source of truth — don't corrupt                          |

---

## 5. Domain Expertise

| Area                    | Confidence | Notes                                                       |
|-------------------------|------------|-------------------------------------------------------------|
| Python / Tkinter        | High       | Standard library, well-documented                           |
| EDMC Plugin API         | Medium     | Documented but not deeply explored yet; Pioneer is reference |
| Elite Dangerous Journal | Medium     | JSON events from game client; Pioneer already parses these  |
| ExploData / SQLAlchemy  | Medium     | Pioneer's dependency; we query but don't own the schema     |
| Exploration Buddy (Win) | Low        | Reverse-engineering from UI screenshots and user reports    |

---

## 6. Preferred Workflows

1. **Session Start Protocol:**
   - Read `conductor/relay.md` first (pending messages, blockers)
   - Then read `conductor/pulse.md` (current state, recent progress)
   - Review `conductor/tracks.md` for next task

2. **Checkpoint Frequency:**
   - Checkpoint after every completed phase
   - Consider mid-phase checkpoints for long phases (>5 tasks)

3. **Decision Logging:**
   - Architectural decisions live in `conductor/adr/` (batched by `/grill`, `/new-track`, or `/checkpoint`)
   - Operational notes live in `conductor/pulse.md` Session Memory
   - Verify actual state on disk before proposing changes

4. **Debugging Protocol:**
   - Test in a running EDMC instance — the plugin only works inside EDMC
   - Check Pioneer's original behavior before assuming bugs are ours
   - If a fix fails twice, stop and escalate
   - Audit actual state vs documented state

---

## 7. Project-Specific Constraints

- **EDMC Flatpak compatibility:** Plugin must work under `io.edcd.EDMarketConnector` Flatpak sandbox
- **Pioneer compatibility:** Display layer changes must not break Pioneer's data flow (`journal_entry`, `dashboard_entry` hooks)
- **No external dependencies beyond Pioneer's:** Can't add packages that EDMC's Python environment doesn't have
- **Theme respect:** Must integrate with EDMC's theme engine (`theme.apply()`) — never hardcode colors that break in non-default themes
- **Python version:** Must work with whatever Python version EDMC bundles (currently 3.11+)

---

## 8. Environment Notes

- **Dev machine:** Pop!_OS (PlutoII), user `solmundur`
- **EDMC install:** Flatpak `io.edcd.EDMarketConnector`
- **Journal path:** `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous/`
- **Plugin dev path:** `~/.var/app/io.edcd.EDMarketConnector/data/EDMarketConnector/plugins/` (Flatpak) or equivalent
- **Pioneer source (upstream):** `https://github.com/Silarn/EDMC-Pioneer`
- **ExploData source (upstream):** `https://github.com/Silarn/EDMC-ExploData`
- **Testing:** Load plugin in EDMC, enter a system with bodies, verify display renders correctly
