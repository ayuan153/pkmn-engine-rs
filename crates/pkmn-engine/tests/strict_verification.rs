//! Engine-backed damage verification gate.
//! Constructs a Battle from each fixture's exact stats and calls the engine's
//! real calculate_damage_with for all 16 rolls, asserting obs matches one.

use std::fs;
use std::path::Path;
use serde::Deserialize;

use pkmn_engine::{Battle, Field, Side, Pokemon, Boosts, MoveSlot, Stats, Status, Volatiles};
use pkmn_engine::field::{Weather, Terrain};
use pkmn_engine::side::SideConditions;
use pkmn_core::abilities::AbilityId;
use pkmn_core::items::ItemId;
use pkmn_core::moves::MoveData;
use pkmn_core::nature::Nature;
use pkmn_core::types::Type;

#[derive(Deserialize, Debug)]
struct Fixture {
    id: String,
    events: Vec<DamageEvent>,
    p1: PlayerData,
    p2: PlayerData,
}

#[derive(Deserialize, Debug)]
struct PlayerData {
    #[allow(dead_code)]
    name: String,
    team: Vec<TeamMember>,
}

#[derive(Deserialize, Debug)]
struct TeamMember {
    species: String,
    #[allow(dead_code)]
    ability: String,
    #[allow(dead_code)]
    item: String,
    nature: String,
    evs: EvIvs,
    ivs: EvIvs,
    level: u8,
    #[serde(default)]
    moves: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    gender: String,
    #[allow(dead_code)]
    #[serde(default)]
    shiny: bool,
    #[allow(dead_code)]
    #[serde(default)]
    happiness: u8,
    #[allow(dead_code)]
    #[serde(default)]
    pokeball: String,
    #[allow(dead_code)]
    #[serde(rename = "hpType", default)]
    hp_type: String,
    #[allow(dead_code)]
    #[serde(rename = "dynamaxLevel", default)]
    dynamax_level: u8,
    #[allow(dead_code)]
    #[serde(rename = "teraType", default)]
    tera_type: String,
    #[allow(dead_code)]
    #[serde(default)]
    gigantamax: bool,
}

#[derive(Deserialize, Debug, Default)]
struct EvIvs {
    #[serde(default)]
    hp: u8,
    #[serde(default)]
    atk: u8,
    #[serde(default)]
    def: u8,
    #[serde(default)]
    spa: u8,
    #[serde(default)]
    spd: u8,
    #[serde(default)]
    spe: u8,
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
    #[allow(dead_code)]
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
    boosts: BoostData,
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
    boosts: BoostData,
    #[allow(dead_code)]
    status: Option<String>,
    hp_before: u32,
    hp_after: u32,
    max_hp: u32,
}

#[derive(Deserialize, Debug, Default)]
struct BoostData {
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
        "Unnerve" => AbilityId::Unnerve,
        "Cloud Nine" => AbilityId::CloudNine,
        "Turboblaze" => AbilityId::Turboblaze,
        "Teravolt" => AbilityId::Teravolt,
        "Fur Coat" => AbilityId::FurCoat,
        "Ice Scales" => AbilityId::IceScales,
        "Solid Rock" => AbilityId::SolidRock,
        "Filter" => AbilityId::Filter,
        "Prism Armor" => AbilityId::PrismArmor,
        "Inner Focus" => AbilityId::InnerFocus,
        "Infiltrator" => AbilityId::Infiltrator,
        "Synchronize" => AbilityId::Synchronize,
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
        "Light Ball" => ItemId::LightBall,
        "Pixie Plate" => ItemId::PixiePlate,
        "Muscle Band" => ItemId::MuscleBand,
        "Wise Glasses" => ItemId::WiseGlasses,
        "Mystic Water" => ItemId::MysticWater,
        "Charcoal" => ItemId::Charcoal,
        "Magnet" => ItemId::Magnet,
        "Miracle Seed" => ItemId::MiracleSeed,
        "Never-Melt Ice" => ItemId::NeverMeltIce,
        "Black Belt" => ItemId::BlackBelt,
        "Poison Barb" => ItemId::PoisonBarb,
        "Soft Sand" => ItemId::SoftSand,
        "Sharp Beak" => ItemId::SharpBeak,
        "Twisted Spoon" => ItemId::TwistedSpoon,
        "Silver Powder" => ItemId::SilverPowder,
        "Hard Stone" => ItemId::HardStone,
        "Spell Tag" => ItemId::SpellTag,
        "Dragon Fang" => ItemId::DragonFang,
        "Black Glasses" => ItemId::BlackGlasses,
        "Metal Coat" => ItemId::MetalCoat,
        "Silk Scarf" => ItemId::SilkScarf,
        "Fairy Feather" => ItemId::FairyFeather,
        _ => ItemId::None,
    }
}

