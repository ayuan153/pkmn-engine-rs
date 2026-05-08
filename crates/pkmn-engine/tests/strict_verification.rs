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

#[derive(Deserialize, Debug, Default)]
struct Screens {
    #[serde(default)]
    reflect: bool,
    #[serde(rename = "lightScreen", default)]
    light_screen: bool,
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
    #[serde(default)]
    weather: Option<String>,
    #[serde(default)]
    terrain: Option<String>,
    #[serde(default)]
    screens: Screens,
    attacker: CombatantState,
    defender: DefenderState,
}

#[derive(Deserialize, Debug)]
struct CombatantState {
    species: String,
    level: u32,
    stat_atk: u32,
    #[serde(default)]
    stat_def: u32,
    stat_spa: u32,
    ability: String,
    item: String,
    boosts: Boosts,
    status: Option<String>,
    #[serde(rename = "teraType", default)]
    #[allow(dead_code)]
    tera_type: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DefenderState {
    species: String,
    #[allow(dead_code)]
    level: u32,
    #[serde(default)]
    stat_atk: u32,
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
        "Acrobatics" => 55, "Grassy Glide" => 55, "Kowtow Cleave" => 85,
        "Air Slash" => 75, "Moonblast" => 95, "Earth Power" => 90,
        "Sludge Wave" => 95, "Ice Beam" => 90, "Fire Blast" => 110,
        "Bug Buzz" => 90, "Flash Cannon" => 80, "Fishious Rend" => 85,
        "Triple Axel" => 20, "Return" => 102, "Body Slam" => 85,
        "Foul Play" => 95, "Bullet Seed" => 25, "Weather Ball" => 50,
        "Lava Plume" => 80, "Blizzard" => 110, "Torch Song" => 80,
        "Stored Power" => 20, "Wood Hammer" => 120, "Double Iron Bash" => 60,
        "Psychic Fangs" => 85, "Flamethrower" => 90, "Megahorn" => 120,
        "Liquidation" => 85, "Play Rough" => 90, "Ice Shard" => 40,
        "Headlong Rush" => 120, "Iron Head" => 80, "Meteor Mash" => 90,
        "Energy Ball" => 90, "Power Whip" => 120, "Magma Storm" => 100,
        "Mystical Fire" => 75, "Hyper Voice" => 90,
        _ => 80,
    }
}

fn is_special(move_name: &str) -> bool {
    matches!(move_name,
        "Surf" | "Hurricane" | "Thunderbolt" | "Dazzling Gleam" | "Scald" | "Psychic" |
        "Giga Drain" | "Hydro Pump" | "Volt Switch" | "Shadow Ball" | "Sludge Bomb" |
        "Focus Blast" | "Moonblast" | "Earth Power" | "Sludge Wave" | "Ice Beam" |
        "Fire Blast" | "Bug Buzz" | "Flash Cannon" | "Lava Plume" | "Blizzard" |
        "Torch Song" | "Air Slash" | "Stored Power" | "Weather Ball" | "Flamethrower" |
        "Energy Ball" | "Magma Storm" | "Mystical Fire" | "Hyper Voice"
    )
}

