use crate::choice::{BattleResult, Choice};
use crate::field::Field;
use crate::pokemon::{MoveSlot, Pokemon, Status, Volatiles};
use crate::side::Side;
use pkmn_core::nature::Nature;
use pkmn_core::species::get_species;

/// Strip form suffix for display name (PS uses base name as nickname).
/// "Rotom-Wash" → "Rotom", "Landorus-Therian" → "Landorus"
/// Exception: names where hyphen is part of the base (Ho-Oh, Porygon-Z, etc.)
fn display_name(species: &'static str) -> &'static str {
    const HYPHENATED_BASE: &[&str] = &[
        "Ho-Oh", "Porygon-Z", "Jangmo-o", "Hakamo-o", "Kommo-o",
        "Wo-Chien", "Chien-Pao", "Ting-Lu", "Chi-Yu",
        "Type: Null", "Mr. Mime", "Mime Jr.", "Nidoran-F", "Nidoran-M",
    ];
    if HYPHENATED_BASE.contains(&species) {
        return species;
    }
    match species.find('-') {
        Some(pos) => &species[..pos],
        None => species,
    }
}

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
    pub protocol: Vec<String>,
    rng_seed: [u16; 4],
    pub rng_call_count: u32,
    /// Queued pivot switch targets per player (set externally by test runner)
    pub pivot_switch_targets: [Vec<u8>; 2],
    /// Last player who executed an attack (for simultaneous faint resolution)
    pub last_attacker: Option<u8>,
}

impl Battle {
    pub fn new(side1: Side, side2: Side, seed: [u16; 4]) -> Self {
        let mut battle = Self {
            sides: [side1, side2],
            field: Field::default(),
            turn: 0,
            result: BattleResult::Ongoing,
            phase: BattlePhase::ActionSelection,
            protocol: Vec::new(),
            rng_seed: seed,
            rng_call_count: 0,
            pivot_switch_targets: [Vec::new(), Vec::new()],
            last_attacker: None,
        };
        // PS consumes 1 RNG call per Pokemon with random gender during init
        // (battle.sample(['M', 'F']) in Pokemon constructor)
        // Species with fixed gender (genderless or single-gender) don't consume RNG
        // Must consume for ALL team members, not just leads
        let mut gender_calls = 0u32;
        for side in &battle.sides {
            for mon in &side.team {
                if !Self::has_fixed_gender(mon.species_id) {
                    gender_calls += 1;
                }
            }
        }
        for _ in 0..gender_calls {
            battle.rand();
        }
        // Emit switch-in for leads
        for p in 0..2u8 {
            let name = battle.species_name(p);
            let full_name = battle.full_species_name(p);
            let mon = battle.sides[p as usize].active();
            let hp = mon.hp;
            let max_hp = mon.max_hp;
            let level = mon.level;
            let level_str = if level == 100 { String::new() } else { format!(", L{}", level) };
            battle.emit(format!("|switch|p{}a: {}|{}{}|{}/{}", p+1, name, full_name, level_str, hp, max_hp));
        }
        // Trigger abilities for starting leads (weather, terrain, Intimidate)
        // Faster Pokemon's ability triggers first (PS behavior)
        let p1_speed = battle.sides[0].active().effective_speed();
        let p2_speed = battle.sides[1].active().effective_speed();
        if p2_speed > p1_speed {
            battle.trigger_ability_on_switch(1);
            battle.trigger_ability_on_switch(0);
        } else {
            battle.trigger_ability_on_switch(0);
            battle.trigger_ability_on_switch(1);
        }
        // Emit first turn marker
        battle.turn = 1;
        battle.emit(format!("|turn|1"));
        battle
    }

    pub(crate) fn emit(&mut self, event: String) {
        self.protocol.push(event);
    }

    /// Format HP display with status suffix (e.g. "250/414 tox")
    pub(crate) fn hp_display(&self, player: u8) -> String {
        let mon = self.sides[player as usize].active();
        let hp = mon.hp;
        let max_hp = mon.max_hp;
        if hp == 0 {
            return "0 fnt".to_string();
        }
        let status_str = match mon.status {
            Status::Toxic => " tox",
            Status::Burn => " brn",
            Status::Poison => " psn",
            Status::Paralyze => " par",
            _ => "",
        };
        format!("{}/{}{}", hp, max_hp, status_str)
    }

    pub fn drain_protocol(&mut self) -> Vec<String> {
        std::mem::take(&mut self.protocol)
    }

