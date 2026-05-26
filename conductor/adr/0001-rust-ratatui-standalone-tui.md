# Rust/Ratatui standalone TUI over Python/Tkinter EDMC plugin

> **Recorded:** 2026-05-27 00:10
> **Status:** accepted

The project was originally conceived as a Python EDMC plugin forked from Silarn/EDMC-Pioneer, replacing Pioneer's single-label display with a `tk.Text`-based rich layout. After evaluating the constraints — Tkinter's limited rendering, EDMC's environment restrictions, and the goal of a premium exploration companion — we pivoted to a standalone Rust TUI using Ratatui.

This gives us full terminal rendering (tables, box drawing, 24-bit color, expandable rows), a single static binary with no runtime dependencies, and architectural independence from EDMC's plugin lifecycle. The trade-off is losing Pioneer's free data pipeline (ExploData integration) and needing to port value calculations and parse journal files ourselves — both tractable problems with existing Rust crates (`ed_journals`) and ~100 lines of formula code.

## Considered Options

- **A) EDMC plugin (Python/Tkinter):** Free data pipeline via ExploData, but display constrained by Tkinter widget system. Fighting the framework.
- **B) Standalone Rust TUI (Ratatui):** Full control, owns its own data pipeline, single binary. Requires porting value calculations and journal parsing (crates exist).
- **C) Python TUI (Textual/urwid):** Keeps Python but loses EDMC integration. Worst of both worlds — still Python performance, no EDMC data, no Rust safety.

Option B chosen for architectural independence and rendering capability.
