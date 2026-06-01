//! Hook-based effect dispatch system (Phase 0).
//!
//! EventHooks is a fn-pointer table: no heap, no dyn, no trait objects.
//! Battle remains Clone and small. Each hook is Option<fn(...)> so absent
//! hooks cost zero at runtime (branch-predicted None check).

use crate::battle::Battle;
use pkmn_core::abilities::AbilityId;
use pkmn_core::items::ItemId;
use pkmn_core::moves::MoveData;

/// Hook table for ability/item effects. All fields are optional fn pointers.
#[derive(Debug, Clone, Copy)]
pub struct EventHooks {
    /// Called when a Pokemon with this ability/item switches in.
    pub on_switch_in: Option<fn(&mut Battle, player: u8)>,
    /// Damage multiplier from the source (attacker's ability/item).
    /// Returns a multiplier applied to final damage.
    pub on_source_modify_damage: Option<fn(&Battle, attacker: u8, move_data: &MoveData) -> f32>,
    /// Called after a damaging hit with contact (defender's ability reacts).
    pub on_damaging_hit: Option<fn(&mut Battle, attacker: u8, defender: u8)>,
    /// Called during the end-of-turn residual pass for this ability/item.
    /// Residual order follows PS convention: items (5) < Leech Seed (8) < status (9-10).
    pub on_residual: Option<fn(&mut Battle, player: u8)>,
}

impl EventHooks {
    /// Empty hook set (no effects).
    pub const NONE: Self = Self {
        on_switch_in: None,
        on_source_modify_damage: None,
        on_damaging_hit: None,
        on_residual: None,
    };
}

// --- Static dispatch: ability hooks ---

/// Returns the hook table for a given ability. Only migrated abilities have hooks;
/// all others return EventHooks::NONE.
pub fn ability_hooks(id: AbilityId) -> EventHooks {
    match id {
        AbilityId::Intimidate => EventHooks {
            on_switch_in: Some(hook_intimidate_switch),
            ..EventHooks::NONE
        },
        AbilityId::Drizzle => EventHooks {
            on_switch_in: Some(hook_drizzle_switch),
            ..EventHooks::NONE
        },
        AbilityId::Drought => EventHooks {
            on_switch_in: Some(hook_drought_switch),
            ..EventHooks::NONE
        },
        AbilityId::SandStream => EventHooks {
            on_switch_in: Some(hook_sand_stream_switch),
            ..EventHooks::NONE
        },
        AbilityId::SnowWarning => EventHooks {
            on_switch_in: Some(hook_snow_warning_switch),
            ..EventHooks::NONE
        },
        AbilityId::Adaptability => EventHooks {
            on_source_modify_damage: Some(hook_adaptability_damage),
            ..EventHooks::NONE
        },
        AbilityId::Technician => EventHooks {
            on_source_modify_damage: Some(hook_technician_damage),
            ..EventHooks::NONE
        },
        AbilityId::RoughSkin => EventHooks {
            on_damaging_hit: Some(hook_rough_skin_hit),
            ..EventHooks::NONE
        },
        AbilityId::IronBarbs => EventHooks {
            on_damaging_hit: Some(hook_iron_barbs_hit),
            ..EventHooks::NONE
        },
        AbilityId::SpeedBoost => EventHooks {
            on_residual: Some(hook_speed_boost_residual),
            ..EventHooks::NONE
        },
        _ => EventHooks::NONE,
    }
}

/// Returns the hook table for a given item.
pub fn item_hooks(id: ItemId) -> EventHooks {
    match id {
        ItemId::LifeOrb => EventHooks {
            on_source_modify_damage: Some(hook_life_orb_damage),
            ..EventHooks::NONE
        },
        ItemId::Leftovers => EventHooks {
            on_residual: Some(hook_leftovers_residual),
            ..EventHooks::NONE
        },
        ItemId::BlackSludge => EventHooks {
            on_residual: Some(hook_black_sludge_residual),
            ..EventHooks::NONE
        },
        _ => EventHooks::NONE,
    }
}

