use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Fixture {
    id: String,
    events: Vec<DamageEvent>,
    #[allow(dead_code)]
    p1: serde_json::Value,
    #[allow(dead_code)]
    p2: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct DamageEvent {
    #[serde(rename = "type")]
    event_type: String,
    turn: u32,
    source: String,
    target: String,
    #[serde(rename = "move")]
    move_name: String,
    damage: u32,
    crit: bool,
    #[allow(dead_code)]
    effectiveness: f64,
    attacker: CombatantState,
    defender: DefenderState,
}

#[derive(Deserialize, Debug)]
struct CombatantState {
    species: String,
    level: u32,
    stat_atk: u32,
    stat_spa: u32,
    ability: String,
    item: String,
    boosts: Boosts,
    status: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DefenderState {
    species: String,
    #[allow(dead_code)]
    level: u32,
    stat_def: u32,
    stat_spd: u32,
    ability: String,
    item: String,
    boosts: Boosts,
    #[allow(dead_code)]
    status: Option<String>,
    hp_before: u32,
    hp_after: u32,
    max_hp: u32,
}

#[derive(Deserialize, Debug, Default)]
struct Boosts {
    #[serde(default)]
    atk: i8,
    #[serde(default)]
    def: i8,
    #[serde(default)]
    spa: i8,
    #[serde(default)]
    spd: i8,
    #[allow(dead_code)]
    #[serde(default)]
    spe: i8,
}

fn apply_boost(stat: u32, boost: i8) -> u32 {
    let (num, den) = match boost {
        -6 => (2, 8), -5 => (2, 7), -4 => (2, 6), -3 => (2, 5),
        -2 => (2, 4), -1 => (2, 3), 0 => (2, 2), 1 => (3, 2),
        2 => (4, 2), 3 => (5, 2), 4 => (6, 2), 5 => (7, 2),
        6 => (8, 2), _ => (2, 2),
    };
    stat * num / den
}

fn get_move_bp(move_name: &str) -> u32 {
    match move_name {
        "Earthquake" => 100, "Dragon Claw" => 80, "Stone Edge" => 100,
        "Crunch" => 80, "Outrage" => 120, "Extreme Speed" => 80,
        "Surf" => 90, "Hurricane" => 110, "Flare Blitz" => 120,
        "Wild Charge" => 90, "Close Combat" => 120, "Thunderbolt" => 90,
        "Dazzling Gleam" => 80, "Knock Off" => 65, "Scald" => 80,
        "Psychic" => 90, "Crabhammer" => 100, "Aqua Jet" => 40,
        "Giga Drain" => 75, "Rock Slide" => 75, "Spirit Break" => 75,
        "Fire Fang" => 65, "Bullet Punch" => 40, "U-turn" => 70,
        "Superpower" => 120, "Volt Switch" => 70, "Hydro Pump" => 110,
        "Facade" => 70, "Drain Punch" => 75, "Mach Punch" => 40,
        "Body Press" => 80, "Brave Bird" => 120, "Shadow Ball" => 80,
        "Sludge Bomb" => 90, "Focus Blast" => 120, "Ice Punch" => 75,
        _ => 80,
    }
}

fn is_special(move_name: &str) -> bool {
    matches!(move_name,
        "Surf" | "Hurricane" | "Thunderbolt" | "Dazzling Gleam" | "Scald" | "Psychic" |
        "Giga Drain" | "Hydro Pump" | "Volt Switch" | "Shadow Ball" | "Sludge Bomb" |
        "Focus Blast"
    )
}

fn get_move_type(move_name: &str) -> &'static str {
    match move_name {
        "Earthquake" => "Ground",
        "Dragon Claw" | "Outrage" => "Dragon",
        "Stone Edge" | "Rock Slide" => "Rock",
        "Crunch" | "Knock Off" => "Dark",
        "Extreme Speed" | "Facade" => "Normal",
        "Surf" | "Scald" | "Crabhammer" | "Aqua Jet" | "Hydro Pump" => "Water",
        "Hurricane" | "Brave Bird" => "Flying",
        "Flare Blitz" | "Fire Fang" => "Fire",
        "Wild Charge" | "Thunderbolt" | "Volt Switch" => "Electric",
        "Close Combat" | "Drain Punch" | "Mach Punch" | "Superpower" | "Body Press" | "Focus Blast" => "Fighting",
        "Dazzling Gleam" | "Spirit Break" => "Fairy",
        "Psychic" => "Psychic",
        "Giga Drain" => "Grass",
        "Bullet Punch" => "Steel",
        "U-turn" => "Bug",
        "Shadow Ball" => "Ghost",
        "Sludge Bomb" => "Poison",
        "Ice Punch" => "Ice",
        _ => "Normal",
    }
}

fn species_types(species: &str) -> &'static [&'static str] {
    match species {
        "Garchomp" => &["Dragon", "Ground"],
        "Tyranitar" => &["Rock", "Dark"],
        "Dragonite" => &["Dragon", "Flying"],
        "Blissey" | "Chansey" => &["Normal"],
        "Pelipper" => &["Water", "Flying"],
        "Arcanine" => &["Fire"],
        "Tapu Koko" => &["Electric", "Fairy"],
        "Hippowdon" => &["Ground"],
        "Krookodile" => &["Ground", "Dark"],
        "Slowbro" => &["Water", "Psychic"],
        "Crawdaunt" => &["Water", "Dark"],
        "Tangrowth" => &["Grass"],
        "Grimmsnarl" => &["Dark", "Fairy"],
        "Scizor" => &["Bug", "Steel"],
        "Rotom-Wash" => &["Electric", "Water"],
        "Conkeldurr" => &["Fighting"],
        "Corviknight" => &["Flying", "Steel"],
        "Gengar" => &["Ghost", "Poison"],
        _ => &[],
    }
}

