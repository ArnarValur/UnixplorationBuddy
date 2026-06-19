# UnixplorationBuddy

Hello, when I finally grew into the Linux world, I missed few tools I used when flying around in Elite, especially Exploration Buddy which is Windows only...

Well... I made a TUI that does the same, it is currently designed for having on the second monitor and a bit curated for my niche needs of looking at what is being explored.

There is some lookups on routes and whether the system has been discovered and reported to EDSM, giving you some clues if you are first encountered. There is more data and improvements I'd like to add, but this is a solid start.

I usually work on this while I play Elite. Fork it, request something, send me a message somehow or just use it if you like it. ( It might feel a bit clonky but it works :D )

CMDR Addinator o7

![Status](https://img.shields.io/badge/status-alpha-orange)
![Language](https://img.shields.io/badge/language-Rust-B7410E)
![Platform](https://img.shields.io/badge/platform-Linux-FCC624)

## What It Does

- **Bodies View** — Hierarchical body table (stars → planets → moons), ordered by the game's naming convention. Shows type, atmosphere, distance, scan/map status icons, calculated value, and bio/geo signal counts.
- **Interesting Codex** — Trip statistics: estimated total value, systems visited, bodies scanned/mapped, first discoveries, bio signals detected.
- **Real-time Updates** — Monitors journal files via background thread. Auto-updates as you honk, scan, and map.
- **Calculate Credits** — Self-contained Rust port of the community-derived exploration value formulas (from [Pioneer's body_calc.py](https://github.com/Silarn/EDMC-Pioneer)), including first discovery/mapping bonuses, Odyssey bonus, and efficiency bonus.

## Lineage

Inspired by [Exploration Buddy](https://panostrede.de/EDEB/) (Windows) and built on data patterns from [EDMC-Pioneer](https://github.com/Silarn/EDMC-Pioneer). Originally planned as an EDMC Tkinter plugin, pivoted to standalone Rust TUI for performance, portability, and terminal aesthetics.

---

## Requirements

- **Linux** (tested on Pop!_OS, should work on any distro)
- **Rust toolchain** — install via [rustup.rs](https://rustup.rs/)
- **Elite Dangerous** via Steam (Proton)

## Installation

```bash
git clone https://github.com/ArnarValur/UnixplorationBuddy.git
cd UnixplorationBuddy
cargo build --release
```

Binary lands at `target/release/unixploration-buddy`.

## Usage

```bash
# Auto-detect journal directory (Flatpak Steam or native Steam)
./target/release/unixploration-buddy

# Or specify your journal path manually
./target/release/unixploration-buddy --journal-path /path/to/journal/dir
```

Trip data persists in `~/.local/share/unixploration-buddy/`.

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch between Bodies / Codex tabs |
| `1` / `2` | Jump to Bodies / Codex tab directly |
| `W` / `S` or `↑` / `↓` | Scroll through body list or codex rows |
| `A` / `D` or `←` / `→` | Switch sub-tabs (Bodies↔Route, codex categories) |
| `v` | Toggle compact / expanded codex view |
| `Scroll wheel` | Scroll body list, codex, or inspector |
| `PageUp` / `PageDown` | Scroll inspector panel |
| `Ctrl+R` | Reset trip statistics |
| `?` | Help overlay |
| `q` / `Esc` | Quit |

## License

MIT — see [LICENSE](LICENSE).