    pub(crate) fn species_name(&self, player: u8) -> &'static str {
        let id = self.sides[player as usize].active().species_id;
        let full_name = pkmn_core::species::get_species_by_id(id)
            .map(|s| s.name)
            .unwrap_or("Unknown");
        display_name(full_name)
    }

    /// Returns true if a species has a fixed gender (genderless or single-gender)
    /// These species don't consume RNG for gender assignment in PS
    fn has_fixed_gender(species_id: u16) -> bool {
        matches!(species_id,
            // Genderless
            81 | 82 | 100 | 101 | 120 | 121 | 137 | 233 | 292 | 337 | 338 |
            343 | 344 | 374 | 375 | 376 | 436 | 437 | 462 | 474 | 479 |
            599 | 600 | 601 | 615 | 622 | 623 | 649 | 703 | 774 | 781 |
            785 | 786 | 787 | 788 | // Tapu Koko/Lele/Bulu/Fini
            854 | 855 | 870 | 874 | 875 | 880 | 881 | 882 | 883 |
            // Always female
            113 | 115 | 124 | 238 | 241 | 242 | 314 | 380 | 413 | 416 |
            440 | 478 | 488 | 548 | 549 | 629 | 630 | 669 | 670 | 671 |
            761 | 762 | 763 | 856 | 857 | 858 |
            // Always male
            106 | 107 | 128 | 236 | 237 | 313 | 381 | 414 | 475 | 538 |
            539 | 627 | 628 | 641 | 642 | 645 | // Landorus (all forms)
            861 | // Grimmsnarl
            905
        )
    }

    pub(crate) fn full_species_name(&self, player: u8) -> &'static str {
        let id = self.sides[player as usize].active().species_id;
        pkmn_core::species::get_species_by_id(id)
            .map(|s| s.name)
            .unwrap_or("Unknown")
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
        // Emit switch line
        let name = self.species_name(player);
        let full_name = self.full_species_name(player);
        let mon = self.sides[player as usize].active();
        let hp = mon.hp;
        let max_hp = mon.max_hp;
        let level = mon.level;
        let level_str = if level == 100 { String::new() } else { format!(", L{}", level) };
        self.emit(format!("|switch|p{}a: {}|{}{}|{}/{}", player+1, name, full_name, level_str, hp, max_hp));
        if !self.has_heavy_duty_boots(player) {
            self.apply_entry_hazards(player);
        }
        self.trigger_ability_on_switch(player);
        self.phase = BattlePhase::ActionSelection;
        // Emit turn marker after forced switch
        self.turn += 1;
        self.emit(format!("|turn|{}", self.turn));
        self.result
    }

    /// Apply both players' choices and advance one turn
    pub fn apply(&mut self, p1_choice: Choice, p2_choice: Choice) -> BattleResult {
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
            // Check win after each action (a faint with no backup ends the game)
            self.check_win();
        }

        if self.result == BattleResult::Ongoing {
            self.end_of_turn();
            // Check if someone fainted during EOT (Leech Seed, burn, etc.)
            self.check_win();
            if self.result == BattleResult::Ongoing {
                self.emit("|upkeep".to_string());
            }
        }

        if let BattleResult::Win(winner) = self.result {
            self.emit(format!("|win|Player {}", winner + 1));
        }

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
            // Emit next turn marker (PS emits this after upkeep)
            self.turn += 1;
            self.emit(format!("|turn|{}", self.turn));
        }

        self.result
    }

    /// PS-compatible Gen5 LCG PRNG
    /// x_{n+1} = (a * x_n + c) mod 2^64
    /// a = 0x5D588B656C078965, c = 0x00269EC3
    /// Returns upper 32 bits as output
    pub(crate) fn rand(&mut self) -> u32 {
        self.rng_call_count += 1;
        let a: [u64; 4] = [0x5D58, 0x8B65, 0x6C07, 0x8965];
        let c: [u64; 4] = [0, 0, 0x0026, 0x9EC3];
        let seed = [
            self.rng_seed[0] as u64,
            self.rng_seed[1] as u64,
            self.rng_seed[2] as u64,
            self.rng_seed[3] as u64,
        ];
        let mut out = [0u16; 4];
        let mut carry: u64 = 0;
        for out_index in (0..4i32).rev() {
            for b_index in out_index..4 {
                let a_index = 3 - (b_index - out_index);
                carry += seed[a_index as usize] * a[b_index as usize];
            }
            carry += c[out_index as usize];
            out[out_index as usize] = (carry & 0xFFFF) as u16;
            carry >>= 16;
        }
        self.rng_seed = out;
        let result = ((self.rng_seed[0] as u32) << 16) + self.rng_seed[1] as u32;
        result
    }

    /// Random number in [from, to) — matches PS's random(from, to)
    #[allow(dead_code)]
    pub(crate) fn rand_range(&mut self, min: u8, max: u8) -> u8 {
        let result = self.rand();
        let range = (max - min + 1) as u32;
        // PS: Math.floor(result * range / 2^32) + from
        ((result as u64 * range as u64 / (1u64 << 32)) as u8) + min
    }

    /// Random check: returns true with probability numerator/denominator
    /// Matches PS's randomChance(numerator, denominator)
    pub(crate) fn rand_check(&mut self, percent: u8) -> bool {
        // PS: randomChance(percent, 100) => random(100) < percent
        let result = self.rand();
        let roll = (result as u64 * 100 / (1u64 << 32)) as u8;
        roll < percent
    }

    /// PS-compatible randomChance(numerator, denominator)
    pub(crate) fn random_chance(&mut self, numerator: u32, denominator: u32) -> bool {
        let result = self.rand();
        let roll = (result as u64 * denominator as u64 / (1u64 << 32)) as u32;
        roll < numerator
    }

    /// PS-compatible random(n): returns value in [0, n)
    pub(crate) fn random(&mut self, n: u32) -> u32 {
        let result = self.rand();
        (result as u64 * n as u64 / (1u64 << 32)) as u32
    }

    /// Create a 6v6 battle with common competitive Pokemon for benchmarking/testing
    pub fn default_test_battle(seed: [u16; 4]) -> Self {
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
