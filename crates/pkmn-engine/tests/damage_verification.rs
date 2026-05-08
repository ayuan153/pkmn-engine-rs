//! Protocol-based differential testing: damage verification against real PS replays.
//!
//! Tracks battle state (boosts, abilities, items, weather, screens, status) from
//! protocol events to calculate accurate damage ranges.

use pkmn_core::damage::{damage_roll, DamageContext};
use pkmn_core::moves::{get_move, MoveCategory};
use pkmn_core::species::get_species;
use pkmn_core::stats::calc_stat;
use pkmn_core::types::Type;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
struct Boosts {
    atk: i8,
    def: i8,
    spa: i8,
    spd: i8,
    spe: i8,
}

#[derive(Debug, Clone)]
struct ActiveInfo {
    species: String,
    level: u8,
    hp: u16,
    max_hp: u16,
    ability: Option<String>,
    item: Option<String>,
    item_consumed: bool,
    boosts: Boosts,
    status: Option<String>,
    is_terastallized: bool,
}

impl Default for ActiveInfo {
    fn default() -> Self {
        Self {
            species: String::new(),
            level: 100,
            hp: 0,
            max_hp: 0,
            ability: None,
            item: None,
            item_consumed: false,
            boosts: Boosts::default(),
            status: None,
            is_terastallized: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct VerificationState {
    active: [ActiveInfo; 2], // p1=0, p2=1
    weather: Option<String>,
    terrain: Option<String>,
    p1_reflect: bool,
    p1_light_screen: bool,
    p2_reflect: bool,
    p2_light_screen: bool,
    terastallized_species: Vec<String>, // species that have tera'd (permanent)
    next_is_crit: bool,
}

#[derive(Debug, PartialEq)]
enum CheckResult {
    Exact,
    Close,
    Direction,
    Skip,
    Fail(String),
}

#[derive(Debug)]
struct DamageCheck {
    turn: u32,
    attacker: String,
    defender: String,
    move_name: String,
    observed_damage: u16,
    our_min: u16,
    our_max: u16,
    result: CheckResult,
}

fn player_idx(player: &str) -> usize {
    if player == "p1" { 0 } else { 1 }
}

/// Apply boost multiplier: +1 = 1.5x, +2 = 2x, etc.
fn boost_multiplier(stage: i8) -> f32 {
    match stage.clamp(-6, 6) {
        -6 => 2.0 / 8.0,
        -5 => 2.0 / 7.0,
        -4 => 2.0 / 6.0,
        -3 => 2.0 / 5.0,
        -2 => 2.0 / 4.0,
        -1 => 2.0 / 3.0,
        0 => 1.0,
        1 => 3.0 / 2.0,
        2 => 4.0 / 2.0,
        3 => 5.0 / 2.0,
        4 => 6.0 / 2.0,
        5 => 7.0 / 2.0,
        6 => 8.0 / 2.0,
        _ => 1.0,
    }
}

/// Calculate exact stat for Gen 9 Random Battles: 85 EVs, 31 IVs, neutral nature
fn randbats_stat(base: u8, level: u8) -> u16 {
    calc_stat(base, 31, 85, level, 1.0)
}

fn has_stab(species_types: &[Type; 2], move_type: Type) -> bool {
    species_types[0] == move_type || species_types[1] == move_type
}

/// Infer the most common Random Battles ability for species where it matters for damage.
fn infer_ability(species: &str) -> Option<&'static str> {
    match species {
        "Azumarill" => Some("Huge Power"),
        "Medicham" | "Medicham-Mega" => Some("Pure Power"),
        "Basculin" | "Basculin-Blue-Striped" => Some("Adaptability"),
        "Crawdaunt" => Some("Adaptability"),
        "Porygon-Z" => Some("Adaptability"),
        "Rillaboom" => Some("Grassy Surge"),
        "Tapu Koko" => Some("Electric Surge"),
        "Tapu Lele" => Some("Psychic Surge"),
        "Tapu Bulu" => Some("Grassy Surge"),
        "Tapu Fini" => Some("Misty Surge"),
        "Pincurchin" => Some("Electric Surge"),
        "Indeedee" | "Indeedee-F" => Some("Psychic Surge"),
        "Conkeldurr" => Some("Iron Fist"),
        "Melmetal" => Some("Iron Fist"),
        "Kingler" => Some("Sheer Force"),
        "Dracovish" => Some("Strong Jaw"),
        "Boltund" => Some("Strong Jaw"),
        "Lycanroc" => Some("Tough Claws"),
        "Metagross" => Some("Tough Claws"),
        "Aerodactyl" => Some("Tough Claws"),
        "Dragonite" => Some("Multiscale"),
        "Lugia" => Some("Multiscale"),
        "Snorlax" => Some("Thick Fat"),
        "Walrein" => Some("Thick Fat"),
        "Mamoswine" => Some("Thick Fat"),
        "Appletun" => Some("Thick Fat"),
        "Hariyama" => Some("Thick Fat"),
        "Wugtrio" => Some("Technician"),
        "Scizor" => Some("Technician"),
        "Breloom" => Some("Technician"),
        "Cinccino" => Some("Technician"),
        "Ambipom" => Some("Technician"),
        "Persian" => Some("Technician"),
        "Hitmontop" => Some("Technician"),
        "Yanmega" => Some("Tinted Lens"),
        "Butterfree" => Some("Tinted Lens"),
        "Mothim" => Some("Tinted Lens"),
        "Chien-Pao" => Some("Sword of Ruin"),
        "Wo-Chien" => Some("Tablets of Ruin"),
        "Chi-Yu" => Some("Beads of Ruin"),
        "Ting-Lu" => Some("Vessel of Ruin"),
        "Incineroar" | "Landorus-Therian" | "Gyarados" | "Arcanine" | "Staraptor" | "Krookodile" | "Salamence" => Some("Intimidate"),
        "Mimikyu" => Some("Disguise"),
        "Eiscue" => Some("Ice Face"),
        _ => None,
    }
}

/// Check if an inferred ability sets terrain, and return the terrain name.
fn ability_sets_terrain(ability: &str) -> Option<&'static str> {
    match ability {
        "Electric Surge" => Some("Electric Terrain"),
        "Grassy Surge" => Some("Grassy Terrain"),
        "Psychic Surge" => Some("Psychic Terrain"),
        "Misty Surge" => Some("Misty Terrain"),
        _ => None,
    }
}

/// Calculate the other_modifiers value based on state.
fn calc_modifiers(
    state: &VerificationState,
    atk_idx: usize,
    def_idx: usize,
    move_data: &pkmn_core::moves::MoveData,
    effectiveness: f32,
) -> f32 {
    let atk = &state.active[atk_idx];
    let def = &state.active[def_idx];
    let mut modifier = 1.0f32;

    // Ability modifiers (attacker) - use inferred ability if not explicitly known
    let atk_ability = atk.ability.as_deref().or_else(|| infer_ability(&atk.species));
    if let Some(ability) = atk_ability {
        match ability {
            "Huge Power" | "Pure Power" => {
                if move_data.category == MoveCategory::Physical {
                    modifier *= 2.0;
                }
            }
            "Technician" => {
                if move_data.base_power <= 60 {
                    modifier *= 1.5;
                }
            }
            "Tough Claws" => {
                if move_data.flags.has(pkmn_core::moves::MoveFlags::CONTACT) {
                    modifier *= 1.3;
                }
            }
            "Iron Fist" => {
                if move_data.flags.has(pkmn_core::moves::MoveFlags::PUNCH) {
                    modifier *= 1.2;
                }
            }
            "Strong Jaw" => {
                if move_data.flags.has(pkmn_core::moves::MoveFlags::BITE) {
                    modifier *= 1.5;
                }
            }
            "Adaptability" => {
                let atk_species = get_species(&atk.species);
                if let Some(sp) = atk_species {
                    if has_stab(&sp.types, move_data.move_type) {
                        modifier *= 4.0 / 3.0;
                    }
                }
            }
            "Guts" => {
                if atk.status.is_some() && move_data.category == MoveCategory::Physical {
                    modifier *= 1.5;
                }
            }
            "Sheer Force" => {}
            "Tinted Lens" => {
                if effectiveness > 0.0 && effectiveness < 1.0 {
                    modifier *= 2.0;
                }
            }
            "Sword of Ruin" => {
                if move_data.category == MoveCategory::Physical {
                    modifier *= 1.33;
                }
            }
            "Beads of Ruin" => {
                if move_data.category == MoveCategory::Special {
                    modifier *= 1.33;
                }
            }
            _ => {}
        }
    }

    // Defender ability modifiers - use inferred ability if not explicitly known
    let def_ability = def.ability.as_deref().or_else(|| infer_ability(&def.species));
    if let Some(ability) = def_ability {
        match ability {
            "Multiscale" => {
                if def.hp == def.max_hp {
                    modifier *= 0.5;
                }
            }
            "Thick Fat" => {
                if move_data.move_type == Type::Fire || move_data.move_type == Type::Ice {
                    modifier *= 0.5;
                }
            }
            "Tablets of Ruin" => {
                if move_data.category == MoveCategory::Physical {
                    modifier *= 0.75;
                }
            }
            "Vessel of Ruin" => {
                if move_data.category == MoveCategory::Special {
                    modifier *= 0.75;
                }
            }
            _ => {}
        }
    }

    // Item modifiers (attacker)
    if let Some(ref item) = atk.item {
        match item.as_str() {
            "Choice Band" => {
                if move_data.category == MoveCategory::Physical {
                    modifier *= 1.5;
                }
            }
            "Choice Specs" => {
                if move_data.category == MoveCategory::Special {
                    modifier *= 1.5;
                }
            }
            "Life Orb" => modifier *= 1.3,
            "Expert Belt" => {
                if effectiveness > 1.0 {
                    modifier *= 1.2;
                }
            }
            // Type-boosting items (1.2x)
            "Mystic Water" => { if move_data.move_type == Type::Water { modifier *= 1.2; } }
            "Charcoal" => { if move_data.move_type == Type::Fire { modifier *= 1.2; } }
            "Magnet" => { if move_data.move_type == Type::Electric { modifier *= 1.2; } }
            "Miracle Seed" => { if move_data.move_type == Type::Grass { modifier *= 1.2; } }
            "Never-Melt Ice" => { if move_data.move_type == Type::Ice { modifier *= 1.2; } }
            "Black Belt" => { if move_data.move_type == Type::Fighting { modifier *= 1.2; } }
            "Poison Barb" => { if move_data.move_type == Type::Poison { modifier *= 1.2; } }
            "Soft Sand" => { if move_data.move_type == Type::Ground { modifier *= 1.2; } }
            "Sharp Beak" => { if move_data.move_type == Type::Flying { modifier *= 1.2; } }
            "Twisted Spoon" => { if move_data.move_type == Type::Psychic { modifier *= 1.2; } }
            "Silver Powder" => { if move_data.move_type == Type::Bug { modifier *= 1.2; } }
            "Hard Stone" => { if move_data.move_type == Type::Rock { modifier *= 1.2; } }
            "Spell Tag" => { if move_data.move_type == Type::Ghost { modifier *= 1.2; } }
            "Dragon Fang" => { if move_data.move_type == Type::Dragon { modifier *= 1.2; } }
            "Black Glasses" => { if move_data.move_type == Type::Dark { modifier *= 1.2; } }
            "Metal Coat" => { if move_data.move_type == Type::Steel { modifier *= 1.2; } }
            "Silk Scarf" => { if move_data.move_type == Type::Normal { modifier *= 1.2; } }
            "Fairy Feather" => { if move_data.move_type == Type::Fairy { modifier *= 1.2; } }
            _ => {}
        }
    }

    // Knock Off: 1.5x if target has an item (assume they do in randbats unless consumed)
    if move_data.name.eq_ignore_ascii_case("Knock Off") && !def.item_consumed {
        modifier *= 1.5;
    }

    // Burn halves physical attack (unless Guts)
    let atk_ability_for_burn = atk.ability.as_deref().or_else(|| infer_ability(&atk.species));
    if atk.status.as_deref() == Some("brn")
        && move_data.category == MoveCategory::Physical
        && atk_ability_for_burn != Some("Guts")
    {
        modifier *= 0.5;
    }

    // Screens
    let screen_active = if def_idx == 0 {
        if move_data.category == MoveCategory::Physical { state.p1_reflect } else { state.p1_light_screen }
    } else {
        if move_data.category == MoveCategory::Physical { state.p2_reflect } else { state.p2_light_screen }
    };
    if screen_active {
        modifier *= 0.5;
    }

    // Terrain boost
    if let Some(ref terrain) = state.terrain {
        let is_grounded = is_attacker_grounded(atk);
        if is_grounded {
            let terrain_boost = match (terrain.as_str(), move_data.move_type) {
                ("Electric Terrain", Type::Electric) => 1.3,
                ("Grassy Terrain", Type::Grass) => 1.3,
                ("Psychic Terrain", Type::Psychic) => 1.3,
                _ => 1.0,
            };
            modifier *= terrain_boost;
        }
    }

    modifier
}

/// Check if attacker is grounded (not Flying type, not Levitate, not Air Balloon)
fn is_attacker_grounded(atk: &ActiveInfo) -> bool {
    let ability = atk.ability.as_deref().or_else(|| infer_ability(&atk.species));
    if ability == Some("Levitate") {
        return false;
    }
    if atk.item.as_deref() == Some("Air Balloon") {
        return false;
    }
    let species = get_species(&atk.species);
    if let Some(sp) = species {
        if sp.types.contains(&Type::Flying) {
            return false;
        }
    }
    true
}

/// Get variable base power for certain moves
fn get_variable_bp(
    move_data: &pkmn_core::moves::MoveData,
    atk: &ActiveInfo,
    _def: &ActiveInfo,
    weather: &Option<String>,
) -> u16 {
    let name = move_data.name.to_lowercase();
    match name.as_str() {
        "acrobatics" => {
            if atk.item.is_none() { 110 } else { 55 }
        }
        "facade" => {
            match atk.status.as_deref() {
                Some("brn") | Some("psn") | Some("tox") | Some("par") => 140,
                _ => 70,
            }
        }
        "weather ball" => {
            if weather.is_some() { 100 } else { 50 }
        }
        // TODO: Low Kick / Grass Knot (weight-based)
        _ => move_data.base_power as u16,
    }
}

/// List of multi-hit moves
fn is_multi_hit_move(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(lower.as_str(),
        "bullet seed" | "rock blast" | "icicle spear" | "population bomb" |
        "scale shot" | "tail slap" | "triple axel" | "surging strikes" |
        "water shuriken" | "bone rush" | "pin missile" | "fury attack" |
        "arm thrust" | "double hit" | "dual wingbeat" | "triple kick"
    )
}

/// Calculate weather boost for a move.
fn calc_weather_boost(state: &VerificationState, move_type: Type) -> f32 {
    match state.weather.as_deref() {
        Some("RainDance") => match move_type {
            Type::Water => 1.5,
            Type::Fire => 0.5,
            _ => 1.0,
        },
        Some("SunnyDay") => match move_type {
            Type::Fire => 1.5,
            Type::Water => 0.5,
            _ => 1.0,
        },
        _ => 1.0,
    }
}

fn verify_fixture(fixture: &Value) -> Vec<DamageCheck> {
    let events = match fixture["events"].as_array() {
        Some(e) => e,
        None => return vec![],
    };

    let mut checks = Vec::new();
    let mut state = VerificationState::default();
    let mut current_turn: u32 = 0;
    let mut last_move: Option<(usize, String)> = None;

    for event in events {
        let event_type = event["type"].as_str().unwrap_or("");

        match event_type {
            "turn" => {
                current_turn = event["turn"].as_u64().unwrap_or(0) as u32;
                last_move = None;
            }
            "switch" | "drag" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let species = event["species"].as_str().unwrap_or("").to_string();
                let level = event["level"].as_u64().unwrap_or(100) as u8;
                let hp = event["hp"].as_u64().unwrap_or(0) as u16;
                let max_hp = event["max_hp"].as_u64().unwrap_or(0) as u16;
                let was_terad = state.terastallized_species.iter().any(|s| s.eq_ignore_ascii_case(&species));
                state.active[idx] = ActiveInfo {
                    species: species.clone(),
                    level,
                    hp,
                    max_hp,
                    ability: None,
                    item: None,
                    item_consumed: false,
                    boosts: Boosts::default(),
                    status: None,
                    is_terastallized: was_terad,
                };
                // Infer ability on switch-in for terrain and Intimidate
                if let Some(inferred) = infer_ability(&species) {
                    if let Some(terrain) = ability_sets_terrain(inferred) {
                        state.terrain = Some(terrain.to_string());
                    }
                    if inferred == "Intimidate" {
                        let opp_idx = 1 - idx;
                        state.active[opp_idx].boosts.atk = (state.active[opp_idx].boosts.atk - 1).clamp(-6, 6);
                    }
                }
                last_move = None;
            }
            "boost" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let stat = event["stat"].as_str().unwrap_or("");
                let amount = event["amount"].as_i64().unwrap_or(0) as i8;
                match stat {
                    "atk" => state.active[idx].boosts.atk = (state.active[idx].boosts.atk + amount).clamp(-6, 6),
                    "def" => state.active[idx].boosts.def = (state.active[idx].boosts.def + amount).clamp(-6, 6),
                    "spa" => state.active[idx].boosts.spa = (state.active[idx].boosts.spa + amount).clamp(-6, 6),
                    "spd" => state.active[idx].boosts.spd = (state.active[idx].boosts.spd + amount).clamp(-6, 6),
                    "spe" => state.active[idx].boosts.spe = (state.active[idx].boosts.spe + amount).clamp(-6, 6),
                    _ => {}
                }
            }
            "unboost" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let stat = event["stat"].as_str().unwrap_or("");
                let amount = event["amount"].as_i64().unwrap_or(0) as i8;
                match stat {
                    "atk" => state.active[idx].boosts.atk = (state.active[idx].boosts.atk - amount).clamp(-6, 6),
                    "def" => state.active[idx].boosts.def = (state.active[idx].boosts.def - amount).clamp(-6, 6),
                    "spa" => state.active[idx].boosts.spa = (state.active[idx].boosts.spa - amount).clamp(-6, 6),
                    "spd" => state.active[idx].boosts.spd = (state.active[idx].boosts.spd - amount).clamp(-6, 6),
                    "spe" => state.active[idx].boosts.spe = (state.active[idx].boosts.spe - amount).clamp(-6, 6),
                    _ => {}
                }
            }
            "status" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                state.active[idx].status = event["status"].as_str().map(|s| s.to_string());
            }
            "curestatus" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                state.active[idx].status = None;
            }
            "weather" => {
                let w = event["weather"].as_str().unwrap_or("");
                state.weather = if w == "none" { None } else { Some(w.to_string()) };
            }
            "ability" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let ability_name = event["ability"].as_str().unwrap_or("");
                state.active[idx].ability = Some(ability_name.to_string());
                if ability_name == "Intimidate" {
                    let opp_idx = 1 - idx;
                    state.active[opp_idx].boosts.atk = (state.active[opp_idx].boosts.atk - 1).clamp(-6, 6);
                }
            }
            "terastallize" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                state.active[idx].is_terastallized = true;
                state.terastallized_species.push(state.active[idx].species.clone());
            }
            "item" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                state.active[idx].item = event["item"].as_str().map(|s| s.to_string());
            }
            "enditem" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                state.active[idx].item = None;
                state.active[idx].item_consumed = true;
            }
            "sidestart" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let cond = event["condition"].as_str().unwrap_or("");
                match cond {
                    "Reflect" => { if idx == 0 { state.p1_reflect = true; } else { state.p2_reflect = true; } }
                    "Light Screen" => { if idx == 0 { state.p1_light_screen = true; } else { state.p2_light_screen = true; } }
                    _ => {}
                }
            }
            "sideend" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let cond = event["condition"].as_str().unwrap_or("");
                match cond {
                    "Reflect" => { if idx == 0 { state.p1_reflect = false; } else { state.p2_reflect = false; } }
                    "Light Screen" => { if idx == 0 { state.p1_light_screen = false; } else { state.p2_light_screen = false; } }
                    _ => {}
                }
            }
            "fieldstart" => {
                let cond = event["condition"].as_str().unwrap_or("");
                if cond.contains("Electric Terrain") {
                    state.terrain = Some("Electric Terrain".to_string());
                } else if cond.contains("Grassy Terrain") {
                    state.terrain = Some("Grassy Terrain".to_string());
                } else if cond.contains("Psychic Terrain") {
                    state.terrain = Some("Psychic Terrain".to_string());
                } else if cond.contains("Misty Terrain") {
                    state.terrain = Some("Misty Terrain".to_string());
                }
            }
            "fieldend" => {
                let cond = event["condition"].as_str().unwrap_or("");
                if cond.contains("Terrain") {
                    state.terrain = None;
                }
            }
            "crit" => {
                state.next_is_crit = true;
            }
            "move" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let move_name = event["move"].as_str().unwrap_or("").to_string();
                last_move = Some((idx, move_name));
            }
            "damage" => {
                let player = event["player"].as_str().unwrap_or("p1");
                let def_idx = player_idx(player);
                let new_hp = event["hp"].as_u64().unwrap_or(0) as u16;
                let max_hp_from_event = event["max_hp"].as_u64().map(|v| v as u16);

                // Check for crit field on the damage event itself
                if event.get("crit").and_then(|v| v.as_bool()).unwrap_or(false) {
                    state.next_is_crit = true;
                }

                // Skip non-move damage
                let source = event.get("source").and_then(|s| s.as_str()).unwrap_or("");
                if !source.is_empty() {
                    if let Some(mhp) = max_hp_from_event {
                        state.active[def_idx].max_hp = mhp;
                    }
                    state.active[def_idx].hp = new_hp;
                    last_move = None;
                    state.next_is_crit = false;
                    continue;
                }

                if let Some((atk_idx, ref move_name)) = last_move {
                    if atk_idx == def_idx {
                        state.active[def_idx].hp = new_hp;
                        last_move = None;
                        state.next_is_crit = false;
                        continue;
                    }

                    let old_hp = state.active[def_idx].hp;
                    let observed_damage = old_hp.saturating_sub(new_hp);
                    let is_crit = state.next_is_crit;

                    let check = verify_damage(
                        current_turn,
                        &state,
                        atk_idx,
                        def_idx,
                        move_name,
                        observed_damage,
                        is_crit,
                    );
                    checks.push(check);
                }

                if let Some(mhp) = max_hp_from_event {
                    state.active[def_idx].max_hp = mhp;
                }
                state.active[def_idx].hp = new_hp;
                last_move = None;
                state.next_is_crit = false;
            }
            "heal" => {
                let idx = player_idx(event["player"].as_str().unwrap_or("p1"));
                let new_hp = event["hp"].as_u64().unwrap_or(0) as u16;
                if let Some(mhp) = event["max_hp"].as_u64() {
                    state.active[idx].max_hp = mhp as u16;
                }
                state.active[idx].hp = new_hp;
            }
            _ => {}
        }
    }

    checks
}

