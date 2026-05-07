use crate::battle::Battle;
use crate::choice::{BattleResult, Choice};
use crate::field::{Terrain, Weather};
use crate::pokemon::{Status, Volatiles};
use pkmn_core::moves::{MoveCategory, MoveData};
use pkmn_core::types::Type;

impl Battle {
    pub fn execute_choice(&mut self, player: u8, choice: Choice) {
        match choice {
            Choice::Switch(target) => self.execute_switch(player, target),
            Choice::Move(idx) => self.execute_move(player, idx),
            Choice::Tera(idx) => {
                self.apply_tera(player);
                self.execute_move(player, idx);
            }
        }
    }

    pub fn apply_tera(&mut self, player: u8) {
        let side = &mut self.sides[player as usize];
        if !side.tera_used {
            let mon = side.active_mut();
            if let Some(tera_type) = mon.tera_type {
                mon.is_terastallized = true;
                mon.types = [tera_type, tera_type];
                side.tera_used = true;
            }
        }
    }

    fn execute_switch(&mut self, player: u8, target: u8) {
        // Clear volatiles on switch
        let mon = self.sides[player as usize].active_mut();
        mon.volatiles = Volatiles::empty();
        mon.locked_move_turns = 0;
        mon.confusion_turns = 0;
        mon.protect_consecutive = 0;

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

        // Must recharge: skip turn and clear
        if self.sides[player as usize]
            .active()
            .volatiles
            .contains(Volatiles::MUST_RECHARGE)
        {
            self.sides[player as usize]
                .active_mut()
                .volatiles
                .remove(Volatiles::MUST_RECHARGE);
            return;
        }

        // Flinch check
        if self.sides[player as usize]
            .active()
            .volatiles
            .contains(Volatiles::FLINCH)
        {
            return;
        }

        // Confusion self-hit: 33% chance
        if self.sides[player as usize]
            .active()
            .volatiles
            .contains(Volatiles::CONFUSED)
        {
            if self.rand_check(33) {
                // Hit self: 40 BP typeless physical
                let atk = self.sides[player as usize].active().effective_atk();
                let def = self.sides[player as usize].active().effective_def();
                let level = self.sides[player as usize].active().level;
                let damage = ((2 * level as u32 / 5 + 2) * 40 * atk as u32 / def as u32 / 50 + 2) as u16;
                self.apply_damage(player, damage);
                return;
            }
            // Decrement confusion
            let mon = self.sides[player as usize].active_mut();
            mon.confusion_turns = mon.confusion_turns.saturating_sub(1);
            if mon.confusion_turns == 0 {
                mon.volatiles.remove(Volatiles::CONFUSED);
            }
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

        // Protect check: blocks all moves targeting the defender
        if self.sides[defender_idx as usize]
            .active()
            .volatiles
            .contains(Volatiles::PROTECT)
            && move_data.category != MoveCategory::Status
        {
            return;
        }

        // Status moves
        if move_data.category == MoveCategory::Status {
            self.apply_status_move(player, defender_idx, &move_data);
            return;
        }

        // Check ability immunity
        if self.check_ability_immunity(defender_idx, move_data.move_type) {
            return;
        }

        // Calculate and apply damage
        let damage = self.calculate_move_damage(player, defender_idx, &move_data);

        // Substitute absorbs damage
        if self.sides[defender_idx as usize]
            .active()
            .volatiles
            .contains(Volatiles::SUBSTITUTE)
        {
            let sub_hp = self.sides[defender_idx as usize].active().substitute_hp;
            if damage >= sub_hp {
                self.sides[defender_idx as usize].active_mut().substitute_hp = 0;
                self.sides[defender_idx as usize]
                    .active_mut()
                    .volatiles
                    .remove(Volatiles::SUBSTITUTE);
            } else {
                self.sides[defender_idx as usize].active_mut().substitute_hp -= damage;
            }
        } else {
            let damage = self.check_focus_sash(defender_idx, damage);
            self.apply_damage(defender_idx, damage);
        }

        // Life Orb recoil
        self.apply_life_orb_recoil(player);

        // Post-damage self-stat drops (Close Combat, etc.)
        self.apply_post_damage_effects(player, &move_data);

        // Multi-turn move handling
        self.handle_multi_turn(player, move_idx, &move_data);

        // Recharge moves
        self.handle_recharge_move(player, &move_data);
    }

    fn handle_multi_turn(&mut self, player: u8, move_idx: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();
        match name.as_str() {
            "outrage" | "petal dance" | "thrash" => {
                let already_locked = self.sides[player as usize]
                    .active()
                    .volatiles
                    .contains(Volatiles::LOCKED_MOVE);
                if !already_locked {
                    let turns = if self.rand_check(50) { 2 } else { 3 };
                    let mon = self.sides[player as usize].active_mut();
                    mon.volatiles.insert(Volatiles::LOCKED_MOVE);
                    mon.locked_move_idx = move_idx;
                    mon.locked_move_turns = turns;
                }
                let mon = self.sides[player as usize].active_mut();
                mon.locked_move_turns = mon.locked_move_turns.saturating_sub(1);
                if mon.locked_move_turns == 0 {
                    mon.volatiles.remove(Volatiles::LOCKED_MOVE);
                    mon.volatiles.insert(Volatiles::CONFUSED);
                    mon.confusion_turns = 2;
                }
            }
            _ => {}
        }
    }

    fn handle_recharge_move(&mut self, player: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();
        match name.as_str() {
            "hyper beam" | "giga impact" | "blast burn" | "frenzy plant" | "hydro cannon" => {
                self.sides[player as usize]
                    .active_mut()
                    .volatiles
                    .insert(Volatiles::MUST_RECHARGE);
            }
            _ => {}
        }
    }

    fn apply_post_damage_effects(&mut self, player: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();
        match name.as_str() {
            "close combat" => {
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.def = (mon.boosts.def - 1).max(-6);
                mon.boosts.spd = (mon.boosts.spd - 1).max(-6);
            }
            "superpower" => {
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk - 1).max(-6);
                mon.boosts.def = (mon.boosts.def - 1).max(-6);
            }
            _ => {}
        }
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

        // STAB calculation with tera
        let stab = if attacker.is_terastallized {
            self.tera_stab(attacker_player, move_data.move_type)
        } else if attacker.types.contains(&move_data.move_type) {
            1.5
        } else {
            1.0
        };

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

        let ability_mod = self.ability_damage_modifier(attacker_player, move_data);
        let item_mod = self.item_damage_modifier(attacker_player, move_data);

        let ctx = pkmn_core::damage::DamageContext {
            attacker_level,
            attacker_stat: atk_stat,
            defender_stat: def_stat,
            base_power: move_data.base_power as u16,
            stab: stab > 1.0,
            type_effectiveness: effectiveness,
            critical,
            weather_boost,
            other_modifiers: burn_mod * ability_mod * item_mod * (stab / if stab > 1.0 { 1.5 } else { 1.0 }),
            random_factor,
        };

        pkmn_core::damage::calculate_damage(&ctx)
    }

