#!/usr/bin/env python3
import os
import sys
import time
import json
from pathlib import Path

# ANSI colors for styling the output
RESET = "\033[0m"
BOLD = "\033[1m"
ORANGE = "\033[38;5;208m"
DIM_ORANGE = "\033[38;5;130m"
YELLOW = "\033[93m"
BLUE = "\033[94m"
GREY = "\033[90m"
GREEN = "\033[92m"
CYAN = "\033[96m"
RED = "\033[91m"

def discover_journal_dir():
    # Check if a custom path was passed as an argument
    if len(sys.argv) > 1:
        path = Path(sys.argv[1])
        if path.is_dir():
            return path
        print(f"{RED}Error: Explicit path {path} is not a directory.{RESET}")
        sys.exit(1)
        
    # Default Steam Proton path on Linux
    home = Path.home()
    default_path = home / ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous"
    if default_path.is_dir():
        return default_path
        
    print(f"{RED}Error: Default journal directory not found.{RESET}")
    print(f"Please run as: {sys.argv[0]} <path_to_journal_dir>")
    sys.exit(1)

def get_latest_log_file(journal_dir):
    log_files = list(journal_dir.glob("Journal.*.log"))
    if not log_files:
        return None
    # Sort by filename which contains timestamp
    log_files.sort()
    return log_files[-1]

def format_event(event_type, data):
    summary = ""
    
    if event_type == "FSDJump":
        star_system = data.get("StarSystem", "Unknown")
        body = data.get("Body", "Unknown")
        dist = data.get("JumpDist", 0.0)
        summary = f"{BOLD}{YELLOW}Jumped to {star_system}{RESET} ({dist} Ls) | Body: {body}"
        
    elif event_type == "Location":
        star_system = data.get("StarSystem", "Unknown")
        body = data.get("Body", "Unknown")
        body_type = data.get("BodyType", "Unknown")
        summary = f"{BOLD}{BLUE}Located in {star_system}{RESET} | Current Body: {body} ({body_type})"
        
    elif event_type == "FSSDiscoveryScan":
        body_count = data.get("BodyCount", 0)
        non_body_count = data.get("NonBodyCount", 0)
        summary = f"{ORANGE}FSS Discovery Scan (Honk) completed!{RESET} | {BOLD}{body_count} bodies{RESET} | {non_body_count} non-bodies"
        
    elif event_type == "Scan":
        body_name = data.get("BodyName", "Unknown")
        scan_type = data.get("ScanType", "Detailed")
        was_disc = data.get("WasDiscovered", True)
        was_mapped = data.get("WasMapped", True)
        dist = data.get("DistanceFromArrivalLS", 0.0)
        
        disc_marker = f"{RED}[NEW]{RESET}" if not was_disc else f"{GREY}[Old]{RESET}"
        map_marker = f"{RED}[DSS NEW]{RESET}" if not was_mapped else f"{GREY}[DSS Old]{RESET}"
        
        if "StarType" in data:
            star_type = data.get("StarType", "?")
            mass = data.get("StellarMass", 0.0)
            summary = f"{YELLOW}Star Scan{RESET} | {BOLD}{body_name}{RESET} (Type {star_type}) | Mass: {mass:.2f} M_sol | Dist: {dist:.1f} Ls | {disc_marker}"
        else:
            planet_class = data.get("PlanetClass", "Unknown")
            mass = data.get("MassEM", 0.0)
            gravity = data.get("SurfaceGravity", 0.0) / 9.80665 # convert to Gs if represented in m/s^2, or check if already in Gs
            # Elite Dangerous journals actually output SurfaceGravity directly in m/s^2 or Gs?
            # Let's print the raw value first so we can verify:
            raw_gravity = data.get("SurfaceGravity", 0.0)
            # Actually, ED journal SurfaceGravity is in m/s^2 (value 9.8 = 1G) or Gs?
            # Let's display both to be safe or raw. EDEB shows e.g. 0.12 g.
            # In SCAN_PLANET_JSON, SurfaceGravity is 34.01 m/s^2. Wait, 34.01 / 9.81 = 3.46 G. Let's show both.
            gravity_str = f"{raw_gravity:.2f} m/s^2" if raw_gravity > 10 else f"{raw_gravity:.2f} G"
            temp = data.get("SurfaceTemperature", 0.0)
            atmo = data.get("Atmosphere", "None")
            terraform = data.get("TerraformState", "")
            terraform_str = f" | {BOLD}{GREEN}Terraformable{RESET}" if terraform in ["Terraformable", "Terraforming"] else ""
            
            summary = f"{BLUE}Planet Scan{RESET} | {BOLD}{body_name}{RESET} ({planet_class}) | Mass: {mass:.4f} M_earth | Gravity: {gravity_str} | Temp: {temp:.0f}K | Atmo: {atmo}{terraform_str} | Dist: {dist:.1f} Ls | {disc_marker} {map_marker}"
            
    elif event_type == "FSSBodySignals":
        body_name = data.get("BodyName", "Unknown")
        signals = data.get("Signals", [])
        signals_summary = []
        for s in signals:
            sig_type = s.get("Type", "Unknown").replace("$SAA_SignalType_", "").replace(";", "")
            sig_count = s.get("Count", 0)
            color = GREEN if sig_type == "Biological" else YELLOW
            signals_summary.append(f"{color}{sig_type}: {sig_count}{RESET}")
        summary = f"{GREEN}Signals Found{RESET} on {BOLD}{body_name}{RESET} | " + ", ".join(signals_summary)
        
    elif event_type == "SAAScanComplete":
        body_name = data.get("BodyName", "Unknown")
        probes = data.get("ProbesUsed", 0)
        target = data.get("EfficiencyTarget", 0)
        efficient = f"{GREEN}EFFICIENT{RESET}" if probes <= target else f"{RED}INEFFICIENT{RESET}"
        summary = f"{CYAN}DSS Surface Mapping Complete{RESET} | {BOLD}{body_name}{RESET} | Probes: {probes}/{target} ({efficient})"
        
    elif event_type == "SAASignalsFound":
        body_name = data.get("BodyName", "Unknown")
        signals = data.get("Signals", [])
        signals_summary = []
        for s in signals:
            sig_type = s.get("Type", "Unknown").replace("$SAA_SignalType_", "").replace(";", "")
            sig_count = s.get("Count", 0)
            color = GREEN if sig_type == "Biological" else YELLOW
            signals_summary.append(f"{color}{sig_type}: {sig_count}{RESET}")
        summary = f"{CYAN}DSS Signals Localized{RESET} on {BOLD}{body_name}{RESET} | " + ", ".join(signals_summary)
        
    else:
        # Fallback for all other events (NavRoute, FSSAllBodiesFound, CodexEntry, etc.)
        # Exclude common noisy events we don't care about, or show them in muted grey
        keys = list(data.keys())
        keys.remove("timestamp")
        keys.remove("event")
        details = ", ".join(f"{k}: {data[k]}" for k in keys[:4])
        if len(keys) > 4:
            details += "..."
        summary = f"{GREY}{event_type}{RESET} | {details}"
        
    return summary

