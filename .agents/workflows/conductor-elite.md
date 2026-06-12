---
description: Elite Dangerous companion mode. Loads CMDR context, API references, and data sources, then runs the standard /conductor protocol.
---

# Conductor Elite — CMDR Companion Protocol

When the user invokes `/conductor-elite`, activate the Elite Dangerous companion context, then delegate to the standard `/conductor` resume protocol.

---

## Step 0: Load CMDR Context

Before running `/conductor`, load this preamble into session awareness. This context persists for the entire session.

### Commander Profile

| Field | Value |
|-------|-------|
| **CMDR Name** | ADDINATOR |
| **Platform** | PC (Steam Proton on Linux) |
| **Frontier ID** | 12208432 |
| **Primary Role** | Explorer |
| **Secondary Role** | Researcher |
| **Allegiance** | Independent |
| **Main Ship** | Mandalay: *EagleHawk* |
| **Play Style** | PvE, Roleplay |
| **Attitude** | Relaxed / casual |
| **Main Base** | Midgard |
| **Canonn Expertise** | Unknown Artefacts, Unknown Probes, Unknown Ships, Planet-based Biological Entities, Ancient Ruins, Code-breaking, Research, Development, Documentation |

### API Services

API keys are stored in `.env` at the project root (gitignored, never committed).

| Service | Base URL | Auth | Docs |
|---------|----------|------|------|
| **EDSM** | `https://www.edsm.net/api-v1/` | API key (`EDSM_API_KEY`) + Commander Name (`EDSM_COMMANDER_NAME`) | [EDSM API docs](https://www.edsm.net/en/api) |
| **Inara** | `https://inara.cz/inapi/v1/` | API key (`INARA_API_KEY`) | [Inara API docs](https://inara.cz/inara-api/) |
| **Spansh** | `https://spansh.co.uk/api/` | None (open) | [Spansh](https://spansh.co.uk/) |
| **Canonn** | `https://canonn.science/` | None (open) | [Canonn Resources](https://canonn.science/resources/) |

#### Common EDSM Endpoints

| Endpoint | Purpose | Example |
|----------|---------|---------|
| `/api-v1/system` | System info by name | `?systemName=Sol&showCoordinates=1` |
| `/api-system-v1/bodies` | Bodies in a system | `?systemName=Sol` |
| `/api-system-v1/estimated-value` | System exploration value | `?systemName=Sol` |
| `/api-v1/sphere-systems` | Systems near coordinates | `?x=0&y=0&z=0&radius=50` |
| `/api-logs-v1/get-position` | CMDR's last known position | Requires API key |

#### Common Spansh Endpoints

| Endpoint | Purpose |
|----------|---------|
| `/api/nearest` | Nearest system to coordinates |
| `/api/search/fields` | Available search fields |
| `/api/bodies/search` | Body search with filters |
| `/api/systems/search` | System search with filters |
| `/api/route/plotter` | Neutron route plotter |

#### Inara API

Inara uses a POST-based JSON API. All requests go to `https://inara.cz/inapi/v1/` with a JSON body containing `header` (API key, app name, CMDR name) and `events` array.

| Event Name | Purpose |
|------------|---------|
| `getCommanderProfile` | CMDR profile lookup |
| `getCommunityGoalsRecent` | Active community goals |
| `getMarketSellOrders` | Station market data |

### Local Data Sources

| Source | Path | Purpose |
|--------|------|---------|
| Player Journals | `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous/` | Live game events |
| EDMC Data | `~/.var/app/io.edcd.EDMarketConnector/data/EDMarketConnector/` | EDMC logs and uploads |
| Canonn Bio Data | `conductor/canonn-data/` | Bundled species prediction dataset |
| Research Reports | `conductor/docs/research/research-reports/` | Deep dives on game mechanics |
| Enhancement Backlog | `conductor/docs/enhancement-backlog.md` | Feature ideas for the TUI |

### Canonn Tools Reference

The following tools at [canonn.science/resources](https://canonn.science/resources/) are useful for research:

- **Canonn R&D** — Codex discoveries, bio/geo/xeno data
- **Signal Searcher** — Find specific signal types
- **Observatory** — Track notable stellar phenomena
- **Patrol Routes** — Community patrol assignments

### Community Profiles

| Platform | Handle |
|----------|--------|
| Canonn | Addinator |
| Inara | Addinator |
| EDSM | Addinator |
| Frontier Forums | Addinator |
| Discord | #Arnarinn2310 |

---

## Step 1–4: Standard Conductor Protocol

After loading the CMDR context above, execute the full `/conductor` protocol as defined in `conductor.md`:

1. **Load Context** — Read all conductor files (project-context, workflow, pulse, tracks, etc.)
2. **Defensive Index Reconcile** — Self-heal `conductor/index.md`
3. **Status Report** — Present structured project status
4. **Await Orders** — Present action options (Grill, New Track, Implement, Review, Checkpoint, Revert)

The status report header changes from `🎵 Conductor Online` to:

```text
🎵 Conductor Elite Online — o7 CMDR ADDINATOR

📍 Last Session: {date from pulse.md} — {focus}
🔄 Active Tracks: {count}
⚠️ Blockers: {count or "None"}
🚀 Ship: Mandalay EagleHawk | Base: Midgard | Role: Explorer/Researcher
```

---

## Session Behavior (Elite Additions)

In addition to standard `/conductor` session behavior:

- **API awareness.** When discussing features that involve EDSM, Spansh, Inara, or Canonn, reference the endpoints above. Keys are in `.env`.
- **Data source awareness.** Know where journals, EDMC data, and Canonn datasets live on disk.
- **Exploration context.** When the user discusses in-game discoveries, route planning, or bio scanning, leverage knowledge of the APIs and local data to assist.
- **Tips & discoveries.** If the user shares gameplay insights, exploration tips, or interesting finds during a session, note them in `conductor/pulse.md` Session Memory with an `_(elite)_` tag for later reference.
