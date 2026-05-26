# UnixplorationBuddy

A visually polished exploration companion plugin for **E:D Market Connector** (EDMC), built for Linux commanders who miss the aesthetics and usability of Windows-only tools like *Exploration Buddy*.

## Vision

Replace the current plain-text, vertically-scrolling body list in EDMC exploration plugins with something that:

- **Groups bodies intelligently** — moons nested under parents, displayed compactly (horizontally where possible)
- **Uses rich formatting** — `tk.Text` tags for per-element styling: bold body names, color-coded values, strike-through for lost data
- **Looks premium** — inspired by [Exploration Buddy](https://panostrede.de) (Windows), which provided a visually pleasing, multi-monitor-friendly companion UI
- **Runs natively on Linux** — no Wine, no workarounds. Built as an EDMC plugin using Tkinter

## Lineage

Forked from [Silarn/EDMC-Pioneer](https://github.com/Silarn/EDMC-Pioneer), which provides the data layer:
- System body scanning and value calculation
- First discovery / first mapped / terraformable tracking
- EDSM integration
- ExploData database backend ([Silarn/EDMC-ExploData](https://github.com/Silarn/EDMC-ExploData))

Pioneer does the hard work of data collection and value math. Our focus is **the display layer** — making that data beautiful and usable.

## Technical Context

### Current Pioneer Architecture
- **GUI**: Single `ttk.Label` inside a `tk.Canvas` scroll container
- **Formatting**: Pure plain text with `\n` and Unicode emoji for status icons
- **Layout**: Vertical list, one body per 4-5 lines, hardcoded dimensions
- **Theme**: EDMC's theme engine applies a single color (orange) uniformly

### Planned Upgrades
| Area | Current | Target |
|------|---------|--------|
| Text widget | `ttk.Label` (single style) | `tk.Text` with tags (rich text) |
| Body layout | Vertical, 4-5 lines each | Compact grouped, moons inline |
| Hierarchy | Flat list | Parent → moon tree structure |
| Formatting | Plain text | Bold names, colored values, strikethrough lost |
| Height | ~~Hardcoded 100px~~ → Dynamic (already patched) | Dynamic with configurable max |

### Key Files (Pioneer)
- `load.py` — Main plugin: GUI construction, display logic, value calculation
- `pioneer/globals.py` — Shared state / settings variables
- `pioneer/body_calc.py` — Body value formulas
- `pioneer/format_util.py` — Credit formatting
- `pioneer/overlay.py` — EDMC overlay integration

### Dependencies
- EDMC (Flatpak: `io.edcd.EDMarketConnector`)
- ExploData plugin (database backend)
- Python 3.x, Tkinter, SQLAlchemy

## Pre-existing Patches

Before forking, we applied a **dynamic scroll height** patch to Pioneer:
- Canvas height auto-sizes to content instead of hardcoded 100px
- Configurable max height (default 400px) in plugin settings
- Backup at `load.py.bak` in the installed plugin directory

## Status

🟡 **Pre-fork** — Project directory created, planning phase. Next steps:
1. Fork `Silarn/EDMC-Pioneer` into this repo
2. Prototype the `tk.Text` display rewrite
3. Implement body grouping logic (parent/moon hierarchy from name parsing)
4. Design the visual layout

---

*Merkurial-studio · Arnar Valur · 2026*
