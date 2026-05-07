#!/usr/bin/env python3
"""
Fetch Pokemon Showdown replay fixtures for differential testing.

Downloads Gen 9 Random Battle replays and extracts turn-by-turn outcomes
into JSON fixtures that our Rust engine can replay and compare against.

Usage:
    python tools/fetch-fixtures.py [--count N] [--output tests/fixtures]
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


def parse_hp(hp_str):
    """Parse HP string like '67/100' or '245/245' or '0 fnt'."""
    if "fnt" in hp_str:
        return 0
    parts = hp_str.split("/")
    return int(parts[0].split()[0])


def parse_max_hp(hp_str):
    """Parse max HP from '245/245' format."""
    if "fnt" in hp_str:
        return None
    parts = hp_str.split("/")
    return int(parts[1].split()[0]) if len(parts) > 1 else 100


def parse_replay_log(log):
    """Parse a replay protocol log into structured fixture data."""
    teams = {"p1": [], "p2": []}
    turns = []
    current_turn = 0
    # Track state per side
    state = {
        "p1": {"hp": None, "max_hp": None, "status": None, "fainted": False, "species": None},
        "p2": {"hp": None, "max_hp": None, "status": None, "fainted": False, "species": None},
    }
    moves_this_turn = {"p1": None, "p2": None}
    winner = None
    seen_species = {"p1": set(), "p2": set()}

    for line in log.split("\n"):
        parts = line.split("|")
        if len(parts) < 2:
            continue
        cmd = parts[1]

        if cmd == "turn":
            # Save previous turn state
            if current_turn > 0:
                turns.append({
                    "turn": current_turn,
                    "p1_choice": moves_this_turn["p1"],
                    "p2_choice": moves_this_turn["p2"],
                    "expected": {
                        "p1_active_hp": state["p1"]["hp"],
                        "p2_active_hp": state["p2"]["hp"],
                        "p1_active_status": state["p1"]["status"],
                        "p2_active_status": state["p2"]["status"],
                        "p1_active_fainted": state["p1"]["fainted"],
                        "p2_active_fainted": state["p2"]["fainted"],
                    },
                })
            current_turn = int(parts[2])
            moves_this_turn = {"p1": None, "p2": None}

        elif cmd == "switch" or cmd == "drag":
            # |switch|p1a: Garchomp|Garchomp, L75, M|245/245
            if len(parts) >= 5:
                player = parts[2][:2]  # p1 or p2
                details = parts[3]
                hp_str = parts[4]
                species = details.split(",")[0]
                level_match = re.search(r"L(\d+)", details)
                level = int(level_match.group(1)) if level_match else 100
                hp = parse_hp(hp_str)
                max_hp = parse_max_hp(hp_str)

                state[player]["species"] = species
                state[player]["hp"] = hp
                state[player]["max_hp"] = max_hp
                state[player]["fainted"] = False
                state[player]["status"] = None

                if species not in seen_species[player]:
                    seen_species[player].add(species)
                    teams[player].append({"species": species, "level": level})

                if current_turn > 0:
                    moves_this_turn[player] = f"switch {species}"

        elif cmd == "move":
            # |move|p1a: Garchomp|Earthquake|p2a: Pikachu
            if len(parts) >= 4:
                player = parts[2][:2]
                move_name = parts[3]
                moves_this_turn[player] = f"move {move_name}"

        elif cmd == "-damage" or cmd == "-heal":
            # |-damage|p2a: Pikachu|67/100
            if len(parts) >= 4:
                player = parts[2][:2]
                hp = parse_hp(parts[3])
                state[player]["hp"] = hp
                if hp == 0:
                    state[player]["fainted"] = True

        elif cmd == "-status":
            # |-status|p2a: Pikachu|brn
            if len(parts) >= 4:
                player = parts[2][:2]
                state[player]["status"] = parts[3]

        elif cmd == "-curestatus":
            if len(parts) >= 4:
                player = parts[2][:2]
                state[player]["status"] = None

        elif cmd == "faint":
            # |faint|p2a: Pikachu
            if len(parts) >= 3:
                player = parts[2][:2]
                state[player]["fainted"] = True
                state[player]["hp"] = 0

        elif cmd == "win":
            if len(parts) >= 3:
                winner = parts[2]

    # Save last turn
    if current_turn > 0:
        turns.append({
            "turn": current_turn,
            "p1_choice": moves_this_turn["p1"],
            "p2_choice": moves_this_turn["p2"],
            "expected": {
                "p1_active_hp": state["p1"]["hp"],
                "p2_active_hp": state["p2"]["hp"],
                "p1_active_status": state["p1"]["status"],
                "p2_active_status": state["p2"]["status"],
                "p1_active_fainted": state["p1"]["fainted"],
                "p2_active_fainted": state["p2"]["fainted"],
            },
        })

    return {"teams": teams, "turns": turns, "winner": winner}


def main():
    parser = argparse.ArgumentParser(description="Fetch PS replay fixtures")
    parser.add_argument("--count", type=int, default=5, help="Number of replays to fetch")
    parser.add_argument("--output", default="tests/fixtures", help="Output directory")
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    print(f"Fetching {args.count} gen9randombattle replays...")
    replay_ids = fetch_replay_ids(args.count)

    for replay_id in replay_ids:
        print(f"  Processing {replay_id}...")
        replay = fetch_replay(replay_id)
        fixture = parse_replay_log(replay["log"])
        fixture["id"] = replay_id

        out_path = os.path.join(args.output, f"{replay_id}.json")
        with open(out_path, "w") as f:
            json.dump(fixture, f, indent=2)
        print(f"    -> {out_path} ({len(fixture['turns'])} turns)")

    print(f"Done! {len(replay_ids)} fixtures saved to {args.output}/")


if __name__ == "__main__":
    main()