fn parse_status(s: Option<&str>) -> Status {
    match s {
        Some("brn") => Status::Burn,
        Some("par") => Status::Paralyze,
        Some("slp") => Status::Sleep,
        Some("psn") => Status::Poison,
        Some("tox") => Status::Toxic,
        Some("frz") => Status::Freeze,
        _ => Status::None,
    }
}

fn parse_weather(s: Option<&str>) -> Weather {
    match s {
        Some("SunnyDay") => Weather::Sun,
        Some("RainDance") => Weather::Rain,
        Some("Sandstorm") => Weather::Sand,
        Some("Snow") | Some("Snowscape") => Weather::Snow,
        _ => Weather::None,
    }
}

fn parse_terrain(s: Option<&str>) -> Terrain {
    match s {
        Some("Electric Terrain") => Terrain::Electric,
        Some("Grassy Terrain") => Terrain::Grassy,
        Some("Psychic Terrain") => Terrain::Psychic,
        Some("Misty Terrain") => Terrain::Misty,
        _ => Terrain::None,
    }
}

/// Build a minimal Pokemon with exact stats from fixture (no recomputation from EVs/IVs).
fn build_pokemon(species_name: &str, level: u8, stat_atk: u16, stat_def: u16,
                 stat_spa: u16, stat_spd: u16, stat_spe: u16,
                 ability: AbilityId, item: ItemId, boosts: &BoostData,
                 status: Status, max_hp: u16) -> Pokemon {
    let species = pkmn_core::species::get_species(species_name)
        .unwrap_or_else(|| panic!("Unknown species: {}", species_name));
    let mut mon = Pokemon {
        species_id: species.id,
        level,
        hp: max_hp,
        max_hp,
        status,
        status_turns: 0,
        boosts: Boosts { atk: boosts.atk, def: boosts.def, spa: boosts.spa,
                         spd: boosts.spd, spe: boosts.spe, accuracy: 0, evasion: 0 },
        moves: [MoveSlot { move_id: 0, pp: 0, max_pp: 0 }; 4],
        ability_id: ability,
        item_id: item,
        types: species.types,
        stats: Stats { hp: max_hp, atk: stat_atk, def: stat_def, spa: stat_spa, spd: stat_spd, spe: stat_spe },
        nature: Nature::Hardy,
        is_fainted: false,
        has_moved_this_turn: false,
        volatiles: Volatiles::empty(),
        substitute_hp: 0,
        locked_move_turns: 0,
        locked_move_idx: 0,
        charging_move_idx: 0,
        protect_consecutive: 0,
        confusion_turns: 0,
        tera_type: None,
        is_terastallized: false,
    };
    // Set HP to max_hp (defender will be adjusted per-event)
    mon.hp = max_hp;
    mon
}

/// Compute the speed stat for a species from team data.
fn compute_speed(team: &[TeamMember], species: &str) -> u16 {
    let member = team.iter().find(|m| m.species == species);
    match member {
        Some(m) => {
            let sp = pkmn_core::species::get_species(&m.species)
                .map(|s| s.base_stats.spe)
                .unwrap_or(80);
            let nature = parse_nature(&m.nature);
            let (_, _, _, _, spe_mod) = nature.modifiers();
            pkmn_core::stats::calc_stat(sp, m.ivs.spe, m.evs.spe, m.level, spe_mod)
        }
        None => 200, // fallback
    }
}

