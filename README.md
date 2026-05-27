# UnixplorationBuddy

A standalone **Rust TUI** exploration companion for Linux commanders playing **Elite Dangerous**.

Runs in a terminal on your second monitor, reads journal files in real-time, and displays everything you need while exploring — body hierarchy, scan states, calculated values, bio/geo signals, and trip statistics.

![Status](https://img.shields.io/badge/status-alpha-orange)
![Language](https://img.shields.io/badge/language-Rust-B7410E)
![Platform](https://img.shields.io/badge/platform-Linux-FCC624)

## What It Does

- **Bodies View** — Hierarchical body table (stars → planets → moons), ordered by the game's naming convention. Shows type, atmosphere, distance, scan/map status icons, calculated value, and bio/geo signal counts.
- **History View** — Trip statistics: estimated total value, systems visited, bodies scanned/mapped, first discoveries, bio signals detected.
- **System Header** — Current system name, discovered body count (e.g., "20 of 25"), total estimated system value.
- **Real-time Updates** — Monitors journal files via background thread. Auto-updates as you honk, scan, and map.
- **Accurate Values** — Self-contained Rust port of the community-derived exploration value formulas (from [Pioneer's body_calc.py](https://github.com/Silarn/EDMC-Pioneer)), including first discovery/mapping bonuses, Odyssey bonus, and efficiency bonus.

## Architecture

```
Journal Files (Steam Proton path)
        │
        ▼
   ed-journals crate ──── deserializes journal events
        │
        ▼
   process_event() ─────── single state transition function
        │                   (replay mode vs live mode)
        ▼
   App state ──────────── bodies, system, trip
        │
        ▼
   Ratatui TUI ──────── renders to terminal
```

**Key design decisions:**
- Own state layer on top of `ed-journals` (the crate's built-in state module is too opaque)
- Replay on startup reconstructs state without inflating trip counters
- Naming convention parser for correct body hierarchy ordering
- Value calculation ported from Pioneer's formulas, not the crate's incomplete `calculate_estimated_worth()`

## Building

```bash
cargo build --release
```

## Running

```bash
# Auto-detects Steam Proton journal path
./target/release/unixploration-buddy

# Or specify journal directory explicitly
./target/release/unixploration-buddy --journal-dir /path/to/journals
```

### Journal Directory

By default, looks for journals at:
```
~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch between Bodies / History views |
| `↑` / `↓` | Navigate body list |
| `q` | Quit |

## Tech Stack

| Component | Choice |
|-----------|--------|
| Language | Rust (2021 edition) |
| TUI Framework | [Ratatui](https://ratatui.rs) 0.30 + Crossterm |
| Journal Parsing | [ed-journals](https://crates.io/crates/ed-journals) 0.12 |
| Value Formulas | Own port of [Pioneer's body_calc.py](https://github.com/Silarn/EDMC-Pioneer) |

## Project Status

### Completed
- ✅ Project bootstrap + data model
- ✅ Journal ingestion (replay + live watcher)
- ✅ Replay/trip separation (trip starts on TUI launch)
- ✅ Naming convention parser (body hierarchy ordering)
- ✅ Value calculation (all body types, all modifiers)
- ✅ Test suite: 65 tests

### In Progress
- 🔄 TUI rendering polish (scrollable table, column widths, colors)

### Planned
- Trip persistence (JSON to XDG data dir)
- Bio species predictions (Canonn data integration)
- EDSM integration (first discoverer lookup)
- Gravity column, sortable columns, route view

## Lineage

Inspired by [Exploration Buddy](https://panostrede.de) (Windows) and built on data patterns from [EDMC-Pioneer](https://github.com/Silarn/EDMC-Pioneer). Originally planned as an EDMC Tkinter plugin, pivoted to standalone Rust TUI for performance, portability, and terminal aesthetics.

---

*Merkurial-studio · Arnar Valur · 2026*
