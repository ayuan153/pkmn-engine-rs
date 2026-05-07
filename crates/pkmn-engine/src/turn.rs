use crate::battle::Battle;
use crate::choice::{BattleResult, Choice};
use crate::field::{Terrain, Weather};
use crate::pokemon::Status;
use pkmn_core::moves::{MoveCategory, MoveData};
use pkmn_core::types::Type;

impl Battle {
    pub fn execute_choice(&mut self, player: u8, choice: Choice) {
        match choice {
            Choice::Switch(target) => self.execute_switch(player, target),
            Choice::Move(idx) => self.execute_move(player, idx),
        }
    }

    fn execute_switch(&mut self, player: u8, target: u8) {
        self.sides[player as usize].active_index = target as usize;
        if !self.has_heavy_duty_boots(player) {
            self.apply_entry_hazards(player);
        }
        self.trigger_ability_on_switch(player);
    }

    fn execute_move(&mut self, player: u8, move_idx: u8) {
        let defender_idx = 1 - player;

        if !self.sides[player as usize].active().is_alive() {
            return;
        }

        let move_id = self.sides[player as usize].active().moves[move_idx as usize].move_id;
        let move_data = match pkmn_core::moves::get_move_by_id(move_id) {
            Some(m) => *m,
            None => return,
        };

        // Deduct PP
        self.sides[player as usize].active_mut().moves[move_idx as usize].pp = self.sides
            [player as usize]
            .active_mut()
            .moves[move_idx as usize]
            .pp
            .saturating_sub(1);

        // Accuracy check
        if move_data.accuracy > 0 && !self.rand_check(move_data.accuracy) {
            return;
        }

        // Status moves
        if move_data.category == MoveCategory::Status {
            self.apply_status_move(defender_idx, &move_data);
            return;
        }

        // Check ability immunity
        if self.check_ability_immunity(defender_idx, move_data.move_type) {
            return;
        }

        // Calculate and apply damage
        let damage = self.calculate_move_damage(player, defender_idx, &move_data);
        let damage = self.check_focus_sash(defender_idx, damage);
        self.apply_damage(defender_idx, damage);

        // Life Orb recoil
        self.apply_life_orb_recoil(player);
    }

    fn calculate_move_damage(
        &mut self,
        attacker_player: u8,
        defender_player: u8,
        move_data: &MoveData,
    ) -> u16 {
        let attacker = self.sides[attacker_player as usize].active();
        let defender = self.sides[defender_player as usize].active();

        let (atk_stat, def_stat) = match move_data.category {
            MoveCategory::Physical => (attacker.effective_atk(), defender.effective_def()),
            MoveCategory::Special => (attacker.effective_spa(), defender.effective_spd()),
            _ => return 0,
        };

        let stab = attacker.types.contains(&move_data.move_type);
        let effectiveness = Type::effectiveness(move_data.move_type, &defender.types);
        let attacker_level = attacker.level;
        let attacker_status = attacker.status;

        if effectiveness == 0.0 {
            return 0;
        }

        let critical = self.rand_check(4);
        let random_factor = self.rand_range(85, 100);
        let weather_boost = self.get_weather_modifier(move_data.move_type);

        let burn_mod =
            if attacker_status == Status::Burn && move_data.category == MoveCategory::Physical {
                0.5
            } else {
                1.0
            };

        // Ability and item damage modifiers
        let ability_mod = self.ability_damage_modifier(attacker_player, move_data);
        let item_mod = self.item_damage_modifier(attacker_player, move_data);

        let ctx = pkmn_core::damage::DamageContext {
            attacker_level,
            attacker_stat: atk_stat,
            defender_stat: def_stat,
            base_power: move_data.base_power as u16,
            stab,
            type_effectiveness: effectiveness,
            critical,
            weather_boost,
            other_modifiers: burn_mod * ability_mod * item_mod,
            random_factor,
        };

        pkmn_core::damage::calculate_damage(&ctx)
    }