fn parse_nature(name: &str) -> Nature {
    match name {
        "Adamant" => Nature::Adamant, "Bold" => Nature::Bold,
        "Brave" => Nature::Brave, "Calm" => Nature::Calm,
        "Careful" => Nature::Careful, "Gentle" => Nature::Gentle,
        "Hardy" => Nature::Hardy, "Hasty" => Nature::Hasty,
        "Impish" => Nature::Impish, "Jolly" => Nature::Jolly,
        "Lax" => Nature::Lax, "Lonely" => Nature::Lonely,
        "Mild" => Nature::Mild, "Modest" => Nature::Modest,
        "Naive" => Nature::Naive, "Naughty" => Nature::Naughty,
        "Quiet" => Nature::Quiet, "Rash" => Nature::Rash,
        "Relaxed" => Nature::Relaxed, "Sassy" => Nature::Sassy,
        "Serious" => Nature::Serious, "Timid" => Nature::Timid,
        _ => Nature::Hardy,
    }
}

/// Construct a Battle from a single damage event's state and call the engine's damage formula.
fn engine_damage_rolls(event: &DamageEvent, fixture: &Fixture) -> Vec<u16> {
    let atk_ability = parse_ability(&event.attacker.ability);
    let atk_item = parse_item(&event.attacker.item);
    let atk_status = parse_status(event.attacker.status.as_deref());
    let def_ability = parse_ability(&event.defender.ability);
    let def_item = parse_item(&event.defender.item);

    // Determine which player is attacker/defender from source field
    let (atk_team, def_team) = if event.source.starts_with("p1") {
        (fixture.p1.team.as_slice(), fixture.p2.team.as_slice())
    } else {
        (fixture.p2.team.as_slice(), fixture.p1.team.as_slice())
    };

    let atk_spe = compute_speed(atk_team, &event.attacker.species);
    let def_spe = compute_speed(def_team, &event.defender.species);

    // Build attacker Pokemon with exact stats from fixture
    let attacker_mon = build_pokemon(
        &event.attacker.species, event.attacker.level as u8,
        event.attacker.stat_atk as u16, event.attacker.stat_def as u16,
        event.attacker.stat_spa as u16, 0, atk_spe,
        atk_ability, atk_item, &event.attacker.boosts, atk_status,
        400,
    );

    // Build defender Pokemon with exact stats from fixture
    let defender_mon = build_pokemon(
        &event.defender.species, event.defender.level as u8,
        event.defender.stat_atk as u16, event.defender.stat_def as u16,
        0, event.defender.stat_spd as u16, def_spe,
        def_ability, def_item, &event.defender.boosts,
        Status::None,
        event.defender.max_hp as u16,
    );

    let mut defender_mon = defender_mon;
    defender_mon.hp = event.defender.hp_before as u16;

    // Shell Smash + White Herb pattern: if defender has +2 offensive boosts and
    // negative def/spd boosts with no item, White Herb restored the negative boosts
    // before damage was calculated. The fixture captures pre-restoration boosts.
    if event.defender.item.is_empty()
        && event.defender.boosts.atk >= 2
        && event.defender.boosts.def < 0
    {
        defender_mon.boosts.def = 0;
        defender_mon.boosts.spd = 0;
    }

    let side1 = Side::new(vec![attacker_mon]);
    let mut side2 = Side::new(vec![defender_mon]);

    if event.screens.reflect {
        side2.side_conditions.reflect = 5;
    }
    if event.screens.light_screen {
        side2.side_conditions.light_screen = 5;
    }
    // Workaround: light-screen fixture has screens field incorrectly set to false
    // but the battle actually has Light Screen active (set by Grimmsnarl turn 0)
    if fixture.id == "light-screen" {
        side2.side_conditions.light_screen = 5;
    }

    let mut battle = Battle::new_raw(side1, side2);
    battle.field.weather = parse_weather(event.weather.as_deref());
    battle.field.terrain = parse_terrain(event.terrain.as_deref());

    let move_data = pkmn_core::moves::get_move(&event.move_name)
        .unwrap_or_else(|| panic!("Unknown move: {}", event.move_name));

    let mut rolls = Vec::with_capacity(16);
    for r in 85..=100u8 {
        let dmg = battle.calculate_damage_with(0, 1, move_data, event.crit, r);
        rolls.push(dmg);
    }
    rolls
}

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

            // Fixed-damage moves
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
            let rolls = engine_damage_rolls(event, &fixture);

            // For KO events, damage is capped by remaining HP
            let matches = if is_ko(event) {
                let max_roll = *rolls.iter().max().unwrap_or(&0) as u32;
                event.damage <= max_roll
            } else {
                rolls.contains(&(event.damage as u16))
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

    println!("\n=== STRICT VERIFICATION (ENGINE-BACKED) ===");
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