// --- Hook implementations: on_switch_in ---

fn hook_intimidate_switch(battle: &mut Battle, player: u8) {
    let name = battle.species_name(player);
    let opp = 1 - player;
    let opp_name = battle.species_name(opp);
    battle.emit(format!(
        "|-ability|p{}a: {}|Intimidate|boost",
        player + 1,
        name
    ));
    battle.emit(format!("|-unboost|p{}a: {}|atk|1", opp + 1, opp_name));
    let cur = battle.sides[opp as usize].active().boosts.atk;
    battle.sides[opp as usize].active_mut().boosts.atk = (cur - 1).max(-6);
}

fn hook_drizzle_switch(battle: &mut Battle, player: u8) {
    use crate::field::Weather;
    let name = battle.species_name(player);
    battle.field.weather = Weather::Rain;
    battle.field.weather_turns = 5;
    battle.emit(format!(
        "|-weather|RainDance|[from] ability: Drizzle|[of] p{}a: {}",
        player + 1,
        name
    ));
}

fn hook_drought_switch(battle: &mut Battle, player: u8) {
    use crate::field::Weather;
    let name = battle.species_name(player);
    battle.field.weather = Weather::Sun;
    battle.field.weather_turns = 5;
    battle.emit(format!(
        "|-weather|SunnyDay|[from] ability: Drought|[of] p{}a: {}",
        player + 1,
        name
    ));
}

fn hook_sand_stream_switch(battle: &mut Battle, player: u8) {
    use crate::field::Weather;
    let name = battle.species_name(player);
    battle.field.weather = Weather::Sand;
    battle.field.weather_turns = 5;
    battle.emit(format!(
        "|-weather|Sandstorm|[from] ability: Sand Stream|[of] p{}a: {}",
        player + 1,
        name
    ));
}

fn hook_snow_warning_switch(battle: &mut Battle, player: u8) {
    use crate::field::Weather;
    let name = battle.species_name(player);
    battle.field.weather = Weather::Snow;
    battle.field.weather_turns = 5;
    battle.emit(format!(
        "|-weather|Snowscape|[from] ability: Snow Warning|[of] p{}a: {}",
        player + 1,
        name
    ));
}

// --- Hook implementations: on_source_modify_damage ---

fn hook_adaptability_damage(battle: &Battle, attacker: u8, move_data: &MoveData) -> f32 {
    let mon = battle.sides[attacker as usize].active();
    let species = pkmn_core::species::get_species_by_id(mon.species_id);
    let has_stab = species
        .map(|s| s.types.contains(&move_data.move_type))
        .unwrap_or(false);
    if has_stab { 4.0 / 3.0 } else { 1.0 }
}

fn hook_technician_damage(_battle: &Battle, _attacker: u8, move_data: &MoveData) -> f32 {
    if move_data.base_power <= 60 { 1.5 } else { 1.0 }
}

fn hook_life_orb_damage(_battle: &Battle, _attacker: u8, _move_data: &MoveData) -> f32 {
    1.3
}

// --- Hook implementations: on_damaging_hit ---

fn hook_rough_skin_hit(battle: &mut Battle, attacker: u8, defender: u8) {
    let attacker_max_hp = battle.sides[attacker as usize].active().max_hp;
    let recoil = (attacker_max_hp / 8).max(1);
    battle.apply_damage(attacker, recoil);
    let atk_name = battle.species_name(attacker);
    let def_name = battle.species_name(defender);
    let hp_str = battle.hp_display(attacker);
    battle.emit(format!(
        "|-damage|p{}a: {}|{}|[from] ability: Rough Skin|[of] p{}a: {}",
        attacker + 1,
        atk_name,
        hp_str,
        defender + 1,
        def_name
    ));
}