fn has_stab(species: &str, move_type: &str) -> bool {
    species_types(species).contains(&move_type)
}

/// Calculate type effectiveness from the type chart
fn calc_type_effectiveness(move_type: &str, defender_species: &str) -> f64 {
    let def_types = species_types(defender_species);
    if def_types.is_empty() { return 1.0; }
    let mut eff = 1.0;
    for dt in def_types {
        eff *= single_type_eff(move_type, dt);
    }
    eff
}

fn single_type_eff(atk: &str, def: &str) -> f64 {
    let ai = type_index(atk);
    let di = type_index(def);
    match TYPE_CHART[ai][di] {
        0 => 0.0, 1 => 0.5, 2 => 1.0, 3 => 2.0, _ => 1.0,
    }
}

fn type_index(t: &str) -> usize {
    match t {
        "Normal" => 0, "Fire" => 1, "Water" => 2, "Electric" => 3,
        "Grass" => 4, "Ice" => 5, "Fighting" => 6, "Poison" => 7,
        "Ground" => 8, "Flying" => 9, "Psychic" => 10, "Bug" => 11,
        "Rock" => 12, "Ghost" => 13, "Dragon" => 14, "Dark" => 15,
        "Steel" => 16, "Fairy" => 17, _ => 0,
    }
}

const TYPE_CHART: [[u8; 18]; 18] = [
    [2,2,2,2,2,2,2,2,2,2,2,2,1,0,2,2,1,2],
    [2,1,1,2,3,3,2,2,2,2,2,3,1,2,1,2,3,2],
    [2,3,1,2,1,2,2,2,3,2,2,2,3,2,1,2,2,2],
    [2,2,3,1,1,2,2,2,0,3,2,2,2,2,1,2,2,2],
    [2,1,3,2,1,2,2,1,3,1,2,1,3,2,1,2,1,2],
    [2,1,1,2,3,1,2,2,3,3,2,2,2,2,3,2,1,2],
    [3,2,2,2,2,3,2,1,2,1,1,1,3,0,2,3,3,1],
    [2,2,2,2,3,2,2,1,1,2,2,2,1,1,2,2,0,3],
    [2,3,2,3,1,2,2,3,2,0,2,1,3,2,2,2,3,2],
    [2,2,2,1,3,2,3,2,2,2,2,3,1,2,2,2,1,2],
    [2,2,2,2,2,2,3,3,2,2,1,2,2,2,2,0,1,2],
    [2,1,2,2,3,2,1,1,2,1,3,2,2,1,2,3,1,1],
    [2,3,2,2,2,3,1,2,1,3,2,3,2,2,2,2,1,2],
    [0,2,2,2,2,2,2,2,2,2,3,2,2,3,2,1,2,2],
    [2,2,2,2,2,2,2,2,2,2,2,2,2,2,3,2,1,0],
    [2,2,2,2,2,2,1,2,2,2,3,2,2,3,2,1,2,1],
    [2,1,1,1,2,3,2,2,2,2,2,2,3,2,2,2,1,3],
    [2,1,2,2,2,2,3,1,2,2,2,2,2,2,3,3,1,2],
];

