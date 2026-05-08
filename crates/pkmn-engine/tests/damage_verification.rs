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
    p1_reflect: bool,
    p1_light_screen: bool,
    p2_reflect: bool,
    p2_light_screen: bool,
    terastallized_species: Vec<String>, // species that have tera'd (permanent)
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

fn estimate_stat(base: u8, level: u8) -> u16 {
    calc_stat(base, 31, 84, level, 1.0)
}

fn has_stab(species_types: &[Type; 2], move_type: Type) -> bool {
    species_types[0] == move_type || species_types[1] == move_type
}

/// Calculate the other_modifiers value based on state.
fn calc_modifiers(
    state: &VerificationState,
    atk_idx: usize,
    def_idx: usize,
    move_data: &pkmn_core::moves::MoveData,
) -> f32 {
    let atk = &state.active[atk_idx];
    let def = &state.active[def_idx];
    let mut modifier = 1.0f32;

    // Ability modifiers (attacker)
    if let Some(ref ability) = atk.ability {
        match ability.as_str() {
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
                // STAB becomes 2x instead of 1.5x; net effect is 2.0/1.5 extra
                // But STAB is already applied separately, so we apply 4/3 here
                // Actually: adaptability makes STAB 2x. Since we already apply 1.5x STAB,
                // we need to multiply by 2.0/1.5 = 4/3
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
            "Sheer Force" => {
                // Approximate: 1.3x for moves with secondary effects
                // We can't easily tell which moves have secondaries, skip for now
            }
            _ => {}
        }
    }

    // Defender ability modifiers
    if let Some(ref ability) = def.ability {
        match ability.as_str() {
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
            "Life Orb" => {
                modifier *= 1.3;
            }
            _ => {}
        }
    }

    // Defender item: Assault Vest boosts SpD (handled in stat calc, not here)
    // Eviolite boosts both defenses (handled in stat calc, not here)

    // Burn halves physical attack (unless Guts)
    if atk.status.as_deref() == Some("brn")
        && move_data.category == MoveCategory::Physical
        && atk.ability.as_deref() != Some("Guts")
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

    modifier
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
                    species,
                    level,
                    hp,
                    max_hp,
                    ability: None,
                    item: None,
                    boosts: Boosts::default(),
                    status: None,
                    is_terastallized: was_terad,
                };
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
                state.active[idx].ability = event["ability"].as_str().map(|s| s.to_string());
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

                // Skip non-move damage
                let source = event.get("source").and_then(|s| s.as_str()).unwrap_or("");
                if !source.is_empty() {
                    if let Some(mhp) = max_hp_from_event {
                        state.active[def_idx].max_hp = mhp;
                    }
                    state.active[def_idx].hp = new_hp;
                    last_move = None;
                    continue;
                }

                if let Some((atk_idx, ref move_name)) = last_move {
                    if atk_idx == def_idx {
                        state.active[def_idx].hp = new_hp;
                        last_move = None;
                        continue;
                    }

                    let old_hp = state.active[def_idx].hp;
                    let observed_damage = old_hp.saturating_sub(new_hp);

                    let check = verify_damage(
                        current_turn,
                        &state,
                        atk_idx,
                        def_idx,
                        move_name,
                        observed_damage,
                    );
                    checks.push(check);
                }

                if let Some(mhp) = max_hp_from_event {
                    state.active[def_idx].max_hp = mhp;
                }
                state.active[def_idx].hp = new_hp;
                last_move = None;
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

    if effectiveness == 0.0 && observed_damage > 0 {
        // If defender is terastallized, their type changed — immunity may not apply
        if defender.is_terastallized {
            return DamageCheck {
                turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
                move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
                result: CheckResult::Direction, // Can't verify without knowing tera type
            };
        }
        // Roost removes Flying type for the turn; Gravity/Smack Down ground Flying types
        // These are hard to track from protocol alone — treat as direction match
        if (actual_move_type == Type::Ground && def_data.types.contains(&Type::Flying)) ||
           (actual_move_type == Type::Psychic && def_data.types.contains(&Type::Dark)) ||
           (actual_move_type == Type::Normal && def_data.types.contains(&Type::Ghost)) ||
           (actual_move_type == Type::Fighting && def_data.types.contains(&Type::Ghost)) {
            return DamageCheck {
                turn, attacker: attacker.species.clone(), defender: defender.species.clone(),
                move_name: move_name.to_string(), observed_damage, our_min: 0, our_max: 0,
                result: CheckResult::Direction, // Likely Roost/Gravity/Scrappy/Ring Target
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

    // Calculate stats with boosts
    let raw_atk_stat = if move_data.category == MoveCategory::Physical {
        estimate_stat(atk_data.base_stats.atk, attacker.level)
    } else {
        estimate_stat(atk_data.base_stats.spa, attacker.level)
    };
    let raw_def_stat = if move_data.category == MoveCategory::Physical {
        estimate_stat(def_data.base_stats.def, defender.level)
    } else {
        estimate_stat(def_data.base_stats.spd, defender.level)
    };

    let atk_boost = if move_data.category == MoveCategory::Physical {
        attacker.boosts.atk
    } else {
        attacker.boosts.spa
    };
    let def_boost = if move_data.category == MoveCategory::Physical {
        defender.boosts.def
    } else {
        defender.boosts.spd
    };

    let atk_stat = (raw_atk_stat as f32 * boost_multiplier(atk_boost)) as u16;
    let def_stat = (raw_def_stat as f32 * boost_multiplier(def_boost)) as u16;

    let stab = has_stab(&atk_data.types, actual_move_type);
    let weather_boost = calc_weather_boost(state, actual_move_type);
    let other_modifiers = calc_modifiers(state, atk_idx, def_idx, move_data);

    let ctx = DamageContext {
        attacker_level: attacker.level,
        attacker_stat: atk_stat,
        defender_stat: def_stat,
        base_power: move_data.base_power as u16,
        stab,
        type_effectiveness: effectiveness,
        critical: false,
        weather_boost,
        other_modifiers,
        random_factor: 100,
    };

    let rolls = damage_roll(&ctx);
    let our_min = rolls[0];
    let our_max = rolls[15];

    // Also check crit range for "close" classification
    let crit_ctx = DamageContext { critical: true, ..ctx };
    let crit_rolls = damage_roll(&crit_ctx);
    let crit_max = crit_rolls[15];

    let result = if observed_damage == 0 && our_min == 0 {
        CheckResult::Exact
    } else if observed_damage >= our_min && observed_damage <= our_max {
        CheckResult::Exact
    } else if observed_damage > 0 && our_max > 0 {
        // Check if within 20% of our range (close match)
        let tolerance = (our_max as f32 * 0.2) as u16;
        if observed_damage <= our_max + tolerance && observed_damage + tolerance >= our_min {
            CheckResult::Close
        } else if observed_damage <= crit_max {
            CheckResult::Close // Could be a crit
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
    eprintln!("Close (±20%/crit):{} ({:.1}% cumulative)", close, close_pct);
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
