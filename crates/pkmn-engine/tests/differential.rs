//! Differential testing harness for pkmn-engine-rs.
//!
//! Reads JSON fixture files and replays battles, comparing our engine's
//! outcomes against expected results from Pokemon Showdown.

use pkmn_core::abilities::AbilityId;
use pkmn_core::nature::Nature;
use pkmn_core::species::get_species;
use pkmn_engine::*;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Parse a nature string into our Nature enum.
fn parse_nature(s: &str) -> Nature {
    match s {
        "Adamant" => Nature::Adamant,
        "Modest" => Nature::Modest,
        "Jolly" => Nature::Jolly,
        "Timid" => Nature::Timid,
        "Bold" => Nature::Bold,
        "Impish" => Nature::Impish,
        "Careful" => Nature::Careful,
        "Calm" => Nature::Calm,
        "Brave" => Nature::Brave,
        "Quiet" => Nature::Quiet,
        _ => Nature::Hardy,
    }
}

/// Parse an ability string into AbilityId.
fn parse_ability(s: &str) -> AbilityId {
    match s {
        "Drizzle" => AbilityId::Drizzle,
        "Drought" => AbilityId::Drought,
        "Sand Stream" => AbilityId::SandStream,
        "Snow Warning" => AbilityId::SnowWarning,
        "Intimidate" => AbilityId::Intimidate,
        "Levitate" => AbilityId::Levitate,
        "Rough Skin" => AbilityId::RoughSkin,
        "Technician" => AbilityId::Technician,
        _ => AbilityId::None,
    }
}

/// Build a Pokemon from fixture JSON.
fn build_pokemon(data: &Value) -> Pokemon {
    let species_name = data["species"].as_str().unwrap();
    let species = get_species(species_name).expect(&format!("Unknown species: {}", species_name));
    let level = data["level"].as_u64().unwrap_or(100) as u8;
    let nature = parse_nature(data["nature"].as_str().unwrap_or("Hardy"));

    let evs: [u8; 6] = if let Some(arr) = data["evs"].as_array() {
        [
            arr[0].as_u64().unwrap_or(0) as u8,
            arr[1].as_u64().unwrap_or(0) as u8,
            arr[2].as_u64().unwrap_or(0) as u8,
            arr[3].as_u64().unwrap_or(0) as u8,
            arr[4].as_u64().unwrap_or(0) as u8,
            arr[5].as_u64().unwrap_or(0) as u8,
        ]
    } else {
        [0; 6]
    };

    let ivs: [u8; 6] = if let Some(arr) = data["ivs"].as_array() {
        [
            arr[0].as_u64().unwrap_or(31) as u8,
            arr[1].as_u64().unwrap_or(31) as u8,
            arr[2].as_u64().unwrap_or(31) as u8,
            arr[3].as_u64().unwrap_or(31) as u8,
            arr[4].as_u64().unwrap_or(31) as u8,
            arr[5].as_u64().unwrap_or(31) as u8,
        ]
    } else {
        [31; 6]
    };

    // Parse moves
    let mut moves = [MoveSlot { move_id: 0, pp: 0, max_pp: 0 }; 4];
    if let Some(move_arr) = data["moves"].as_array() {
        for (i, m) in move_arr.iter().enumerate().take(4) {
            if let Some(name) = m.as_str() {
                if let Some(move_data) = pkmn_core::moves::get_move(name) {
                    moves[i] = MoveSlot {
                        move_id: move_data.id,
                        pp: move_data.pp,
                        max_pp: move_data.pp,
                    };
                }
            }
        }
    }

    let mut mon = Pokemon::new(species, level, nature, moves, evs, ivs);
    mon.ability_id = parse_ability(data["ability"].as_str().unwrap_or("None"));
    mon
}

/// Parse a choice string like "move 0", "switch 1" into a Choice.
fn parse_choice(s: &str) -> Choice {
    let parts: Vec<&str> = s.split_whitespace().collect();
    match parts[0] {
        "move" => Choice::Move(parts[1].parse().unwrap()),
        "switch" => Choice::Switch(parts[1].parse().unwrap()),
        "tera" => Choice::Tera(parts[1].parse().unwrap()),
        _ => panic!("Unknown choice: {}", s),
    }
}

/// Result of comparing one turn.
#[derive(Debug)]
struct TurnDivergence {
    turn: u64,
    field: &'static str,
    expected: String,
    actual: String,
}

