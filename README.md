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

## Lineage

Inspired by [Exploration Buddy](https://panostrede.de/EDEB/) (Windows) and built on data patterns from [EDMC-Pioneer](https://github.com/Silarn/EDMC-Pioneer). Originally planned as an EDMC Tkinter plugin, pivoted to standalone Rust TUI for performance, portability, and terminal aesthetics.

---

*Merkurial-studio · Arnar Valur · 2026*

## Quick Start

### Build and Run

To compile and run the TUI:

```bash
# Clone the repository
git clone https://github.com/ArnarValur/UnixplorationBuddy.git
cd UnixplorationBuddy

# Compile and run
cargo run --release
```

By default, the TUI automatically discovers your Elite Dangerous journal directory on Linux under Steam Flatpak. If you are using a different Wine/Proton path, you can specify it using the `--journal-path` argument:

```bash
cargo run --release -- --journal-path "/path/to/journal/folder"
```

### Keybindings

- **Tab** or **1** / **2** — Switch between **Bodies** and **History** tabs.
- **Up / Down Arrow** — Navigate/scroll the body hierarchy list.
- **Ctrl+R** — Reset trip statistics.
- **?** — Toggle keybindings overlay help window.
- **q** or **Esc** — Quit.
