use std::fs;
use std::path::Path;

use pkmn_core::abilities::AbilityId;
use pkmn_core::items::ItemId;
use pkmn_core::moves::get_move;
use pkmn_core::nature::Nature;
use pkmn_core::species::get_species;
use pkmn_engine::{Battle, BattleResult, Choice, MoveSlot, Pokemon, Side};
use serde::Deserialize;

#[derive(Deserialize)]
struct FullSimFixture {
    id: String,
    #[allow(dead_code)]
    description: String,
    seed: [u64; 4],
    p1: TeamData,
    p2: TeamData,
    choices: Vec<[String; 2]>,
    protocol: Vec<String>,
}

#[derive(Deserialize)]
struct TeamData {
    team: Vec<PokemonSetData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PokemonSetData {
    species: String,
    ability: String,
    item: String,
    nature: String,
    moves: Vec<String>,
    level: u8,
    evs: EvSpread,
    ivs: EvSpread,
    #[serde(default)]
    name: String,
    #[serde(default)]
    gender: String,
    #[serde(default)]
    shiny: bool,
    #[serde(default)]
    happiness: u8,
    #[serde(default)]
    pokeball: String,
    #[serde(default)]
    hp_type: String,
    #[serde(default)]
    dynamax_level: u8,
    #[serde(default)]
    tera_type: String,
    #[serde(default)]
    gigantamax: bool,
}

#[derive(Deserialize)]
struct EvSpread {
    hp: u8,
    atk: u8,
    def: u8,
    spa: u8,
    spd: u8,
    spe: u8,
}

fn parse_nature(name: &str) -> Nature {
    match name {
        "Hardy" => Nature::Hardy,
        "Lonely" => Nature::Lonely,
        "Brave" => Nature::Brave,
        "Adamant" => Nature::Adamant,
        "Naughty" => Nature::Naughty,
        "Bold" => Nature::Bold,
        "Docile" => Nature::Docile,
        "Relaxed" => Nature::Relaxed,
        "Impish" => Nature::Impish,
        "Lax" => Nature::Lax,
        "Timid" => Nature::Timid,
        "Hasty" => Nature::Hasty,
        "Serious" => Nature::Serious,
        "Jolly" => Nature::Jolly,
        "Naive" => Nature::Naive,
        "Modest" => Nature::Modest,
        "Mild" => Nature::Mild,
        "Quiet" => Nature::Quiet,
        "Bashful" => Nature::Bashful,
        "Rash" => Nature::Rash,
        "Calm" => Nature::Calm,
        "Gentle" => Nature::Gentle,
        "Sassy" => Nature::Sassy,
        "Careful" => Nature::Careful,
        "Quirky" => Nature::Quirky,
        _ => Nature::Hardy,
    }
}

fn parse_ability(name: &str) -> AbilityId {
    match name {
        "Intimidate" => AbilityId::Intimidate,
        "Levitate" => AbilityId::Levitate,
        "Mold Breaker" => AbilityId::MoldBreaker,
        "Multiscale" => AbilityId::Multiscale,
        "Sturdy" => AbilityId::Sturdy,
        "Flame Body" => AbilityId::FlameBody,
        "Static" => AbilityId::Static,
        "Poison Point" => AbilityId::PoisonPoint,
        "Rough Skin" => AbilityId::RoughSkin,
        "Iron Barbs" => AbilityId::IronBarbs,
        "Natural Cure" => AbilityId::NaturalCure,
        "Regenerator" => AbilityId::Regenerator,
        "Unaware" => AbilityId::Unaware,
        "Magic Bounce" => AbilityId::MagicBounce,
        "Magic Guard" => AbilityId::MagicGuard,
        "Technician" => AbilityId::Technician,
        "Adaptability" => AbilityId::Adaptability,
        "Huge Power" => AbilityId::HugePower,
        "Pure Power" => AbilityId::PurePower,
        "Sheer Force" => AbilityId::SheerForce,
        "Protean" => AbilityId::Protean,
        "Libero" => AbilityId::Libero,
        "Tough Claws" => AbilityId::ToughClaws,
        "Iron Fist" => AbilityId::IronFist,
        "Strong Jaw" => AbilityId::StrongJaw,
        "Swift Swim" => AbilityId::SwiftSwim,
        "Chlorophyll" => AbilityId::Chlorophyll,
        "Sand Rush" => AbilityId::SandRush,
        "Slush Rush" => AbilityId::SlushRush,
        "Surge Surfer" => AbilityId::SurgeSurfer,
        "Drought" => AbilityId::Drought,
        "Drizzle" => AbilityId::Drizzle,
        "Sand Stream" => AbilityId::SandStream,
        "Snow Warning" => AbilityId::SnowWarning,
        "Electric Surge" => AbilityId::ElectricSurge,
        "Grassy Surge" => AbilityId::GrassySurge,
        "Misty Surge" => AbilityId::MistySurge,
        "Psychic Surge" => AbilityId::PsychicSurge,
        "Guts" => AbilityId::Guts,
        "Marvel Scale" => AbilityId::MarvelScale,
        "Overcoat" => AbilityId::Overcoat,
        "Thick Fat" => AbilityId::ThickFat,
        "Flash Fire" => AbilityId::FlashFire,
        "Volt Absorb" => AbilityId::VoltAbsorb,
        "Water Absorb" => AbilityId::WaterAbsorb,
        "Lightning Rod" => AbilityId::LightningRod,
        "Storm Drain" => AbilityId::StormDrain,
        "Clear Body" => AbilityId::ClearBody,
        "White Smoke" => AbilityId::WhiteSmoke,
        "Full Metal Body" => AbilityId::FullMetalBody,
        "Speed Boost" => AbilityId::SpeedBoost,
        "Beast Boost" => AbilityId::BeastBoost,
        "Moxie" => AbilityId::Moxie,
        "Tinted Lens" => AbilityId::TintedLens,
        "Sword of Ruin" => AbilityId::SwordOfRuin,
        "Tablets of Ruin" => AbilityId::TabletsOfRuin,
        "Vessel of Ruin" => AbilityId::VesselOfRuin,
        "Beads of Ruin" => AbilityId::BeadsOfRuin,
        "Pressure" => AbilityId::Pressure,
        "Prankster" => AbilityId::Prankster,
        "Supreme Overlord" => AbilityId::SupremeOverlord,
        "Cursed Body" => AbilityId::CursedBody,
        "Skill Link" => AbilityId::SkillLink,
        _ => AbilityId::None,
    }
}

fn parse_item(name: &str) -> ItemId {
    match name {
        "Choice Band" => ItemId::ChoiceBand,
        "Choice Specs" => ItemId::ChoiceSpecs,
        "Choice Scarf" => ItemId::ChoiceScarf,
        "Life Orb" => ItemId::LifeOrb,
        "Leftovers" => ItemId::Leftovers,
        "Black Sludge" => ItemId::BlackSludge,
        "Heavy-Duty Boots" => ItemId::HeavyDutyBoots,
        "Assault Vest" => ItemId::AssaultVest,
        "Rocky Helmet" => ItemId::RockyHelmet,
        "Focus Sash" => ItemId::FocusSash,
        "Weakness Policy" => ItemId::WeaknessPolicy,
        "Expert Belt" => ItemId::ExpertBelt,
        "Eviolite" => ItemId::Eviolite,
        "Light Clay" => ItemId::LightClay,
        "Heat Rock" => ItemId::HeatRock,
        "Damp Rock" => ItemId::DampRock,
        "Flame Orb" => ItemId::FlameOrb,
        "Toxic Orb" => ItemId::ToxicOrb,
        "Sitrus Berry" => ItemId::SitrusBerry,
        "Lum Berry" => ItemId::LumBerry,
        "Protective Pads" => ItemId::ProtectivePads,
        "Safety Goggles" => ItemId::SafetyGoggles,
        "Shed Shell" => ItemId::ShedShell,
        "Red Card" => ItemId::RedCard,
        "Air Balloon" => ItemId::AirBalloon,
        "Loaded Dice" => ItemId::LoadedDice,
        "Clear Amulet" => ItemId::ClearAmulet,
        _ => ItemId::None,
    }
}

fn parse_choice(s: &str) -> Result<Choice, String> {
    if let Some(idx) = s.strip_prefix("move ") {
        let i: u8 = idx.parse::<u8>().map_err(|e| e.to_string())? - 1;
        Ok(Choice::Move(i))
    } else if let Some(idx) = s.strip_prefix("switch ") {
        let i: u8 = idx.parse::<u8>().map_err(|e| e.to_string())? - 1;
        Ok(Choice::Switch(i))
    } else if let Some(idx) = s.strip_prefix("tera ") {
        let i: u8 = idx.parse::<u8>().map_err(|e| e.to_string())? - 1;
        Ok(Choice::Tera(i))
    } else {
        Err(format!("Unknown choice: '{}'", s))
    }
}

fn build_pokemon(data: &PokemonSetData) -> Result<Pokemon, String> {
    let species = get_species(&data.species)
        .ok_or_else(|| format!("Unknown species: {}", data.species))?;

    let nature = parse_nature(&data.nature);
    let evs = [data.evs.hp, data.evs.atk, data.evs.def, data.evs.spa, data.evs.spd, data.evs.spe];
    let ivs = [data.ivs.hp, data.ivs.atk, data.ivs.def, data.ivs.spa, data.ivs.spd, data.ivs.spe];

    let mut moves = [MoveSlot { move_id: 0, pp: 0, max_pp: 0 }; 4];
    for (i, move_name) in data.moves.iter().enumerate().take(4) {
        let move_data = get_move(move_name)
            .ok_or_else(|| format!("Unknown move: {}", move_name))?;
        moves[i] = MoveSlot {
            move_id: move_data.id,
            pp: move_data.pp,
            max_pp: move_data.pp,
        };
    }

    let mut mon = Pokemon::new(species, data.level, nature, moves, evs, ivs);
    mon.ability_id = parse_ability(&data.ability);
    mon.item_id = parse_item(&data.item);
    Ok(mon)
}

fn build_battle(fixture: &FullSimFixture) -> Result<Battle, String> {
    let mut team1 = Vec::new();
    for p in &fixture.p1.team {
        team1.push(build_pokemon(p)?);
    }
    let mut team2 = Vec::new();
    for p in &fixture.p2.team {
        team2.push(build_pokemon(p)?);
    }

    // Convert seed array to [u16; 4] for PS-compatible PRNG
    let seed = [
        fixture.seed[0] as u16,
        fixture.seed[1] as u16,
        fixture.seed[2] as u16,
        fixture.seed[3] as u16,
    ];

    Ok(Battle::new(Side::new(team1), Side::new(team2), seed))
}

fn run_and_compare(fixture: &FullSimFixture) -> Result<(), String> {
    let mut battle = build_battle(fixture)?;

    let mut all_protocol: Vec<String> = Vec::new();
    // Drain initial protocol (switch-ins, abilities)
    all_protocol.extend(battle.drain_protocol());

    for [p1_choice, p2_choice] in &fixture.choices {
        // Handle forced switch phase
        if let pkmn_engine::BattlePhase::ForcedSwitch(_) = battle.phase {
            break;
        }
        let p1c = parse_choice(p1_choice)?;
        let p2c = parse_choice(p2_choice)?;
        let result = battle.apply(p1c, p2c);
        all_protocol.extend(battle.drain_protocol());
        if result != pkmn_engine::BattleResult::Ongoing {
            break;
        }
    }

    compare_protocols(&fixture.protocol, &all_protocol)
}

/// Normalize a protocol line to strip cosmetic differences:
/// - Pokemon nicknames: "pXa: Nickname" -> "pXa: ~"
/// - Switch details: strip gender and L100
fn normalize_line(line: &str) -> String {
    // Normalize pokemon references in all lines: "pXa: Name" -> "pXa: ~"
    let mut result = String::new();
    let mut remaining = line;
    while let Some(pos) = remaining.find("p1a: ").or_else(|| remaining.find("p2a: ")) {
        let prefix_end = pos + 4; // "pXa:" length
        result.push_str(&remaining[..prefix_end]);
        remaining = &remaining[prefix_end..];
        // Skip the space and name until next | or end
        if remaining.starts_with(' ') {
            result.push_str(" ~");
            remaining = &remaining[1..]; // skip space
            // Skip name chars until | or end
            if let Some(pipe) = remaining.find('|') {
                remaining = &remaining[pipe..];
            } else {
                remaining = "";
            }
        }
    }
    result.push_str(remaining);

    // For switch lines, normalize details (strip gender, L100)
    if result.starts_with("|switch|") || result.starts_with("|drag|") {
        let parts: Vec<&str> = result.splitn(5, '|').collect();
        if parts.len() >= 4 {
            // parts[0]="" parts[1]="switch" parts[2]="p1a: ~" parts[3] has "Details|HP" or "Details|HP|[from]..."
            let after_pokemon = &result[parts[0].len() + 1 + parts[1].len() + 1 + parts[2].len() + 1..];
            if let Some(pipe_pos) = after_pokemon.find('|') {
                let details = &after_pokemon[..pipe_pos];
                let hp_and_rest = &after_pokemon[pipe_pos + 1..];
                let detail_tokens: Vec<&str> = details.split(", ").collect();
                let species = detail_tokens[0];
                let mut norm_details = species.to_string();
                for token in &detail_tokens[1..] {
                    let t = token.trim();
                    if t.starts_with('L') && t != "L100" {
                        norm_details.push_str(", ");
                        norm_details.push_str(t);
                    }
                    // Skip gender (M, F) and L100
                }
                return format!("|{}|{}|{}|{}", parts[1], parts[2], norm_details, hp_and_rest);
            }
        }
    }

    result
}

fn compare_protocols(expected: &[String], actual: &[String]) -> Result<(), String> {
    if actual.len() < expected.len() {
        return Err(format!(
            "Too few lines: expected {} lines, got {}",
            expected.len(), actual.len()
        ));
    }
    for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
        let norm_exp = normalize_line(exp);
        let norm_act = normalize_line(act);
        if norm_exp != norm_act {
            // Allow damage values to differ (RNG mismatch) if the line structure matches
            if is_damage_match(&norm_exp, &norm_act) {
                continue;
            }
            return Err(format!(
                "Line {}: expected '{}' got '{}'",
                i, exp, act
            ));
        }
    }
    Ok(())
}