    fn apply_damage(&mut self, player: u8, damage: u16) {
        let mon = self.sides[player as usize].active_mut();
        mon.hp = mon.hp.saturating_sub(damage);
        if mon.hp == 0 {
            mon.is_fainted = true;
        }
    }

    fn get_weather_modifier(&self, move_type: Type) -> f32 {
        match (self.field.weather, move_type) {
            (Weather::Sun, Type::Fire) => 1.5,
            (Weather::Sun, Type::Water) => 0.5,
            (Weather::Rain, Type::Water) => 1.5,
            (Weather::Rain, Type::Fire) => 0.5,
            _ => 1.0,
        }
    }

    fn apply_status_move(&mut self, defender: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();
        let def_mon = self.sides[defender as usize].active_mut();
        match name.as_str() {
            "toxic" => {
                if def_mon.status == Status::None {
                    def_mon.status = Status::Toxic;
                }
            }
            "will-o-wisp" => {
                if def_mon.status == Status::None {
                    def_mon.status = Status::Burn;
                }
            }
            "thunder wave" => {
                if def_mon.status == Status::None {
                    def_mon.status = Status::Paralyze;
                }
            }
            _ => {}
        }
    }

    pub fn end_of_turn(&mut self) {
        for player in 0..2u8 {
            // Ability end-of-turn effects
            self.trigger_ability_end_of_turn(player);
            // Item end-of-turn effects
            self.trigger_item_end_of_turn(player);
        }

        for player in 0..2 {
            let weather = self.field.weather;
            let mon = self.sides[player].active_mut();
            if !mon.is_alive() {
                continue;
            }

            // Sandstorm damage
            if weather == Weather::Sand
                && !mon.types.contains(&Type::Rock)
                && !mon.types.contains(&Type::Ground)
                && !mon.types.contains(&Type::Steel)
            {
                let dmg = (mon.max_hp / 16).max(1);
                mon.hp = mon.hp.saturating_sub(dmg);
            }

            // Status damage
            match mon.status {
                Status::Burn => {
                    let dmg = (mon.max_hp / 16).max(1);
                    mon.hp = mon.hp.saturating_sub(dmg);
                }
                Status::Poison => {
                    let dmg = (mon.max_hp / 8).max(1);
                    mon.hp = mon.hp.saturating_sub(dmg);
                }
                Status::Toxic => {
                    mon.status_turns += 1;
                    let dmg = (mon.max_hp * mon.status_turns as u16 / 16).max(1);
                    mon.hp = mon.hp.saturating_sub(dmg);
                }
                _ => {}
            }

            if mon.hp == 0 {
                mon.is_fainted = true;
            }
        }

        // Decrement field turns
        if self.field.weather_turns > 0 {
            self.field.weather_turns -= 1;
            if self.field.weather_turns == 0 {
                self.field.weather = Weather::None;
            }
        }
        if self.field.terrain_turns > 0 {
            self.field.terrain_turns -= 1;
            if self.field.terrain_turns == 0 {
                self.field.terrain = Terrain::None;
            }
        }
        if self.field.trick_room > 0 {
            self.field.trick_room -= 1;
        }

        for side in &mut self.sides {
            if side.side_conditions.reflect > 0 {
                side.side_conditions.reflect -= 1;
            }
            if side.side_conditions.light_screen > 0 {
                side.side_conditions.light_screen -= 1;
            }
            if side.side_conditions.tailwind > 0 {
                side.side_conditions.tailwind -= 1;
            }
        }
    }

    pub fn check_win(&mut self) {
        let p1_alive = self.sides[0].alive_count();
        let p2_alive = self.sides[1].alive_count();

        if p1_alive == 0 && p2_alive == 0 {
            self.result = BattleResult::Tie;
        } else if p1_alive == 0 {
            self.result = BattleResult::Win(1);
        } else if p2_alive == 0 {
            self.result = BattleResult::Win(0);
        }
    }
}