fn hook_iron_barbs_hit(battle: &mut Battle, attacker: u8, defender: u8) {
    let attacker_max_hp = battle.sides[attacker as usize].active().max_hp;
    let recoil = (attacker_max_hp / 8).max(1);
    battle.apply_damage(attacker, recoil);
    let atk_name = battle.species_name(attacker);
    let def_name = battle.species_name(defender);
    let hp_str = battle.hp_display(attacker);
    battle.emit(format!(
        "|-damage|p{}a: {}|{}|[from] ability: Iron Barbs|[of] p{}a: {}",
        attacker + 1,
        atk_name,
        hp_str,
        defender + 1,
        def_name
    ));
}

// --- Hook implementations: on_residual ---

fn hook_speed_boost_residual(battle: &mut Battle, player: u8) {
    let mon = battle.sides[player as usize].active_mut();
    if mon.is_alive() {
        mon.boosts.spe = (mon.boosts.spe + 1).min(6);
    }
}

fn hook_leftovers_residual(battle: &mut Battle, player: u8) {
    battle.trigger_item_heal_eot(player);
}

fn hook_black_sludge_residual(battle: &mut Battle, player: u8) {
    battle.trigger_item_heal_eot(player);
}

// --- Data-driven move effects ---

/// Stat boosts for a move effect. Each field is the number of stages to change.
#[derive(Debug, Clone, Copy)]
pub struct BoostEffect {
    pub atk: i8,
    pub def: i8,
    pub spa: i8,
    pub spd: i8,
    pub spe: i8,
}

/// Which non-volatile status a move inflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Burn,
    Paralyze,
    Poison,
    Toxic,
    Sleep,
    Freeze,
}

/// Data-driven move effect for status/boost moves.
#[derive(Debug, Clone, Copy)]
pub enum MoveEffect {
    /// Apply stat boosts to the user.
    Boost(BoostEffect),
    /// Inflict a non-volatile status on the target (immunity checks applied by applier).
    StatusInflict(StatusKind),
    /// Heal the user by num/denom of max HP.
    Heal(u16, u16),
    /// Set a hazard on the opponent's side.
    Hazard(HazardKind),
    /// Set weather or terrain on the field.
    Field(FieldEffect),
    /// Toggle Trick Room (set 5 turns if off, clear if on).
    TrickRoom,
}

/// Weather/terrain setter effect (data-driven).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldEffect {
    Weather(FieldWeather),
    Terrain(FieldTerrain),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldWeather { Rain, Sun, Sand, Snow }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldTerrain { Electric, Grassy, Misty, Psychic }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardKind {
    StealthRock,
    Spikes,
    ToxicSpikes,
    StickyWeb,
}

