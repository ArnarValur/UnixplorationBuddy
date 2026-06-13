---
name: cmdr-snapshot
description: Capture the current Elite Dangerous game state by parsing player journal files. Use this skill whenever you need to know the CMDR's current system, bodies, scan status, bio signals, or trip state. Run the snapshot script to "see" what UnixplorationBuddy shows.
---

# CMDR Snapshot — Journal State Reader

Tap into the player journal to capture a snapshot of the current game state — the same data UnixplorationBuddy displays.

## When to Use

- User asks "where am I?" or "what system am I in?"
- You need current system context for API lookups or research
- User asks about bodies, scans, bio signals in their current system
- You want to verify what UB is showing
- Any time you need live game state awareness

## How to Use

Run the snapshot script:

```bash
python3 /home/solmundur/Projects/EliteDangerous/UnixplorationBuddy/.agents/skills/cmdr-snapshot/scripts/snapshot.py
```

### Output

The script outputs a JSON object with:

| Field | Description |
|-------|-------------|
| `system` | Current system name, address, coordinates, region |
| `bodies` | Array of all scanned bodies with type, subtype, mass, temp, gravity, atmosphere, volcanism, signals |
| `honk` | FSS discovery scan results (body count, non-body count) |
| `signals` | Bio/geo signal counts per body |
| `scans_complete` | Which bodies have been DSS'd |
| `commander` | CMDR name, ship, game version |
| `timestamp` | When the last relevant event occurred |
| `journal_file` | Which journal file was parsed |

### Options

```bash
# Default: current system snapshot
python3 .../snapshot.py

# Include last N systems visited (for trip context)
python3 .../snapshot.py --history 5

# Raw JSON (no pretty print)
python3 .../snapshot.py --raw
```

## Journal Path

The script reads from the Steam Proton journal path:
```
~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous/
```

## Notes

- The script reads journal files in reverse chronological order until it finds the current system context
- It may need to read multiple journal files if the current session spans a file boundary
- Body data accumulates from `Scan` events since the last `FSDJump` or `Location` event
- This is read-only — it never modifies journal files
