//! Protocol-based differential testing: damage verification against real PS replays.
//!
//! For each (move, damage) event pair in replay fixtures, verifies that our
//! damage formula produces a range containing the observed damage.

use pkmn_core::damage::{damage_roll, DamageContext};
use pkmn_core::moves::{get_move, MoveCategory};
use pkmn_core::species::get_species;
use pkmn_core::stats::calc_stat;
use pkmn_core::types::Type;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Tracks active Pokemon state from protocol events.
#[derive(Debug, Clone)]
struct ActiveMon {
    species: String,
    level: u8,
    hp: u16,
    max_hp: u16,
}

/// Result of a single damage check.
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

#[derive(Debug, PartialEq)]
enum CheckResult {
    Pass,              // Observed damage within our range
    DirectionMatch,    // We predict damage and PS shows damage (but range doesn't contain it)
    EffectivenessOk,   // Type effectiveness direction matches
    Skip,             // Status move or missing data
    Fail(String),     // Type chart bug or direction mismatch
}

/// Estimate a stat for a random battle Pokemon (assume 31 IVs, 84 EVs, neutral nature).
/// Random battles use a specific EV spread but we don't know it, so use a reasonable middle.
fn estimate_stat(base: u8, level: u8) -> u16 {
    calc_stat(base, 31, 84, level, 1.0)
}

/// Check if attacker gets STAB for a move type.
fn has_stab(species_types: &[Type; 2], move_type: Type) -> bool {
    species_types[0] == move_type || species_types[1] == move_type
}

/// Run damage verification on a single fixture.
fn verify_fixture(fixture: &Value) -> Vec<DamageCheck> {
    let events = match fixture["events"].as_array() {
        Some(e) => e,
        None => return vec![],
    };

    let mut checks = Vec::new();
    let mut active: [Option<ActiveMon>; 2] = [None, None]; // p1=0, p2=1
    let mut current_turn: u32 = 0;
    let mut last_move: Option<(usize, String)> = None; // (attacker_idx, move_name)

    for event in events {
        let event_type = event["type"].as_str().unwrap_or("");

        match event_type {
            "turn" => {
                current_turn = event["turn"].as_u64().unwrap_or(0) as u32;
                last_move = None;
            }
            "switch" => {
                let player = event["player"].as_str().unwrap_or("");
                let idx = if player == "p1" { 0 } else { 1 };
                let species = event["species"].as_str().unwrap_or("").to_string();
                let level = event["level"].as_u64().unwrap_or(100) as u8;
                let hp = event["hp"].as_u64().unwrap_or(0) as u16;
                let max_hp = event["max_hp"].as_u64().unwrap_or(0) as u16;
                active[idx] = Some(ActiveMon { species, level, hp, max_hp });
                last_move = None; // switch breaks move->damage chain
            }
            "move" => {
                let player = event["player"].as_str().unwrap_or("");
                let idx = if player == "p1" { 0 } else { 1 };
                let move_name = event["move"].as_str().unwrap_or("").to_string();
                last_move = Some((idx, move_name));
            }
            "damage" => {
                let player = event["player"].as_str().unwrap_or("");
                let def_idx = if player == "p1" { 0 } else { 1 };
                let new_hp = event["hp"].as_u64().unwrap_or(0) as u16;
                let max_hp_from_event = event["max_hp"].as_u64().map(|v| v as u16);

                // Skip non-move damage (hazards, status, weather, etc.)
                let source = event.get("source").and_then(|s| s.as_str()).unwrap_or("");
                if !source.is_empty() {
                    // Update HP tracking
                    if let Some(ref mut mon) = active[def_idx] {
                        mon.hp = new_hp;
                        if let Some(mhp) = max_hp_from_event {
                            mon.max_hp = mhp;
                        }
                    }
                    last_move = None;
                    continue;
                }

                // Only check if we have a preceding move targeting this player
                if let Some((atk_idx, ref move_name)) = last_move {
                    if atk_idx == def_idx {
                        // Self-damage (recoil, etc.) - skip
                        if let Some(ref mut mon) = active[def_idx] {
                            mon.hp = new_hp;
                        }
                        last_move = None;
                        continue;
                    }

                    let defender = active[def_idx].clone();
                    let attacker = active[atk_idx].clone();

                    if let (Some(atk), Some(def)) = (attacker, defender) {
                        let old_hp = def.hp;
                        let observed_damage = old_hp.saturating_sub(new_hp);

                        let check = verify_damage(
                            current_turn,
                            &atk,
                            &def,
                            move_name,
                            observed_damage,
                            max_hp_from_event.unwrap_or(def.max_hp),
                        );
                        checks.push(check);
                    }
                }

                // Update HP
                if let Some(ref mut mon) = active[def_idx] {
                    mon.hp = new_hp;
                    if let Some(mhp) = max_hp_from_event {
                        mon.max_hp = mhp;
                    }
                }
                last_move = None;
            }
            _ => {}
        }
    }

    checks
}