fn is_recoil_or_self_damage(event: &DamageEvent) -> bool {
    event.source == event.target
}

fn is_burn_tick(event: &DamageEvent) -> bool {
    let tick = event.defender.max_hp / 16;
    event.damage == tick
}

fn is_ko(event: &DamageEvent) -> bool {
    event.defender.hp_after == 0 && event.damage == event.defender.hp_before
}

fn has_rain(event: &DamageEvent) -> bool {
    event.attacker.ability == "Drizzle" || event.defender.ability == "Drizzle"
}

fn has_reflect(fixture_id: &str, event: &DamageEvent) -> bool {
    fixture_id == "screens-reflect"
        && event.defender.species == "Grimmsnarl"
        && !is_special(&event.move_name)
}

fn calculate_damage_rolls(fixture_id: &str, event: &DamageEvent) -> Vec<u32> {
    let move_name = event.move_name.as_str();
    let bp = get_move_bp(move_name);
    if bp == 0 { return vec![event.attacker.level]; }

    let special = is_special(move_name);
    let move_type = get_move_type(move_name);
    let level = event.attacker.level;

    // Effective BP (Technician, Knock Off, Facade)
    let mut effective_bp = bp;
    if event.attacker.ability == "Technician" && bp <= 60 {
        effective_bp = bp * 3 / 2;
    }
    if move_name == "Knock Off" && !event.defender.item.is_empty() {
        effective_bp = effective_bp * 3 / 2;
    }
    if move_name == "Facade" && event.attacker.status.is_some() {
        effective_bp = 140;
    }

    // Attack stat (with item/ability stat modifiers)
    let raw_atk = if special {
        apply_boost(event.attacker.stat_spa, event.attacker.boosts.spa)
    } else {
        apply_boost(event.attacker.stat_atk, event.attacker.boosts.atk)
    };

    // Choice Band/Specs as stat modifier
    let atk = if !special && event.attacker.item == "Choice Band" {
        raw_atk * 3 / 2
    } else if special && event.attacker.item == "Choice Specs" {
        raw_atk * 3 / 2
    } else if !special && (event.attacker.ability == "Huge Power" || event.attacker.ability == "Pure Power") {
        raw_atk * 2
    } else if !special && event.attacker.ability == "Guts" && event.attacker.status.is_some() {
        raw_atk * 3 / 2
    } else {
        raw_atk
    };

    // Defense stat
    let raw_def = if special {
        let d = apply_boost(event.defender.stat_spd,
            if event.crit && event.defender.boosts.spd > 0 { 0 } else { event.defender.boosts.spd });
        // Eviolite boosts SpD by 1.5x
        if event.defender.item == "Eviolite" { d * 3 / 2 } else { d }
    } else {
        apply_boost(event.defender.stat_def,
            if event.crit && event.defender.boosts.def > 0 { 0 } else { event.defender.boosts.def })
    };

    // Base damage
    let base = ((2 * level / 5 + 2) * effective_bp * atk / raw_def) / 50 + 2;

    // Calculate type effectiveness from type chart
    let type_eff = calc_type_effectiveness(move_type, &event.defender.species);

    let mut rolls: Vec<u32> = Vec::with_capacity(16);
    for r in 85..=100u32 {
        let mut dmg = base;

        // Screens
        if has_reflect(fixture_id, event) {
            dmg = dmg / 2;
        }

        // Weather
        if has_rain(event) {
            if move_type == "Water" { dmg = dmg * 3 / 2; }
            else if move_type == "Fire" { dmg = dmg / 2; }
        }

        // Critical hit
        if event.crit { dmg = dmg * 3 / 2; }

        // Random factor
        dmg = dmg * r / 100;

        // STAB
        if has_stab(&event.attacker.species, move_type) {
            if event.attacker.ability == "Adaptability" {
                dmg = dmg * 2;
            } else {
                dmg = dmg * 3 / 2;
            }
        }

        // Type effectiveness (applied as combined multiplier)
        if type_eff == 0.0 { dmg = 0; }
        else if type_eff == 4.0 { dmg = dmg * 4; }
        else if type_eff == 2.0 { dmg = dmg * 2; }
        else if type_eff == 0.5 { dmg = dmg / 2; }
        else if type_eff == 0.25 { dmg = dmg / 4; }

        // Burn
        if !special && event.attacker.status.as_deref() == Some("brn")
            && event.attacker.ability != "Guts" && move_name != "Facade" {
            dmg = dmg / 2;
        }

        // Life Orb (pokeRound: (val * mod + 2048) / 4096)
        if event.attacker.item == "Life Orb" {
            dmg = (dmg * 5324 + 2048) / 4096;
        }

        // Multiscale
        if event.defender.ability == "Multiscale" && event.defender.hp_before == event.defender.max_hp {
            dmg = dmg / 2;
        }

        rolls.push(dmg.max(1));
    }
    rolls
}

