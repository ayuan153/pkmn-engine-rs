#!/usr/bin/env python3
"""
Fetch Pokemon Showdown replay fixtures for differential testing.

Downloads Gen 9 Random Battle replays and parses protocol logs into
structured event-based fixtures for damage verification.

Usage:
    python tools/fetch-fixtures.py [--count N] [--output tests/fixtures/replay_events]
    python tools/fetch-fixtures.py --parse-local tests/fixtures/replays
"""

import argparse
import json
import os
import re
import sys

import requests

REPLAY_SEARCH_URL = "https://replay.pokemonshowdown.com/search.json"
REPLAY_URL = "https://replay.pokemonshowdown.com/{}.json"


def fetch_replay_ids(count=5):
    """Fetch recent gen9randombattle replay IDs."""
    resp = requests.get(REPLAY_SEARCH_URL, params={"format": "gen9randombattle"}, timeout=10)
    resp.raise_for_status()
    replays = resp.json()
    return [r["id"] for r in replays[:count]]


def fetch_replay(replay_id):
    """Fetch a single replay's JSON data."""
    resp = requests.get(REPLAY_URL.format(replay_id), timeout=10)
    resp.raise_for_status()
    return resp.json()


def parse_hp_field(hp_str):
    """Parse HP string like '245/245', '67/100', '0 fnt', or percentage '45/100'."""
    hp_str = hp_str.strip()
    if "fnt" in hp_str:
        return 0, None
    # Remove status conditions appended after space (e.g., "245/245 brn")
    hp_str = hp_str.split()[0]
    parts = hp_str.split("/")
    hp = int(parts[0])
    max_hp = int(parts[1]) if len(parts) > 1 else None
    return hp, max_hp


def parse_protocol_log(log):
    """Parse a PS protocol log into structured events."""
    events = []
    current_turn = 0
    # Track max_hp per player slot for percentage conversion
    known_max_hp = {"p1": None, "p2": None}
    # Accumulate crit/effectiveness flags for the next damage event
    pending_flags = {}

    for line in log.split("\n"):
        line = line.strip()
        if not line or not line.startswith("|"):
            continue

        parts = line.split("|")
        # parts[0] is empty string before first |
        if len(parts) < 2:
            continue
        cmd = parts[1]

        if cmd == "turn":
            current_turn = int(parts[2])
            events.append({"type": "turn", "turn": current_turn})

        elif cmd in ("switch", "drag"):
            if len(parts) >= 5:
                ident = parts[2]  # "p1a: Garchomp"
                player = ident[:2]
                details = parts[3]  # "Garchomp, L75, M"
                hp_str = parts[4]  # "245/245"

                species = details.split(",")[0].strip()
                level_match = re.search(r"L(\d+)", details)
                level = int(level_match.group(1)) if level_match else 100

                hp, max_hp = parse_hp_field(hp_str)
                if max_hp:
                    known_max_hp[player] = max_hp

                event = {"type": "switch", "player": player, "species": species, "level": level}
                if max_hp:
                    event["hp"] = hp
                    event["max_hp"] = max_hp
                events.append(event)

        elif cmd == "move":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                move_name = parts[3].strip()
                target = parts[4][:2] if len(parts) >= 5 and parts[4].startswith("p") else None
                event = {"type": "move", "player": player, "move": move_name}
                if target:
                    event["target"] = target
                events.append(event)

        elif cmd == "-crit":
            if len(parts) >= 3:
                pending_flags["crit"] = True

        elif cmd == "-supereffective":
            pending_flags["effectiveness"] = "super"

        elif cmd == "-resisted":
            pending_flags["effectiveness"] = "resisted"

        elif cmd == "-miss":
            if len(parts) >= 3:
                ident = parts[2]
                player = ident[:2]
                target = parts[3][:2] if len(parts) >= 4 and parts[3].startswith("p") else None
                event = {"type": "miss", "player": player}
                if target:
                    event["target"] = target
                events.append(event)
                pending_flags = {}

        elif cmd == "-damage":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                hp, max_hp = parse_hp_field(parts[3])
                if max_hp:
                    known_max_hp[player] = max_hp
                event = {"type": "damage", "player": player, "hp": hp}
                if max_hp:
                    event["max_hp"] = max_hp
                # Source of damage if present
                if len(parts) >= 5 and parts[4].strip():
                    event["source"] = parts[4].strip()
                # Merge pending crit/effectiveness flags
                event.update(pending_flags)
                pending_flags = {}
                events.append(event)

        elif cmd == "-heal":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                hp, max_hp = parse_hp_field(parts[3])
                if max_hp:
                    known_max_hp[player] = max_hp
                event = {"type": "heal", "player": player, "hp": hp}
                if max_hp:
                    event["max_hp"] = max_hp
                events.append(event)

        elif cmd == "-status":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                status = parts[3].strip()
                events.append({"type": "status", "player": player, "status": status})

        elif cmd == "-curestatus":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                events.append({"type": "curestatus", "player": player})

        elif cmd == "faint":
            if len(parts) >= 3:
                ident = parts[2]
                player = ident[:2]
                events.append({"type": "faint", "player": player})

        elif cmd == "-weather":
            if len(parts) >= 3:
                weather = parts[2].strip()
                event = {"type": "weather", "weather": weather}
                if len(parts) >= 4 and parts[3].strip():
                    event["source"] = parts[3].strip()
                events.append(event)

        elif cmd == "win":
            if len(parts) >= 3:
                events.append({"type": "win", "player": parts[2].strip()})

        elif cmd == "-boost" or cmd == "-unboost":
            if len(parts) >= 5:
                ident = parts[2]
                player = ident[:2]
                stat = parts[3].strip()
                amount = int(parts[4].strip())
                events.append({
                    "type": "boost" if cmd == "-boost" else "unboost",
                    "player": player,
                    "stat": stat,
                    "amount": amount,
                })

        elif cmd == "-ability":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                ability = parts[3].strip()
                events.append({"type": "ability", "player": player, "ability": ability})

        elif cmd == "-terastallize":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                tera_type = parts[3].strip()
                events.append({"type": "terastallize", "player": player, "tera_type": tera_type})

        elif cmd == "-item":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                item = parts[3].strip()
                events.append({"type": "item", "player": player, "item": item})

        elif cmd == "-enditem":
            if len(parts) >= 4:
                ident = parts[2]
                player = ident[:2]
                events.append({"type": "enditem", "player": player})

        elif cmd == "-sidestart":
            if len(parts) >= 4:
                side = parts[2].strip()
                player = side[:2]
                condition = parts[3].strip()
                # Normalize: "move: Reflect" -> "Reflect"
                if condition.startswith("move: "):
                    condition = condition[6:]
                events.append({"type": "sidestart", "player": player, "condition": condition})

        elif cmd == "-sideend":
            if len(parts) >= 4:
                side = parts[2].strip()
                player = side[:2]
                condition = parts[3].strip()
                if condition.startswith("move: "):
                    condition = condition[6:]
                events.append({"type": "sideend", "player": player, "condition": condition})

    return events