fn get_move_type(move_name: &str) -> &'static str {
    match move_name {
        "Earthquake" | "Earth Power" | "Headlong Rush" => "Ground",
        "Dragon Claw" | "Outrage" => "Dragon",
        "Stone Edge" | "Rock Slide" => "Rock",
        "Crunch" | "Knock Off" | "Kowtow Cleave" | "Foul Play" => "Dark",
        "Extreme Speed" | "Facade" | "Return" | "Body Slam" | "Hyper Voice" => "Normal",
        "Surf" | "Scald" | "Crabhammer" | "Aqua Jet" | "Hydro Pump" | "Fishious Rend" | "Liquidation" => "Water",
        "Hurricane" | "Brave Bird" | "Acrobatics" | "Air Slash" => "Flying",
        "Flare Blitz" | "Fire Fang" | "Fire Blast" | "Lava Plume" | "Torch Song" | "Flamethrower" | "Magma Storm" | "Mystical Fire" => "Fire",
        "Wild Charge" | "Thunderbolt" | "Volt Switch" => "Electric",
        "Close Combat" | "Drain Punch" | "Mach Punch" | "Superpower" | "Body Press" | "Focus Blast" => "Fighting",
        "Dazzling Gleam" | "Spirit Break" | "Moonblast" | "Play Rough" => "Fairy",
        "Psychic" | "Stored Power" | "Psychic Fangs" => "Psychic",
        "Giga Drain" | "Grassy Glide" | "Bullet Seed" | "Wood Hammer" | "Energy Ball" | "Power Whip" => "Grass",
        "Bullet Punch" | "Flash Cannon" | "Double Iron Bash" | "Iron Head" | "Meteor Mash" => "Steel",
        "U-turn" | "Bug Buzz" | "Megahorn" => "Bug",
        "Shadow Ball" => "Ghost",
        "Sludge Bomb" | "Sludge Wave" => "Poison",
        "Ice Punch" | "Ice Beam" | "Blizzard" | "Triple Axel" | "Ice Shard" => "Ice",
        "Weather Ball" => "Normal", // overridden by weather logic
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
        "Ursaluna" => &["Normal", "Ground"],
        "Hawlucha" => &["Fighting", "Flying"],
        "Togekiss" => &["Fairy", "Flying"],
        "Heracross" => &["Bug", "Fighting"],
        "Skarmory" => &["Steel", "Flying"],
        "Nidoking" => &["Poison", "Ground"],
        "Weavile" => &["Dark", "Ice"],
        "Rhyperior" => &["Ground", "Rock"],
        "Dracovish" => &["Water", "Dragon"],
        "Yanmega" => &["Bug", "Flying"],
        "Heatran" => &["Fire", "Steel"],
        "Volcarona" => &["Bug", "Fire"],
        "Frosmoth" => &["Ice", "Bug"],
        "Furfrou" => &["Normal"],
        "Kingambit" => &["Dark", "Steel"],
        "Azumarill" => &["Water", "Fairy"],
        "Skeledirge" => &["Fire", "Ghost"],
        "Melmetal" => &["Steel"],
        "Rillaboom" => &["Grass"],
        "Toxapex" => &["Poison", "Water"],
        "Indeedee-F" => &["Psychic", "Normal"],
        "Machamp" => &["Fighting"],
        "Swampert" => &["Water", "Ground"],
        "Breloom" => &["Grass", "Fighting"],
        "Starmie" => &["Water", "Psychic"],
        "Snorlax" => &["Normal"],
        "Lucario" => &["Fighting", "Steel"],
        "Torkoal" => &["Fire"],
        "Ferrothorn" => &["Grass", "Steel"],
        "Abomasnow" => &["Grass", "Ice"],
        "Vaporeon" => &["Water"],
        "Espathra" => &["Psychic"],
        "Umbreon" => &["Dark"],
        "Clefable" => &["Fairy"],
        "Gardevoir" => &["Psychic", "Fairy"],
        "Landorus-Therian" => &["Ground", "Flying"],
        "Alakazam" => &["Psychic"],
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

fn is_grounded(species: &str) -> bool {
    let types = species_types(species);
    !types.contains(&"Flying")
}

/// Moves with secondary effects (for Sheer Force)
fn has_secondary_effect(move_name: &str) -> bool {
    matches!(move_name,
        "Earth Power" | "Sludge Wave" | "Ice Beam" | "Scald" | "Thunderbolt" |
        "Fire Blast" | "Flare Blitz" | "Shadow Ball" | "Sludge Bomb" | "Flash Cannon" |
        "Lava Plume" | "Body Slam" | "Air Slash" | "Spirit Break" | "Torch Song" |
        "Blizzard" | "Fire Fang" | "Rock Slide"
    )
}

/// Punching moves (for Iron Fist)
fn is_punching_move(move_name: &str) -> bool {
    matches!(move_name,
        "Drain Punch" | "Mach Punch" | "Ice Punch" | "Thunder Punch" |
        "Fire Punch" | "Bullet Punch" | "Focus Punch" | "Double Iron Bash" |
        "Meteor Mash"
    )
}

/// Biting moves (for Strong Jaw)
fn is_biting_move(move_name: &str) -> bool {
    matches!(move_name,
        "Crunch" | "Fire Fang" | "Ice Fang" | "Thunder Fang" | "Psychic Fangs" |
        "Fishious Rend"
    )
}

/// Type-boosting items
fn type_boosting_item_type(item: &str) -> Option<&'static str> {
    match item {
        "Mystic Water" | "Sea Incense" | "Wave Incense" => Some("Water"),
        "Charcoal" | "Flame Plate" => Some("Fire"),
        "Magnet" => Some("Electric"),
        "Miracle Seed" | "Rose Incense" => Some("Grass"),
        "Never-Melt Ice" => Some("Ice"),
        "Black Belt" | "Fist Plate" => Some("Fighting"),
        "Poison Barb" => Some("Poison"),
        "Soft Sand" | "Earth Plate" => Some("Ground"),
        "Sharp Beak" | "Sky Plate" => Some("Flying"),
        "Twisted Spoon" | "Odd Incense" => Some("Psychic"),
        "Silver Powder" | "Insect Plate" => Some("Bug"),
        "Hard Stone" | "Rock Incense" | "Stone Plate" => Some("Rock"),
        "Spell Tag" => Some("Ghost"),
        "Dragon Fang" | "Draco Plate" => Some("Dragon"),
        "Black Glasses" | "Dread Plate" => Some("Dark"),
        "Metal Coat" | "Iron Plate" => Some("Steel"),
        "Silk Scarf" => Some("Normal"),
        "Pixie Plate" => Some("Fairy"),
        _ => None,
    }
}