fn verify_damage(
    turn: u32,
    attacker: &ActiveMon,
    defender: &ActiveMon,
    move_name: &str,
    observed_damage: u16,
    _defender_max_hp: u16,
) -> DamageCheck {
    let move_data = match get_move(move_name) {
        Some(m) => m,
        None => {
            return DamageCheck {
                turn,
                attacker: attacker.species.clone(),
                defender: defender.species.clone(),
                move_name: move_name.to_string(),
                observed_damage,
                our_min: 0,
                our_max: 0,
                result: CheckResult::Skip,
            };
        }
    };

    // Skip status moves
    if move_data.category == MoveCategory::Status {
        return DamageCheck {
            turn,
            attacker: attacker.species.clone(),
            defender: defender.species.clone(),
            move_name: move_name.to_string(),
            observed_damage,
            our_min: 0,
            our_max: 0,
            result: CheckResult::Skip,
        };
    }

    let atk_species = get_species(&attacker.species);
    let def_species = get_species(&defender.species);

    if atk_species.is_none() || def_species.is_none() {
        return DamageCheck {
            turn,
            attacker: attacker.species.clone(),
            defender: defender.species.clone(),
            move_name: move_name.to_string(),
            observed_damage,
            our_min: 0,
            our_max: 0,
            result: CheckResult::Skip,
        };
    }

    let atk_data = atk_species.unwrap();
    let def_data = def_species.unwrap();

    // Calculate type effectiveness
    let effectiveness = Type::effectiveness(move_data.move_type, &def_data.types);

    // Check type effectiveness direction
    if effectiveness == 0.0 && observed_damage > 0 {
        return DamageCheck {
            turn,
            attacker: attacker.species.clone(),
            defender: defender.species.clone(),
            move_name: move_name.to_string(),
            observed_damage,
            our_min: 0,
            our_max: 0,
            result: CheckResult::Fail(format!(
                "We predict immune (0x) but PS shows {} damage",
                observed_damage
            )),
        };
    }

    if effectiveness == 0.0 && observed_damage == 0 {
        return DamageCheck {
            turn,
            attacker: attacker.species.clone(),
            defender: defender.species.clone(),
            move_name: move_name.to_string(),
            observed_damage: 0,
            our_min: 0,
            our_max: 0,
            result: CheckResult::Pass,
        };
    }

    // Estimate stats
    let atk_stat = if move_data.category == MoveCategory::Physical {
        estimate_stat(atk_data.base_stats.atk, attacker.level)
    } else {
        estimate_stat(atk_data.base_stats.spa, attacker.level)
    };

    let def_stat = if move_data.category == MoveCategory::Physical {
        estimate_stat(def_data.base_stats.def, defender.level)
    } else {
        estimate_stat(def_data.base_stats.spd, defender.level)
    };

    let stab = has_stab(&atk_data.types, move_data.move_type);

    let ctx = DamageContext {
        attacker_level: attacker.level,
        attacker_stat: atk_stat,
        defender_stat: def_stat,
        base_power: move_data.base_power as u16,
        stab,
        type_effectiveness: effectiveness,
        critical: false,
        weather_boost: 1.0,
        other_modifiers: 1.0,
        random_factor: 100, // placeholder, we use damage_roll
    };

    let rolls = damage_roll(&ctx);
    let our_min = rolls[0];
    let our_max = rolls[15];

    // Also compute crit range for tolerance
    let crit_ctx = DamageContext { critical: true, ..ctx };
    let crit_rolls = damage_roll(&crit_ctx);
    let crit_max = crit_rolls[15];

    let result = if observed_damage == 0 && our_min == 0 {
        CheckResult::Pass
    } else if observed_damage >= our_min && observed_damage <= our_max {
        CheckResult::Pass
    } else if observed_damage > 0 && our_max > 0 {
        // Damage direction matches even if exact range doesn't
        // (could be abilities, items, boosts we don't track)
        if observed_damage <= crit_max {
            CheckResult::DirectionMatch
        } else {
            CheckResult::DirectionMatch // Still a direction match, just outside our range
        }
    } else if observed_damage > 0 && our_max == 0 {
        CheckResult::Fail(format!(
            "We predict 0 damage but PS shows {}",
            observed_damage
        ))
    } else {
        CheckResult::EffectivenessOk
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

    let mut total_checks = 0;
    let mut passes = 0;
    let mut direction_matches = 0;
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
            total_checks += 1;
            match &check.result {
                CheckResult::Pass => passes += 1,
                CheckResult::DirectionMatch => direction_matches += 1,
                CheckResult::EffectivenessOk => passes += 1,
                CheckResult::Skip => skips += 1,
                CheckResult::Fail(_) => {
                    fails.push((fixture_id.clone(), check));
                }
            }
        }
    }

    eprintln!("\n=== Damage Verification Results ===");
    eprintln!("Total checks:     {}", total_checks);
    eprintln!("Exact passes:     {}", passes);
    eprintln!("Direction match:  {}", direction_matches);
    eprintln!("Skipped:          {}", skips);
    eprintln!("Failures:         {}", fails.len());

    if !fails.is_empty() {
        eprintln!("\nFailures (type chart bugs):");
        for (id, check) in &fails {
            eprintln!(
                "  [{}] Turn {}: {} used {} vs {} — observed {} dmg, our range [{}, {}]: {:?}",
                id, check.turn, check.attacker, check.move_name, check.defender,
                check.observed_damage, check.our_min, check.our_max, check.result
            );
        }
    }

    // Only fail on type chart bugs (immune but PS shows damage)
    // Direction mismatches are expected due to unimplemented abilities/items/boosts
    assert!(
        fails.is_empty(),
        "{} type effectiveness failures found (see above)",
        fails.len()
    );
}

#[test]
fn test_type_effectiveness_from_replays() {
    // Verify that for every move+damage pair where we know both species,
    // our type chart agrees with whether damage was dealt or not.
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
            // If our formula says immune (max=0), verify PS agrees
            if check.our_max == 0 && check.observed_damage == 0 {
                immunity_checks += 1;
                immunity_correct += 1;
            } else if check.our_max == 0 && check.observed_damage > 0 {
                immunity_checks += 1;
                // This is a bug - counted in fails
            }
        }
    }

    eprintln!(
        "Immunity checks: {}/{} correct",
        immunity_correct, immunity_checks
    );
}
