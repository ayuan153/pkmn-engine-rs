use crate::battle::Battle;
use crate::choice::{BattleResult, Choice};
use crate::field::{Terrain, Weather};
use crate::pokemon::{Status, Volatiles};
use pkmn_core::moves::{MoveCategory, MoveData, MoveFlags};
use pkmn_core::types::Type;

impl Battle {
    pub fn execute_choice(&mut self, player: u8, choice: Choice) {
        match choice {
            Choice::Switch(target) => {
                // If switching to already-active Pokemon, fall back to move 0
                if target as usize == self.sides[player as usize].active_index {
                    self.execute_move(player, 0);
                    return;
                }
                self.execute_switch(player, target);
            }
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
        self.execute_switch_from(player, target, None);
    }

    fn execute_switch_from(&mut self, player: u8, target: u8, from_move: Option<&str>) {
        // Clear volatiles on switch
        let mon = self.sides[player as usize].active_mut();
        mon.volatiles = Volatiles::empty();
        mon.locked_move_turns = 0;
        mon.confusion_turns = 0;
        mon.protect_consecutive = 0;

        self.sides[player as usize].active_index = target as usize;
        let name = self.species_name(player);
        let mon = self.sides[player as usize].active();
        let hp = mon.hp;
        let max_hp = mon.max_hp;
        let level = mon.level;
        let level_str = if level == 100 { String::new() } else { format!(", L{}", level) };
        let from_str = match from_move {
            Some(m) => format!("|[from] {}", m),
            None => String::new(),
        };
        self.emit(format!("|switch|p{}a: {}|{}{}|{}/{}{}", player+1, name, name, level_str, hp, max_hp, from_str));
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

        // Accuracy check (skip for multi-hit moves - they check per hit)
        let is_multi_hit = self.get_multi_hit_count(player, &move_data).is_some();
        if !is_multi_hit && move_data.accuracy > 0 && !self.rand_check(move_data.accuracy) {
            let atk_name = self.species_name(player);
            let def_name = self.species_name(defender_idx);
            self.emit(format!("|move|p{}a: {}|{}|p{}a: {}|[miss]", player+1, atk_name, move_data.name, defender_idx+1, def_name));
            self.emit(format!("|-miss|p{}a: {}|p{}a: {}", player+1, atk_name, defender_idx+1, def_name));
            return;
        }

        // Multi-hit moves: handle entirely before normal move announcement
        if is_multi_hit {
            // First hit accuracy check
            if move_data.accuracy > 0 && !self.rand_check(move_data.accuracy) {
                let atk_name = self.species_name(player);
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|move|p{}a: {}|{}|p{}a: {}|[miss]", player+1, atk_name, move_data.name, defender_idx+1, def_name));
                self.emit(format!("|-miss|p{}a: {}|p{}a: {}", player+1, atk_name, defender_idx+1, def_name));
                return;
            }
            // First hit passed - emit normal move line and execute multi-hit
            let atk_name = self.species_name(player);
            let def_name = self.species_name(defender_idx);
            self.emit(format!("|move|p{}a: {}|{}|p{}a: {}", player+1, atk_name, move_data.name, defender_idx+1, def_name));
            let max_hits = self.get_multi_hit_count(player, &move_data).unwrap();
            self.execute_multi_hit(player, defender_idx, &move_data, max_hits);
            return;
        }

        // Emit move announcement
        {
            let atk_name = self.species_name(player);
            // Self-targeting moves show the user as target
            let is_self_target = move_data.category == MoveCategory::Status && matches!(
                move_data.name.to_lowercase().as_str(),
                "swords dance" | "dragon dance" | "calm mind" | "iron defense" | "acid armor"
                | "nasty plot" | "agility" | "rock polish" | "quiver dance" | "shell smash"
                | "substitute" | "protect" | "detect"
                | "reflect" | "light screen" | "aurora veil" | "tailwind"
                | "recover" | "soft-boiled" | "roost" | "slack off" | "rest"
            );
            // Check if targeting status move will be blocked by Substitute
            let is_sub_blocked = move_data.category == MoveCategory::Status
                && !is_self_target
                && matches!(move_data.name.to_lowercase().as_str(), "toxic" | "will-o-wisp" | "thunder wave")
                && self.sides[defender_idx as usize].active().volatiles.contains(Volatiles::SUBSTITUTE);

            // Check if healing move will fail (at full HP)
            let is_heal_fail = is_self_target && matches!(
                move_data.name.to_lowercase().as_str(),
                "recover" | "soft-boiled" | "roost" | "slack off"
            ) && self.sides[player as usize].active().hp >= self.sides[player as usize].active().max_hp;

            if is_sub_blocked {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if is_heal_fail {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if is_self_target {
                self.emit(format!("|move|p{}a: {}|{}|p{}a: {}", player+1, atk_name, move_data.name, player+1, atk_name));
            } else {
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|move|p{}a: {}|{}|p{}a: {}", player+1, atk_name, move_data.name, defender_idx+1, def_name));
            }
        }

        // Protect check: blocks all moves targeting the defender
        if self.sides[defender_idx as usize]
            .active()
            .volatiles
            .contains(Volatiles::PROTECT)
            && move_data.category != MoveCategory::Status
        {
            let def_name = self.species_name(defender_idx);
            self.emit(format!("|-activate|p{}a: {}|move: Protect", defender_idx+1, def_name));
            return;
        }

        // Status moves
        if move_data.category == MoveCategory::Status {
            self.apply_status_move(player, defender_idx, &move_data);
            return;
        }

        // Check type immunity
        {
            let defender = self.sides[defender_idx as usize].active();
            let effectiveness = Type::effectiveness(move_data.move_type, &defender.types);
            if effectiveness == 0.0 {
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|-immune|p{}a: {}", defender_idx+1, def_name));
                return;
            }
        }

        // Check ability immunity
        if self.check_ability_immunity(defender_idx, move_data.move_type) {
            let def_name = self.species_name(defender_idx);
            self.emit(format!("|-immune|p{}a: {}", defender_idx+1, def_name));
            return;
        }

        // Calculate and apply damage
        let damage = self.calculate_move_damage(player, defender_idx, &move_data);

        // Emit effectiveness messages
        {
            let defender = self.sides[defender_idx as usize].active();
            let effectiveness = Type::effectiveness(move_data.move_type, &defender.types);
            let def_name = self.species_name(defender_idx);
            if effectiveness > 1.0 {
                self.emit(format!("|-supereffective|p{}a: {}", defender_idx+1, def_name));
            } else if effectiveness < 1.0 {
                self.emit(format!("|-resisted|p{}a: {}", defender_idx+1, def_name));
            }
        }

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
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|-end|p{}a: {}|Substitute", defender_idx+1, def_name));
            } else {
                self.sides[defender_idx as usize].active_mut().substitute_hp -= damage;
            }
        } else {
            let damage = self.check_focus_sash(defender_idx, damage);
            self.apply_damage(defender_idx, damage);
            let def_name = self.species_name(defender_idx);
            let hp_str = self.hp_display(defender_idx);
            self.emit(format!("|-damage|p{}a: {}|{}", defender_idx+1, def_name, hp_str));

            // Drain moves (Drain Punch, Giga Drain, etc.) — heal 1/2 of damage dealt
            let move_name_lower2 = move_data.name.to_lowercase();
            if matches!(move_name_lower2.as_str(), "drain punch" | "giga drain" | "horn leech" | "leech life" | "oblivion wing" | "parabolic charge") {
                let heal = (damage / 2).max(1);
                let mon = self.sides[player as usize].active_mut();
                if mon.is_alive() && mon.hp < mon.max_hp {
                    mon.hp = (mon.hp + heal).min(mon.max_hp);
                    let atk_name = self.species_name(player);
                    let atk_hp = self.sides[player as usize].active().hp;
                    let atk_max = self.sides[player as usize].active().max_hp;
                    let status_str = match self.sides[player as usize].active().status {
                        Status::Toxic => " tox",
                        Status::Burn => " brn",
                        Status::Poison => " psn",
                        Status::Paralyze => " par",
                        _ => "",
                    };
                    self.emit(format!("|-heal|p{}a: {}|{}/{}{}|[from] drain|[of] p{}a: {}", player+1, atk_name, atk_hp, atk_max, status_str, defender_idx+1, def_name));
                }
            }

            // Contact recoil abilities (Rough Skin, Iron Barbs)
            if move_data.flags.has(MoveFlags::CONTACT) {
                let defender_ability = self.sides[defender_idx as usize].active().ability_id;
                let defender_item = self.sides[defender_idx as usize].active().item_id;
                // Rough Skin / Iron Barbs: 1/8 max HP
                if matches!(defender_ability, pkmn_core::abilities::AbilityId::RoughSkin | pkmn_core::abilities::AbilityId::IronBarbs) {
                    let attacker_max_hp = self.sides[player as usize].active().max_hp;
                    let recoil = (attacker_max_hp / 8).max(1);
                    self.apply_damage(player, recoil);
                    let atk_name = self.species_name(player);
                    let ability_name = if defender_ability == pkmn_core::abilities::AbilityId::RoughSkin { "Rough Skin" } else { "Iron Barbs" };
                    let hp_str = self.hp_display(player);
                    self.emit(format!("|-damage|p{}a: {}|{}|[from] ability: {}|[of] p{}a: {}", player+1, atk_name, hp_str, ability_name, defender_idx+1, def_name));
                }
                // Rocky Helmet: 1/6 max HP
                if defender_item == pkmn_core::items::ItemId::RockyHelmet {
                    let attacker_max_hp = self.sides[player as usize].active().max_hp;
                    let recoil = (attacker_max_hp / 6).max(1);
                    self.apply_damage(player, recoil);
                    let atk_name = self.species_name(player);
                    let hp_str = self.hp_display(player);
                    self.emit(format!("|-damage|p{}a: {}|{}|[from] item: Rocky Helmet|[of] p{}a: {}", player+1, atk_name, hp_str, defender_idx+1, def_name));
                }
            }

            // Emit faint after contact recoil
            if self.sides[defender_idx as usize].active().hp == 0 {
                self.emit(format!("|faint|p{}a: {}", defender_idx+1, self.species_name(defender_idx)));
            }
            if self.sides[player as usize].active().hp == 0 && self.sides[player as usize].active().is_fainted {
                self.emit(format!("|faint|p{}a: {}", player+1, self.species_name(player)));
            }
        }

        // 4. Secondary effects (RNG consumed after damage)
        if self.sides[defender_idx as usize].active().is_alive() {
            self.apply_secondaries(player, defender_idx, &move_data);
        }

        // Move recoil (Brave Bird, Flare Blitz, etc.) — 1/3 of damage dealt
        let move_name_lower = move_data.name.to_lowercase();
        if matches!(move_name_lower.as_str(), "brave bird" | "flare blitz" | "double-edge" | "head smash" | "wild charge" | "wood hammer" | "take down") {
            let recoil_fraction = match move_name_lower.as_str() {
                "head smash" => 2, // 1/2
                "take down" => 4,  // 1/4
                _ => 3,            // 1/3
            };
            let recoil = (damage / recoil_fraction).max(1);
            if self.sides[player as usize].active().is_alive() {
                self.apply_damage(player, recoil);
                let atk_name = self.species_name(player);
                let hp = self.sides[player as usize].active().hp;
                let max_hp = self.sides[player as usize].active().max_hp;
                if hp == 0 {
                    self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] Recoil", player+1, atk_name));
                    self.emit(format!("|faint|p{}a: {}", player+1, atk_name));
                } else {
                    self.emit(format!("|-damage|p{}a: {}|{}/{}|[from] Recoil", player+1, atk_name, hp, max_hp));
                }
            }
        }

        // Drain moves (Drain Punch, Giga Drain, etc.) — heal 1/2 of damage dealt
        // (handled inside the damage application block above)

        // Life Orb recoil
        self.apply_life_orb_recoil(player);

        // Post-damage self-stat drops (Close Combat, etc.)
        self.apply_post_damage_effects(player, &move_data);

        // Multi-turn move handling
        self.handle_multi_turn(player, move_idx, &move_data);

        // Recharge moves
        self.handle_recharge_move(player, &move_data);

        // U-turn / Volt Switch / Flip Turn: switch out after dealing damage
        self.handle_switch_out_move(player, &move_data);
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

    fn apply_secondaries(&mut self, attacker: u8, defender: u8, move_data: &MoveData) {
        use pkmn_core::moves::{get_secondaries, SecondaryEffect, SecondaryStatus, Stat};

        let secondaries = get_secondaries(move_data.id);
        for secondary in secondaries {
            let roll = self.random(100);
            if roll >= secondary.chance {
                continue;
            }
            let def_name = self.species_name(defender);
            match secondary.effect {
                SecondaryEffect::Status(status) => {
                    let mon = self.sides[defender as usize].active_mut();
                    if mon.status != Status::None {
                        continue;
                    }
                    match status {
                        SecondaryStatus::Burn => {
                            if mon.types.contains(&Type::Fire) { continue; }
                            mon.status = Status::Burn;
                            self.emit(format!("|-status|p{}a: {}|brn", defender+1, def_name));
                        }
                        SecondaryStatus::Paralyze => {
                            if mon.types.contains(&Type::Electric) { continue; }
                            mon.status = Status::Paralyze;
                            self.emit(format!("|-status|p{}a: {}|par", defender+1, def_name));
                        }
                        SecondaryStatus::Freeze => {
                            if mon.types.contains(&Type::Ice) { continue; }
                            mon.status = Status::Freeze;
                            self.emit(format!("|-status|p{}a: {}|frz", defender+1, def_name));
                        }
                        SecondaryStatus::Poison => {
                            if mon.types.contains(&Type::Poison) || mon.types.contains(&Type::Steel) { continue; }
                            mon.status = Status::Poison;
                            self.emit(format!("|-status|p{}a: {}|psn", defender+1, def_name));
                        }
                        SecondaryStatus::Flinch => {
                            mon.volatiles.insert(Volatiles::FLINCH);
                        }
                    }
                }
                SecondaryEffect::StatDrop(stat, amount) => {
                    let mon = self.sides[defender as usize].active_mut();
                    match stat {
                        Stat::Atk => mon.boosts.atk = (mon.boosts.atk + amount).clamp(-6, 6),
                        Stat::Def => mon.boosts.def = (mon.boosts.def + amount).clamp(-6, 6),
                        Stat::Spa => mon.boosts.spa = (mon.boosts.spa + amount).clamp(-6, 6),
                        Stat::Spd => mon.boosts.spd = (mon.boosts.spd + amount).clamp(-6, 6),
                        Stat::Spe => mon.boosts.spe = (mon.boosts.spe + amount).clamp(-6, 6),
                    }
                    let (stat_name, abs_amount) = match stat {
                        Stat::Atk => ("atk", amount.unsigned_abs()),
                        Stat::Def => ("def", amount.unsigned_abs()),
                        Stat::Spa => ("spa", amount.unsigned_abs()),
                        Stat::Spd => ("spd", amount.unsigned_abs()),
                        Stat::Spe => ("spe", amount.unsigned_abs()),
                    };
                    if amount < 0 {
                        self.emit(format!("|-unboost|p{}a: {}|{}|{}", defender+1, def_name, stat_name, abs_amount));
                    } else {
                        self.emit(format!("|-boost|p{}a: {}|{}|{}", defender+1, def_name, stat_name, abs_amount));
                    }
                }
                SecondaryEffect::SelfStatBoost(stat, amount) => {
                    let atk_name = self.species_name(attacker);
                    let mon = self.sides[attacker as usize].active_mut();
                    match stat {
                        Stat::Atk => mon.boosts.atk = (mon.boosts.atk + amount).clamp(-6, 6),
                        Stat::Def => mon.boosts.def = (mon.boosts.def + amount).clamp(-6, 6),
                        Stat::Spa => mon.boosts.spa = (mon.boosts.spa + amount).clamp(-6, 6),
                        Stat::Spd => mon.boosts.spd = (mon.boosts.spd + amount).clamp(-6, 6),
                        Stat::Spe => mon.boosts.spe = (mon.boosts.spe + amount).clamp(-6, 6),
                    }
                    let (stat_name, abs_amount) = match stat {
                        Stat::Atk => ("atk", amount.unsigned_abs()),
                        Stat::Def => ("def", amount.unsigned_abs()),
                        Stat::Spa => ("spa", amount.unsigned_abs()),
                        Stat::Spd => ("spd", amount.unsigned_abs()),
                        Stat::Spe => ("spe", amount.unsigned_abs()),
                    };
                    if amount > 0 {
                        self.emit(format!("|-boost|p{}a: {}|{}|{}", attacker+1, atk_name, stat_name, abs_amount));
                    } else {
                        self.emit(format!("|-unboost|p{}a: {}|{}|{}", attacker+1, atk_name, stat_name, abs_amount));
                    }
                }
            }
        }
    }

    fn calculate_move_damage(
        &mut self,
        attacker_player: u8,
        defender_player: u8,
        move_data: &MoveData,
    ) -> u16 {
        // RNG order: 1. crit check, 2. damage roll (random(16))
        let critical = self.random_chance(1, 24);
        let roll = self.random(16); // 0-15
        let random_factor = (100 - roll) as u8; // 85-100
        self.calculate_damage_with(attacker_player, defender_player, move_data, critical, random_factor)
    }

    fn calculate_damage_with(
        &self,
        attacker_player: u8,
        defender_player: u8,
        move_data: &MoveData,
        critical: bool,
        random_factor: u8,
    ) -> u16 {
        let attacker = self.sides[attacker_player as usize].active();
        let defender = self.sides[defender_player as usize].active();

        let (atk_stat, def_stat) = match move_data.category {
            MoveCategory::Physical => {
                let atk = if move_data.name == "Body Press" {
                    attacker.effective_def()
                } else {
                    attacker.effective_atk()
                };
                (atk, defender.effective_def())
            }
            MoveCategory::Special => (attacker.effective_spa(), defender.effective_spd()),
            _ => return 0,
        };

        let base_power = self.get_variable_bp(attacker_player, defender_player, move_data);

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
        let attacker_ability = attacker.ability_id;

        if effectiveness == 0.0 {
            return 0;
        }

        let weather_boost = self.get_weather_modifier(move_data.move_type);
        let terrain_mod = self.get_terrain_modifier(attacker_player, move_data.move_type);

        let burn_mod =
            if attacker_status == Status::Burn
                && move_data.category == MoveCategory::Physical
                && attacker_ability != pkmn_core::abilities::AbilityId::Guts
            {
                0.5
            } else {
                1.0
            };

        let ability_mod = self.ability_damage_modifier(attacker_player, move_data);
        let item_mod = self.item_damage_modifier(attacker_player, move_data);

        let screen_mod = if !critical {
            match move_data.category {
                MoveCategory::Physical if self.sides[defender_player as usize].side_conditions.reflect > 0 => 0.5,
                MoveCategory::Special if self.sides[defender_player as usize].side_conditions.light_screen > 0 => 0.5,
                _ => 1.0,
            }
        } else {
            1.0
        };

        let ctx = pkmn_core::damage::DamageContext {
            attacker_level,
            attacker_stat: atk_stat,
            defender_stat: def_stat,
            base_power,
            stab: stab > 1.0,
            type_effectiveness: effectiveness,
            critical,
            weather_boost: weather_boost * terrain_mod,
            other_modifiers: burn_mod * ability_mod * item_mod * screen_mod * (stab / if stab > 1.0 { 1.5 } else { 1.0 }),
            random_factor,
        };

        pkmn_core::damage::calculate_damage(&ctx)
    }

    fn get_variable_bp(&self, attacker_player: u8, defender_player: u8, move_data: &MoveData) -> u16 {
        let name = move_data.name.to_lowercase();
        let attacker = self.sides[attacker_player as usize].active();
        let _defender = self.sides[defender_player as usize].active();
        match name.as_str() {
            "acrobatics" => {
                if attacker.item_id == pkmn_core::items::ItemId::None { 110 } else { 55 }
            }
            "facade" => {
                if attacker.status == Status::Burn
                    || attacker.status == Status::Poison
                    || attacker.status == Status::Toxic
                    || attacker.status == Status::Paralyze
                {
                    140
                } else {
                    70
                }
            }
            "weather ball" => {
                if self.field.weather != crate::field::Weather::None { 100 } else { 50 }
            }
            _ => move_data.base_power as u16,
        }
    }

    fn get_terrain_modifier(&self, attacker_player: u8, move_type: Type) -> f32 {
        let attacker = self.sides[attacker_player as usize].active();
        // Check if grounded (not Flying type, not Levitate, not Air Balloon)
        let is_grounded = !attacker.types.contains(&Type::Flying)
            && attacker.ability_id != pkmn_core::abilities::AbilityId::Levitate
            && attacker.item_id != pkmn_core::items::ItemId::AirBalloon;
        if !is_grounded {
            return 1.0;
        }
        match (self.field.terrain, move_type) {
            (Terrain::Electric, Type::Electric) => 1.3,
            (Terrain::Grassy, Type::Grass) => 1.3,
            (Terrain::Psychic, Type::Psychic) => 1.3,
            _ => 1.0,
        }
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

        // Prankster immunity: Dark types are immune to Prankster-boosted status moves
        let is_targeting_status = matches!(name.as_str(), "toxic" | "will-o-wisp" | "thunder wave");
        if is_targeting_status
            && self.sides[attacker as usize].active().ability_id == pkmn_core::abilities::AbilityId::Prankster
            && self.sides[defender as usize].active().types.contains(&Type::Dark)
        {
            let def_name = self.species_name(defender);
            self.emit(format!("|-immune|p{}a: {}", defender+1, def_name));
            return;
        }

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

        // Substitute blocks targeting status moves
        let is_targeting = matches!(name.as_str(), "toxic" | "will-o-wisp" | "thunder wave");
        if is_targeting && self.sides[defender as usize].active().volatiles.contains(Volatiles::SUBSTITUTE) {
            let atk_name = self.species_name(attacker);
            self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
            return;
        }

        match name.as_str() {
            "toxic" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None
                    && !def_mon.types.contains(&Type::Poison)
                    && !def_mon.types.contains(&Type::Steel)
                {
                    def_mon.status = Status::Toxic;
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-status|p{}a: {}|tox", defender+1, def_name));
                } else if def_mon.types.contains(&Type::Poison) || def_mon.types.contains(&Type::Steel) {
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-immune|p{}a: {}", defender+1, def_name));
                }
            }
            "will-o-wisp" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    def_mon.status = Status::Burn;
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-status|p{}a: {}|brn", defender+1, def_name));
                }
            }
            "thunder wave" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    def_mon.status = Status::Paralyze;
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-status|p{}a: {}|par", defender+1, def_name));
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
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-singleturn|p{}a: {}|Protect", attacker+1, atk_name));
                return;
            }
            "substitute" => {
                let mon = self.sides[attacker as usize].active_mut();
                let cost = mon.max_hp / 4;
                if mon.hp > cost && !mon.volatiles.contains(Volatiles::SUBSTITUTE) {
                    mon.hp -= cost;
                    mon.substitute_hp = cost;
                    mon.volatiles.insert(Volatiles::SUBSTITUTE);
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-start|p{}a: {}|Substitute", attacker+1, atk_name));
                    let hp = self.sides[attacker as usize].active().hp;
                    let max_hp = self.sides[attacker as usize].active().max_hp;
                    self.emit(format!("|-damage|p{}a: {}|{}/{}", attacker+1, atk_name, hp, max_hp));
                }
                return;
            }
            // Boost moves
            "swords dance" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk + 2).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|atk|2", attacker+1, atk_name));
            }
            "dragon dance" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk + 1).min(6);
                mon.boosts.spe = (mon.boosts.spe + 1).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|atk|1", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|spe|1", attacker+1, atk_name));
            }
            "calm mind" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                mon.boosts.spd = (mon.boosts.spd + 1).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spa|1", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|spd|1", attacker+1, atk_name));
            }
            "iron defense" | "acid armor" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.def = (mon.boosts.def + 2).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|def|2", attacker+1, atk_name));
            }
            "nasty plot" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 2).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spa|2", attacker+1, atk_name));
            }
            "agility" | "rock polish" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spe = (mon.boosts.spe + 2).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spe|2", attacker+1, atk_name));
            }
            "quiver dance" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                mon.boosts.spd = (mon.boosts.spd + 1).min(6);
                mon.boosts.spe = (mon.boosts.spe + 1).min(6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spa|1", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|spd|1", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|spe|1", attacker+1, atk_name));
            }
            "shell smash" => {
                let mon = self.sides[attacker as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk + 2).min(6);
                mon.boosts.spa = (mon.boosts.spa + 2).min(6);
                mon.boosts.spe = (mon.boosts.spe + 2).min(6);
                mon.boosts.def = (mon.boosts.def - 1).max(-6);
                mon.boosts.spd = (mon.boosts.spd - 1).max(-6);
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-unboost|p{}a: {}|def|1", attacker+1, atk_name));
                self.emit(format!("|-unboost|p{}a: {}|spd|1", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|atk|2", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|spa|2", attacker+1, atk_name));
                self.emit(format!("|-boost|p{}a: {}|spe|2", attacker+1, atk_name));
            }
            "reflect" => {
                self.sides[attacker as usize].side_conditions.reflect = 5;
                self.emit(format!("|-sidestart|p{}: Player {}|Reflect", attacker+1, attacker+1));
            }
            "light screen" => {
                self.sides[attacker as usize].side_conditions.light_screen = 5;
                self.emit(format!("|-sidestart|p{}: Player {}|move: Light Screen", attacker+1, attacker+1));
            }
            "stealth rock" => {
                self.sides[defender as usize].side_conditions.stealth_rock = true;
                self.emit(format!("|-sidestart|p{}: Player {}|move: Stealth Rock", defender+1, defender+1));
            }
            "recover" | "soft-boiled" | "roost" | "slack off" => {
                let mon = self.sides[attacker as usize].active_mut();
                if mon.hp >= mon.max_hp {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}|heal", attacker+1, atk_name));
                } else {
                    let heal = mon.max_hp / 2;
                    mon.hp = (mon.hp + heal).min(mon.max_hp);
                    let atk_name = self.species_name(attacker);
                    let hp = self.sides[attacker as usize].active().hp;
                    let max_hp = self.sides[attacker as usize].active().max_hp;
                    self.emit(format!("|-heal|p{}a: {}|{}/{}", attacker+1, atk_name, hp, max_hp));
                }
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
        }

        // Weather upkeep emission (before item healing)
        if self.field.weather != Weather::None && self.field.weather_turns > 0 {
            let weather_str = match self.field.weather {
                Weather::Rain => "RainDance",
                Weather::Sun => "SunnyDay",
                Weather::Sand => "Sandstorm",
                Weather::Snow => "Snow",
                _ => "none",
            };
            self.emit(format!("|-weather|{}|[upkeep]", weather_str));
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
                    let dmg = (mon.max_hp / 16).max(1) * mon.status_turns as u16;
                    mon.hp = mon.hp.saturating_sub(dmg);
                }
                _ => {}
            }

            if mon.hp == 0 {
                mon.is_fainted = true;
            }

            // Emit damage from weather/status
            let species_id = mon.species_id;
            let hp = mon.hp;
            let max_hp = mon.max_hp;
            let status = mon.status;
            let is_fainted = mon.is_fainted;
            let name = pkmn_core::species::get_species_by_id(species_id)
                .map(|s| s.name)
                .unwrap_or("Unknown");

            let status_str = match status {
                Status::Toxic => " tox",
                Status::Burn => " brn",
                Status::Poison => " psn",
                Status::Paralyze => " par",
                _ => "",
            };

            if weather == Weather::Sand
                && !self.sides[player].active().types.contains(&Type::Rock)
                && !self.sides[player].active().types.contains(&Type::Ground)
                && !self.sides[player].active().types.contains(&Type::Steel)
            {
                if hp == 0 {
                    self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] Sandstorm", player+1, name));
                } else {
                    self.emit(format!("|-damage|p{}a: {}|{}/{}{}|[from] Sandstorm", player+1, name, hp, max_hp, status_str));
                }
            }
            match status {
                Status::Burn => {
                    if hp == 0 {
                        self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] brn", player+1, name));
                    } else {
                        self.emit(format!("|-damage|p{}a: {}|{}/{} brn|[from] brn", player+1, name, hp, max_hp));
                    }
                }
                Status::Poison => {
                    if hp == 0 {
                        self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] psn", player+1, name));
                    } else {
                        self.emit(format!("|-damage|p{}a: {}|{}/{} psn|[from] psn", player+1, name, hp, max_hp));
                    }
                }
                Status::Toxic => {
                    if hp == 0 {
                        self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] psn", player+1, name));
                    } else {
                        self.emit(format!("|-damage|p{}a: {}|{}/{} tox|[from] psn", player+1, name, hp, max_hp));
                    }
                }
                _ => {}
            }
            if is_fainted {
                self.emit(format!("|faint|p{}a: {}", player+1, name));
            }
        }

        // Item end-of-turn (Leftovers, etc.) — after weather/status damage, faster first
        let p1_speed = self.sides[0].active().effective_speed();
        let p2_speed = self.sides[1].active().effective_speed();
        let item_order: [u8; 2] = if p2_speed > p1_speed { [1, 0] } else { [0, 1] };
        for &player in &item_order {
            self.trigger_item_end_of_turn(player);
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

    fn handle_switch_out_move(&mut self, player: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();
        let is_switch_out = matches!(name.as_str(), "u-turn" | "volt switch" | "flip turn" | "parting shot" | "teleport");
        if !is_switch_out {
            return;
        }
        if !self.sides[player as usize].active().is_alive() {
            return;
        }
        if !self.sides[player as usize].has_alive_switch() {
            return;
        }
        let current = self.sides[player as usize].active_index;
        let target = self.sides[player as usize].team.iter().enumerate()
            .find(|(i, p)| *i != current && p.is_alive())
            .map(|(i, _)| i);
        if let Some(target_idx) = target {
            self.execute_switch_from(player, target_idx as u8, Some(move_data.name));
        }
    }

    fn get_multi_hit_count(&self, player: u8, move_data: &MoveData) -> Option<u8> {
        match move_data.name {
            "Double Iron Bash" | "Dual Wingbeat" | "Dragon Darts" => Some(2),
            "Surging Strikes" | "Triple Axel" | "Triple Kick" => Some(3),
            "Population Bomb" => Some(10),
            "Bullet Seed" | "Icicle Spear" | "Rock Blast" | "Scale Shot"
            | "Pin Missile" | "Tail Slap" | "Bone Rush" | "Arm Thrust"
            | "Double Slap" | "Comet Punch" => {
                // 2-5 hits; Skill Link always gives 5
                if self.sides[player as usize].active().ability_id
                    == pkmn_core::abilities::AbilityId::SkillLink
                {
                    Some(5)
                } else {
                    Some(0) // sentinel: roll in execute_multi_hit
                }
            }
            _ => None,
        }
    }

    fn execute_multi_hit(&mut self, player: u8, defender: u8, move_data: &MoveData, max_hits: u8) {
        let hits = if max_hits == 0 {
            // 2-5 hit distribution: 35/35/15/15
            let roll = self.random(20);
            if roll < 7 { 2 } else if roll < 14 { 3 } else if roll < 17 { 4 } else { 5 }
        } else {
            // PS still consumes RNG for hit count even with Skill Link
            let _ = self.random(20);
            max_hits as u32
        };

        let mut actual_hits: u32 = 0;
        for _hit_num in 0..hits {
            let critical = self.random_chance(1, 24);
            let roll = self.random(16);
            let random_factor = (100 - roll) as u8;
            let damage = self.calculate_damage_with(player, defender, move_data, critical, random_factor);

            if self.sides[defender as usize].active().volatiles.contains(Volatiles::SUBSTITUTE) {
                let sub_hp = self.sides[defender as usize].active().substitute_hp;
                if damage >= sub_hp {
                    self.sides[defender as usize].active_mut().substitute_hp = 0;
                    self.sides[defender as usize].active_mut().volatiles.remove(Volatiles::SUBSTITUTE);
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-end|p{}a: {}|Substitute", defender+1, def_name));
                } else {
                    self.sides[defender as usize].active_mut().substitute_hp -= damage;
                }
            } else {
                let damage = self.check_focus_sash(defender, damage);
                self.apply_damage(defender, damage);
                let def_name = self.species_name(defender);
                let hp = self.sides[defender as usize].active().hp;
                let max_hp = self.sides[defender as usize].active().max_hp;
                if hp == 0 {
                    self.emit(format!("|-damage|p{}a: {}|0 fnt", defender+1, def_name));
                } else {
                    self.emit(format!("|-damage|p{}a: {}|{}/{}", defender+1, def_name, hp, max_hp));
                }
            }

            actual_hits += 1;
            if self.sides[defender as usize].active().hp == 0 {
                break;
            }
        }

        let def_name = self.species_name(defender);
        self.emit(format!("|-hitcount|p{}a: {}|{}", defender+1, def_name, actual_hits));

        if self.sides[defender as usize].active().hp == 0 {
            self.emit(format!("|faint|p{}a: {}", defender+1, def_name));
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