/// Run a single fixture and return any divergences.
fn run_fixture(fixture: &Value) -> Vec<TurnDivergence> {
    let mut divergences = Vec::new();
    let id = fixture["id"].as_str().unwrap_or("unknown");

    // Build teams
    let p1_team: Vec<Pokemon> = fixture["teams"]["p1"]
        .as_array()
        .unwrap()
        .iter()
        .map(build_pokemon)
        .collect();
    let p2_team: Vec<Pokemon> = fixture["teams"]["p2"]
        .as_array()
        .unwrap()
        .iter()
        .map(build_pokemon)
        .collect();

    let side1 = Side::new(p1_team);
    let side2 = Side::new(p2_team);
    let mut battle = Battle::new(side1, side2, [42, 0, 0, 0]);

    // Apply setup conditions
    if let Some(setup) = fixture.get("setup") {
        if setup["p1_stealth_rock"].as_bool().unwrap_or(false) {
            battle.sides[0].side_conditions.stealth_rock = true;
        }
        if setup["p2_stealth_rock"].as_bool().unwrap_or(false) {
            battle.sides[1].side_conditions.stealth_rock = true;
        }
    }

    // Execute turns
    if let Some(turns) = fixture["turns"].as_array() {
        for turn_data in turns {
            let turn_num = turn_data["turn"].as_u64().unwrap_or(0);
            let p1_choice = parse_choice(turn_data["p1_choice"].as_str().unwrap());
            let p2_choice = parse_choice(turn_data["p2_choice"].as_str().unwrap());

            let p1_hp_before = battle.sides[0].active().hp;
            let p2_hp_before = battle.sides[1].active().hp;

            // Handle switches vs normal turns
            match (&p1_choice, &p2_choice) {
                (Choice::Switch(_), _) | (_, Choice::Switch(_)) => {
                    battle.apply(p1_choice, p2_choice);
                }
                _ => {
                    battle.apply(p1_choice, p2_choice);
                }
            }

            // Handle forced switches after faints
            if let BattlePhase::ForcedSwitch(_) = battle.phase {
                // For now, skip forced switch handling in fixtures
                // Real fixtures would specify the switch choice
            }

            let expected = &turn_data["expected"];

            // Check faint expectations
            if let Some(p1_fainted) = expected["p1_active_fainted"].as_bool() {
                let actual = battle.sides[0].active().is_fainted;
                if actual != p1_fainted {
                    divergences.push(TurnDivergence {
                        turn: turn_num,
                        field: "p1_active_fainted",
                        expected: p1_fainted.to_string(),
                        actual: actual.to_string(),
                    });
                }
            }
            if let Some(p2_fainted) = expected["p2_active_fainted"].as_bool() {
                let actual = battle.sides[1].active().is_fainted;
                if actual != p2_fainted {
                    divergences.push(TurnDivergence {
                        turn: turn_num,
                        field: "p2_active_fainted",
                        expected: p2_fainted.to_string(),
                        actual: actual.to_string(),
                    });
                }
            }

            // Check damage direction (did HP decrease?)
            if let Some(true) = expected["p1_took_damage"].as_bool() {
                if battle.sides[0].active().hp >= p1_hp_before && !battle.sides[0].active().is_fainted {
                    divergences.push(TurnDivergence {
                        turn: turn_num,
                        field: "p1_took_damage",
                        expected: "true".into(),
                        actual: format!("hp unchanged: {} -> {}", p1_hp_before, battle.sides[0].active().hp),
                    });
                }
            }
            if let Some(true) = expected["p2_took_damage"].as_bool() {
                if battle.sides[1].active().hp >= p2_hp_before && !battle.sides[1].active().is_fainted {
                    divergences.push(TurnDivergence {
                        turn: turn_num,
                        field: "p2_took_damage",
                        expected: "true".into(),
                        actual: format!("hp unchanged: {} -> {}", p2_hp_before, battle.sides[1].active().hp),
                    });
                }
            }

            // Check HP fraction (for hazard damage verification)
            if let Some(frac) = expected["p1_hp_fraction_max"].as_f64() {
                let mon = battle.sides[0].active();
                let actual_frac = mon.hp as f64 / mon.max_hp as f64;
                // Allow 5% tolerance for rounding
                if (actual_frac - frac).abs() > 0.05 {
                    divergences.push(TurnDivergence {
                        turn: turn_num,
                        field: "p1_hp_fraction_max",
                        expected: format!("{:.2}", frac),
                        actual: format!("{:.2} ({}/{})", actual_frac, mon.hp, mon.max_hp),
                    });
                }
            }

            // Check weather
            if let Some(weather_str) = expected["weather"].as_str() {
                let actual_weather = match battle.field.weather {
                    Weather::None => "None",
                    Weather::Sun => "Sun",
                    Weather::Rain => "Rain",
                    Weather::Sand => "Sand",
                    Weather::Snow => "Snow",
                    _ => "Other",
                };
                if actual_weather != weather_str {
                    divergences.push(TurnDivergence {
                        turn: turn_num,
                        field: "weather",
                        expected: weather_str.into(),
                        actual: actual_weather.into(),
                    });
                }
            }
        }
    }

    // Check winner
    if let Some(winner_str) = fixture.get("winner") {
        if !winner_str.is_null() {
            let expected_winner = winner_str.as_str().unwrap();
            let actual_winner = match battle.result {
                BattleResult::Win(0) => "p1",
                BattleResult::Win(1) => "p2",
                BattleResult::Tie => "tie",
                BattleResult::Ongoing => "ongoing",
                _ => "unknown",
            };
            if actual_winner != expected_winner {
                divergences.push(TurnDivergence {
                    turn: 0,
                    field: "winner",
                    expected: expected_winner.into(),
                    actual: actual_winner.into(),
                });
            }
        }
    }

    if !divergences.is_empty() {
        eprintln!("  Fixture '{}' divergences:", id);
        for d in &divergences {
            eprintln!("    Turn {}: {} expected={} actual={}", d.turn, d.field, d.expected, d.actual);
        }
    }

    divergences
}

#[test]
fn differential_test_all_fixtures() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures");

    let mut total = 0;
    let mut passed = 0;
    let mut all_divergences = Vec::new();

    for entry in fs::read_dir(&fixture_dir).expect("Cannot read fixtures directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            total += 1;
            let content = fs::read_to_string(&path).unwrap();
            let fixture: Value = serde_json::from_str(&content)
                .expect(&format!("Failed to parse {}", path.display()));

            let divergences = run_fixture(&fixture);
            if divergences.is_empty() {
                passed += 1;
            } else {
                all_divergences.push((path.file_name().unwrap().to_string_lossy().to_string(), divergences));
            }
        }
    }

    eprintln!("\nDifferential test results: {}/{} fixtures passed", passed, total);

    assert!(
        all_divergences.is_empty(),
        "Divergences found in {} fixture(s): {:?}",
        all_divergences.len(),
        all_divergences.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>()
    );
}