    pub fn tera_stab(&self, player: u8, move_type: Type) -> f32 {
        let mon = self.sides[player as usize].active();
        let tera_type = mon.tera_type.unwrap_or(Type::Normal);
        if move_type == tera_type {
            // Check if original types contained this type (stored in species)
            let species = pkmn_core::species::get_species_by_id(mon.species_id);
            let original_has_type = species
                .map(|s| s.types.contains(&move_type))
                .unwrap_or(false);
            if original_has_type {
                2.25 // Adaptability-like
            } else {
                2.0
            }
        } else {
            1.0
        }
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

    fn apply_status_move(&mut self, attacker: u8, defender: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();

        // Protect check for status moves too
        if self.sides[defender as usize]
            .active()
            .volatiles
            .contains(Volatiles::PROTECT)
        {
            // Protect blocks targeting status moves
            match name.as_str() {
                "toxic" | "will-o-wisp" | "thunder wave" => return,
                _ => {}
            }
        }

        match name.as_str() {
            "toxic" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    def_mon.status = Status::Toxic;
                }
            }
            "will-o-wisp" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    def_mon.status = Status::Burn;
                }
            }
            "thunder wave" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    def_mon.status = Status::Paralyze;
                }
            }
            "protect" | "detect" => {
                let consecutive = self.sides[attacker as usize].active().protect_consecutive;
                if consecutive > 0 && self.rand_check(50) {
                    return;
                }
                let mon = self.sides[attacker as usize].active_mut();
                mon.volatiles.insert(Volatiles::PROTECT);
                mon.protect_consecutive += 1;
                return;
            }
            "substitute" => {
                let mon = self.sides[attacker as usize].active_mut();
                let cost = mon.max_hp / 4;
                if mon.hp > cost && !mon.volatiles.contains(Volatiles::SUBSTITUTE) {
                    mon.hp -= cost;
                    mon.substitute_hp = cost;
                    mon.volatiles.insert(Volatiles::SUBSTITUTE);
                }
                return;
            }
            // Boost moves
            "swords dance" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk + 2).min(6);
            }
            "dragon dance" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk + 1).min(6);
                mon.boosts.spe = (mon.boosts.spe + 1).min(6);
            }
            "calm mind" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                mon.boosts.spd = (mon.boosts.spd + 1).min(6);
            }
            "iron defense" | "acid armor" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.def = (mon.boosts.def + 2).min(6);
            }
            "nasty plot" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 2).min(6);
            }
            "agility" | "rock polish" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spe = (mon.boosts.spe + 2).min(6);
            }
            "quiver dance" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                mon.boosts.spd = (mon.boosts.spd + 1).min(6);
                mon.boosts.spe = (mon.boosts.spe + 1).min(6);
            }
            "shell smash" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk + 2).min(6);
                mon.boosts.spa = (mon.boosts.spa + 2).min(6);
                mon.boosts.spe = (mon.boosts.spe + 2).min(6);
                mon.boosts.def = (mon.boosts.def - 1).max(-6);
                mon.boosts.spd = (mon.boosts.spd - 1).max(-6);
            }
            _ => {}
        }

        // Reset protect consecutive if not using protect
        if name != "protect" && name != "detect" {
            self.sides[attacker as usize].active_mut().protect_consecutive = 0;
        }
    }

    pub fn end_of_turn(&mut self) {
        for player in 0..2u8 {
            self.trigger_ability_end_of_turn(player);
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