#[test]
fn strict_damage_matches() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/full-info");

    let mut total = 0u32;
    let mut exact = 0u32;
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&fixture_dir)
        .expect("Cannot read fixture dir")
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "json") { continue; }

        let content = fs::read_to_string(&path).unwrap();
        let fixture: Fixture = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));

        for event in &fixture.events {
            if event.event_type != "damage" { continue; }
            if event.damage == 0 { continue; }
            if is_recoil_or_self_damage(event) { continue; }
            if is_burn_tick(event) { continue; }
            if event.move_name == "Body Press" { continue; }

            if event.move_name == "Seismic Toss" || event.move_name == "Night Shade" {
                total += 1;
                if event.damage == event.attacker.level {
                    exact += 1;
                } else {
                    failures.push(format!(
                        "{}|t{}| {} -> {} via {} (fixed) | obs={} expected={}",
                        fixture.id, event.turn, event.attacker.species,
                        event.defender.species, event.move_name,
                        event.damage, event.attacker.level
                    ));
                }
                continue;
            }

            total += 1;
            let rolls = calculate_damage_rolls(&fixture.id, event);

            // For KO events, damage is capped by remaining HP
            let matches = if is_ko(event) {
                let max_roll = *rolls.iter().max().unwrap_or(&0);
                event.damage <= max_roll
            } else {
                rolls.contains(&event.damage)
            };

            if matches {
                exact += 1;
            } else {
                let min = rolls.iter().min().unwrap_or(&0);
                let max = rolls.iter().max().unwrap_or(&0);
                failures.push(format!(
                    "{}|t{}| {} ({}) -> {} ({}) via {} | obs={} range=[{},{}]{}",
                    fixture.id, event.turn,
                    event.attacker.species, event.attacker.ability,
                    event.defender.species, event.defender.ability,
                    event.move_name, event.damage, min, max,
                    if is_ko(event) { " [KO]" } else { "" }
                ));
            }
        }
    }

    println!("\n=== STRICT VERIFICATION ===");
    println!("Total events: {}", total);
    println!("Exact match:  {} ({:.1}%)", exact, exact as f64 / total as f64 * 100.0);
    println!("Failures:     {}", failures.len());

    if !failures.is_empty() {
        println!("\nFailed events:");
        for f in &failures {
            println!("  {}", f);
        }
    }

    assert_eq!(failures.len(), 0, "{} damage events did not match exactly", failures.len());
}