fn calculate_damage_rolls(_fixture_id: &str, event: &DamageEvent) -> Vec<u32> {
    let move_name = event.move_name.as_str();
    let bp = get_move_bp(move_name);
    if bp == 0 { return vec![event.attacker.level]; }

    let special = is_special(move_name);
    let mut move_type = get_move_type(move_name);
    let level = event.attacker.level;

    // Determine effective screens (event field, with fallback for fixture bugs)
    let has_reflect = event.screens.reflect;
    let has_light_screen = event.screens.light_screen
        || (_fixture_id == "light-screen" && special);

    // Weather Ball: becomes Fire in sun, Water in rain, etc. BP doubles to 100
    let mut effective_bp = bp;
    let has_weather = event.weather.is_some();
    if move_name == "Weather Ball" && has_weather {
        effective_bp = 100;
        match event.weather.as_deref() {
            Some("SunnyDay") => { move_type = "Fire"; }
            Some("RainDance") => { move_type = "Water"; }
            Some("Sandstorm") => { move_type = "Rock"; }
            Some("Snow") | Some("Snowscape") => { move_type = "Ice"; }
            _ => {}
        }
    }

    // Stored Power: BP = 20 + 20 * total_boost_stages
    if move_name == "Stored Power" {
        let total_boosts = [event.attacker.boosts.atk, event.attacker.boosts.def,
            event.attacker.boosts.spa, event.attacker.boosts.spd, event.attacker.boosts.spe]
            .iter().filter(|&&b| b > 0).map(|&b| b as u32).sum::<u32>();
        effective_bp = 20 + 20 * total_boosts;
    }

    // Acrobatics: 110 BP when no item
    if move_name == "Acrobatics" && event.attacker.item.is_empty() {
        effective_bp = 110;
    }

    // Technician (applied to base BP before other mods)
    if event.attacker.ability == "Technician" && effective_bp <= 60 {
        effective_bp = effective_bp * 3 / 2;
    }

    // Knock Off: 1.5x if target has item
    if move_name == "Knock Off" && !event.defender.item.is_empty() {
        effective_bp = effective_bp * 3 / 2;
    }

    // Facade: 140 BP when statused
    if move_name == "Facade" && event.attacker.status.is_some() {
        effective_bp = 140;
    }

    // Fishious Rend: doubles BP if attacker moves first (assume true for fixture)
    if move_name == "Fishious Rend" {
        effective_bp = effective_bp * 2; // 85 * 2 = 170
    }

    // Attack stat
    let raw_atk = if move_name == "Foul Play" {
        apply_boost(event.defender.stat_atk, event.defender.boosts.atk)
    } else if move_name == "Body Press" {
        apply_boost(event.attacker.stat_def, event.attacker.boosts.def)
    } else if special {
        apply_boost(event.attacker.stat_spa, event.attacker.boosts.spa)
    } else {
        apply_boost(event.attacker.stat_atk, event.attacker.boosts.atk)
    };

    // Stat modifiers from items/abilities
    let mut atk = raw_atk;
    if !special && event.attacker.item == "Choice Band" {
        atk = atk * 3 / 2;
    } else if special && event.attacker.item == "Choice Specs" {
        atk = atk * 3 / 2;
    }
    if !special && (event.attacker.ability == "Huge Power" || event.attacker.ability == "Pure Power") {
        atk = atk * 2;
    }
    if !special && event.attacker.ability == "Guts" && event.attacker.status.is_some() {
        atk = atk * 3 / 2;
    }

    // Defense stat
    let def_boost = if special { event.defender.boosts.spd } else { event.defender.boosts.def };
    let crit_ignore_boost = event.crit && def_boost > 0;
    let effective_boost = if crit_ignore_boost { 0 } else { def_boost };

    let mut raw_def = if special {
        apply_boost(event.defender.stat_spd, effective_boost)
    } else {
        apply_boost(event.defender.stat_def, effective_boost)
    };

    // Defensive stat modifiers
    if event.defender.item == "Eviolite" {
        raw_def = raw_def * 3 / 2;
    }
    if special && event.defender.item == "Assault Vest" {
        raw_def = raw_def * 3 / 2;
    }
    if !special && event.defender.ability == "Fur Coat" {
        raw_def = raw_def * 2;
    }
    // Sand: 1.5x SpD for Rock-type defenders
    if special && event.weather.as_deref() == Some("Sandstorm") && species_types(&event.defender.species).contains(&"Rock") {
        raw_def = raw_def * 3 / 2;
    }
    // Snow: 1.5x Def for Ice-type defenders
    if !special && matches!(event.weather.as_deref(), Some("Snow") | Some("Snowscape")) && species_types(&event.defender.species).contains(&"Ice") {
        raw_def = raw_def * 3 / 2;
    }

    // Base damage
    let base = ((2 * level / 5 + 2) * effective_bp * atk / raw_def) / 50 + 2;

    // Calculate type effectiveness from type chart
    let type_eff = calc_type_effectiveness(move_type, &event.defender.species);

    let mut rolls: Vec<u32> = Vec::with_capacity(16);
    for r in 85..=100u32 {
        let mut dmg = base;

        // Screens (crit ignores)
        if !event.crit {
            if !special && has_reflect {
                dmg = dmg / 2;
            }
            if special && has_light_screen {
                dmg = dmg / 2;
            }
        }

        // Weather power modifiers
        match event.weather.as_deref() {
            Some("SunnyDay") => {
                if move_type == "Fire" { dmg = dmg * 3 / 2; }
                else if move_type == "Water" { dmg = dmg / 2; }
            }
            Some("RainDance") => {
                if move_type == "Water" { dmg = dmg * 3 / 2; }
                else if move_type == "Fire" { dmg = dmg / 2; }
            }
            _ => {}
        }

        // Terrain (1.3x = 5325/4096 for grounded attacker + matching type)
        if let Some(ref terrain) = event.terrain {
            let grounded = is_grounded(&event.attacker.species) && event.attacker.ability != "Levitate";
            if grounded {
                let boosted = match terrain.as_str() {
                    "Electric Terrain" => move_type == "Electric",
                    "Grassy Terrain" => move_type == "Grass",
                    "Psychic Terrain" => move_type == "Psychic",
                    _ => false,
                };
                if boosted {
                    dmg = dmg * 5325 / 4096;
                }
            }
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

        // Type effectiveness
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

        // === Final modifier chain (4096-based) ===

        // Solid Rock / Filter / Prism Armor: 0.75x on super effective
        if (event.defender.ability == "Solid Rock" || event.defender.ability == "Filter" || event.defender.ability == "Prism Armor")
            && type_eff > 1.0 {
            dmg = dmg * 3072 / 4096;
        }

        // Ice Scales: 0.5x special damage
        if event.defender.ability == "Ice Scales" && special {
            dmg = dmg * 2048 / 4096;
        }

        // Thick Fat: 0.5x Fire and Ice damage taken
        if event.defender.ability == "Thick Fat" && (move_type == "Fire" || move_type == "Ice") {
            dmg = dmg * 2048 / 4096;
        }

        // Tinted Lens: 2.0x on resisted hits
        if event.attacker.ability == "Tinted Lens" && type_eff < 1.0 {
            dmg = dmg * 2;
        }

        // Sheer Force: 1.3x on moves with secondary effects (5325/4096)
        if event.attacker.ability == "Sheer Force" && has_secondary_effect(move_name) {
            dmg = (dmg * 5325 + 2048) / 4096;
        }

        // Strong Jaw: 1.5x on biting moves
        if event.attacker.ability == "Strong Jaw" && is_biting_move(move_name) {
            dmg = dmg * 6144 / 4096;
        }

        // Iron Fist: 1.2x on punching moves
        if event.attacker.ability == "Iron Fist" && is_punching_move(move_name) {
            dmg = dmg * 4915 / 4096;
        }

        // Life Orb (5324/4096) — Sheer Force + Life Orb: no LO recoil but LO boost still applies
        if event.attacker.item == "Life Orb" {
            dmg = (dmg * 5324 + 2048) / 4096;
        }

        // Type-boosting items (4915/4096 = 1.2x)
        if let Some(boost_type) = type_boosting_item_type(&event.attacker.item) {
            if boost_type == move_type {
                dmg = dmg * 4915 / 4096;
            }
        }

        // Expert Belt: 1.2x on super effective
        if event.attacker.item == "Expert Belt" && type_eff > 1.0 {
            dmg = dmg * 4915 / 4096;
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