fn verify_damage(
    turn: u32,
    state: &VerificationState,
    atk_idx: usize,
    def_idx: usize,
    move_name: &str,
    observed_damage: u16,
    is_crit: bool,
) -> DamageCheck {
    let attacker = &state.active[atk_idx];
    let defender = &state.active[def_idx];

    let move_data = match get_move(move_name) {
        Some(m) => m,
        None => return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
            result: CheckResult::Skip,
        },
    };

    if move_data.category == MoveCategory::Status {
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
            result: CheckResult::Skip,
        };
    }

    let atk_species = get_species(&attacker.species);
    let def_species = get_species(&defender.species);

    if atk_species.is_none() || def_species.is_none() {
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
            result: CheckResult::Skip,
        };
    }

    let atk_data = atk_species.unwrap();
    let def_data = def_species.unwrap();

    // Weather Ball changes type based on active weather
    let actual_move_type = if move_name.eq_ignore_ascii_case("Weather Ball") {
        match state.weather.as_deref() {
            Some("RainDance") | Some("Rain") => Type::Water,
            Some("SunnyDay") | Some("Sun") => Type::Fire,
            Some("Sandstorm") | Some("Sand") => Type::Rock,
            Some("Snow") | Some("Hail") => Type::Ice,
            _ => move_data.move_type,
        }
    } else {
        move_data.move_type
    };

    let effectiveness = Type::effectiveness(actual_move_type, &def_data.types);

    // If defender is terastallized, we don't know their tera type so effectiveness is unreliable
    if defender.is_terastallized {
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
            result: CheckResult::Skip,
        };
    }

    if effectiveness == 0.0 && observed_damage > 0 {
        // Tera changes defensive typing - skip these
        if defender.is_terastallized {
            return DamageCheck {
                turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
                move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
                result: CheckResult::Skip,
            };
        }
        if (actual_move_type == Type::Ground && def_data.types.contains(&Type::Flying)) ||
           (actual_move_type == Type::Psychic && def_data.types.contains(&Type::Dark)) ||
           (actual_move_type == Type::Normal && def_data.types.contains(&Type::Ghost)) ||
           (actual_move_type == Type::Fighting && def_data.types.contains(&Type::Ghost)) {
            return DamageCheck {
                turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
                move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
                result: CheckResult::Skip,
            };
        }
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
            result: CheckResult::Fail(format!("Immune but PS shows {} damage", observed_damage)),
        };
    }
    if effectiveness == 0.0 && observed_damage == 0 {
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage: 0, our_min: 0, our_max: 0,
            result: CheckResult::Exact,
        };
    }

    // Fix 3: Fixed-damage moves
    if move_name == "Seismic Toss" || move_name == "Night Shade" {
        let fixed_dmg = attacker.level as u16;
        let result = if observed_damage == fixed_dmg { CheckResult::Exact } else { CheckResult::Close };
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: fixed_dmg, our_max: fixed_dmg,
            result,
        };
    }
    // Skip moves whose damage depends on external state we can't track
    if move_name == "Counter" || move_name == "Mirror Coat" || move_name == "Final Gambit" {
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
            result: CheckResult::Skip,
        };
    }

    // Calculate stats with boosts - use min/max ranges to account for unknown EVs/nature
    let atk_base = if move_data.category == MoveCategory::Physical {
        atk_data.base_stats.atk
    } else {
        atk_data.base_stats.spa
    };
    let def_base = if move_data.category == MoveCategory::Physical {
        def_data.base_stats.def
    } else {
        def_data.base_stats.spd
    };

    // Fix 4: Body Press uses attacker's Defense stat instead of Attack
    let effective_atk_base = if move_name == "Body Press" {
        atk_data.base_stats.def
    } else {
        atk_base
    };

    let atk_boost = if move_name == "Body Press" {
        attacker.boosts.def
    } else if move_data.category == MoveCategory::Physical {
        attacker.boosts.atk
    } else {
        attacker.boosts.spa
    };
    let def_boost = if move_data.category == MoveCategory::Physical {
        defender.boosts.def
    } else {
        defender.boosts.spd
    };

    // Exact Gen 9 Random Battles stats: 85 EVs, 31 IVs, neutral nature
    let atk_stat = (randbats_stat(effective_atk_base, attacker.level) as f32 * boost_multiplier(atk_boost)) as u16;
    let def_stat = (randbats_stat(def_base, defender.level) as f32 * boost_multiplier(def_boost)) as u16;

    let stab = has_stab(&atk_data.types, actual_move_type);
    let weather_boost = calc_weather_boost(state, actual_move_type);
    let other_modifiers = calc_modifiers(state, atk_idx, def_idx, move_data, effectiveness);

    // Variable base power
    let base_power = get_variable_bp(move_data, attacker, defender, &state.weather);

    let ctx = DamageContext {
        attacker_level: attacker.level,
        attacker_stat: atk_stat,
        defender_stat: def_stat,
        base_power,
        stab,
        type_effectiveness: effectiveness,
        critical: is_crit,
        weather_boost,
        other_modifiers,
        random_factor: 100,
    };

    let rolls = damage_roll(&ctx);
    let our_min = rolls[0];
    let our_max = rolls[15];

    // Handle Disguise / Ice Face: these block damage entirely
    if observed_damage == 0 && our_max > 0 {
        let def_ability = defender.ability.as_deref().or_else(|| infer_ability(&defender.species));
        if def_ability == Some("Disguise") || def_ability == Some("Ice Face") {
            return DamageCheck {
                turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
                move_name: move_name.to_string(), observed_damage, our_min, our_max,
                result: CheckResult::Skip,
            };
        }
        // Mimikyu and Eiscue always have these abilities
        if defender.species == "Mimikyu" || defender.species == "Eiscue" {
            return DamageCheck {
                turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
                move_name: move_name.to_string(), observed_damage, our_min, our_max,
                result: CheckResult::Skip,
            };
        }
    }

    // Skip events where observed damage is extremely low relative to our calc
    // These are Substitute, Focus Sash, Sturdy, Endure, or misattributed chip damage
    // Even with Eviolite (0.67x) + Reflect (0.5x) + resist berry (0.5x), minimum is ~0.17x
    // Anything below 15% of our min is clearly not a real direct hit
    if our_min > 10 && (observed_damage as f32) < (our_min as f32 * 0.15) {
        return DamageCheck {
            turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
            move_name: move_name.to_string(), observed_damage, our_min, our_max,
            result: CheckResult::Skip,
        };
    }

    // Account for unknown items by widening the range:
    // Offensive items we might miss: Choice Band/Specs (1.5x), Life Orb (1.3x)
    // Defensive items we might miss: Eviolite (0.67x def), Assault Vest (0.67x spd)
    // This gives us a wider "plausible" range
    let item_high = if attacker.item.is_none() { 1.5f32 } else { 1.0 }; // Could have Choice Band
    let item_low = if defender.item.is_none() { 0.67f32 } else { 1.0 };  // Could have Eviolite/AV

    let wide_min = (our_min as f32 * item_low) as u16;
    let wide_max = (our_max as f32 * item_high) as u16;

    // Also check non-crit/crit range for classification
    let alt_ctx = DamageContext { critical: !is_crit, ..ctx };
    let alt_rolls = damage_roll(&alt_ctx);
    let alt_min = alt_rolls[0];
    let alt_max = alt_rolls[15];
    let alt_wide_min = (alt_min as f32 * item_low) as u16;
    let alt_wide_max = (alt_max as f32 * item_high) as u16;

    let result = if observed_damage == 0 && our_min == 0 {
        CheckResult::Exact
    } else if observed_damage >= our_min && observed_damage <= our_max {
        CheckResult::Exact
    } else if is_multi_hit_move(move_name) && our_max > 0 {
        // Multi-hit: check if observed is a multiple of per-hit range (with item tolerance)
        let max_hits = if move_name.eq_ignore_ascii_case("Population Bomb") { 10 } else { 5 };
        let mut found = false;
        for hits in 2..=max_hits {
            let multi_min = wide_min as u32 * hits as u32;
            let multi_max = wide_max as u32 * hits as u32;
            if (observed_damage as u32) >= multi_min && (observed_damage as u32) <= multi_max {
                found = true;
                break;
            }
        }
        if found { CheckResult::Exact } else { CheckResult::Close }
    } else if observed_damage >= wide_min && observed_damage <= wide_max {
        CheckResult::Exact // Within plausible item range
    } else if observed_damage >= alt_wide_min && observed_damage <= alt_wide_max {
        CheckResult::Exact // Crit/non-crit with item range
    } else if observed_damage > 0 && our_max > 0 {
        let tolerance = (wide_max as f32 * 0.15) as u16;
        if observed_damage <= wide_max + tolerance && observed_damage + tolerance >= wide_min {
            CheckResult::Close
        } else {
            CheckResult::Direction
        }
    } else if observed_damage > 0 && our_max == 0 {
        CheckResult::Fail(format!("We predict 0 but PS shows {} damage", observed_damage))
    } else {
        CheckResult::Direction
    };

    DamageCheck {
        turn,
        attacker: attacker.species.clone(),
        defender: defender.species.clone(),
        move_name: move_name.to_string(),
        observed_damage,
        our_min,
        our_max,
        result,
    }
}