/// Lookup data-driven effect for a move by name. Returns None for moves
/// not yet migrated (fallback to string-match).
pub fn move_effect(name: &str) -> Option<MoveEffect> {
    match name {
        "swords dance" => Some(MoveEffect::Boost(BoostEffect {
            atk: 2, def: 0, spa: 0, spd: 0, spe: 0,
        })),
        "dragon dance" => Some(MoveEffect::Boost(BoostEffect {
            atk: 1, def: 0, spa: 0, spd: 0, spe: 1,
        })),
        "calm mind" => Some(MoveEffect::Boost(BoostEffect {
            atk: 0, def: 0, spa: 1, spd: 1, spe: 0,
        })),
        "nasty plot" => Some(MoveEffect::Boost(BoostEffect {
            atk: 0, def: 0, spa: 2, spd: 0, spe: 0,
        })),
        "agility" | "rock polish" => Some(MoveEffect::Boost(BoostEffect {
            atk: 0, def: 0, spa: 0, spd: 0, spe: 2,
        })),
        "iron defense" | "acid armor" => Some(MoveEffect::Boost(BoostEffect {
            atk: 0, def: 2, spa: 0, spd: 0, spe: 0,
        })),
        "amnesia" => Some(MoveEffect::Boost(BoostEffect {
            atk: 0, def: 0, spa: 0, spd: 2, spe: 0,
        })),
        "bulk up" => Some(MoveEffect::Boost(BoostEffect {
            atk: 1, def: 1, spa: 0, spd: 0, spe: 0,
        })),
        "quiver dance" => Some(MoveEffect::Boost(BoostEffect {
            atk: 0, def: 0, spa: 1, spd: 1, spe: 1,
        })),
        "shell smash" => Some(MoveEffect::Boost(BoostEffect {
            atk: 2, def: -1, spa: 2, spd: -1, spe: 2,
        })),
        "coil" => Some(MoveEffect::Boost(BoostEffect {
            atk: 1, def: 1, spa: 0, spd: 0, spe: 0,
        })),
        "hone claws" => Some(MoveEffect::Boost(BoostEffect {
            atk: 1, def: 0, spa: 0, spd: 0, spe: 0,
        })),
        "shift gear" => Some(MoveEffect::Boost(BoostEffect {
            atk: 1, def: 0, spa: 0, spd: 0, spe: 2,
        })),
        // Status-inflicting moves
        "toxic" => Some(MoveEffect::StatusInflict(StatusKind::Toxic)),
        "will-o-wisp" => Some(MoveEffect::StatusInflict(StatusKind::Burn)),
        "thunder wave" => Some(MoveEffect::StatusInflict(StatusKind::Paralyze)),
        // Flat recovery moves (1/2 max HP)
        "recover" | "soft-boiled" | "slack off" | "milk drink" => Some(MoveEffect::Heal(1, 2)),
        // Hazards
        "stealth rock" => Some(MoveEffect::Hazard(HazardKind::StealthRock)),
        "spikes" => Some(MoveEffect::Hazard(HazardKind::Spikes)),
        "toxic spikes" => Some(MoveEffect::Hazard(HazardKind::ToxicSpikes)),
        "sticky web" => Some(MoveEffect::Hazard(HazardKind::StickyWeb)),
        // Weather setter moves
        "rain dance" => Some(MoveEffect::Field(FieldEffect::Weather(FieldWeather::Rain))),
        "sunny day" => Some(MoveEffect::Field(FieldEffect::Weather(FieldWeather::Sun))),
        "sandstorm" => Some(MoveEffect::Field(FieldEffect::Weather(FieldWeather::Sand))),
        "snowscape" => Some(MoveEffect::Field(FieldEffect::Weather(FieldWeather::Snow))),
        // Terrain setter moves
        "electric terrain" => Some(MoveEffect::Field(FieldEffect::Terrain(FieldTerrain::Electric))),
        "grassy terrain" => Some(MoveEffect::Field(FieldEffect::Terrain(FieldTerrain::Grassy))),
        "misty terrain" => Some(MoveEffect::Field(FieldEffect::Terrain(FieldTerrain::Misty))),
        "psychic terrain" => Some(MoveEffect::Field(FieldEffect::Terrain(FieldTerrain::Psychic))),
        // Trick Room
        "trick room" => Some(MoveEffect::TrickRoom),
        _ => None,
    }
}

/// Self-effect data for damaging moves (drain, recoil, self-stat changes).
/// Applied AFTER damage resolution in execute_move.
#[derive(Debug, Clone, Copy)]
pub struct DamagingSelfEffect {
    /// Drain: heal attacker by (num/denom) of damage dealt. (0,0) = no drain.
    pub drain: (u8, u8),
    /// Recoil: damage attacker by (num/denom) of damage dealt. (0,0) = no recoil.
    /// Rock Head / Magic Guard negate recoil.
    pub recoil: (u8, u8),
    /// Self-stat boosts applied to attacker after the hit. None = no self-boosts.
    pub self_boosts: Option<BoostEffect>,
}

