use crate::choice::{BattleResult, Choice};
use crate::field::Field;
use crate::side::Side;

#[derive(Debug, Clone)]
pub struct Battle {
    pub sides: [Side; 2],
    pub field: Field,
    pub turn: u16,
    pub result: BattleResult,
    rng_seed: u64,
}

impl Battle {
    pub fn new(side1: Side, side2: Side, seed: u64) -> Self {
        Self {
            sides: [side1, side2],
            field: Field::default(),
            turn: 0,
            result: BattleResult::Ongoing,
            rng_seed: seed,
        }
    }

    /// Get legal choices for a player (0 or 1)
    pub fn choices(&self, player: u8) -> Vec<Choice> {
        let side = &self.sides[player as usize];
        let mut choices = Vec::new();

        let active = side.active();
        if active.is_alive() {
            for i in 0..4 {
                if active.moves[i].pp > 0 && active.moves[i].move_id != 0 {
                    choices.push(Choice::Move(i as u8));
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

    /// Apply both players' choices and advance one turn
    pub fn apply(&mut self, p1_choice: Choice, p2_choice: Choice) -> BattleResult {
        self.turn += 1;

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
}
