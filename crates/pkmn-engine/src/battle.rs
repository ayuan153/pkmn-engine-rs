use crate::choice::{BattleResult, Choice};
use crate::field::Field;
use crate::pokemon::{MoveSlot, Pokemon, Volatiles};
use crate::side::Side;
use pkmn_core::nature::Nature;
use pkmn_core::species::get_species;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlePhase {
    ActionSelection,
    ForcedSwitch(u8), // Player N must switch (their active fainted)
}

#[derive(Debug, Clone)]
pub struct Battle {
    pub sides: [Side; 2],
    pub field: Field,
    pub turn: u16,
    pub result: BattleResult,
    pub phase: BattlePhase,
    rng_seed: u64,
}

impl Battle {
    pub fn new(side1: Side, side2: Side, seed: u64) -> Self {
        Self {
            sides: [side1, side2],
            field: Field::default(),
            turn: 0,
            result: BattleResult::Ongoing,
            phase: BattlePhase::ActionSelection,
            rng_seed: seed,
        }
    }

    /// Get legal choices for a player (0 or 1)
    pub fn choices(&self, player: u8) -> Vec<Choice> {
        let side = &self.sides[player as usize];

        // Forced switch: only switch options
        if let BattlePhase::ForcedSwitch(p) = self.phase {
            if p == player {
                let mut choices = Vec::new();
                for (i, mon) in side.team.iter().enumerate() {
                    if i != side.active_index && mon.is_alive() {
                        choices.push(Choice::Switch(i as u8));
                    }
                }
                return choices;
            }
        }

        let active = side.active();
        let mut choices = Vec::new();

        // Must recharge: no choices (forced to do nothing)
        if active.volatiles.contains(Volatiles::MUST_RECHARGE) {
            return choices;
        }

        // Locked move: only that move
        if active.volatiles.contains(Volatiles::LOCKED_MOVE) {
            choices.push(Choice::Move(active.locked_move_idx));
            return choices;
        }

        if active.is_alive() {
            for i in 0..4 {
                if active.moves[i].pp > 0 && active.moves[i].move_id != 0 {
                    choices.push(Choice::Move(i as u8));
                }
            }

            // Tera option: if side hasn't used tera and mon has a tera type
            if !side.tera_used && active.tera_type.is_some() && !active.is_terastallized {
                for i in 0..4 {
                    if active.moves[i].pp > 0 && active.moves[i].move_id != 0 {
                        choices.push(Choice::Tera(i as u8));
                    }
                }
            }
        }

        for (i, mon) in side.team.iter().enumerate() {
            if i != side.active_index && mon.is_alive() {
                choices.push(Choice::Switch(i as u8));
            }
        }

        choices
    }

    /// Apply forced switch after faint
    pub fn apply_switch(&mut self, player: u8, target: u8) -> BattleResult {
        self.sides[player as usize].active_index = target as usize;
        if !self.has_heavy_duty_boots(player) {
            self.apply_entry_hazards(player);
        }
        self.trigger_ability_on_switch(player);
        self.phase = BattlePhase::ActionSelection;
        self.result
    }

    /// Apply both players' choices and advance one turn
    pub fn apply(&mut self, p1_choice: Choice, p2_choice: Choice) -> BattleResult {
        self.turn += 1;

        // Clear per-turn volatiles
        for side in &mut self.sides {
            let mon = side.active_mut();
            mon.volatiles.remove(Volatiles::FLINCH);
            mon.volatiles.remove(Volatiles::PROTECT);
        }

        let order = self.determine_order(p1_choice, p2_choice);

        for (player, choice) in order {
            if self.result != BattleResult::Ongoing {
                break;
            }
            self.execute_choice(player, choice);
        }

        if self.result == BattleResult::Ongoing {
            self.end_of_turn();
        }

        self.check_win();

        // Check for forced switch after faint
        if self.result == BattleResult::Ongoing {
            for p in 0..2u8 {
                if !self.sides[p as usize].active().is_alive()
                    && self.sides[p as usize].has_alive_switch()
                {
                    self.phase = BattlePhase::ForcedSwitch(p);
                    return BattleResult::Ongoing;
                }
            }
        }

        self.result
    }

    /// Simple seeded RNG (xorshift64)
    pub(crate) fn rand(&mut self) -> u64 {
        self.rng_seed ^= self.rng_seed << 13;
        self.rng_seed ^= self.rng_seed >> 7;
        self.rng_seed ^= self.rng_seed << 17;
        self.rng_seed
    }

    /// Random u8 in range [min, max] inclusive
    pub(crate) fn rand_range(&mut self, min: u8, max: u8) -> u8 {
        let range = (max - min + 1) as u64;
        (self.rand() % range) as u8 + min
    }

    /// Random check: returns true with probability percent/100
    pub(crate) fn rand_check(&mut self, percent: u8) -> bool {
        self.rand_range(1, 100) <= percent
    }

    /// Create a 6v6 battle with common competitive Pokemon for benchmarking/testing
    pub fn default_test_battle(seed: u64) -> Self {
        let evs_phys = [0, 252, 0, 0, 0, 252u8];
        let evs_spec = [0, 0, 0, 252, 0, 252u8];
        let ivs = [31u8; 6];

        let make = |name: &str, nature: Nature, evs: [u8; 6], moves: [MoveSlot; 4]| -> Pokemon {
            let species = get_species(name).unwrap();
            Pokemon::new(species, 100, nature, moves, evs, ivs)
        };

        let ms = |id: u16, pp: u8| MoveSlot { move_id: id, pp, max_pp: pp };

        let team1 = vec![
            make("Garchomp", Nature::Jolly, evs_phys, [ms(89, 10), ms(370, 5), ms(444, 5), ms(14, 20)]),
            make("Dragapult", Nature::Timid, evs_spec, [ms(247, 15), ms(53, 15), ms(85, 15), ms(434, 5)]),
            make("Kingambit", Nature::Adamant, evs_phys, [ms(282, 20), ms(442, 15), ms(14, 20), ms(89, 10)]),
            make("Heatran", Nature::Modest, evs_spec, [ms(53, 15), ms(126, 5), ms(446, 20), ms(85, 15)]),
            make("Corviknight", Nature::Impish, [252, 0, 252, 0, 0, 0], [ms(413, 15), ms(355, 5), ms(432, 15), ms(369, 20)]),
            make("Breloom", Nature::Jolly, evs_phys, [ms(370, 5), ms(89, 10), ms(14, 20), ms(282, 20)]),
        ];

        let team2 = vec![
            make("Dragonite", Nature::Adamant, evs_phys, [ms(200, 10), ms(89, 10), ms(349, 20), ms(583, 10)]),
            make("Volcarona", Nature::Timid, evs_spec, [ms(53, 15), ms(585, 15), ms(347, 20), ms(355, 5)]),
            make("Great Tusk", Nature::Jolly, evs_phys, [ms(89, 10), ms(370, 5), ms(229, 40), ms(446, 20)]),
            make("Gholdengo", Nature::Timid, evs_spec, [ms(247, 15), ms(85, 15), ms(399, 15), ms(105, 5)]),
            make("Iron Valiant", Nature::Naive, evs_phys, [ms(370, 5), ms(585, 15), ms(282, 20), ms(14, 20)]),
            make("Tyranitar", Nature::Adamant, evs_phys, [ms(444, 5), ms(89, 10), ms(282, 20), ms(349, 20)]),
        ];

        Self::new(Side::new(team1), Side::new(team2), seed)
    }
}