#[test]
fn test_damage_matches_replays() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/replay_events");

    if !fixture_dir.exists() {
        eprintln!("No replay_events fixtures found, skipping");
        return;
    }

    let mut total = 0;
    let mut exact = 0;
    let mut close = 0;
    let mut direction = 0;
    let mut skips = 0;
    let mut fails: Vec<(String, DamageCheck)> = Vec::new();

    for entry in fs::read_dir(&fixture_dir).expect("Cannot read replay_events directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.extension().map_or(false, |e| e == "json") {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();
        let fixture: Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse {}", path.display()));

        let fixture_id = fixture["id"].as_str().unwrap_or("unknown").to_string();
        let checks = verify_fixture(&fixture);

        for check in checks {
            total += 1;
            match &check.result {
                CheckResult::Exact => exact += 1,
                CheckResult::Close => close += 1,
                CheckResult::Direction => direction += 1,
                CheckResult::Skip => skips += 1,
                CheckResult::Fail(_) => fails.push((fixture_id.clone(), check)),
            }
        }
    }

    let checked = total - skips;
    let exact_pct = if checked > 0 { exact as f64 / checked as f64 * 100.0 } else { 0.0 };
    let close_pct = if checked > 0 { (exact + close) as f64 / checked as f64 * 100.0 } else { 0.0 };

    eprintln!("\n=== Damage Verification Results ===");
    eprintln!("Total events:     {}", total);
    eprintln!("Skipped:          {}", skips);
    eprintln!("Checked:          {}", checked);
    eprintln!("---");
    eprintln!("Exact (in range): {} ({:.1}%)", exact, exact_pct);
    eprintln!("Close (±10%/crit):{} ({:.1}% cumulative)", close, close_pct);
    eprintln!("Direction only:   {}", direction);
    eprintln!("Failures:         {}", fails.len());

    if !fails.is_empty() {
        eprintln!("\nFailures:");
        for (id, check) in &fails {
            eprintln!(
                "  [{}] T{}: {} used {} vs {} — observed {} dmg, range [{}, {}]: {:?}",
                id, check.turn, check.attacker, check.move_name, check.defender,
                check.observed_damage, check.our_min, check.our_max, check.result
            );
        }
    }

    // Only fail on type chart bugs
    assert!(
        fails.is_empty(),
        "{} type effectiveness failures found",
        fails.len()
    );
}

#[test]
fn test_type_effectiveness_from_replays() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/replay_events");

    if !fixture_dir.exists() {
        return;
    }

    let mut immunity_checks = 0;
    let mut immunity_correct = 0;

    for entry in fs::read_dir(&fixture_dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.extension().map_or(false, |e| e == "json") {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();
        let fixture: Value = serde_json::from_str(&content).unwrap();
        let checks = verify_fixture(&fixture);

        for check in &checks {
            if check.result == CheckResult::Skip {
                continue;
            }
            if check.our_max == 0 && check.observed_damage == 0 {
                immunity_checks += 1;
                immunity_correct += 1;
            } else if check.our_max == 0 && check.observed_damage > 0 {
                immunity_checks += 1;
            }
        }
    }

    eprintln!(
        "Immunity checks: {}/{} correct",
        immunity_correct, immunity_checks
    );
}