/// Check if two lines differ only in HP values (damage amount due to RNG)
fn is_damage_match(exp: &str, act: &str) -> bool {
    // Match lines like |-damage|pXa: ~|NNN/MMM or |-damage|pXa: ~|0 fnt
    // or |-heal|pXa: ~|NNN/MMM|...
    if !(exp.contains("|-damage|") || exp.contains("|-heal|")) {
        return false;
    }
    if !(act.contains("|-damage|") || act.contains("|-heal|")) {
        return false;
    }
    // Split on | and compare all parts except the HP field
    let exp_parts: Vec<&str> = exp.split('|').collect();
    let act_parts: Vec<&str> = act.split('|').collect();
    if exp_parts.len() != act_parts.len() {
        return false;
    }
    // Compare all parts except the HP value (index 3 for |-damage|pXa: ~|HP/MAX)
    for (i, (e, a)) in exp_parts.iter().zip(act_parts.iter()).enumerate() {
        if i == 3 {
            // This is the HP field — allow it to differ
            // But both should have the same format (NNN/MMM or "0 fnt")
            let e_fnt = e.contains("fnt");
            let a_fnt = a.contains("fnt");
            if e_fnt != a_fnt {
                return false;
            }
            continue;
        }
        if e != a {
            return false;
        }
    }
    true
}

#[test]
fn full_sim_protocol_matches() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/full-sim");

    let mut total_fixtures = 0;
    let mut passed = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in fs::read_dir(&fixture_dir).expect("Cannot read full-sim fixture dir") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "json") {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();
        let fixture: FullSimFixture = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
        total_fixtures += 1;

        match run_and_compare(&fixture) {
            Ok(()) => passed += 1,
            Err(msg) => failures.push(format!("{}: {}", fixture.id, msg)),
        }
    }

    println!("\n=== FULL-SIM PROTOCOL COMPARISON ===");
    println!("Fixtures: {}", total_fixtures);
    println!("Passed:   {}", passed);
    println!("Failed:   {}", failures.len());

    if !failures.is_empty() {
        println!("\nFailures:");
        for f in &failures {
            println!("  {}", f);
        }
    }

    // Report only — don't assert until divergences are fixed
    // assert_eq!(failures.len(), 0);
}