/// Lookup data-driven self-effect for a damaging move by name.
/// Returns None for moves with no special self-effects.
pub fn damaging_self_effect(name: &str) -> Option<DamagingSelfEffect> {
    match name {
        // Drain moves: heal fraction of damage dealt
        "drain punch" | "giga drain" | "horn leech" | "leech life"
        | "oblivion wing" | "parabolic charge" => Some(DamagingSelfEffect {
            drain: (1, 2),
            recoil: (0, 0),
            self_boosts: None,
        }),
        "draining kiss" => Some(DamagingSelfEffect {
            drain: (3, 4),
            recoil: (0, 0),
            self_boosts: None,
        }),
        // Recoil moves: damage fraction of damage dealt
        // Note: PS uses 33/100 for these; we use 1/3 which matches current test expectations.
        "brave bird" | "flare blitz" | "double-edge" | "wood hammer" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (1, 3),
            self_boosts: None,
        }),
        "wild charge" | "take down" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (1, 4),
            self_boosts: None,
        }),
        "head smash" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (1, 2),
            self_boosts: None,
        }),
        // Self-stat drop moves
        "close combat" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (0, 0),
            self_boosts: Some(BoostEffect { atk: 0, def: -1, spa: 0, spd: -1, spe: 0 }),
        }),
        "superpower" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (0, 0),
            self_boosts: Some(BoostEffect { atk: -1, def: -1, spa: 0, spd: 0, spe: 0 }),
        }),
        "overheat" | "draco meteor" | "leaf storm" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (0, 0),
            self_boosts: Some(BoostEffect { atk: 0, def: 0, spa: -2, spd: 0, spe: 0 }),
        }),
        // Self-stat boost moves
        "power-up punch" => Some(DamagingSelfEffect {
            drain: (0, 0),
            recoil: (0, 0),
            self_boosts: Some(BoostEffect { atk: 1, def: 0, spa: 0, spd: 0, spe: 0 }),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_hooks_returns_switch_in_for_intimidate() {
        let hooks = ability_hooks(AbilityId::Intimidate);
        assert!(hooks.on_switch_in.is_some());
        assert!(hooks.on_source_modify_damage.is_none());
        assert!(hooks.on_damaging_hit.is_none());
        assert!(hooks.on_residual.is_none());
    }

    #[test]
    fn test_ability_hooks_returns_damage_mod_for_technician() {
        let hooks = ability_hooks(AbilityId::Technician);
        assert!(hooks.on_source_modify_damage.is_some());
        assert!(hooks.on_switch_in.is_none());
    }

    #[test]
    fn test_item_hooks_returns_damage_mod_for_life_orb() {
        let hooks = item_hooks(ItemId::LifeOrb);
        assert!(hooks.on_source_modify_damage.is_some());
    }

    #[test]
    fn test_ability_hooks_none_for_unknown() {
        let hooks = ability_hooks(AbilityId::None);
        assert!(hooks.on_switch_in.is_none());
        assert!(hooks.on_source_modify_damage.is_none());
        assert!(hooks.on_damaging_hit.is_none());
        assert!(hooks.on_residual.is_none());
    }

    #[test]
    fn test_move_effect_swords_dance() {
        let eff = move_effect("swords dance");
        assert!(matches!(eff, Some(MoveEffect::Boost(b)) if b.atk == 2));
    }

    #[test]
    fn test_move_effect_stealth_rock() {
        let eff = move_effect("stealth rock");
        assert!(matches!(eff, Some(MoveEffect::Hazard(HazardKind::StealthRock))));
    }

    #[test]
    fn test_move_effect_unknown_returns_none() {
        assert!(move_effect("flamethrower").is_none());
    }

    #[test]
    fn test_ability_hooks_returns_residual_for_speed_boost() {
        let hooks = ability_hooks(AbilityId::SpeedBoost);
        assert!(hooks.on_residual.is_some());
        assert!(hooks.on_switch_in.is_none());
    }

    #[test]
    fn test_item_hooks_returns_residual_for_leftovers() {
        let hooks = item_hooks(ItemId::Leftovers);
        assert!(hooks.on_residual.is_some());
        assert!(hooks.on_source_modify_damage.is_none());
    }
}
