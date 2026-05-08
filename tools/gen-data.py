#!/usr/bin/env python3
"""Generate Rust data files from Pokemon Showdown JSON data."""

import json
import os
import re
import urllib.request
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent
CACHE_DIR = SCRIPT_DIR / "cache"
GEN_DIR = PROJECT_ROOT / "crates" / "pkmn-core" / "src" / "gen"

URLS = {
    "pokedex": "https://play.pokemonshowdown.com/data/pokedex.json",
    "moves": "https://play.pokemonshowdown.com/data/moves.json",
    "abilities": "https://play.pokemonshowdown.com/data/abilities.js",
}

TYPE_MAP = {
    "Normal": "Type::Normal", "Fire": "Type::Fire", "Water": "Type::Water",
    "Electric": "Type::Electric", "Grass": "Type::Grass", "Ice": "Type::Ice",
    "Fighting": "Type::Fighting", "Poison": "Type::Poison", "Ground": "Type::Ground",
    "Flying": "Type::Flying", "Psychic": "Type::Psychic", "Bug": "Type::Bug",
    "Rock": "Type::Rock", "Ghost": "Type::Ghost", "Dragon": "Type::Dragon",
    "Dark": "Type::Dark", "Steel": "Type::Steel", "Fairy": "Type::Fairy",
}

FLAG_MAP = {
    "contact": "MoveFlags::CONTACT",
    "sound": "MoveFlags::SOUND",
    "bullet": "MoveFlags::BULLET",
    "punch": "MoveFlags::PUNCH",
    "bite": "MoveFlags::BITE",
    "pulse": "MoveFlags::PULSE",
    "slicing": "MoveFlags::SLICING",
}


def download(name: str) -> dict:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    cache_file = CACHE_DIR / f"{name}.json"
    if not cache_file.exists():
        print(f"Downloading {name}...")
        req = urllib.request.Request(URLS[name], headers={"User-Agent": "pkmn-engine-rs/1.0"})
        with urllib.request.urlopen(req) as resp:
            raw = resp.read().decode("utf-8")
        # abilities.js has format: exports.BattleAbilities = {...}
        if raw.startswith("exports."):
            raw = raw[raw.index("{"):]
            # Convert JS object to JSON: quote unquoted keys
            raw = re.sub(r'(?<=[{,])(\s*)([a-zA-Z_]\w*)(\s*):', r'\1"\2"\3:', raw)
            # Strip trailing semicolons
            raw = raw.rstrip().rstrip(";")
        cache_file.write_text(raw)
    with open(cache_file) as f:
        return json.load(f)


