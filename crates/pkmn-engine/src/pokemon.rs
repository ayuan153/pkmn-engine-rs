use pkmn_core::nature::Nature;
use pkmn_core::species::SpeciesData;
use pkmn_core::stats;
use pkmn_core::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    None = 0,
    Burn,
    Paralyze,
    Sleep,
    Poison,
    Toxic,
    Freeze,
}

#[derive(Debug, Clone, Copy)]
pub struct Boosts {
    pub atk: i8,
    pub def: i8,
    pub spa: i8,
    pub spd: i8,
    pub spe: i8,
    pub accuracy: i8,
    pub evasion: i8,
}

impl Boosts {
    pub fn new() -> Self {
        Self { atk: 0, def: 0, spa: 0, spd: 0, spe: 0, accuracy: 0, evasion: 0 }
    }

    /// Apply boost multiplier: stages -6 to +6
    pub fn multiplier(stage: i8) -> f32 {
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
}

#[derive(Debug, Clone, Copy)]
pub struct MoveSlot {
    pub move_id: u16,
    pub pp: u8,
    pub max_pp: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub spa: u16,
    pub spd: u16,
    pub spe: u16,
}

#[derive(Debug, Clone)]
pub struct Pokemon {
    pub species_id: u16,
    pub level: u8,
    pub hp: u16,
    pub max_hp: u16,
    pub status: Status,
    pub status_turns: u8,
    pub boosts: Boosts,
    pub moves: [MoveSlot; 4],
    pub ability_id: u16,
    pub item_id: u16,
    pub types: [Type; 2],
    pub stats: Stats,
    pub nature: Nature,
    pub is_fainted: bool,
    pub has_moved_this_turn: bool,
}

impl Pokemon {
    pub fn new(
        species: &SpeciesData,
        level: u8,
        nature: Nature,
        moves: [MoveSlot; 4],
        evs: [u8; 6],
        ivs: [u8; 6],
    ) -> Self {
        let (atk_mod, def_mod, spa_mod, spd_mod, spe_mod) = nature.modifiers();
        let hp = stats::calc_hp(species.base_stats.hp, ivs[0], evs[0], level);
        let computed = Stats {
            hp,
            atk: stats::calc_stat(species.base_stats.atk, ivs[1], evs[1], level, atk_mod),
            def: stats::calc_stat(species.base_stats.def, ivs[2], evs[2], level, def_mod),
            spa: stats::calc_stat(species.base_stats.spa, ivs[3], evs[3], level, spa_mod),
            spd: stats::calc_stat(species.base_stats.spd, ivs[4], evs[4], level, spd_mod),
            spe: stats::calc_stat(species.base_stats.spe, ivs[5], evs[5], level, spe_mod),
        };
        Self {
            species_id: species.id,
            level,
            hp,
            max_hp: hp,
            status: Status::None,
            status_turns: 0,
            boosts: Boosts::new(),
            moves,
            ability_id: 0,
            item_id: 0,
            types: species.types,
            stats: computed,
            nature,
            is_fainted: false,
            has_moved_this_turn: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.is_fainted && self.hp > 0
    }

    pub fn effective_speed(&self) -> u16 {
        let base = self.stats.spe as f32 * Boosts::multiplier(self.boosts.spe);
        let base = if self.status == Status::Paralyze { base * 0.5 } else { base };
        base as u16
    }

    pub fn effective_atk(&self) -> u16 {
        (self.stats.atk as f32 * Boosts::multiplier(self.boosts.atk)) as u16
    }

    pub fn effective_def(&self) -> u16 {
        (self.stats.def as f32 * Boosts::multiplier(self.boosts.def)) as u16
    }

    pub fn effective_spa(&self) -> u16 {
        (self.stats.spa as f32 * Boosts::multiplier(self.boosts.spa)) as u16
    }

    pub fn effective_spd(&self) -> u16 {
        (self.stats.spd as f32 * Boosts::multiplier(self.boosts.spd)) as u16
    }
}
