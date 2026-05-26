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
- **Tagline:** A standalone TUI exploration companion for Linux commanders.
- **Description:** A Rust-based terminal user interface (TUI) application for Elite Dangerous explorers on Linux. Monitors the player journal in real-time and displays system body hierarchy, scan states, calculated exploration values, bio/geo signals, and trip statistics on a second monitor while playing. Built with Ratatui for rich terminal rendering — tables, color-coded values, expandable rows, and the signature Elite orange-on-black aesthetic.
- **Target Audience:** Linux Elite Dangerous pilots who want a dedicated exploration companion without depending on EDMC or Windows-only tools.
- **Key Differentiators:**
  - Standalone terminal app — no EDMC, no Wine, no GUI framework
  - Single static binary — zero runtime dependencies
  - Rust performance — instant journal parsing, sub-millisecond rendering
  - Second-monitor optimized — auto-updates, high contrast, readable at a glance
  - Open-source (community-driven, unlike the Windows Exploration Buddy)

---

## 2. Product Guidelines

- **Brand Voice:** Technical but approachable — speaks the language of the Elite Dangerous pilot community. Open-source ethos: clear documentation, welcoming to contributors, no gatekeeping.
- **UX Principles:**
  - **Information density** — tabular layout like Windows Exploration Buddy; no wasted vertical space
  - **Hierarchy clarity** — parent→moon grouping visible at a glance via indentation and naming convention parsing
  - **Glanceable** — second-monitor use means high contrast, clear status at a distance
  - **Interactive on demand** — expandable bio species rows when you focus the terminal
  - **Compact by default** — moons nested under parents, bio details collapsed until expanded
- **Accessibility:** Full keyboard navigation. Terminal-native — respects user font/terminal settings. High-contrast orange-on-black as default theme.

---

## 3. Tech Stack

- **Language:** Rust (latest stable)
- **TUI Framework:** Ratatui (v0.30+)
- **Terminal Backend:** Crossterm (Ratatui default)
- **Journal Parsing:** `ed_journals` crate (journal reading, file watching, game state tracking)
- **Value Calculation:** Self-contained Rust port of community-derived formulas (see ADR-0003)
- **Bio Data:** Canonn species data bundled as static dataset (stored in `conductor/canonn-data/`)
- **Build:** Cargo (standard Rust toolchain)
- **Distribution:** GitHub releases (prebuilt Linux binary) + `cargo install`

---

## 4. Caution Levels

| Domain                        | Level       | Notes                                                    |
|-------------------------------|-------------|----------------------------------------------------------|
| Journal parsing               | 🔴 Critical | Data correctness depends on parsing every event type     |
| Value calculation formulas    | 🔴 Critical | Must match community-verified values — test thoroughly   |
| TUI rendering / layout        | 🟡 Careful  | Visual impact — test in multiple terminal emulators      |
| Bio species prediction        | 🟡 Careful  | Must match Canonn data accurately                        |
| Canonn data processing        | 🟡 Careful  | Source of truth for bio predictions                      |
| conductor/ files              | 🟡 Careful  | Source of truth — don't corrupt                          |
| Terminal compatibility        | 🟢 Normal   | Ratatui handles most backends                            |

---

## 5. Domain Expertise

| Area                    | Confidence | Notes                                                       |
|-------------------------|------------|-------------------------------------------------------------|
| Rust / Ratatui          | Medium     | Well-documented ecosystem; community templates available     |
| Elite Dangerous Journal | Medium     | JSON events from game client; `ed_journals` crate handles parsing |
| Exploration value formulas | Medium  | Community-derived; Pioneer's `body_calc.py` is reference    |
| Canonn bio data         | Medium     | Rich dataset gathered in `conductor/canonn-data/`           |
| EDSM / Spansh APIs      | Low       | Phase 2 enrichment — investigate when needed                |

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
   - Test with real journal files from the player's journal directory
   - Verify value calculations against known systems
   - Test TUI in at least 2 terminal emulators (kitty, alacritty, gnome-terminal)
   - If a fix fails twice, stop and escalate

---

## 7. Project-Specific Constraints

- **Journal file path:** `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous/` (Steam Proton). Must support auto-detection or config.
- **No GUI dependencies:** Pure terminal app — no X11, no Wayland, no Tkinter
- **Offline-first:** Core functionality (journal parsing, value calculation) works without network. EDSM/Spansh are optional enrichment.
- **Single binary:** `cargo build --release` produces one executable. No config files required for basic operation.

---

## 8. Environment Notes

- **Dev machine:** Pop!_OS (PlutoII), user `solmundur`
- **Game install:** Steam (Flatpak) with Proton
- **Journal path:** `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous/`
- **EDMC install:** Flatpak `io.edcd.EDMarketConnector` (still used for trade/EDSM uploads — UnixplorationBuddy is independent)
- **Canonn data:** `conductor/canonn-data/` — Obsidian markdown clippings + JSON files
- **Pioneer source (reference):** `https://github.com/Silarn/EDMC-Pioneer` — value calculation formulas
- **CETI (reference):** `https://github.com/carsonbfl/CETI` — journal monitoring and API querying patterns