def gen_species(pokedex: dict) -> str:
    lines = [
        "// Auto-generated from Pokemon Showdown data. Do not edit manually.",
        "use crate::types::Type;",
        "use crate::species::{BaseStats, SpeciesData};",
        "",
        "pub const SPECIES_DATA: &[SpeciesData] = &[",
    ]

    entries = []
    for key, data in pokedex.items():
        num = data.get("num", 0)
        if num <= 0:
            continue
        # Skip CAP, future, etc
        ns = data.get("isNonstandard")
        if ns in ("CAP", "Custom", "Future"):
            continue
        name = data["name"]
        types = data.get("types", ["Normal"])
        t1 = TYPE_MAP.get(types[0], "Type::Normal")
        t2 = TYPE_MAP.get(types[1], t1) if len(types) > 1 else t1
        bs = data.get("baseStats", {})
        hp = bs.get("hp", 1)
        atk = bs.get("atk", 1)
        dfn = bs.get("def", 1)
        spa = bs.get("spa", 1)
        spd = bs.get("spd", 1)
        spe = bs.get("spe", 1)
        weight = int(data.get("weightkg", 0) * 10)
        # Clamp stats to u8
        hp, atk, dfn, spa, spd, spe = [min(255, s) for s in [hp, atk, dfn, spa, spd, spe]]
        weight = min(65535, weight)
        entries.append((num, name, t1, t2, hp, atk, dfn, spa, spd, spe, weight))

    # Sort by national dex number
    entries.sort(key=lambda x: x[0])

    for num, name, t1, t2, hp, atk, dfn, spa, spd, spe, weight in entries:
        # Escape quotes in name
        escaped = name.replace('"', '\\"')
        lines.append(
            f'    SpeciesData {{ id: {num}, name: "{escaped}", '
            f'types: [{t1}, {t2}], '
            f'base_stats: BaseStats {{ hp: {hp}, atk: {atk}, def: {dfn}, spa: {spa}, spd: {spd}, spe: {spe} }}, '
            f'weight_hg: {weight} }},'
        )

    lines.append("];")
    lines.append("")
    lines.append("pub fn get_species_by_name(name: &str) -> Option<&'static SpeciesData> {")
    lines.append("    SPECIES_DATA.iter().find(|s| s.name.eq_ignore_ascii_case(name))")
    lines.append("}")
    lines.append("")
    lines.append("pub fn get_species_by_id(id: u16) -> Option<&'static SpeciesData> {")
    lines.append("    SPECIES_DATA.iter().find(|s| s.id == id)")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def gen_moves(moves: dict) -> str:
    lines = [
        "// Auto-generated from Pokemon Showdown data. Do not edit manually.",
        "use crate::types::Type;",
        "use crate::moves::{MoveCategory, MoveData, MoveFlags};",
        "",
        "pub const MOVE_DATA: &[MoveData] = &[",
    ]

    entries = []
    for key, data in moves.items():
        num = data.get("num", 0)
        if num <= 0:
            continue
        ns = data.get("isNonstandard")
        if ns in ("CAP", "Custom", "Future"):
            continue
        name = data["name"]
        mtype = TYPE_MAP.get(data.get("type", "Normal"), "Type::Normal")
        cat = data.get("category", "Status")
        category = f"MoveCategory::{cat}" if cat in ("Physical", "Special", "Status") else "MoveCategory::Status"
        bp = min(255, data.get("basePower", 0))
        acc = data.get("accuracy")
        if acc is True or acc is None:
            acc = 0
        else:
            acc = min(255, int(acc))
        pp = min(255, data.get("pp", 1))
        priority = data.get("priority", 0)
        # Clamp priority to i8
        priority = max(-128, min(127, priority))

        # Build flags
        flag_bits = []
        flags_dict = data.get("flags", {})
        for ps_flag, rust_flag in FLAG_MAP.items():
            if ps_flag in flags_dict:
                flag_bits.append(rust_flag)

        if flag_bits:
            flags_expr = f"MoveFlags::new({' | '.join(flag_bits)})"
        else:
            flags_expr = "MoveFlags::NONE"

        entries.append((num, name, mtype, category, bp, acc, pp, priority, flags_expr))

    entries.sort(key=lambda x: x[0])

    for num, name, mtype, category, bp, acc, pp, priority, flags_expr in entries:
        escaped = name.replace('"', '\\"')
        lines.append(
            f'    MoveData {{ id: {num}, name: "{escaped}", '
            f'move_type: {mtype}, category: {category}, '
            f'base_power: {bp}, accuracy: {acc}, pp: {pp}, priority: {priority}, '
            f'flags: {flags_expr} }},'
        )

    lines.append("];")
    lines.append("")
    lines.append("pub fn get_move_by_name(name: &str) -> Option<&'static MoveData> {")
    lines.append("    MOVE_DATA.iter().find(|m| m.name.eq_ignore_ascii_case(name))")
    lines.append("}")
    lines.append("")
    lines.append("pub fn get_move_by_id(id: u16) -> Option<&'static MoveData> {")
    lines.append("    MOVE_DATA.iter().find(|m| m.id == id)")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def gen_abilities(abilities: dict) -> str:
    lines = [
        "// Auto-generated from Pokemon Showdown data. Do not edit manually.",
        "",
        "pub struct GenAbilityData {",
        "    pub id: u16,",
        "    pub name: &'static str,",
        "}",
        "",
        "pub const ABILITY_DATA: &[GenAbilityData] = &[",
    ]

    entries = []
    for key, data in abilities.items():
        num = data.get("num", 0)
        if num <= 0:
            continue
        ns = data.get("isNonstandard")
        if ns in ("CAP", "Custom", "Future"):
            continue
        name = data["name"]
        entries.append((num, name))

    entries.sort(key=lambda x: x[0])

    for num, name in entries:
        escaped = name.replace('"', '\\"')
        lines.append(f'    GenAbilityData {{ id: {num}, name: "{escaped}" }},')

    lines.append("];")
    lines.append("")
    lines.append("pub fn get_ability_by_name(name: &str) -> Option<&'static GenAbilityData> {")
    lines.append("    ABILITY_DATA.iter().find(|a| a.name.eq_ignore_ascii_case(name))")
    lines.append("}")
    lines.append("")
    lines.append("pub fn get_ability_by_id(id: u16) -> Option<&'static GenAbilityData> {")
    lines.append("    ABILITY_DATA.iter().find(|a| a.id == id)")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def gen_mod() -> str:
    return """// Auto-generated module. Do not edit manually.
pub mod species_data;
pub mod move_data;
pub mod ability_data;

pub use species_data::SPECIES_DATA;
pub use move_data::MOVE_DATA;
pub use ability_data::{GenAbilityData, ABILITY_DATA};
"""


def main():
    pokedex = download("pokedex")
    moves = download("moves")
    abilities = download("abilities")

    GEN_DIR.mkdir(parents=True, exist_ok=True)

    species_rs = gen_species(pokedex)
    moves_rs = gen_moves(moves)
    abilities_rs = gen_abilities(abilities)
    mod_rs = gen_mod()

    (GEN_DIR / "species_data.rs").write_text(species_rs)
    (GEN_DIR / "move_data.rs").write_text(moves_rs)
    (GEN_DIR / "ability_data.rs").write_text(abilities_rs)
    (GEN_DIR / "mod.rs").write_text(mod_rs)

    # Count entries
    species_count = species_rs.count("SpeciesData {")
    moves_count = moves_rs.count("MoveData {")
    abilities_count = abilities_rs.count("GenAbilityData {") - 1  # subtract struct def
    print(f"Generated {species_count} species, {moves_count} moves, {abilities_count} abilities")
    print(f"Output: {GEN_DIR}")


if __name__ == "__main__":
    main()
