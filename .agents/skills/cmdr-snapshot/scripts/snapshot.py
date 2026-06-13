#!/usr/bin/env python3
"""
CMDR Snapshot — Journal State Reader

Parses Elite Dangerous player journal files to capture a snapshot of
the current game state. Outputs JSON with system info, bodies, scans,
bio signals, and commander context.

Same data source as UnixplorationBuddy — the player journal files.
"""

import json
import glob
import os
import sys
import argparse
from pathlib import Path

JOURNAL_DIR = os.path.expanduser(
    "~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/"
    "compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/"
    "Frontier Developments/Elite Dangerous"
)

def get_journal_files():
    """Get journal files sorted newest first."""
    pattern = os.path.join(JOURNAL_DIR, "Journal.*.log")
    files = glob.glob(pattern)
    files.sort(reverse=True)
    return files

def parse_journal_file(filepath):
    """Parse a journal file and return list of events."""
    events = []
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                events.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return events

def build_snapshot(history_count=0):
    """Build a snapshot of the current game state."""
    journal_files = get_journal_files()
    if not journal_files:
        return {"error": "No journal files found", "journal_dir": JOURNAL_DIR}

    # Collect all events from recent journals (newest first)
    all_events = []
    for jf in journal_files[:5]:  # Look at last 5 journal files max
        events = parse_journal_file(jf)
        all_events.append((jf, events))

    # Find current system — look for most recent FSDJump or Location
    current_system = None
    system_event = None
    system_journal = None
    bodies = []
    honk = None
    signals = {}
    scans_complete = []
    commander = {}
    fileheader = {}
    history = []

    # Process events from newest journal to oldest
    for jf, events in all_events:
        # Process events in reverse (newest first) to find current state
        for event in reversed(events):
            ev_type = event.get("event", "")

            # Commander info
            if ev_type == "Commander" and not commander:
                commander = {
                    "name": event.get("Name"),
                    "frontier_id": event.get("FID"),
                }

            if ev_type == "Fileheader" and not fileheader:
                fileheader = {
                    "game_version": event.get("gameversion"),
                    "game_build": event.get("build"),
                    "language": event.get("language"),
                }

            if ev_type == "LoadGame" and "ship" not in commander:
                commander["ship"] = event.get("Ship_Localised", event.get("Ship"))
                commander["ship_name"] = event.get("ShipName")
                commander["ship_ident"] = event.get("ShipIdent")
                commander["game_mode"] = event.get("GameMode")

            # Current system detection
            if ev_type in ("FSDJump", "Location", "CarrierJump"):
                if current_system is None:
                    current_system = {
                        "name": event.get("StarSystem"),
                        "address": event.get("SystemAddress"),
                        "coordinates": event.get("StarPos"),
                        "economy": event.get("SystemEconomy_Localised"),
                        "government": event.get("SystemGovernment_Localised"),
                        "allegiance": event.get("SystemAllegiance"),
                        "population": event.get("Population"),
                        "security": event.get("SystemSecurity_Localised"),
                    }
                    system_event = event
                    system_journal = jf
                elif history_count > 0 and len(history) < history_count:
                    history.append({
                        "name": event.get("StarSystem"),
                        "coordinates": event.get("StarPos"),
                        "timestamp": event.get("timestamp"),
                    })

        # If we found the current system, now collect bodies from that point forward
        if current_system and system_journal == jf:
            system_address = current_system["address"]
            # Re-process events forward from the system entry
            found_system = False
            for event in events:
                ev_type = event.get("event", "")

                # Find the system entry point
                if not found_system:
                    if ev_type in ("FSDJump", "Location", "CarrierJump"):
                        if event.get("SystemAddress") == system_address:
                            found_system = True
                    continue

                # Stop if we jump to another system
                if ev_type in ("FSDJump", "CarrierJump"):
                    break

                # Collect body scans
                if ev_type == "Scan":
                    body = {
                        "name": event.get("BodyName"),
                        "body_id": event.get("BodyID"),
                        "type": event.get("StarType") and "Star" or (
                            "Belt Cluster" if "Belt Cluster" in event.get("BodyName", "") else "Body"
                        ),
                        "star_type": event.get("StarType"),
                        "planet_class": event.get("PlanetClass"),
                        "mass": event.get("StellarMass") or event.get("MassEM"),
                        "radius": event.get("Radius"),
                        "surface_temp": event.get("SurfaceTemperature"),
                        "surface_gravity": event.get("SurfaceGravity"),
                        "surface_pressure": event.get("SurfacePressure"),
                        "atmosphere": event.get("Atmosphere"),
                        "atmosphere_type": event.get("AtmosphereType"),
                        "volcanism": event.get("Volcanism"),
                        "distance_from_arrival": event.get("DistanceFromArrivalLS"),
                        "orbital_period": event.get("OrbitalPeriod"),
                        "rotation_period": event.get("RotationPeriod"),
                        "axial_tilt": event.get("AxialTilt"),
                        "eccentricity": event.get("Eccentricity"),
                        "semi_major_axis": event.get("SemiMajorAxis"),
                        "landable": event.get("Landable"),
                        "terraform_state": event.get("TerraformState"),
                        "was_discovered": event.get("WasDiscovered"),
                        "was_mapped": event.get("WasMapped"),
                        "parents": event.get("Parents"),
                        "rings": [
                            {
                                "name": r.get("Name"),
                                "class": r.get("RingClass"),
                                "mass": r.get("MassMT"),
                            }
                            for r in event.get("Rings", [])
                        ] or None,
                        "timestamp": event.get("timestamp"),
                    }
                    # Clean out None values
                    body = {k: v for k, v in body.items() if v is not None}
                    bodies.append(body)

                # FSS Discovery Scan (honk)
                if ev_type == "FSSDiscoveryScan":
                    honk = {
                        "body_count": event.get("BodyCount"),
                        "non_body_count": event.get("NonBodyCount"),
                        "progress": event.get("Progress"),
                        "timestamp": event.get("timestamp"),
                    }

                # Body signals (bio/geo)
                if ev_type == "FSSBodySignals":
                    body_name = event.get("BodyName")
                    sigs = []
                    for s in event.get("Signals", []):
                        sigs.append({
                            "type": s.get("Type_Localised", s.get("Type")),
                            "count": s.get("Count"),
                        })
                    signals[body_name] = sigs

                # SAA scan complete (DSS)
                if ev_type == "SAAScanComplete":
                    scans_complete.append({
                        "body_name": event.get("BodyName"),
                        "body_id": event.get("BodyID"),
                        "probes_used": event.get("ProbesUsed"),
                        "efficiency_target": event.get("EfficiencyTarget"),
                        "timestamp": event.get("timestamp"),
                    })

                # SAASignalsFound (detailed surface signals from DSS)
                if ev_type == "SAASignalsFound":
                    body_name = event.get("BodyName")
                    if body_name not in signals:
                        signals[body_name] = []
                    for s in event.get("Signals", []):
                        sig = {
                            "type": s.get("Type_Localised", s.get("Type")),
                            "count": s.get("Count"),
                        }
                        if sig not in signals[body_name]:
                            signals[body_name].append(sig)
                    # Genuses from DSS
                    genuses = event.get("Genuses", [])
                    if genuses:
                        if "_genuses" not in signals:
                            signals["_genuses"] = {}
                        signals["_genuses"][body_name] = [
                            g.get("Genus_Localised", g.get("Genus"))
                            for g in genuses
                        ]

            break  # Done — we found and processed the current system

    # Continue collecting history from older journals if needed
    if history_count > 0 and len(history) < history_count:
        for jf, events in all_events:
            if jf == system_journal:
                continue
            for event in reversed(events):
                if event.get("event") in ("FSDJump", "Location", "CarrierJump"):
                    if len(history) < history_count:
                        history.append({
                            "name": event.get("StarSystem"),
                            "coordinates": event.get("StarPos"),
                            "timestamp": event.get("timestamp"),
                        })

    # Build final snapshot
    snapshot = {
        "snapshot_source": "cmdr-snapshot skill — journal parser",
        "journal_file": system_journal,
        "commander": commander,
        "game": fileheader,
        "system": current_system,
        "honk": honk,
        "bodies": bodies,
        "body_count": len(bodies),
        "signals": {k: v for k, v in signals.items() if k != "_genuses"},
        "genuses": signals.get("_genuses", {}),
        "dss_scans": scans_complete,
        "timestamp": system_event.get("timestamp") if system_event else None,
    }

    if history:
        snapshot["recent_systems"] = history

    return snapshot


def main():
    parser = argparse.ArgumentParser(description="CMDR Snapshot — Journal State Reader")
    parser.add_argument("--history", type=int, default=0,
                        help="Include last N systems visited")
    parser.add_argument("--raw", action="store_true",
                        help="Output raw JSON (no pretty print)")
    args = parser.parse_args()

    snapshot = build_snapshot(history_count=args.history)

    if args.raw:
        print(json.dumps(snapshot))
    else:
        print(json.dumps(snapshot, indent=2))


if __name__ == "__main__":
    main()
