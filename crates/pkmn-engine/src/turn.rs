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
        // Regenerator: heal 1/3 HP on switch-out
        {
            let mon = self.sides[player as usize].active();
            if mon.ability_id == pkmn_core::abilities::AbilityId::Regenerator && mon.is_alive() && mon.hp < mon.max_hp {
                let heal = mon.max_hp / 3;
                let mon = self.sides[player as usize].active_mut();
                mon.hp = (mon.hp + heal).min(mon.max_hp);
            }
        }

        // Clear volatiles on switch
        let mon = self.sides[player as usize].active_mut();
        mon.volatiles = Volatiles::empty();
        mon.locked_move_turns = 0;
        mon.confusion_turns = 0;
        mon.protect_consecutive = 0;

        self.sides[player as usize].active_index = target as usize;
        let name = self.species_name(player);
        let full_name = self.full_species_name(player);
        let mon = self.sides[player as usize].active();
        let hp = mon.hp;
        let max_hp = mon.max_hp;
        let level = mon.level;
        let level_str = if level == 100 { String::new() } else { format!(", L{}", level) };
        let from_str = match from_move {
            Some(m) => format!("|[from] {}", m),
            None => String::new(),
        };
        self.emit(format!("|switch|p{}a: {}|{}{}|{}/{}{}", player+1, name, full_name, level_str, hp, max_hp, from_str));
        if !self.has_heavy_duty_boots(player) {
            self.apply_entry_hazards(player);
        }
        self.trigger_ability_on_switch(player);
    }

    fn execute_move(&mut self, player: u8, move_idx: u8) {
        let defender_idx = 1 - player;

        self.last_attacker = Some(player);

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
            let name = self.species_name(player);
            self.emit(format!("|cant|p{}a: {}|flinch", player+1, name));
            return;
        }

        // Paralysis full-para check: 25% chance to be fully paralyzed
        if self.sides[player as usize].active().status == Status::Paralyze {
            let roll = self.random(4);
            if roll == 0 {
                let name = self.species_name(player);
                self.emit(format!("|cant|p{}a: {}|par", player+1, name));
                return;
            }
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

        // Check if move will fail (skip accuracy for these - PS behavior)
        let will_fail_still = {
            let mn = move_data.name.to_lowercase();
            match mn.as_str() {
                "stealth rock" => self.sides[defender_idx as usize].side_conditions.stealth_rock,
                "toxic spikes" => self.sides[defender_idx as usize].side_conditions.toxic_spikes >= 2,
                "sticky web" => self.sides[defender_idx as usize].side_conditions.sticky_web,
                "leech seed" => {
                    let def = self.sides[defender_idx as usize].active();
                    def.volatiles.contains(Volatiles::LEECH_SEED) || def.types.contains(&Type::Grass)
                }
                "thunder wave" => {
                    // [still] only for type immunity (Electric, Ground) — Thunder Wave has ignoreImmunity: false in PS
                    let def = self.sides[defender_idx as usize].active();
                    def.types.contains(&Type::Electric) || def.types.contains(&Type::Ground)
                }
                "toxic" => {
                    // PS: status moves ignore type immunity by default, so Toxic vs Steel/Poison
                    // still consumes accuracy RNG. Only [still] if target already has status.
                    false
                }
                "will-o-wisp" => {
                    // PS: status moves ignore type immunity, so Will-O-Wisp vs Fire
                    // still consumes accuracy RNG. Only [still] if target already has status.
                    false
                }
                _ => false,
            }
        };

        // Check type immunity BEFORE accuracy (PS order: immunity → accuracy → damage)
        let is_type_immune = if move_data.category != MoveCategory::Status {
            let def_types = self.defender_types(defender_idx);
            Type::effectiveness(move_data.move_type, &def_types) == 0.0
        } else {
            false
        };

        // Also check ability immunity before accuracy
        let is_ability_immune = move_data.category != MoveCategory::Status
            && self.check_ability_immunity(defender_idx, move_data.move_type);

        // Accuracy check (skip for multi-hit moves, moves that will fail, and immune targets)
        // accuracy=0 means "never miss" (PS accuracy: true), no RNG consumed
        // accuracy=100 means "always hits" — PS still consumes RNG via randomChance(100, 100)
        let is_multi_hit = self.get_multi_hit_count(player, &move_data).is_some();
        if !will_fail_still && !is_multi_hit && !is_type_immune && !is_ability_immune && move_data.accuracy > 0 {
            if !self.rand_check(move_data.accuracy) {
                let atk_name = self.species_name(player);
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|move|p{}a: {}|{}|p{}a: {}|[miss]", player+1, atk_name, move_data.name, defender_idx+1, def_name));
                self.emit(format!("|-miss|p{}a: {}|p{}a: {}", player+1, atk_name, defender_idx+1, def_name));
                return;
            }
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
            // Apply secondary effects after multi-hit (e.g. Scale Shot def-1/spe+1)
            if self.sides[defender_idx as usize].active().is_alive() {
                let has_secondaries = !pkmn_core::moves::get_secondaries(move_data.id).is_empty();
                let sheer_force_active = self.sides[player as usize].active().ability_id == pkmn_core::abilities::AbilityId::SheerForce && has_secondaries;
                if !sheer_force_active {
                    self.apply_secondaries(player, defender_idx, &move_data);
                }
            }
            // Life Orb recoil for multi-hit moves
            if self.sides[player as usize].active().is_alive() {
                let has_secondaries = !pkmn_core::moves::get_secondaries(move_data.id).is_empty();
                let sheer_force_active = self.sides[player as usize].active().ability_id == pkmn_core::abilities::AbilityId::SheerForce && has_secondaries;
                if !sheer_force_active {
                    self.apply_life_orb_recoil(player);
                }
            }
            return;
        }

        // Emit move announcement
        {
            let atk_name = self.species_name(player);
            // Self-targeting moves show the user as target
            // Opponent-targeting status moves show the opponent as target
            let is_opponent_status = move_data.category == MoveCategory::Status && matches!(
                move_data.name.to_lowercase().as_str(),
                "toxic" | "will-o-wisp" | "thunder wave" | "sleep powder" | "spore"
                | "stun spore" | "hypnosis" | "sing" | "lovely kiss" | "dark void"
                | "yawn" | "glare" | "nuzzle" | "confuse ray" | "swagger" | "flatter"
                | "taunt" | "encore" | "torment" | "disable" | "heal block"
                | "leech seed" | "whirlwind" | "roar"
                | "trick" | "switcheroo"
                | "pain split"
                | "stealth rock" | "spikes" | "toxic spikes" | "sticky web"
                | "defog"
            );
            // Self-targeting: any Status move that doesn't target the opponent
            let is_self_target = move_data.category == MoveCategory::Status && !is_opponent_status;
            // Check if targeting status move will be blocked by Substitute
            let is_sub_blocked = move_data.category == MoveCategory::Status
                && !is_self_target
                && matches!(move_data.name.to_lowercase().as_str(), "toxic" | "will-o-wisp" | "thunder wave")
                && self.sides[defender_idx as usize].active().volatiles.contains(Volatiles::SUBSTITUTE);

            // Check if healing move will fail (at full HP)
            let is_heal_fail = is_self_target && matches!(
                move_data.name.to_lowercase().as_str(),
                "recover" | "soft-boiled" | "roost" | "slack off" | "synthesis" | "morning sun" | "moonlight"
            ) && self.sides[player as usize].active().hp >= self.sides[player as usize].active().max_hp;

            // Moves that use [still] format when failing
            let heal_still = is_heal_fail && matches!(
                move_data.name.to_lowercase().as_str(),
                "recover" | "soft-boiled" | "roost" | "slack off"
            );

            // Check if hazard move will fail (already set)
            let is_hazard_fail = matches!(move_data.name.to_lowercase().as_str(), "stealth rock" | "spikes" | "toxic spikes" | "sticky web") && {
                let sc = &self.sides[defender_idx as usize].side_conditions;
                match move_data.name.to_lowercase().as_str() {
                    "stealth rock" => sc.stealth_rock,
                    "toxic spikes" => sc.toxic_spikes >= 2,
                    "sticky web" => sc.sticky_web,
                    _ => false,
                }
            };

            // Check if Leech Seed will fail (already seeded or Grass type)
            let is_leech_seed_fail = move_data.name == "Leech Seed" && {
                let def = self.sides[defender_idx as usize].active();
                def.volatiles.contains(Volatiles::LEECH_SEED) || def.types.contains(&Type::Grass)
            };

            // Check if status move will fail because target has a DIFFERENT status
            let is_status_fail = {
                let mn = move_data.name.to_lowercase();
                let def = self.sides[defender_idx as usize].active();
                match mn.as_str() {
                    "thunder wave" => def.status != Status::None && def.status != Status::Paralyze,
                    "toxic" => def.status != Status::None && def.status != Status::Toxic && def.status != Status::Poison,
                    "will-o-wisp" => def.status != Status::None && def.status != Status::Burn,
                    _ => false,
                }
            };

            // Check if this is a locked move continuation (Outrage turn 2+)
            let is_locked = self.sides[player as usize].active().volatiles.contains(Volatiles::LOCKED_MOVE);
            let locked_suffix = if is_locked { "|[from] lockedmove" } else { "" };

            if is_sub_blocked {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if heal_still {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if is_hazard_fail {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if is_leech_seed_fail {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if is_status_fail {
                self.emit(format!("|move|p{}a: {}|{}||[still]", player+1, atk_name, move_data.name));
            } else if is_self_target {
                self.emit(format!("|move|p{}a: {}|{}|p{}a: {}{}", player+1, atk_name, move_data.name, player+1, atk_name, locked_suffix));
            } else {
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|move|p{}a: {}|{}|p{}a: {}{}", player+1, atk_name, move_data.name, defender_idx+1, def_name, locked_suffix));
            }

            if is_heal_fail {
                self.emit(format!("|-fail|p{}a: {}|heal", player+1, atk_name));
                return;
            }

            // Status move already-has-status: emit |-fail| and return
            if is_status_fail {
                self.emit(format!("|-fail|p{}a: {}", player+1, atk_name));
                return;
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
            let def_types = self.defender_types(defender_idx);
            let effectiveness = Type::effectiveness(move_data.move_type, &def_types);
            if effectiveness == 0.0 {
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|-immune|p{}a: {}", defender_idx+1, def_name));
                return;
            }
        }

        // Check ability immunity
        if self.check_ability_immunity(defender_idx, move_data.move_type) {
            let def_name = self.species_name(defender_idx);
            let defender_ability = self.sides[defender_idx as usize].active().ability_id;
            if defender_ability == pkmn_core::abilities::AbilityId::FlashFire {
                self.emit(format!("|-start|p{}a: {}|ability: Flash Fire", defender_idx+1, def_name));
                self.sides[defender_idx as usize].active_mut().volatiles.insert(Volatiles::FLASH_FIRE);
            } else if defender_ability == pkmn_core::abilities::AbilityId::LightningRod
                || defender_ability == pkmn_core::abilities::AbilityId::StormDrain
            {
                let ability_name = pkmn_core::abilities::get_ability(defender_ability).name;
                self.emit(format!("|-ability|p{}a: {}|{}|boost", defender_idx+1, def_name, ability_name));
                // Boost SpA by 1
                self.sides[defender_idx as usize].active_mut().boosts.spa += 1;
                self.emit(format!("|-boost|p{}a: {}|spa|1", defender_idx+1, def_name));
            } else if defender_ability == pkmn_core::abilities::AbilityId::VoltAbsorb
                || defender_ability == pkmn_core::abilities::AbilityId::WaterAbsorb
            {
                let ability_name = pkmn_core::abilities::get_ability(defender_ability).name;
                self.emit(format!("|-immune|p{}a: {}|[from] ability: {}", defender_idx+1, def_name, ability_name));
            } else {
                let ability_name = pkmn_core::abilities::get_ability(defender_ability).name;
                self.emit(format!("|-immune|p{}a: {}|[from] ability: {}", defender_idx+1, def_name, ability_name));
            }
            return;
        }

        // Calculate and apply damage
        let (damage, critical) = self.calculate_move_damage(player, defender_idx, &move_data);

        // Emit effectiveness messages (skip for fixed-damage moves like Seismic Toss/Night Shade)
        let is_fixed_damage = matches!(move_data.name, "Seismic Toss" | "Night Shade");
        if !is_fixed_damage {
            let def_types = self.defender_types(defender_idx);
            let effectiveness = Type::effectiveness(move_data.move_type, &def_types);
            let def_name = self.species_name(defender_idx);
            if effectiveness > 1.0 {
                self.emit(format!("|-supereffective|p{}a: {}", defender_idx+1, def_name));
            } else if effectiveness < 1.0 {
                self.emit(format!("|-resisted|p{}a: {}", defender_idx+1, def_name));
            }
            if critical {
                self.emit(format!("|-crit|p{}a: {}", defender_idx+1, def_name));
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

            // Rapid Spin: +1 Speed then remove hazards from user's side (Gen 9)
            if move_data.name == "Rapid Spin" && self.sides[player as usize].active().is_alive() {
                let atk_name = self.species_name(player);
                let current_spe = self.sides[player as usize].active().boosts.spe;
                if current_spe < 6 {
                    let mon = self.sides[player as usize].active_mut();
                    mon.boosts.spe = (mon.boosts.spe + 1).min(6);
                    self.emit(format!("|-boost|p{}a: {}|spe|1", player+1, atk_name));
                }
                let side = player as usize;
                if self.sides[side].side_conditions.stealth_rock {
                    self.sides[side].side_conditions.stealth_rock = false;
                    self.emit(format!("|-sideend|p{}: Player {}|Stealth Rock|[from] move: Rapid Spin|[of] p{}a: {}", player+1, player+1, player+1, atk_name));
                }
                if self.sides[side].side_conditions.spikes > 0 {
                    self.sides[side].side_conditions.spikes = 0;
                    self.emit(format!("|-sideend|p{}: Player {}|Spikes|[from] move: Rapid Spin|[of] p{}a: {}", player+1, player+1, player+1, atk_name));
                }
                if self.sides[side].side_conditions.toxic_spikes > 0 {
                    self.sides[side].side_conditions.toxic_spikes = 0;
                    self.emit(format!("|-sideend|p{}: Player {}|Toxic Spikes|[from] move: Rapid Spin|[of] p{}a: {}", player+1, player+1, player+1, atk_name));
                }
                if self.sides[side].side_conditions.sticky_web {
                    self.sides[side].side_conditions.sticky_web = false;
                    self.emit(format!("|-sideend|p{}: Player {}|Sticky Web|[from] move: Rapid Spin|[of] p{}a: {}", player+1, player+1, player+1, atk_name));
                }
            }

            // Secondary effects: must come BEFORE self-stat drops and contact recoil (PS order)
            // Only apply if target is alive (skip RNG consumption on KO)
            let has_secondaries = !pkmn_core::moves::get_secondaries(move_data.id).is_empty();
            let sheer_force_active = self.sides[player as usize].active().ability_id == pkmn_core::abilities::AbilityId::SheerForce && has_secondaries;
            if self.sides[defender_idx as usize].active().is_alive() && !sheer_force_active {
                self.apply_secondaries(player, defender_idx, &move_data);
            }

            // Self-stat drops (Close Combat, Superpower) emit BEFORE contact recoil per PS protocol
            if matches!(move_data.name, "Close Combat") {
                let atk_name = self.species_name(player);
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.def = (mon.boosts.def - 1).max(-6);
                mon.boosts.spd = (mon.boosts.spd - 1).max(-6);
                self.emit(format!("|-unboost|p{}a: {}|def|1", player+1, atk_name));
                self.emit(format!("|-unboost|p{}a: {}|spd|1", player+1, atk_name));
            }
            if move_data.name == "Superpower" {
                let atk_name = self.species_name(player);
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.atk = (mon.boosts.atk - 1).max(-6);
                mon.boosts.def = (mon.boosts.def - 1).max(-6);
                self.emit(format!("|-unboost|p{}a: {}|atk|1", player+1, atk_name));
                self.emit(format!("|-unboost|p{}a: {}|def|1", player+1, atk_name));
            }

            // Contact recoil abilities (Rough Skin, Iron Barbs) — triggers even if defender fainted
            if move_data.flags.has(MoveFlags::CONTACT) {
                let defender_ability = self.sides[defender_idx as usize].active().ability_id;
                let defender_item = self.sides[defender_idx as usize].active().item_id;
                // Contact recoil abilities dispatched via EventHooks
                let hooks = crate::events::ability_hooks(defender_ability);
                if let Some(hook) = hooks.on_damaging_hit {
                    hook(self, player, defender_idx);
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
            if self.sides[defender_idx as usize].active().hp == 0 {
                self.emit(format!("|faint|p{}a: {}", defender_idx+1, self.species_name(defender_idx)));
            }
            if self.sides[player as usize].active().hp == 0 && self.sides[player as usize].active().is_fainted {
                self.emit(format!("|faint|p{}a: {}", player+1, self.species_name(player)));
            }
        }

        // Sheer Force / secondary tracking for Life Orb
        let has_secondaries = !pkmn_core::moves::get_secondaries(move_data.id).is_empty();
        let sheer_force_active = self.sides[player as usize].active().ability_id == pkmn_core::abilities::AbilityId::SheerForce && has_secondaries;

        // Knock Off: remove target's item
        if move_data.name == "Knock Off" && self.sides[defender_idx as usize].active().is_alive() {
            let target_item = self.sides[defender_idx as usize].active().item_id;
            if target_item != pkmn_core::items::ItemId::None {
                let item_name = pkmn_core::items::get_item(target_item).name;
                self.sides[defender_idx as usize].active_mut().item_id = pkmn_core::items::ItemId::None;
                let def_name = self.species_name(defender_idx);
                let atk_name = self.species_name(player);
                self.emit(format!("|-enditem|p{}a: {}|{}|[from] move: Knock Off|[of] p{}a: {}", defender_idx+1, def_name, item_name, player+1, atk_name));
            }
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

        // Life Orb recoil (skipped by Sheer Force when move has secondaries)
        if !sheer_force_active {
            self.apply_life_orb_recoil(player);
        }

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
        let pname = self.species_name(player);
        let name = move_data.name.to_lowercase();
        match name.as_str() {
            "overheat" => {
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa - 2).max(-6);
                self.emit(format!("|-unboost|p{}a: {}|spa|2", player+1, pname));
            }
            "draco meteor" => {
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa - 2).max(-6);
                self.emit(format!("|-unboost|p{}a: {}|spa|2", player+1, pname));
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
                SecondaryEffect::None => {
                    // RNG already consumed above, nothing to apply
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
    ) -> (u16, bool) {
        // RNG order: 1. crit check, 2. damage roll (random(16))
        // Crit stage table (Gen 9): stage 0 → 1/24, stage 1 → 1/8, stage 2 → 1/2, stage 3+ → always
        // crit_ratio: 0 = normal (1/24), 1 = high-crit (1/8), 2 = super (1/2), 3+ = always
        let crit_denom = match move_data.crit_ratio {
            0 => 24,
            1 => 8,
            2 => 2,
            _ => 1,
        };
        // Fixed-damage moves: return attacker's level (PS skips crit+roll for these)
        if matches!(move_data.name, "Seismic Toss" | "Night Shade") {
            let attacker = self.sides[attacker_player as usize].active();
            return (attacker.level as u16, false);
        }
        let critical = self.random_chance(1, crit_denom);
        let roll = self.random(16); // 0-15
        let random_factor = (100 - roll) as u8; // 85-100
        (self.calculate_damage_with(attacker_player, defender_player, move_data, critical, random_factor), critical)
    }

    pub fn calculate_damage_with(
        &self,
        attacker_player: u8,
        defender_player: u8,
        move_data: &MoveData,
        critical: bool,
        random_factor: u8,
    ) -> u16 {
        let attacker = self.sides[attacker_player as usize].active();
        let defender = self.sides[defender_player as usize].active();

        // Fixed-damage moves: return attacker's level
        if matches!(move_data.name, "Seismic Toss" | "Night Shade") {
            return attacker.level as u16;
        }

        // --- Foul Play uses defender's Atk stat ---
        let (mut atk_stat, mut def_stat) = match move_data.category {
            MoveCategory::Physical => {
                let atk = if move_data.name == "Body Press" {
                    attacker.effective_def()
                } else if move_data.name == "Foul Play" {
                    defender.effective_atk()
                } else {
                    attacker.effective_atk()
                };
                // Crit ignores positive def boosts
                let def = if critical && defender.boosts.def > 0 {
                    defender.stats.def // raw stat, no boost
                } else {
                    defender.effective_def()
                };
                (atk, def)
            }
            MoveCategory::Special => {
                let def = if critical && defender.boosts.spd > 0 {
                    defender.stats.spd
                } else {
                    defender.effective_spd()
                };
                (attacker.effective_spa(), def)
            }
            _ => return 0,
        };

        let atk_item = attacker.item_id;
        let atk_ability = attacker.ability_id;
        let def_ability = defender.ability_id;

        // --- Offensive stat modifiers ---
        match move_data.category {
            MoveCategory::Physical => {
                if atk_item == pkmn_core::items::ItemId::ChoiceBand {
                    atk_stat = (atk_stat as u32 * 3 / 2) as u16;
                }
                if atk_item == pkmn_core::items::ItemId::LightBall {
                    atk_stat = (atk_stat as u32 * 2) as u16;
                }
                if matches!(atk_ability, pkmn_core::abilities::AbilityId::HugePower | pkmn_core::abilities::AbilityId::PurePower) {
                    atk_stat = (atk_stat as u32 * 2) as u16;
                }
                // Guts: 1.5x Atk when statused (replaces burn penalty)
                if atk_ability == pkmn_core::abilities::AbilityId::Guts
                    && attacker.status != Status::None
                {
                    atk_stat = (atk_stat as u32 * 3 / 2) as u16;
                }
                // Tablets of Ruin (opponent has it): attacker's Atk * 3/4
                if def_ability == pkmn_core::abilities::AbilityId::TabletsOfRuin {
                    atk_stat = (atk_stat as u32 * 3 / 4) as u16;
                }
            }
            MoveCategory::Special => {
                if atk_item == pkmn_core::items::ItemId::ChoiceSpecs {
                    atk_stat = (atk_stat as u32 * 3 / 2) as u16;
                }
                if atk_item == pkmn_core::items::ItemId::LightBall {
                    atk_stat = (atk_stat as u32 * 2) as u16;
                }
                // Vessel of Ruin (opponent has it): attacker's SpA * 3/4
                if def_ability == pkmn_core::abilities::AbilityId::VesselOfRuin {
                    atk_stat = (atk_stat as u32 * 3 / 4) as u16;
                }
            }
            _ => {}
        }

        // --- Defensive stat modifiers ---
        let def_item = defender.item_id;
        if def_item == pkmn_core::items::ItemId::Eviolite {
            def_stat = (def_stat as u32 * 3 / 2) as u16;
        }
        if def_item == pkmn_core::items::ItemId::AssaultVest && move_data.category == MoveCategory::Special {
            def_stat = (def_stat as u32 * 3 / 2) as u16;
        }
        // Fur Coat: 2x physical Def
        if def_ability == pkmn_core::abilities::AbilityId::FurCoat
            && move_data.category == MoveCategory::Physical
        {
            def_stat = (def_stat as u32 * 2) as u16;
        }
        // Thick Fat: halves Fire/Ice damage (applied as 0.5x to attacker's offensive stat in PS)
        // Actually in PS it's onSourceModifyAtk/SpA, effectively halving the attacking stat
        if def_ability == pkmn_core::abilities::AbilityId::ThickFat
            && (move_data.move_type == Type::Fire || move_data.move_type == Type::Ice)
        {
            atk_stat = (atk_stat as u32 / 2) as u16;
        }
        // Sword of Ruin (attacker has it): defender's Def * 3/4
        if atk_ability == pkmn_core::abilities::AbilityId::SwordOfRuin
            && move_data.category == MoveCategory::Physical
        {
            def_stat = (def_stat as u32 * 3 / 4) as u16;
        }
        // Beads of Ruin (attacker has it): defender's SpD * 3/4
        if atk_ability == pkmn_core::abilities::AbilityId::BeadsOfRuin
            && move_data.category == MoveCategory::Special
        {
            def_stat = (def_stat as u32 * 3 / 4) as u16;
        }
        // Sandstorm: 1.5x SpD for Rock-type defenders
        if self.field.weather == crate::field::Weather::Sand
            && move_data.category == MoveCategory::Special
            && defender.types.contains(&Type::Rock)
        {
            def_stat = (def_stat as u32 * 3 / 2) as u16;
        }
        // Snow: 1.5x Def for Ice-type defenders
        if self.field.weather == crate::field::Weather::Snow
            && move_data.category == MoveCategory::Physical
            && defender.types.contains(&Type::Ice)
        {
            def_stat = (def_stat as u32 * 3 / 2) as u16;
        }

        // --- Base power calculation ---
        let base_power = self.get_variable_bp(attacker_player, defender_player, move_data);

        // Determine effective move type (Weather Ball changes type in weather)
        let move_type = if move_data.name == "Weather Ball" {
            match self.field.weather {
                crate::field::Weather::Sun => Type::Fire,
                crate::field::Weather::Rain => Type::Water,
                crate::field::Weather::Sand => Type::Rock,
                crate::field::Weather::Snow => Type::Ice,
                _ => move_data.move_type,
            }
        } else {
            move_data.move_type
        };

        // Knock Off: 1.5x BP when target has item (onBasePower)
        let base_power = if move_data.name == "Knock Off"
            && defender.item_id != pkmn_core::items::ItemId::None
        {
            (base_power as u32 * 6144 + 2048) / 4096
        } else {
            base_power as u32
        } as u16;

        // Terrain boost (onBasePower)
        let (terrain_num, terrain_denom) = self.get_terrain_modifier(attacker_player, move_type);
        let base_power = (base_power as u32 * terrain_num / terrain_denom) as u16;

        // onBasePower modifiers (Tough Claws, type-boosting items, Muscle Band, etc.)
        let base_power = self.apply_bp_modifiers_with_type(attacker_player, move_data, base_power, move_type);

        // --- STAB ---
        let stab = if attacker.is_terastallized {
            self.tera_stab(attacker_player, move_type)
        } else if attacker.types.contains(&move_type) {
            // Adaptability: 2.0x STAB instead of 1.5x
            if atk_ability == pkmn_core::abilities::AbilityId::Adaptability { 2.0 } else { 1.5 }
        } else {
            1.0
        };

        // --- Type effectiveness ---
        let def_types = self.defender_types(defender_player);
        let effectiveness = Type::effectiveness(move_type, &def_types);
        if effectiveness == 0.0 {
            return 0;
        }

        // --- Weather power modifier ---
        let weather_boost = self.get_weather_modifier(move_type);

        // --- Burn (halves physical damage, Guts/Facade exempt) ---
        let burn_mod: u32 = if attacker.status == Status::Burn
            && move_data.category == MoveCategory::Physical
            && atk_ability != pkmn_core::abilities::AbilityId::Guts
            && move_data.name != "Facade"
        {
            2048 // 0.5x
        } else {
            4096
        };

        // --- Screen modifier ---
        let screen_mod: u32 = if !critical {
            match move_data.category {
                MoveCategory::Physical if self.sides[defender_player as usize].side_conditions.reflect > 0 => 2048,
                MoveCategory::Special if self.sides[defender_player as usize].side_conditions.light_screen > 0 => 2048,
                _ => 4096,
            }
        } else {
            4096
        };

        // --- Base damage formula ---
        let level = attacker.level as u32;
        let power = base_power as u32;
        let atk = atk_stat as u32;
        let def = def_stat as u32;
        let mut damage = ((2 * level / 5 + 2) * power * atk / def) / 50 + 2;

        // Weather: chainModify
        let weather_mod = (weather_boost * 4096.0) as u32;
        damage = (damage * weather_mod + 2047) / 4096;

        // Critical hit: simple truncation (1.5x)
        if critical {
            damage = damage * 3 / 2;
        }

        // Random factor: simple truncation
        damage = damage * random_factor as u32 / 100;

        // STAB: chainModify(6144/4096) for normal, 8192 for Adaptability
        if stab > 1.0 {
            let stab_mod = (stab / 1.5 * 6144.0) as u32; // 6144 for 1.5x, 8192 for 2.0x
            damage = (damage * stab_mod + 2047) / 4096;
        }

        // Type effectiveness: direct multiplication
        if effectiveness == 4.0 { damage *= 4; }
        else if effectiveness == 2.0 { damage *= 2; }
        else if effectiveness == 0.5 { damage /= 2; }
        else if effectiveness == 0.25 { damage /= 4; }

        // Burn: chainModify
        damage = (damage * burn_mod + 2047) / 4096;

        // Screen: chainModify
        damage = (damage * screen_mod + 2047) / 4096;

        // --- Final modifier chain (each applied separately via chainModify) ---

        // Solid Rock / Filter / Prism Armor: 0.75x on super effective
        if (def_ability == pkmn_core::abilities::AbilityId::SolidRock
            || def_ability == pkmn_core::abilities::AbilityId::Filter
            || def_ability == pkmn_core::abilities::AbilityId::PrismArmor)
            && effectiveness > 1.0
        {
            damage = (damage * 3072 + 2047) / 4096;
        }

        // Ice Scales: 0.5x special damage
        if def_ability == pkmn_core::abilities::AbilityId::IceScales
            && move_data.category == MoveCategory::Special
        {
            damage = (damage * 2048 + 2047) / 4096;
        }

        // Multiscale: 0.5x at full HP
        if def_ability == pkmn_core::abilities::AbilityId::Multiscale
            && defender.hp == defender.max_hp
        {
            damage = (damage * 2048 + 2047) / 4096;
        }

        // Tinted Lens: 2x on resisted hits
        if atk_ability == pkmn_core::abilities::AbilityId::TintedLens
            && effectiveness < 1.0
        {
            damage = (damage * 8192 + 2047) / 4096;
        }

        // Sheer Force: 1.3x (5325/4096) on moves with secondary effects
        if atk_ability == pkmn_core::abilities::AbilityId::SheerForce
            && !pkmn_core::moves::get_secondaries(move_data.id).is_empty()
        {
            damage = (damage * 5325 + 2047) / 4096;
        }

        // Iron Fist: 1.2x (4915/4096) on punching moves
        if atk_ability == pkmn_core::abilities::AbilityId::IronFist
            && move_data.flags.has(MoveFlags::PUNCH)
        {
            damage = (damage * 4915 + 2047) / 4096;
        }

        // Strong Jaw: 1.5x (6144/4096) on biting moves
        if atk_ability == pkmn_core::abilities::AbilityId::StrongJaw
            && move_data.flags.has(MoveFlags::BITE)
        {
            damage = (damage * 6144 + 2047) / 4096;
        }

        // Life Orb: 1.3x (5324/4096)
        if atk_item == pkmn_core::items::ItemId::LifeOrb {
            damage = (damage * 5324 + 2047) / 4096;
        }

        // Expert Belt: 1.2x (4915/4096) on super effective
        if atk_item == pkmn_core::items::ItemId::ExpertBelt && effectiveness > 1.0 {
            damage = (damage * 4915 + 2047) / 4096;
        }

        damage.max(1) as u16
    }

    fn get_variable_bp(&self, attacker_player: u8, defender_player: u8, move_data: &MoveData) -> u16 {
        let name = move_data.name.to_lowercase();
        let attacker = self.sides[attacker_player as usize].active();
        let defender = self.sides[defender_player as usize].active();
        let mut bp = match name.as_str() {
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
            // Gyro Ball: BP = min(150, floor(25 * target_speed / max(1, user_speed))), min 1
            "gyro ball" => {
                let user_spe = attacker.effective_speed().max(1) as u32;
                let target_spe = defender.effective_speed() as u32;
                ((25 * target_spe / user_spe).clamp(1, 150)) as u16
            }
            // Fishious Rend: doubles BP when attacker moves first
            "fishious rend" | "bolt beak" => {
                if !defender.has_moved_this_turn { 170 } else { 85 }
            }
            // Return: BP = happiness * 2 / 5 (max 102 at 255 happiness)
            "return" => 102,
            _ => move_data.base_power as u16,
        };

        // Grassy Terrain halves Earthquake/Bulldoze/Magnitude BP against grounded targets
        if self.field.terrain == Terrain::Grassy
            && matches!(move_data.name, "Earthquake" | "Bulldoze" | "Magnitude")
        {
            let def_grounded = !defender.types.contains(&Type::Flying)
                && defender.ability_id != pkmn_core::abilities::AbilityId::Levitate
                && defender.item_id != pkmn_core::items::ItemId::AirBalloon;
            if def_grounded { bp /= 2; }
        }

        // Misty Terrain halves Dragon move BP against grounded targets
        if self.field.terrain == Terrain::Misty && move_data.move_type == Type::Dragon {
            let def_grounded = !defender.types.contains(&Type::Flying)
                && defender.ability_id != pkmn_core::abilities::AbilityId::Levitate
                && defender.item_id != pkmn_core::items::ItemId::AirBalloon;
            if def_grounded { bp /= 2; }
        }

        bp
    }

    /// Apply onBasePower modifiers (abilities + items that modify BP in PS).
    /// Uses pokeround: floor((bp * modifier + 2048) / 4096).
    fn apply_bp_modifiers_with_type(&self, attacker_player: u8, move_data: &MoveData, bp: u16, move_type: Type) -> u16 {
        let attacker = self.sides[attacker_player as usize].active();
        let mut bp = bp as u32;

        // Technician: 1.5x (6144/4096) on moves with effective BP <= 60
        if attacker.ability_id == pkmn_core::abilities::AbilityId::Technician
            && bp > 0 && bp <= 60
        {
            bp = (bp * 6144 + 2048) / 4096;
        }

        // Tough Claws: 1.3x (5325/4096) on contact moves
        if attacker.ability_id == pkmn_core::abilities::AbilityId::ToughClaws
            && move_data.flags.has(MoveFlags::CONTACT)
        {
            bp = (bp * 5325 + 2048) / 4096;
        }

        // Muscle Band: 1.1x (4505/4096) on physical moves
        if attacker.item_id == pkmn_core::items::ItemId::MuscleBand
            && move_data.category == MoveCategory::Physical
        {
            bp = (bp * 4505 + 2048) / 4096;
        }
        // Wise Glasses: 1.1x (4505/4096) on special moves
        if attacker.item_id == pkmn_core::items::ItemId::WiseGlasses
            && move_data.category == MoveCategory::Special
        {
            bp = (bp * 4505 + 2048) / 4096;
        }

        // Type-boosting items: 1.2x (4915/4096) for matching move type
        let type_boost = match attacker.item_id {
            pkmn_core::items::ItemId::MysticWater => move_type == Type::Water,
            pkmn_core::items::ItemId::Charcoal => move_type == Type::Fire,
            pkmn_core::items::ItemId::Magnet => move_type == Type::Electric,
            pkmn_core::items::ItemId::MiracleSeed => move_type == Type::Grass,
            pkmn_core::items::ItemId::NeverMeltIce => move_type == Type::Ice,
            pkmn_core::items::ItemId::BlackBelt => move_type == Type::Fighting,
            pkmn_core::items::ItemId::PoisonBarb => move_type == Type::Poison,
            pkmn_core::items::ItemId::SoftSand => move_type == Type::Ground,
            pkmn_core::items::ItemId::SharpBeak => move_type == Type::Flying,
            pkmn_core::items::ItemId::TwistedSpoon => move_type == Type::Psychic,
            pkmn_core::items::ItemId::SilverPowder => move_type == Type::Bug,
            pkmn_core::items::ItemId::HardStone => move_type == Type::Rock,
            pkmn_core::items::ItemId::SpellTag => move_type == Type::Ghost,
            pkmn_core::items::ItemId::DragonFang => move_type == Type::Dragon,
            pkmn_core::items::ItemId::BlackGlasses => move_type == Type::Dark,
            pkmn_core::items::ItemId::MetalCoat => move_type == Type::Steel,
            pkmn_core::items::ItemId::SilkScarf => move_type == Type::Normal,
            pkmn_core::items::ItemId::FairyFeather | pkmn_core::items::ItemId::PixiePlate => move_type == Type::Fairy,
            _ => false,
        };
        if type_boost {
            bp = (bp * 4915 + 2048) / 4096;
        }

        bp as u16
    }

    /// Returns terrain modifier as (numerator, 4096) pair matching PS's chainModify values.
    fn get_terrain_modifier(&self, attacker_player: u8, move_type: Type) -> (u32, u32) {
        let attacker = self.sides[attacker_player as usize].active();
        // Check if grounded (not Flying type, not Levitate, not Air Balloon)
        let is_grounded = !attacker.types.contains(&Type::Flying)
            && attacker.ability_id != pkmn_core::abilities::AbilityId::Levitate
            && attacker.item_id != pkmn_core::items::ItemId::AirBalloon;
        if !is_grounded {
            return (4096, 4096);
        }
        match (self.field.terrain, move_type) {
            (Terrain::Electric, Type::Electric) => (5325, 4096),
            (Terrain::Grassy, Type::Grass) => (5325, 4096),
            (Terrain::Psychic, Type::Psychic) => (5325, 4096),
            _ => (4096, 4096),
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

    pub(crate) fn apply_damage(&mut self, player: u8, damage: u16) {
        let mon = self.sides[player as usize].active_mut();
        mon.hp = mon.hp.saturating_sub(damage);
        if mon.hp == 0 {
            mon.is_fainted = true;
        }
    }

    /// Apply a data-driven boost effect to a player's active Pokemon.
    /// Emits the appropriate protocol messages for each non-zero boost.
    fn apply_boost_effect(&mut self, player: u8, boosts: &crate::events::BoostEffect) {
        let atk_name = self.species_name(player);
        let mon = self.sides[player as usize].active_mut();

        if boosts.atk != 0 {
            let old = mon.boosts.atk;
            mon.boosts.atk = (mon.boosts.atk + boosts.atk).clamp(-6, 6);
            let actual = (mon.boosts.atk - old).unsigned_abs();
            if boosts.atk > 0 {
                self.emit(format!("|-boost|p{}a: {}|atk|{}", player + 1, atk_name, actual));
            } else {
                self.emit(format!("|-unboost|p{}a: {}|atk|{}", player + 1, atk_name, actual));
            }
        }
        if boosts.def != 0 {
            let old = self.sides[player as usize].active().boosts.def;
            self.sides[player as usize].active_mut().boosts.def = (old + boosts.def).clamp(-6, 6);
            let actual = (self.sides[player as usize].active().boosts.def - old).unsigned_abs();
            if boosts.def > 0 {
                self.emit(format!("|-boost|p{}a: {}|def|{}", player + 1, atk_name, actual));
            } else {
                self.emit(format!("|-unboost|p{}a: {}|def|{}", player + 1, atk_name, actual));
            }
        }
        if boosts.spa != 0 {
            let old = self.sides[player as usize].active().boosts.spa;
            self.sides[player as usize].active_mut().boosts.spa = (old + boosts.spa).clamp(-6, 6);
            let actual = (self.sides[player as usize].active().boosts.spa - old).unsigned_abs();
            if boosts.spa > 0 {
                self.emit(format!("|-boost|p{}a: {}|spa|{}", player + 1, atk_name, actual));
            } else {
                self.emit(format!("|-unboost|p{}a: {}|spa|{}", player + 1, atk_name, actual));
            }
        }
        if boosts.spd != 0 {
            let old = self.sides[player as usize].active().boosts.spd;
            self.sides[player as usize].active_mut().boosts.spd = (old + boosts.spd).clamp(-6, 6);
            let actual = (self.sides[player as usize].active().boosts.spd - old).unsigned_abs();
            if boosts.spd > 0 {
                self.emit(format!("|-boost|p{}a: {}|spd|{}", player + 1, atk_name, actual));
            } else {
                self.emit(format!("|-unboost|p{}a: {}|spd|{}", player + 1, atk_name, actual));
            }
        }
        if boosts.spe != 0 {
            let old = self.sides[player as usize].active().boosts.spe;
            self.sides[player as usize].active_mut().boosts.spe = (old + boosts.spe).clamp(-6, 6);
            let actual = (self.sides[player as usize].active().boosts.spe - old).unsigned_abs();
            if boosts.spe > 0 {
                self.emit(format!("|-boost|p{}a: {}|spe|{}", player + 1, atk_name, actual));
            } else {
                self.emit(format!("|-unboost|p{}a: {}|spe|{}", player + 1, atk_name, actual));
            }
        }
    }

    pub fn defender_types(&self, player: u8) -> [Type; 2] {
        let mon = self.sides[player as usize].active();
        let mut types = mon.types;
        if mon.volatiles.contains(Volatiles::ROOST) {
            for t in types.iter_mut() {
                if *t == Type::Flying {
                    *t = Type::Normal;
                }
            }
        }
        types
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

        // Data-driven move effect dispatch (migrated moves skip the string match)
        if let Some(effect) = crate::events::move_effect(name.as_str()) {
            match effect {
                crate::events::MoveEffect::Boost(boosts) => {
                    self.apply_boost_effect(attacker, &boosts);
                    return;
                }
                crate::events::MoveEffect::Hazard(crate::events::HazardType::StealthRock) => {
                    if self.sides[defender as usize].side_conditions.stealth_rock {
                        let atk_name = self.species_name(attacker);
                        self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                    } else {
                        self.sides[defender as usize].side_conditions.stealth_rock = true;
                        self.emit(format!("|-sidestart|p{}: Player {}|move: Stealth Rock", defender+1, defender+1));
                    }
                    return;
                }
            }
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
                } else {
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-fail|p{}a: {}|tox", defender+1, def_name));
                }
            }
            "will-o-wisp" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    if def_mon.types.contains(&Type::Fire) {
                        let def_name = self.species_name(defender);
                        self.emit(format!("|-immune|p{}a: {}", defender+1, def_name));
                    } else {
                        def_mon.status = Status::Burn;
                        let def_name = self.species_name(defender);
                        self.emit(format!("|-status|p{}a: {}|brn", defender+1, def_name));
                    }
                } else {
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-fail|p{}a: {}|brn", defender+1, def_name));
                }
            }
            "thunder wave" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.status == Status::None {
                    if def_mon.types.contains(&Type::Electric) || def_mon.types.contains(&Type::Ground) {
                        let def_name = self.species_name(defender);
                        self.emit(format!("|-immune|p{}a: {}", defender+1, def_name));
                    } else {
                        def_mon.status = Status::Paralyze;
                        let def_name = self.species_name(defender);
                        self.emit(format!("|-status|p{}a: {}|par", defender+1, def_name));
                    }
                } else {
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-fail|p{}a: {}|par", defender+1, def_name));
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
            // Boost moves (swords dance, dragon dance, calm mind now data-driven via events.rs)
            "iron defense" | "acid armor" => {
                let mon = self.sides[attacker as usize].active_mut();
                let old = mon.boosts.def;
                mon.boosts.def = (mon.boosts.def + 2).min(6);
                let actual = mon.boosts.def - old;
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|def|{}", attacker+1, atk_name, actual));
            }
            "nasty plot" => {
                let mon = self.sides[attacker as usize].active_mut();
                let old = mon.boosts.spa;
                mon.boosts.spa = (mon.boosts.spa + 2).min(6);
                let actual = mon.boosts.spa - old;
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spa|{}", attacker+1, atk_name, actual));
            }
            "agility" | "rock polish" => {
                let mon = self.sides[attacker as usize].active_mut();
                let old = mon.boosts.spe;
                mon.boosts.spe = (mon.boosts.spe + 2).min(6);
                let actual = mon.boosts.spe - old;
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spe|{}", attacker+1, atk_name, actual));
            }
            "shift gear" => {
                let (actual_spe, actual_atk) = {
                    let mon = self.sides[attacker as usize].active_mut();
                    let old_spe = mon.boosts.spe;
                    let old_atk = mon.boosts.atk;
                    mon.boosts.spe = (mon.boosts.spe + 2).min(6);
                    mon.boosts.atk = (mon.boosts.atk + 1).min(6);
                    (mon.boosts.spe - old_spe, mon.boosts.atk - old_atk)
                };
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spe|{}", attacker+1, atk_name, actual_spe));
                self.emit(format!("|-boost|p{}a: {}|atk|{}", attacker+1, atk_name, actual_atk));
            }
            "quiver dance" => {
                let (actual_spa, actual_spd, actual_spe) = {
                    let mon = self.sides[attacker as usize].active_mut();
                    let old_spa = mon.boosts.spa;
                    let old_spd = mon.boosts.spd;
                    let old_spe = mon.boosts.spe;
                    mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                    mon.boosts.spd = (mon.boosts.spd + 1).min(6);
                    mon.boosts.spe = (mon.boosts.spe + 1).min(6);
                    (mon.boosts.spa - old_spa, mon.boosts.spd - old_spd, mon.boosts.spe - old_spe)
                };
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-boost|p{}a: {}|spa|{}", attacker+1, atk_name, actual_spa));
                self.emit(format!("|-boost|p{}a: {}|spd|{}", attacker+1, atk_name, actual_spd));
                self.emit(format!("|-boost|p{}a: {}|spe|{}", attacker+1, atk_name, actual_spe));
            }
            "shell smash" => {
                let (actual_atk, actual_spa, actual_spe, actual_def, actual_spd) = {
                    let mon = self.sides[attacker as usize].active_mut();
                    let old_atk = mon.boosts.atk;
                    let old_spa = mon.boosts.spa;
                    let old_spe = mon.boosts.spe;
                    let old_def = mon.boosts.def;
                    let old_spd = mon.boosts.spd;
                    mon.boosts.atk = (mon.boosts.atk + 2).min(6);
                    mon.boosts.spa = (mon.boosts.spa + 2).min(6);
                    mon.boosts.spe = (mon.boosts.spe + 2).min(6);
                    mon.boosts.def = (mon.boosts.def - 1).max(-6);
                    mon.boosts.spd = (mon.boosts.spd - 1).max(-6);
                    (mon.boosts.atk - old_atk, mon.boosts.spa - old_spa, mon.boosts.spe - old_spe,
                     (old_def - mon.boosts.def).unsigned_abs(), (old_spd - mon.boosts.spd).unsigned_abs())
                };
                let atk_name = self.species_name(attacker);
                self.emit(format!("|-unboost|p{}a: {}|def|{}", attacker+1, atk_name, actual_def));
                self.emit(format!("|-unboost|p{}a: {}|spd|{}", attacker+1, atk_name, actual_spd));
                self.emit(format!("|-boost|p{}a: {}|atk|{}", attacker+1, atk_name, actual_atk));
                self.emit(format!("|-boost|p{}a: {}|spa|{}", attacker+1, atk_name, actual_spa));
                self.emit(format!("|-boost|p{}a: {}|spe|{}", attacker+1, atk_name, actual_spe));
            }
            "reflect" => {
                self.sides[attacker as usize].side_conditions.reflect = 5;
                self.emit(format!("|-sidestart|p{}: Player {}|Reflect", attacker+1, attacker+1));
            }
            "light screen" => {
                self.sides[attacker as usize].side_conditions.light_screen = 5;
                self.emit(format!("|-sidestart|p{}: Player {}|move: Light Screen", attacker+1, attacker+1));
            }
            // stealth rock is now data-driven via events.rs
            "toxic spikes" => {
                let layers = &mut self.sides[defender as usize].side_conditions.toxic_spikes;
                if *layers < 2 {
                    *layers += 1;
                    self.emit(format!("|-sidestart|p{}: Player {}|move: Toxic Spikes", defender+1, defender+1));
                } else {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                }
            }
            "sticky web" => {
                if self.sides[defender as usize].side_conditions.sticky_web {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                } else {
                    self.sides[defender as usize].side_conditions.sticky_web = true;
                    self.emit(format!("|-sidestart|p{}: Player {}|move: Sticky Web", defender+1, defender+1));
                }
            }
            "recover" | "soft-boiled" | "roost" | "slack off" | "moonlight" | "synthesis" | "morning sun" => {
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
                    if name == "roost" {
                        self.emit(format!("|-singleturn|p{}a: {}|move: Roost", attacker+1, atk_name));
                        self.sides[attacker as usize].active_mut().volatiles.insert(Volatiles::ROOST);
                    }
                }
            }
            "belly drum" => {
                let mon = self.sides[attacker as usize].active_mut();
                let cost = mon.max_hp / 2;
                if mon.hp > cost {
                    mon.hp -= cost;
                    mon.boosts.atk = 6;
                    let atk_name = self.species_name(attacker);
                    let hp = self.sides[attacker as usize].active().hp;
                    let max_hp = self.sides[attacker as usize].active().max_hp;
                    self.emit(format!("|-damage|p{}a: {}|{}/{}", attacker+1, atk_name, hp, max_hp));
                    self.emit(format!("|-setboost|p{}a: {}|atk|6|[from] move: Belly Drum", attacker+1, atk_name));
                    // Trigger Sitrus Berry if applicable
                    if self.sides[attacker as usize].active().item_id == pkmn_core::items::ItemId::SitrusBerry {
                        let mon = self.sides[attacker as usize].active_mut();
                        let heal = mon.max_hp / 4;
                        mon.hp = (mon.hp + heal).min(mon.max_hp);
                        mon.item_id = pkmn_core::items::ItemId::None;
                        let atk_name = self.species_name(attacker);
                        self.emit(format!("|-enditem|p{}a: {}|Sitrus Berry|[eat]", attacker+1, atk_name));
                        let hp = self.sides[attacker as usize].active().hp;
                        let max_hp = self.sides[attacker as usize].active().max_hp;
                        self.emit(format!("|-heal|p{}a: {}|{}/{}|[from] item: Sitrus Berry", attacker+1, atk_name, hp, max_hp));
                    }
                } else {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}|move: Belly Drum", attacker+1, atk_name));
                }
            }
            "leech seed" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if def_mon.types.contains(&Type::Grass) {
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-immune|p{}a: {}", defender+1, def_name));
                } else if def_mon.volatiles.contains(Volatiles::LEECH_SEED) {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                } else {
                    def_mon.volatiles.insert(Volatiles::LEECH_SEED);
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-start|p{}a: {}|move: Leech Seed", defender+1, def_name));
                }
            }
            "defog" => {
                // Lower target's evasion by 1
                let def_name = self.species_name(defender);
                let mon = self.sides[defender as usize].active_mut();
                mon.boosts.evasion = (mon.boosts.evasion - 1).max(-6);
                self.emit(format!("|-unboost|p{}a: {}|evasion|1", defender+1, def_name));
                // Remove hazards from both sides
                let atk_name = self.species_name(attacker);
                for side_idx in 0..2u8 {
                    let has_sr = self.sides[side_idx as usize].side_conditions.stealth_rock;
                    let has_spikes = self.sides[side_idx as usize].side_conditions.spikes > 0;
                    let has_tspikes = self.sides[side_idx as usize].side_conditions.toxic_spikes > 0;
                    let has_web = self.sides[side_idx as usize].side_conditions.sticky_web;
                    if has_sr {
                        self.sides[side_idx as usize].side_conditions.stealth_rock = false;
                        self.emit(format!("|-sideend|p{}: Player {}|Stealth Rock|[from] move: Defog|[of] p{}a: {}", side_idx+1, side_idx+1, attacker+1, atk_name));
                    }
                    if has_spikes {
                        self.sides[side_idx as usize].side_conditions.spikes = 0;
                        self.emit(format!("|-sideend|p{}: Player {}|Spikes|[from] move: Defog|[of] p{}a: {}", side_idx+1, side_idx+1, attacker+1, atk_name));
                    }
                    if has_tspikes {
                        self.sides[side_idx as usize].side_conditions.toxic_spikes = 0;
                        self.emit(format!("|-sideend|p{}: Player {}|Toxic Spikes|[from] move: Defog|[of] p{}a: {}", side_idx+1, side_idx+1, attacker+1, atk_name));
                    }
                    if has_web {
                        self.sides[side_idx as usize].side_conditions.sticky_web = false;
                        self.emit(format!("|-sideend|p{}: Player {}|Sticky Web|[from] move: Defog|[of] p{}a: {}", side_idx+1, side_idx+1, attacker+1, atk_name));
                    }
                }
            }
            "pain split" => {
                let atk_hp = self.sides[attacker as usize].active().hp;
                let def_hp = self.sides[defender as usize].active().hp;
                let avg = (atk_hp as u32 + def_hp as u32) / 2;
                let atk_max = self.sides[attacker as usize].active().max_hp;
                let def_max = self.sides[defender as usize].active().max_hp;
                self.sides[attacker as usize].active_mut().hp = (avg as u16).min(atk_max);
                self.sides[defender as usize].active_mut().hp = (avg as u16).min(def_max);
            }
            _ => {}
        }

        // Reset protect consecutive if not using protect
        if name != "protect" && name != "detect" {
            self.sides[attacker as usize].active_mut().protect_consecutive = 0;
        }
    }

    pub fn end_of_turn(&mut self) {
        // PS processes EOT effects in speed order (faster first)
        let order: [u8; 2] = if self.sides[0].active().effective_speed() >= self.sides[1].active().effective_speed() {
            [0, 1]
        } else {
            [1, 0]
        };

        for &player in &order {
            self.trigger_ability_end_of_turn(player);
        }

        // Weather upkeep: decrement first, then emit (PS order)
        if self.field.weather != Weather::None && self.field.weather_turns > 0 {
            self.field.weather_turns -= 1;
            if self.field.weather_turns == 0 {
                self.field.weather = Weather::None;
                self.emit("|-weather|none".to_string());
            } else {
                let weather_str = match self.field.weather {
                    Weather::Rain => "RainDance",
                    Weather::Sun => "SunnyDay",
                    Weather::Sand => "Sandstorm",
                    Weather::Snow => "Snowscape",
                    _ => "none",
                };
                self.emit(format!("|-weather|{}|[upkeep]", weather_str));
            }
        }

        // Sandstorm damage
        for &player in &order {
            let mon = self.sides[player as usize].active_mut();
            if !mon.is_alive() { continue; }
            if self.field.weather == Weather::Sand
                && !mon.types.contains(&Type::Rock)
                && !mon.types.contains(&Type::Ground)
                && !mon.types.contains(&Type::Steel)
            {
                let dmg = (mon.max_hp / 16).max(1);
                mon.hp = mon.hp.saturating_sub(dmg);
                if mon.hp == 0 { mon.is_fainted = true; }
                let name = pkmn_core::species::get_species_by_id(mon.species_id).map(|s| s.name).unwrap_or("Unknown");
                let hp = mon.hp;
                let max_hp = mon.max_hp;
                if hp == 0 {
                    self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] Sandstorm", player+1, name));
                    self.emit(format!("|faint|p{}a: {}", player+1, name));
                } else {
                    let status_str = match mon.status {
                        Status::Toxic => " tox", Status::Burn => " brn", Status::Poison => " psn", Status::Paralyze => " par", _ => "",
                    };
                    self.emit(format!("|-damage|p{}a: {}|{}/{}{}|[from] Sandstorm", player+1, name, hp, max_hp, status_str));
                }
            }
        }

        // Grassy Terrain EOT heal (1/16 HP to grounded Pokemon)
        if self.field.terrain == Terrain::Grassy {
            for &player in &order {
                let mon = self.sides[player as usize].active();
                if !mon.is_alive() { continue; }
                let is_grounded = !mon.types.contains(&Type::Flying)
                    && mon.ability_id != pkmn_core::abilities::AbilityId::Levitate
                    && mon.item_id != pkmn_core::items::ItemId::AirBalloon;
                if is_grounded && mon.hp < mon.max_hp {
                    let heal = (mon.max_hp / 16).max(1);
                    let mon = self.sides[player as usize].active_mut();
                    mon.hp = (mon.hp + heal).min(mon.max_hp);
                    let name = self.species_name(player);
                    let hp = self.sides[player as usize].active().hp;
                    let max_hp = self.sides[player as usize].active().max_hp;
                    self.emit(format!("|-heal|p{}a: {}|{}/{}|[from] Grassy Terrain", player+1, name, hp, max_hp));
                }
            }
        }

        // Item end-of-turn: healing items (Leftovers, Black Sludge) — PS residualOrder 5
        // Dispatched via on_residual hooks for migrated items, fallback for others.
        let p1_speed = self.sides[0].active().effective_speed();
        let p2_speed = self.sides[1].active().effective_speed();
        let item_order: [u8; 2] = if p2_speed > p1_speed { [1, 0] } else { [0, 1] };
        for &player in &item_order {
            let item_id = self.sides[player as usize].active().item_id;
            let hooks = crate::events::item_hooks(item_id);
            if let Some(hook) = hooks.on_residual {
                hook(self, player);
            } else {
                self.trigger_item_heal_eot(player);
            }
        }

        // Leech Seed end-of-turn: drain 1/8 HP from seeded mon, heal the seeder
        for &player in &order {
            let seeded = self.sides[player as usize].active().volatiles.contains(Volatiles::LEECH_SEED);
            if !seeded || !self.sides[player as usize].active().is_alive() { continue; }
            let max_hp = self.sides[player as usize].active().max_hp;
            let dmg = (max_hp / 8).max(1);
            self.apply_damage(player, dmg);
            let name = self.species_name(player);
            let other = 1 - player;
            let other_name = self.species_name(other);
            let hp = self.sides[player as usize].active().hp;
            let max_hp = self.sides[player as usize].active().max_hp;
            let status_str = match self.sides[player as usize].active().status {
                Status::Toxic => " tox", Status::Burn => " brn", Status::Poison => " psn", Status::Paralyze => " par", _ => "",
            };
            let fainted = hp == 0;
            if fainted {
                self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] Leech Seed|[of] p{}a: {}", player+1, name, other+1, other_name));
            } else {
                self.emit(format!("|-damage|p{}a: {}|{}/{}{}|[from] Leech Seed|[of] p{}a: {}", player+1, name, hp, max_hp, status_str, other+1, other_name));
            }
            // Heal the seeder
            if self.sides[other as usize].active().is_alive() {
                let other_mon = self.sides[other as usize].active_mut();
                if other_mon.hp < other_mon.max_hp {
                    other_mon.hp = (other_mon.hp + dmg).min(other_mon.max_hp);
                    let other_name = self.species_name(other);
                    let ohp = self.sides[other as usize].active().hp;
                    let omax = self.sides[other as usize].active().max_hp;
                    self.emit(format!("|-heal|p{}a: {}|{}/{}|[silent]", other+1, other_name, ohp, omax));
                }
            }
            // Emit faint after heal
            if fainted {
                let name = self.species_name(player);
                self.emit(format!("|faint|p{}a: {}", player+1, name));
            }
        }

        // Status damage (burn/poison/toxic)
        for &player in &order {
            let mon = self.sides[player as usize].active_mut();
            if !mon.is_alive() { continue; }
            let status = mon.status;
            match status {
                Status::Burn => {
                    let dmg = (mon.max_hp / 16).max(1);
                    mon.hp = mon.hp.saturating_sub(dmg);
                    if mon.hp == 0 { mon.is_fainted = true; }
                }
                Status::Poison => {
                    let dmg = (mon.max_hp / 8).max(1);
                    mon.hp = mon.hp.saturating_sub(dmg);
                    if mon.hp == 0 { mon.is_fainted = true; }
                }
                Status::Toxic => {
                    mon.status_turns += 1;
                    let dmg = (mon.max_hp / 16).max(1) * mon.status_turns as u16;
                    mon.hp = mon.hp.saturating_sub(dmg);
                    if mon.hp == 0 { mon.is_fainted = true; }
                }
                _ => {}
            }
            let name = pkmn_core::species::get_species_by_id(mon.species_id).map(|s| s.name).unwrap_or("Unknown");
            let hp = mon.hp;
            let max_hp = mon.max_hp;
            let is_fainted = mon.is_fainted;
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

        // Flame Orb / Toxic Orb: apply status after status damage
        for &player in &item_order {
            self.trigger_item_orb_eot(player);
        }

        // Decrement field turns
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
            side.active_mut().volatiles.remove(Volatiles::ROOST);
            side.active_mut().volatiles.remove(Volatiles::FLINCH);
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
        // Use queued pivot target if available, otherwise pick first alive
        let target = if !self.pivot_switch_targets[player as usize].is_empty() {
            self.pivot_switch_targets[player as usize].remove(0)
        } else {
            let current = self.sides[player as usize].active_index;
            self.sides[player as usize].team.iter().enumerate()
                .find(|(i, p)| *i != current && p.is_alive())
                .map(|(i, _)| i as u8)
                .unwrap_or(0)
        };
        self.execute_switch_from(player, target, Some(move_data.name));
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
            // 2-5 hit distribution
            let has_loaded_dice = self.sides[player as usize].active().item_id == pkmn_core::items::ItemId::LoadedDice;
            if has_loaded_dice {
                // Loaded Dice: 4 or 5 hits
                let roll = self.random(2);
                roll + 4
            } else {
                // 35/35/15/15 distribution via sample of 20
                let roll = self.random(20);
                if roll < 7 { 2 } else if roll < 14 { 3 } else if roll < 17 { 4 } else { 5 }
            }
        } else {
            max_hits as u32
        };

        let mut actual_hits: u32 = 0;
        for _hit_num in 0..hits {
            // Crit stage table: 0 → 1/24, 1 → 1/8, 2 → 1/2, 3+ → always
            let crit_denom = match move_data.crit_ratio {
                0 => 24,
                1 => 8,
                2 => 2,
                _ => 1,
            };
            let critical = self.random_chance(1, crit_denom);
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
                if critical {
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-crit|p{}a: {}", defender+1, def_name));
                }
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
            // Simultaneous faint: attacker wins (defender fainted first)
            if let Some(attacker) = self.last_attacker {
                self.result = BattleResult::Win(attacker);
            } else {
                self.result = BattleResult::Tie;
            }
        } else if p1_alive == 0 {
            self.result = BattleResult::Win(1);
        } else if p2_alive == 0 {
            self.result = BattleResult::Win(0);
        }
    }
}