def parse_inputlog(inputlog):
    """Parse inputlog into structured choices."""
    if not inputlog:
        return None
    choices = []
    for line in inputlog.split("\n"):
        line = line.strip()
        if not line.startswith(">"):
            continue
        # >p1 move 1
        parts = line[1:].split(None, 2)
        if len(parts) >= 2:
            choices.append({"player": parts[0], "action": parts[1], "args": parts[2] if len(parts) > 2 else ""})
    return choices


def replay_to_event_fixture(replay_data):
    """Convert a full replay JSON to our event fixture format."""
    log = replay_data.get("log", "")
    events = parse_protocol_log(log)

    fixture = {
        "id": replay_data.get("id", "unknown"),
        "format": replay_data.get("format", "gen9randombattle"),
        "events": events,
    }

    # Include inputlog if available (randbats have this)
    inputlog = replay_data.get("inputlog", "")
    if inputlog:
        fixture["inputlog"] = parse_inputlog(inputlog)

    # Extract winner from events
    for e in reversed(events):
        if e["type"] == "win":
            fixture["winner"] = e["player"]
            break

    return fixture


def main():
    parser = argparse.ArgumentParser(description="Fetch PS replay fixtures (event-based)")
    parser.add_argument("--count", type=int, default=5, help="Number of replays to fetch")
    parser.add_argument("--output", default="tests/fixtures/replay_events", help="Output directory")
    parser.add_argument("--parse-local", help="Parse existing replay JSON files from this directory instead of fetching")
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    if args.parse_local:
        # Parse existing downloaded replays
        local_dir = args.parse_local
        count = 0
        for fname in sorted(os.listdir(local_dir)):
            if not fname.endswith(".json"):
                continue
            path = os.path.join(local_dir, fname)
            with open(path) as f:
                data = json.load(f)

            # Skip hand-crafted fixtures (they have "teams" with full data)
            if "teams" in data and "turns" in data and "log" not in data:
                # This is an old-format fixture from the turn-based parser.
                # We can still convert it by re-parsing if it has raw log.
                print(f"  Skipping {fname} (old format, no raw log)")
                continue

            # If it has a "log" field, parse it
            if "log" in data:
                fixture = replay_to_event_fixture(data)
            else:
                # It's already a parsed fixture from our old format - skip
                print(f"  Skipping {fname} (no log field)")
                continue

            out_path = os.path.join(args.output, fname)
            with open(out_path, "w") as f:
                json.dump(fixture, f, indent=2)
            count += 1
            print(f"  {fname} -> {len(fixture['events'])} events")

        print(f"Done! {count} fixtures saved to {args.output}/")
        return

    # Fetch from PS API
    print(f"Fetching {args.count} gen9randombattle replays...")
    replay_ids = fetch_replay_ids(args.count)

    for replay_id in replay_ids:
        print(f"  Processing {replay_id}...")
        replay_data = fetch_replay(replay_id)

        # Save raw replay for future re-parsing
        raw_dir = os.path.join(os.path.dirname(args.output), "replays_raw")
        os.makedirs(raw_dir, exist_ok=True)
        raw_path = os.path.join(raw_dir, f"{replay_id}.json")
        with open(raw_path, "w") as f:
            json.dump(replay_data, f, indent=2)

        # Parse into event format
        fixture = replay_to_event_fixture(replay_data)
        out_path = os.path.join(args.output, f"{replay_id}.json")
        with open(out_path, "w") as f:
            json.dump(fixture, f, indent=2)
        print(f"    -> {out_path} ({len(fixture['events'])} events)")

    print(f"Done! {len(replay_ids)} fixtures saved to {args.output}/")


if __name__ == "__main__":
    main()