def main():
    print(f"\n{BOLD}{ORANGE}🎵 UnixplorationBuddy — Telemetry Monitor v1.0{RESET}")
    print(f"{DIM_ORANGE}A pairing tool to audit live Elite Dangerous journal events.{RESET}\n")
    
    journal_dir = discover_journal_dir()
    print(f"Watching Journal Directory: {BOLD}{journal_dir}{RESET}")
    
    latest_file = get_latest_log_file(journal_dir)
    if not latest_file:
        print(f"{RED}No journal log files found in directory!{RESET}")
        sys.exit(1)
        
    print(f"Tracking Live Log File:   {BOLD}{latest_file.name}{RESET}")
    print(f"{GREY}Session capture is active. Press Ctrl+C to stop recording and generate analysis.{RESET}")
    print("-" * 80)
    
    captured_events = []
    
    # Open the file and seek to the end to only capture live telemetry
    with open(latest_file, "r", encoding="utf-8") as f:
        f.seek(0, os.SEEK_END)
        
        try:
            while True:
                # Check if file has rotated (a new journal file is created on game relaunch)
                new_latest = get_latest_log_file(journal_dir)
                if new_latest and new_latest != latest_file:
                    print(f"\n{YELLOW}Journal file rotated! Switching to: {new_latest.name}{RESET}\n")
                    latest_file = new_latest
                    f.close()
                    f = open(latest_file, "r", encoding="utf-8")
                    f.seek(0, os.SEEK_END)
                    
                line = f.readline()
                if not line:
                    time.sleep(0.1)
                    continue
                    
                line = line.strip()
                if not line:
                    continue
                    
                try:
                    data = json.loads(line)
                    event_type = data.get("event", "Unknown")
                    timestamp = data.get("timestamp", "")
                    
                    # Capture it
                    captured_events.append(data)
                    
                    # Format and print the event
                    formatted = format_event(event_type, data)
                    time_str = timestamp.split("T")[-1].replace("Z", "")
                    print(f"[{GREY}{time_str}{RESET}] {formatted}")
                    
                except json.JSONDecodeError:
                    print(f"{RED}Error: Failed to parse JSON line: {line}{RESET}")
                    
        except KeyboardInterrupt:
            print(f"\n\n{BOLD}{GREEN}Telemetry Session Stopped.{RESET}")
            
    # Save the captured session
    session_file = Path("scratch/telemetry_session_log.json")
    session_file.parent.mkdir(parents=True, exist_ok=True)
    with open(session_file, "w", encoding="utf-8") as out:
        json.dump(captured_events, out, indent=2)
        
    print(f"\n{BOLD}Analysis Summary:{RESET}")
    print(f"• Total events captured: {BOLD}{len(captured_events)}{RESET}")
    print(f"• Session log saved to:  {BOLD}{session_file.absolute()}{RESET}")
    print(f"\nReady to build more precise TUI bindings based on this telemetry!")

if __name__ == "__main__":
    main()
