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
        mon.taunt_turns = 0;
        mon.encore_turns = 0;
        mon.disable_turns = 0;
        mon.trap_turns = 0;
        mon.trap_move_id = 0;

        // Trapper leaving: release opponent's partial trap
        let opp = 1 - player;
        if self.sides[opp as usize].active().volatiles.contains(Volatiles::TRAPPED) {
            let opp_mon = self.sides[opp as usize].active_mut();
            opp_mon.volatiles.remove(Volatiles::TRAPPED);
            opp_mon.trap_turns = 0;
            opp_mon.trap_move_id = 0;
        }

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

        // Struggle: typeless 50 BP physical, 1/4 max HP recoil, ignores type immunity
        if move_idx == 255 {
            let atk_name = self.species_name(player);
            let def_name = self.species_name(1 - player);
            self.emit(format!("|move|p{}a: {}|Struggle|p{}a: {}", player+1, atk_name, 2-player, def_name));
            let attacker_mon = self.sides[player as usize].active();
            let level = attacker_mon.level as u32;
            let atk = attacker_mon.effective_atk() as u32;
            let def = self.sides[(1-player) as usize].active().effective_def() as u32;
            let damage = ((2 * level / 5 + 2) * 50 * atk / def / 50 + 2) as u16;
            let defender_idx = 1 - player;
            self.apply_damage(defender_idx, damage);
            let def_name = self.species_name(defender_idx);
            let hp_str = self.hp_display(defender_idx);
            self.emit(format!("|-damage|p{}a: {}|{}", defender_idx+1, def_name, hp_str));
            if self.sides[defender_idx as usize].active().hp == 0 {
                self.emit(format!("|faint|p{}a: {}", defender_idx+1, def_name));
            }
            // 1/4 max HP recoil to user
            let max_hp = self.sides[player as usize].active().max_hp;
            let recoil = (max_hp / 4).max(1);
            self.apply_damage(player, recoil);
            let atk_name = self.species_name(player);
            let hp_str = self.hp_display(player);
            if self.sides[player as usize].active().hp == 0 {
                self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] Recoil", player+1, atk_name));
                self.emit(format!("|faint|p{}a: {}", player+1, atk_name));
            } else {
                self.emit(format!("|-damage|p{}a: {}|{}|[from] Recoil", player+1, atk_name, hp_str));
            }
            return;
        }

        // Must recharge: skip turn, emit cant, and clear
        if self.sides[player as usize]
            .active()
            .volatiles
            .contains(Volatiles::MUST_RECHARGE)
        {
            let name = self.species_name(player);
            self.emit(format!("|cant|p{}a: {}|recharge", player+1, name));
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

        // Sleep check: decrement counter, wake if 0, otherwise can't move
        if self.sides[player as usize].active().status == Status::Sleep {
            let mon = self.sides[player as usize].active_mut();
            mon.status_turns = mon.status_turns.saturating_sub(1);
            if mon.status_turns == 0 {
                mon.status = Status::None;
                let name = self.species_name(player);
                self.emit(format!("|-curestatus|p{}a: {}|slp|[msg]", player+1, name));
                // Decrement sleep clause counter when waking
                self.sides[player as usize].sleep_clause_count =
                    self.sides[player as usize].sleep_clause_count.saturating_sub(1);
                // Pokemon wakes and may act this turn — continue execution
            } else {
                let name = self.species_name(player);
                self.emit(format!("|cant|p{}a: {}|slp", player+1, name));
                return;
            }
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

        // Deduct PP (skip for locked move continuations and charge/semi-invulnerable turn 2)
        let is_locked_continuation = self.sides[player as usize]
            .active()
            .volatiles
            .contains(Volatiles::LOCKED_MOVE);
        let is_charge_turn2 = self.sides[player as usize]
            .active()
            .volatiles
            .intersects(Volatiles::CHARGING | Volatiles::SEMI_INVULNERABLE);
        if !is_locked_continuation && !is_charge_turn2 {
            self.sides[player as usize].active_mut().moves[move_idx as usize].pp = self.sides
                [player as usize]
                .active_mut()
                .moves[move_idx as usize]
                .pp
                .saturating_sub(1);
            // Track last used move for Encore/Disable targeting
            self.sides[player as usize].active_mut().last_used_move_idx = move_idx;
        }

        // --- Multi-turn move state machine: charge / semi-invulnerable ---
        // Turn 2: clear charging state and proceed with normal execution
        if is_charge_turn2 {
            let mon = self.sides[player as usize].active_mut();
            mon.volatiles.remove(Volatiles::CHARGING);
            mon.volatiles.remove(Volatiles::SEMI_INVULNERABLE);
        } else {
            // Turn 1: check if this is a charge/semi-invulnerable move that needs setup
            let charge_result = self.check_charge_turn1(player, move_idx, &move_data);
            if charge_result {
                return; // Charge turn consumed; move executes next turn
            }
        }

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

        // Semi-invulnerable: most moves auto-miss against a semi-invulnerable target
        // TODO: Earthquake hits Dig, Surf hits Dive, Thunder/Hurricane hit Fly/Bounce, etc.
        if self.sides[defender_idx as usize]
            .active()
            .volatiles
            .contains(Volatiles::SEMI_INVULNERABLE)
            && move_data.category != MoveCategory::Status
        {
            let atk_name = self.species_name(player);
            let def_name = self.species_name(defender_idx);
            self.emit(format!("|move|p{}a: {}|{}|p{}a: {}|[miss]", player+1, atk_name, move_data.name, defender_idx+1, def_name));
            self.emit(format!("|-miss|p{}a: {}|p{}a: {}", player+1, atk_name, defender_idx+1, def_name));
            return;
        }

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
                | "grass whistle"
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
            // Interrupt breaks locking moves (no confusion)
            self.break_lock(player);
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
                self.break_lock(player);
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
            self.break_lock(player);
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

            // Drain moves — heal attacker by fraction of damage dealt (data-driven)
            let move_name_lower2 = move_data.name.to_lowercase();
            if let Some(effect) = crate::events::damaging_self_effect(move_name_lower2.as_str()) {
                if effect.drain.1 > 0 {
                    // PS rounding: floor(damage * num / denom), min 1
                    let heal = ((damage as u32 * effect.drain.0 as u32) / effect.drain.1 as u32).max(1) as u16;
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

            // Self-stat drops (data-driven) — emit BEFORE contact recoil per PS protocol
            {
                let move_name_lower3 = move_data.name.to_lowercase();
                if let Some(effect) = crate::events::damaging_self_effect(move_name_lower3.as_str()) {
                    if let Some(ref boosts) = effect.self_boosts {
                        self.apply_boost_effect(player, boosts);
                    }
                }
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

        // Move recoil (data-driven) — Rock Head / Magic Guard negate
        let move_name_lower = move_data.name.to_lowercase();
        if let Some(effect) = crate::events::damaging_self_effect(move_name_lower.as_str()) {
            if effect.recoil.1 > 0 {
                let attacker_ability = self.sides[player as usize].active().ability_id;
                let negated = attacker_ability == pkmn_core::abilities::AbilityId::RockHead
                    || attacker_ability == pkmn_core::abilities::AbilityId::MagicGuard;
                if !negated {
                    // PS rounding: floor(damage * num / denom), min 1
                    let recoil = ((damage as u32 * effect.recoil.0 as u32) / effect.recoil.1 as u32).max(1) as u16;
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
            }
        }

        // Drain moves (Drain Punch, Giga Drain, etc.) — heal 1/2 of damage dealt
        // (handled inside the damage application block above)

        // Life Orb recoil (skipped by Sheer Force when move has secondaries)
        if !sheer_force_active {
            self.apply_life_orb_recoil(player);
        }

        // Partial trapping moves: trap the target for 4-5 turns
        if self.sides[defender_idx as usize].active().is_alive() {
            let mn = move_data.name.to_lowercase();
            let is_trapping = matches!(mn.as_str(),
                "bind" | "wrap" | "fire spin" | "whirlpool" | "sand tomb" | "magma storm" | "infestation"
            );
            if is_trapping && !self.sides[defender_idx as usize].active().volatiles.contains(Volatiles::TRAPPED) {
                let turns = if self.random(2) == 0 { 4 } else { 5 };
                let def_mon = self.sides[defender_idx as usize].active_mut();
                def_mon.volatiles.insert(Volatiles::TRAPPED);
                def_mon.trap_turns = turns;
                def_mon.trap_move_id = move_data.id;
                let def_name = self.species_name(defender_idx);
                self.emit(format!("|-activate|p{}a: {}|move: {}", defender_idx+1, def_name, move_data.name));
            }
        }

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

    /// Break a locking move early (no confusion). Called when the move is interrupted.
    fn break_lock(&mut self, player: u8) {
        let mon = self.sides[player as usize].active_mut();
        if mon.volatiles.contains(Volatiles::LOCKED_MOVE) {
            mon.volatiles.remove(Volatiles::LOCKED_MOVE);
            mon.locked_move_turns = 0;
        }
    }

    fn handle_recharge_move(&mut self, player: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();
        match name.as_str() {
            "hyper beam" | "giga impact" | "blast burn" | "frenzy plant" | "hydro cannon"
            | "eternabeam" | "prismatic laser" | "roar of time" => {
                self.sides[player as usize]
                    .active_mut()
                    .volatiles
                    .insert(Volatiles::MUST_RECHARGE);
            }
            _ => {}
        }
    }

    /// Check if a move is a charge or semi-invulnerable move on turn 1.
    /// Returns true if the charge turn was consumed (caller should return).
    /// Handles Power Herb skip and Solar Beam/Blade sun skip.
    fn check_charge_turn1(&mut self, player: u8, move_idx: u8, move_data: &MoveData) -> bool {
        let name = move_data.name.to_lowercase();
        let is_charge = matches!(name.as_str(), "solar beam" | "solar blade" | "meteor beam");
        let is_semi_inv = matches!(name.as_str(), "fly" | "bounce" | "dig" | "dive" | "phantom force" | "shadow force");
        if !is_charge && !is_semi_inv {
            return false;
        }

        // Check if charge can be skipped: Sun for Solar Beam/Blade, Power Herb for all
        let skip_charge = if matches!(name.as_str(), "solar beam" | "solar blade")
            && self.field.weather == Weather::Sun
        {
            true
        } else {
            self.sides[player as usize].active().item_id == pkmn_core::items::ItemId::PowerHerb
        };

        if skip_charge {
            // Consume Power Herb if that's what skipped it
            if self.sides[player as usize].active().item_id == pkmn_core::items::ItemId::PowerHerb
                && !(matches!(name.as_str(), "solar beam" | "solar blade") && self.field.weather == Weather::Sun)
            {
                self.sides[player as usize].active_mut().item_id = pkmn_core::items::ItemId::None;
                let atk_name = self.species_name(player);
                self.emit(format!("|-enditem|p{}a: {}|Power Herb", player+1, atk_name));
            }
            // For Meteor Beam, the SpA boost still happens on the skip turn
            if name == "meteor beam" {
                let atk_name = self.species_name(player);
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                self.emit(format!("|-boost|p{}a: {}|spa|1", player+1, atk_name));
            }
            return false; // Proceed with normal execution (no charge turn)
        }

        // Turn 1: set up the charge state
        let atk_name = self.species_name(player);
        let defender_idx = 1 - player;
        let def_name = self.species_name(defender_idx);

        if is_semi_inv {
            // Semi-invulnerable: emit move line then prepare message
            self.emit(format!("|move|p{}a: {}|{}|p{}a: {}", player+1, atk_name, move_data.name, defender_idx+1, def_name));
            self.emit(format!("|-prepare|p{}a: {}|{}", player+1, atk_name, move_data.name));
            let mon = self.sides[player as usize].active_mut();
            mon.volatiles.insert(Volatiles::SEMI_INVULNERABLE);
            mon.charging_move_idx = move_idx;
        } else {
            // Charge move: emit move line then prepare message
            self.emit(format!("|move|p{}a: {}|{}|p{}a: {}", player+1, atk_name, move_data.name, defender_idx+1, def_name));
            self.emit(format!("|-prepare|p{}a: {}|{}", player+1, atk_name, move_data.name));
            // Meteor Beam: +1 SpA on charge turn
            if name == "meteor beam" {
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.spa = (mon.boosts.spa + 1).min(6);
                let atk_name2 = self.species_name(player);
                self.emit(format!("|-boost|p{}a: {}|spa|1", player+1, atk_name2));
            }
            let mon = self.sides[player as usize].active_mut();
            mon.volatiles.insert(Volatiles::CHARGING);
            mon.charging_move_idx = move_idx;
        }
        true
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

        // -ate abilities: Normal moves become a different type + 1.2x (4915/4096)
        let (move_type, ate_boost) = if move_type == Type::Normal {
            match atk_ability {
                pkmn_core::abilities::AbilityId::Pixilate => (Type::Fairy, true),
                pkmn_core::abilities::AbilityId::Refrigerate => (Type::Ice, true),
                pkmn_core::abilities::AbilityId::Aerilate => (Type::Flying, true),
                pkmn_core::abilities::AbilityId::Galvanize => (Type::Electric, true),
                _ => (move_type, false),
            }
        } else {
            (move_type, false)
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

        // -ate abilities: 1.2x (4915/4096) BP boost when type was changed from Normal
        let base_power = if ate_boost {
            ((base_power as u32 * 4915 + 2048) / 4096) as u16
        } else {
            base_power
        };

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

        // Punk Rock: 0.5x (2048/4096) on sound moves (defender takes less)
        if def_ability == pkmn_core::abilities::AbilityId::PunkRock
            && move_data.flags.has(MoveFlags::SOUND)
        {
            damage = (damage * 2048 + 2047) / 4096;
        }

        // Steelworker / Steely Spirit: 1.5x (6144/4096) on Steel moves
        if (atk_ability == pkmn_core::abilities::AbilityId::Steelworker
            || atk_ability == pkmn_core::abilities::AbilityId::SteelySpirit)
            && move_type == Type::Steel
        {
            damage = (damage * 6144 + 2047) / 4096;
        }

        // Water Bubble: 2x (8192/4096) on Water moves (attacker)
        if atk_ability == pkmn_core::abilities::AbilityId::WaterBubble
            && move_type == Type::Water
        {
            damage = (damage * 8192 + 2047) / 4096;
        }

        // Water Bubble: 0.5x (2048/4096) on Fire moves (defender takes less)
        if def_ability == pkmn_core::abilities::AbilityId::WaterBubble
            && move_type == Type::Fire
        {
            damage = (damage * 2048 + 2047) / 4096;
        }

        // Neuroforce: 1.25x (5120/4096) on super-effective hits
        if atk_ability == pkmn_core::abilities::AbilityId::Neuroforce
            && effectiveness > 1.0
        {
            damage = (damage * 5120 + 2047) / 4096;
        }

        // Analytic: 1.3x (5325/4096) when user moves last
        // Uses has_moved_this_turn: if defender already moved, attacker is slower/moved last
        if atk_ability == pkmn_core::abilities::AbilityId::Analytic
            && defender.has_moved_this_turn
        {
            damage = (damage * 5325 + 2047) / 4096;
        }

        // TODO: Stakeout: 2x (8192/4096) when target switched in this turn
        // Requires tracking switched_in_this_turn on Pokemon (not yet available)

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

        // Solar Beam/Blade: 0.5x BP in Rain/Sand/Snow (weather weakening)
        if matches!(name.as_str(), "solar beam" | "solar blade")
            && matches!(self.field.weather, Weather::Rain | Weather::Sand | Weather::Snow)
        {
            bp /= 2;
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

        // Sharpness: 1.5x (6144/4096) on slicing moves (onBasePower in PS)
        if attacker.ability_id == pkmn_core::abilities::AbilityId::Sharpness
            && move_data.flags.has(MoveFlags::SLICING)
        {
            bp = (bp * 6144 + 2048) / 4096;
        }

        // Reckless: 1.2x (4915/4096) on recoil moves (onBasePower in PS)
        if attacker.ability_id == pkmn_core::abilities::AbilityId::Reckless
            && pkmn_core::moves::is_recoil_move(move_data.id)
        {
            bp = (bp * 4915 + 2048) / 4096;
        }

        // Mega Launcher: 1.5x (6144/4096) on pulse/aura moves (onBasePower in PS)
        if attacker.ability_id == pkmn_core::abilities::AbilityId::MegaLauncher
            && move_data.flags.has(MoveFlags::PULSE)
        {
            bp = (bp * 6144 + 2048) / 4096;
        }

        // Punk Rock: 1.3x (5325/4096) on sound moves (onBasePower in PS)
        if attacker.ability_id == pkmn_core::abilities::AbilityId::PunkRock
            && move_data.flags.has(MoveFlags::SOUND)
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
    /// Emits unboosts first, then boosts (PS protocol order for mixed moves like Shell Smash).
    fn apply_boost_effect(&mut self, player: u8, boosts: &crate::events::BoostEffect) {
        let atk_name = self.species_name(player);

        // Stat order for iteration: atk, def, spa, spd, spe
        let stats: [(i8, &str); 5] = [
            (boosts.atk, "atk"),
            (boosts.def, "def"),
            (boosts.spa, "spa"),
            (boosts.spd, "spd"),
            (boosts.spe, "spe"),
        ];

        // First pass: apply all stat changes
        let mon = self.sides[player as usize].active_mut();
        let old_atk = mon.boosts.atk;
        let old_def = mon.boosts.def;
        let old_spa = mon.boosts.spa;
        let old_spd = mon.boosts.spd;
        let old_spe = mon.boosts.spe;
        if boosts.atk != 0 { mon.boosts.atk = (mon.boosts.atk + boosts.atk).clamp(-6, 6); }
        if boosts.def != 0 { mon.boosts.def = (mon.boosts.def + boosts.def).clamp(-6, 6); }
        if boosts.spa != 0 { mon.boosts.spa = (mon.boosts.spa + boosts.spa).clamp(-6, 6); }
        if boosts.spd != 0 { mon.boosts.spd = (mon.boosts.spd + boosts.spd).clamp(-6, 6); }
        if boosts.spe != 0 { mon.boosts.spe = (mon.boosts.spe + boosts.spe).clamp(-6, 6); }

        let new_vals = [
            self.sides[player as usize].active().boosts.atk,
            self.sides[player as usize].active().boosts.def,
            self.sides[player as usize].active().boosts.spa,
            self.sides[player as usize].active().boosts.spd,
            self.sides[player as usize].active().boosts.spe,
        ];
        let old_vals = [old_atk, old_def, old_spa, old_spd, old_spe];

        // Emit unboosts first (PS order)
        for (i, &(change, stat_name)) in stats.iter().enumerate() {
            if change < 0 {
                let actual = (old_vals[i] - new_vals[i]).unsigned_abs();
                self.emit(format!("|-unboost|p{}a: {}|{}|{}", player + 1, atk_name, stat_name, actual));
            }
        }
        // Then emit boosts
        for (i, &(change, stat_name)) in stats.iter().enumerate() {
            if change > 0 {
                let actual = (new_vals[i] - old_vals[i]).unsigned_abs();
                self.emit(format!("|-boost|p{}a: {}|{}|{}", player + 1, atk_name, stat_name, actual));
            }
        }
    }

    /// Apply a data-driven status infliction. Enforces type/ability immunity
    /// exactly as the old per-move match arms did.
    fn apply_status_inflict(&mut self, _attacker: u8, defender: u8, kind: crate::events::StatusKind) {
        use crate::events::StatusKind;
        use pkmn_core::abilities::AbilityId;
        let def_mon = self.sides[defender as usize].active();
        // Already statused → fail
        if def_mon.status != Status::None {
            let def_name = self.species_name(defender);
            let tag = match kind {
                StatusKind::Burn => "brn",
                StatusKind::Paralyze => "par",
                StatusKind::Poison | StatusKind::Toxic => "tox",
                StatusKind::Sleep => "slp",
                StatusKind::Freeze => "frz",
            };
            self.emit(format!("|-fail|p{}a: {}|{}", defender + 1, def_name, tag));
            return;
        }
        // Type immunity checks
        let immune = match kind {
            StatusKind::Burn => def_mon.types.contains(&Type::Fire),
            StatusKind::Paralyze => {
                def_mon.types.contains(&Type::Electric) || def_mon.types.contains(&Type::Ground)
            }
            StatusKind::Poison | StatusKind::Toxic => {
                def_mon.types.contains(&Type::Poison) || def_mon.types.contains(&Type::Steel)
            }
            StatusKind::Freeze => def_mon.types.contains(&Type::Ice),
            // Grass-type immunity to powder/spore sleep moves (Gen 6+)
            StatusKind::Sleep => def_mon.types.contains(&Type::Grass),
        };
        if immune {
            let def_name = self.species_name(defender);
            self.emit(format!("|-immune|p{}a: {}", defender + 1, def_name));
            return;
        }
        // Sleep-specific immunity checks
        if kind == StatusKind::Sleep {
            let ability = self.sides[defender as usize].active().ability_id;
            // Ability immunity: Insomnia, Vital Spirit, Sweet Veil
            if ability == AbilityId::Insomnia
                || ability == AbilityId::VitalSpirit
                || ability == AbilityId::SweetVeil
            {
                let def_name = self.species_name(defender);
                self.emit(format!("|-immune|p{}a: {}", defender + 1, def_name));
                return;
            }
            // Terrain immunity: Electric Terrain and Misty Terrain block sleep for grounded mons
            let is_grounded = !self.sides[defender as usize].active().types.contains(&Type::Flying)
                && self.sides[defender as usize].active().ability_id != AbilityId::Levitate;
            if is_grounded
                && (self.field.terrain == crate::field::Terrain::Electric
                    || self.field.terrain == crate::field::Terrain::Misty)
            {
                let def_name = self.species_name(defender);
                self.emit(format!("|-immune|p{}a: {}", defender + 1, def_name));
                return;
            }
            // Sleep Clause Mod: can't put a second mon to sleep via a move
            if self.sides[defender as usize].sleep_clause_count > 0 {
                let def_name = self.species_name(defender);
                self.emit(format!("|-fail|p{}a: {}|slp", defender + 1, def_name));
                return;
            }
        }
        // Apply the status
        let status = match kind {
            StatusKind::Burn => Status::Burn,
            StatusKind::Paralyze => Status::Paralyze,
            StatusKind::Poison => Status::Poison,
            StatusKind::Toxic => Status::Toxic,
            StatusKind::Sleep => Status::Sleep,
            StatusKind::Freeze => Status::Freeze,
        };
        let tag = match kind {
            StatusKind::Burn => "brn",
            StatusKind::Paralyze => "par",
            StatusKind::Poison => "psn",
            StatusKind::Toxic => "tox",
            StatusKind::Sleep => "slp",
            StatusKind::Freeze => "frz",
        };
        self.sides[defender as usize].active_mut().status = status;
        // Sleep: set random 1-3 turn counter and track for Sleep Clause
        if kind == StatusKind::Sleep {
            let turns = self.random(3) as u8 + 1; // 1, 2, or 3
            self.sides[defender as usize].active_mut().status_turns = turns;
            self.sides[defender as usize].sleep_clause_count += 1;
        }
        let def_name = self.species_name(defender);
        self.emit(format!("|-status|p{}a: {}|{}", defender + 1, def_name, tag));
    }

    /// Apply a data-driven flat heal (num/denom of max HP).
    fn apply_heal(&mut self, player: u8, num: u16, denom: u16) {
        let mon = self.sides[player as usize].active_mut();
        if mon.hp >= mon.max_hp {
            let name = self.species_name(player);
            self.emit(format!("|-fail|p{}a: {}|heal", player + 1, name));
        } else {
            let heal = mon.max_hp * num / denom;
            mon.hp = (mon.hp + heal).min(mon.max_hp);
            let name = self.species_name(player);
            let hp = self.sides[player as usize].active().hp;
            let max_hp = self.sides[player as usize].active().max_hp;
            self.emit(format!("|-heal|p{}a: {}|{}/{}", player + 1, name, hp, max_hp));
        }
    }

    /// Apply a data-driven hazard set on the defender's side.
    fn apply_hazard(&mut self, attacker: u8, defender: u8, kind: crate::events::HazardKind) {
        use crate::events::HazardKind;
        let sc = &self.sides[defender as usize].side_conditions;
        let fail = match kind {
            HazardKind::StealthRock => sc.stealth_rock,
            HazardKind::Spikes => sc.spikes >= 3,
            HazardKind::ToxicSpikes => sc.toxic_spikes >= 2,
            HazardKind::StickyWeb => sc.sticky_web,
        };
        if fail {
            let atk_name = self.species_name(attacker);
            self.emit(format!("|-fail|p{}a: {}", attacker + 1, atk_name));
        } else {
            match kind {
                HazardKind::StealthRock => {
                    self.sides[defender as usize].side_conditions.stealth_rock = true;
                    self.emit(format!("|-sidestart|p{}: Player {}|move: Stealth Rock", defender + 1, defender + 1));
                }
                HazardKind::Spikes => {
                    self.sides[defender as usize].side_conditions.spikes += 1;
                    self.emit(format!("|-sidestart|p{}: Player {}|move: Spikes", defender + 1, defender + 1));
                }
                HazardKind::ToxicSpikes => {
                    self.sides[defender as usize].side_conditions.toxic_spikes += 1;
                    self.emit(format!("|-sidestart|p{}: Player {}|move: Toxic Spikes", defender + 1, defender + 1));
                }
                HazardKind::StickyWeb => {
                    self.sides[defender as usize].side_conditions.sticky_web = true;
                    self.emit(format!("|-sidestart|p{}: Player {}|move: Sticky Web", defender + 1, defender + 1));
                }
            }
        }
    }

    /// Apply a data-driven weather/terrain field effect.
    fn apply_field_effect(&mut self, attacker: u8, effect: crate::events::FieldEffect) {
        use crate::events::{FieldEffect, FieldWeather, FieldTerrain};
        use crate::field::{Weather as W, Terrain as T};
        use pkmn_core::items::ItemId;

        match effect {
            FieldEffect::Weather(w) => {
                let (weather, name_str, rock_item) = match w {
                    FieldWeather::Rain => (W::Rain, "RainDance", ItemId::DampRock),
                    FieldWeather::Sun => (W::Sun, "SunnyDay", ItemId::HeatRock),
                    FieldWeather::Sand => (W::Sand, "Sandstorm", ItemId::SmoothRock),
                    FieldWeather::Snow => (W::Snow, "Snowscape", ItemId::IcyRock),
                };
                let turns = if self.sides[attacker as usize].active().item_id == rock_item { 8 } else { 5 };
                self.field.weather = weather;
                self.field.weather_turns = turns;
                self.emit(format!("|-weather|{}", name_str));
            }
            FieldEffect::Terrain(t) => {
                let (terrain, name_str) = match t {
                    FieldTerrain::Electric => (T::Electric, "Electric Terrain"),
                    FieldTerrain::Grassy => (T::Grassy, "Grassy Terrain"),
                    FieldTerrain::Misty => (T::Misty, "Misty Terrain"),
                    FieldTerrain::Psychic => (T::Psychic, "Psychic Terrain"),
                };
                let turns = if self.sides[attacker as usize].active().item_id == ItemId::TerrainExtender { 8 } else { 5 };
                self.field.terrain = terrain;
                self.field.terrain_turns = turns;
                self.emit(format!("|-fieldstart|move: {}", name_str));
            }
        }
    }

    /// Toggle Trick Room: set 5 turns if off, clear if already on.
    fn apply_trick_room(&mut self, _attacker: u8) {
        if self.field.trick_room > 0 {
            // Already active: toggle off
            self.field.trick_room = 0;
            self.emit("|-fieldend|move: Trick Room".to_string());
        } else {
            // Set Trick Room for 5 turns
            self.field.trick_room = 5;
            self.emit("|-fieldstart|move: Trick Room".to_string());
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

    /// Test helper: call apply_status_move with a move looked up by name.
    #[cfg(test)]
    pub(crate) fn apply_status_move_for_test(&mut self, attacker: u8, defender: u8, move_name: &str) {
        let move_data = pkmn_core::moves::get_move(move_name)
            .expect("test move not found");
        self.apply_status_move(attacker, defender, move_data);
    }

    fn apply_status_move(&mut self, attacker: u8, defender: u8, move_data: &MoveData) {
        let name = move_data.name.to_lowercase();

        // Prankster immunity: Dark types are immune to Prankster-boosted status moves
        let is_targeting_status = matches!(name.as_str(),
            "toxic" | "will-o-wisp" | "thunder wave"
            | "spore" | "sleep powder" | "hypnosis" | "lovely kiss" | "sing"
            | "grass whistle"
        );
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
                "toxic" | "will-o-wisp" | "thunder wave"
                | "spore" | "sleep powder" | "hypnosis" | "lovely kiss" | "sing"
                | "grass whistle" => return,
                _ => {}
            }
        }

        // Substitute blocks targeting status moves
        let is_targeting = matches!(name.as_str(),
            "toxic" | "will-o-wisp" | "thunder wave"
            | "spore" | "sleep powder" | "hypnosis" | "lovely kiss" | "sing"
            | "grass whistle"
        );
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
                crate::events::MoveEffect::StatusInflict(kind) => {
                    self.apply_status_inflict(attacker, defender, kind);
                    return;
                }
                crate::events::MoveEffect::Heal(num, denom) => {
                    self.apply_heal(attacker, num, denom);
                    return;
                }
                crate::events::MoveEffect::Hazard(kind) => {
                    self.apply_hazard(attacker, defender, kind);
                    return;
                }
                crate::events::MoveEffect::Field(field_effect) => {
                    self.apply_field_effect(attacker, field_effect);
                    return;
                }
                crate::events::MoveEffect::TrickRoom => {
                    self.apply_trick_room(attacker);
                    return;
                }
            }
        }

        match name.as_str() {
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
            // Boost moves (swords dance, dragon dance, calm mind, nasty plot, agility, etc. now data-driven via events.rs)
            "reflect" => {
                self.sides[attacker as usize].side_conditions.reflect = 5;
                self.emit(format!("|-sidestart|p{}: Player {}|Reflect", attacker+1, attacker+1));
            }
            "light screen" => {
                self.sides[attacker as usize].side_conditions.light_screen = 5;
                self.emit(format!("|-sidestart|p{}: Player {}|move: Light Screen", attacker+1, attacker+1));
            }
            // stealth rock, spikes, toxic spikes, sticky web are now data-driven via events.rs
            "roost" | "moonlight" | "synthesis" | "morning sun" => {
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
            "rest" => {
                use pkmn_core::abilities::AbilityId;
                let mon = self.sides[attacker as usize].active();
                // Fail if at full HP or if ability prevents sleep
                let ability = mon.ability_id;
                if mon.hp >= mon.max_hp {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}|heal", attacker+1, atk_name));
                    return;
                }
                if ability == AbilityId::Insomnia
                    || ability == AbilityId::VitalSpirit
                    || ability == AbilityId::SweetVeil
                {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                    return;
                }
                // Rest: full heal, set sleep for exactly 2 turns (Rest-sleep is exempt from Sleep Clause)
                let mon = self.sides[attacker as usize].active_mut();
                mon.status = Status::Sleep;
                mon.status_turns = 2;
                mon.hp = mon.max_hp;
                let atk_name = self.species_name(attacker);
                let hp = self.sides[attacker as usize].active().hp;
                let max_hp = self.sides[attacker as usize].active().max_hp;
                self.emit(format!("|-status|p{}a: {}|slp|[from] move: Rest", attacker+1, atk_name));
                self.emit(format!("|-heal|p{}a: {}|{}/{}|[silent]", attacker+1, atk_name, hp, max_hp));
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
            "taunt" => {
                let def_mon = self.sides[defender as usize].active_mut();
                if !def_mon.volatiles.contains(Volatiles::TAUNT) {
                    def_mon.volatiles.insert(Volatiles::TAUNT);
                    def_mon.taunt_turns = 3;
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-start|p{}a: {}|move: Taunt", defender+1, def_name));
                }
            }
            "encore" => {
                let last_idx = self.sides[defender as usize].active().last_used_move_idx;
                if last_idx == 255 || self.sides[defender as usize].active().volatiles.contains(Volatiles::ENCORE) {
                    // Fail: no last move or already encored
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                } else {
                    let def_mon = self.sides[defender as usize].active_mut();
                    def_mon.volatiles.insert(Volatiles::ENCORE);
                    def_mon.encore_turns = 3;
                    def_mon.encore_move_idx = last_idx;
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-start|p{}a: {}|move: Encore", defender+1, def_name));
                }
            }
            "disable" => {
                let last_idx = self.sides[defender as usize].active().last_used_move_idx;
                if last_idx == 255 || self.sides[defender as usize].active().volatiles.contains(Volatiles::DISABLE) {
                    let atk_name = self.species_name(attacker);
                    self.emit(format!("|-fail|p{}a: {}", attacker+1, atk_name));
                } else {
                    let move_name = pkmn_core::moves::get_move_by_id(
                        self.sides[defender as usize].active().moves[last_idx as usize].move_id
                    ).map(|m| m.name).unwrap_or("???");
                    let def_mon = self.sides[defender as usize].active_mut();
                    def_mon.volatiles.insert(Volatiles::DISABLE);
                    def_mon.disable_turns = 4;
                    def_mon.disable_move_idx = last_idx;
                    let def_name = self.species_name(defender);
                    self.emit(format!("|-start|p{}a: {}|Disable|{}", defender+1, def_name, move_name));
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
        // PS processes EOT effects in speed order (faster first, respecting abilities)
        let order: [u8; 2] = if self.ordering_speed(0) >= self.ordering_speed(1) {
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
        let p1_speed = self.ordering_speed(0);
        let p2_speed = self.ordering_speed(1);
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

        // Partial trapping: 1/8 max HP chip each end-of-turn, decrement counter
        for &player in &order {
            let mon = self.sides[player as usize].active();
            if !mon.is_alive() || !mon.volatiles.contains(Volatiles::TRAPPED) { continue; }
            let trap_move_id = mon.trap_move_id;
            let max_hp = mon.max_hp;
            let dmg = (max_hp / 8).max(1);
            self.apply_damage(player, dmg);
            let move_name = pkmn_core::moves::get_move_by_id(trap_move_id)
                .map(|m| m.name).unwrap_or("???");
            let name = self.species_name(player);
            let hp_str = self.hp_display(player);
            self.emit(format!("|-damage|p{}a: {}|{}|[from] move: {}", player+1, name, hp_str, move_name));
            if self.sides[player as usize].active().hp == 0 {
                self.emit(format!("|faint|p{}a: {}", player+1, name));
            }
            let mon = self.sides[player as usize].active_mut();
            mon.trap_turns = mon.trap_turns.saturating_sub(1);
            if mon.trap_turns == 0 {
                mon.volatiles.remove(Volatiles::TRAPPED);
                mon.trap_move_id = 0;
            }
        }

        // Action-constraining volatile counters: decrement at end of turn
        for &player in &order {
            if !self.sides[player as usize].active().is_alive() { continue; }
            // Taunt
            if self.sides[player as usize].active().volatiles.contains(Volatiles::TAUNT) {
                let mon = self.sides[player as usize].active_mut();
                mon.taunt_turns = mon.taunt_turns.saturating_sub(1);
                if mon.taunt_turns == 0 {
                    mon.volatiles.remove(Volatiles::TAUNT);
                    let name = self.species_name(player);
                    self.emit(format!("|-end|p{}a: {}|move: Taunt", player+1, name));
                }
            }
            // Encore: also ends if PP of encored move is 0
            if self.sides[player as usize].active().volatiles.contains(Volatiles::ENCORE) {
                let idx = self.sides[player as usize].active().encore_move_idx as usize;
                let pp_zero = self.sides[player as usize].active().moves[idx].pp == 0;
                let mon = self.sides[player as usize].active_mut();
                if pp_zero { mon.encore_turns = 0; }
                mon.encore_turns = mon.encore_turns.saturating_sub(1);
                if mon.encore_turns == 0 {
                    mon.volatiles.remove(Volatiles::ENCORE);
                    let name = self.species_name(player);
                    self.emit(format!("|-end|p{}a: {}|move: Encore", player+1, name));
                }
            }
            // Disable
            if self.sides[player as usize].active().volatiles.contains(Volatiles::DISABLE) {
                let mon = self.sides[player as usize].active_mut();
                mon.disable_turns = mon.disable_turns.saturating_sub(1);
                if mon.disable_turns == 0 {
                    mon.volatiles.remove(Volatiles::DISABLE);
                    let name = self.species_name(player);
                    self.emit(format!("|-end|p{}a: {}|Disable", player+1, name));
                }
            }
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
            if self.field.trick_room == 0 {
                self.emit("|-fieldend|move: Trick Room".to_string());
            }
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
            // Fixed 2-hit moves
            "Double Iron Bash" | "Dual Wingbeat" | "Dragon Darts"
            | "Double Kick" | "Dual Chop" | "Bonemerang" | "Gear Grind"
            | "Twin Beam" | "Double Hit" => Some(2),
            // Fixed 3-hit
            "Surging Strikes" => Some(3),
            // Escalating: Triple Axel (20/40/60) and Triple Kick (10/20/30)
            // Max 3 hits but each hit rolls accuracy; stop on miss
            "Triple Axel" | "Triple Kick" => Some(3),
            // Population Bomb: up to 10 hits, per-hit accuracy unless Loaded Dice
            "Population Bomb" => Some(10),
            // 2-5 hit moves
            "Bullet Seed" | "Icicle Spear" | "Rock Blast" | "Scale Shot"
            | "Pin Missile" | "Tail Slap" | "Bone Rush" | "Arm Thrust"
            | "Double Slap" | "Comet Punch" | "Fury Attack" | "Fury Swipes" => {
                // Skill Link always gives 5
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
        let is_escalating = matches!(move_data.name, "Triple Axel" | "Triple Kick");
        let is_pop_bomb = move_data.name == "Population Bomb";
        let has_loaded_dice = self.sides[player as usize].active().item_id == pkmn_core::items::ItemId::LoadedDice;
        // Per-hit accuracy: escalating moves and Population Bomb (unless Loaded Dice)
        let per_hit_accuracy = is_escalating || (is_pop_bomb && !has_loaded_dice);

        let hits = if max_hits == 0 {
            // 2-5 hit distribution
            if has_loaded_dice {
                // Loaded Dice: 4 or 5 hits
                let roll = self.random(2);
                roll + 4
            } else {
                // 35/35/15/15 distribution via sample of 20
                let roll = self.random(20);
                if roll < 7 { 2 } else if roll < 14 { 3 } else if roll < 17 { 4 } else { 5 }
            }
        } else if is_pop_bomb && has_loaded_dice {
            // Loaded Dice + Population Bomb: guaranteed 4-5 hits, no per-hit acc
            let roll = self.random(2);
            roll + 4
        } else {
            max_hits as u32
        };

        let mut actual_hits: u32 = 0;
        for hit_num in 0..hits {
            // Per-hit accuracy check (skip first hit — already checked before entering)
            if per_hit_accuracy && hit_num > 0 && move_data.accuracy > 0
                && !self.rand_check(move_data.accuracy)
            {
                break;
            }

            // Escalating BP: create a modified move_data with overridden base_power
            let effective_bp = if is_escalating {
                let base = move_data.base_power as u16;
                base * (hit_num as u16 + 1) // Triple Kick: 10/20/30, Triple Axel: 20/40/60
            } else {
                move_data.base_power as u16
            };

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

            let damage = if is_escalating {
                // Use a modified MoveData with escalating BP
                let mut modified = *move_data;
                modified.base_power = effective_bp as u8;
                self.calculate_damage_with(player, defender, &modified, critical, random_factor)
            } else {
                self.calculate_damage_with(player, defender, move_data, critical, random_factor)
            };

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

                // on_damaging_hit per hit for contact moves
                if move_data.flags.has(MoveFlags::CONTACT) && self.sides[defender as usize].active().is_alive() {
                    let defender_ability = self.sides[defender as usize].active().ability_id;
                    let defender_item = self.sides[defender as usize].active().item_id;
                    let hooks = crate::events::ability_hooks(defender_ability);
                    if let Some(hook) = hooks.on_damaging_hit {
                        hook(self, player, defender);
                    }
                    if defender_item == pkmn_core::items::ItemId::RockyHelmet {
                        let attacker_max_hp = self.sides[player as usize].active().max_hp;
                        let recoil = (attacker_max_hp / 6).max(1);
                        self.apply_damage(player, recoil);
                        let atk_name = self.species_name(player);
                        let hp_str = self.hp_display(player);
                        let def_name = self.species_name(defender);
                        self.emit(format!("|-damage|p{}a: {}|{}|[from] item: Rocky Helmet|[of] p{}a: {}", player+1, atk_name, hp_str, defender+1, def_name));
                    }
                }
            }

            actual_hits += 1;
            if self.sides[defender as usize].active().hp == 0 {
                break;
            }
            // Stop if attacker fainted (e.g. from Rough Skin recoil)
            if !self.sides[player as usize].active().is_alive() {
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

#[cfg(test)]
mod tests_data_driven_moves {
    use crate::battle::Battle;
    use crate::pokemon::{MoveSlot, Pokemon, Status};
    use crate::side::Side;
    use pkmn_core::nature::Nature;
    use pkmn_core::species::get_species;

    fn toxic_slot() -> MoveSlot { MoveSlot { move_id: 92, pp: 10, max_pp: 10 } }
    fn wow_slot() -> MoveSlot { MoveSlot { move_id: 261, pp: 15, max_pp: 15 } } // Will-O-Wisp
    fn twave_slot() -> MoveSlot { MoveSlot { move_id: 86, pp: 20, max_pp: 20 } } // Thunder Wave
    fn recover_slot() -> MoveSlot { MoveSlot { move_id: 105, pp: 10, max_pp: 10 } }
    fn spikes_slot() -> MoveSlot { MoveSlot { move_id: 191, pp: 20, max_pp: 20 } }
    fn tspikes_slot() -> MoveSlot { MoveSlot { move_id: 390, pp: 20, max_pp: 20 } } // Toxic Spikes
    fn sweb_slot() -> MoveSlot { MoveSlot { move_id: 564, pp: 20, max_pp: 20 } } // Sticky Web
    fn empty_slot() -> MoveSlot { MoveSlot { move_id: 0, pp: 0, max_pp: 0 } }

    fn make_battle_with_moves(p1_name: &str, p1_moves: [MoveSlot; 4], p2_name: &str) -> Battle {
        let species1 = get_species(p1_name).unwrap();
        let species2 = get_species(p2_name).unwrap();
        let p1 = Pokemon::new(species1, 100, Nature::Hardy, p1_moves, [0; 6], [31; 6]);
        let p2 = Pokemon::new(species2, 100, Nature::Hardy, [empty_slot(); 4], [0; 6], [31; 6]);
        Battle::new(Side::new(vec![p1]), Side::new(vec![p2]), [1, 2, 3, 4])
    }

    // --- StatusInflict tests ---

    #[test]
    fn toxic_poisons_neutral_target() {
        let mut battle = make_battle_with_moves("Blissey", [toxic_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        // Garchomp is Dragon/Ground — not immune to Toxic
        battle.apply_status_move_for_test(0, 1, "Toxic");
        assert_eq!(battle.sides[1].active().status, Status::Toxic);
        assert!(battle.protocol.iter().any(|l| l.contains("|-status|") && l.contains("tox")));
    }

    #[test]
    fn toxic_immune_vs_steel() {
        let mut battle = make_battle_with_moves("Blissey", [toxic_slot(), empty_slot(), empty_slot(), empty_slot()], "Ferrothorn");
        // Ferrothorn is Grass/Steel — immune to Toxic
        battle.apply_status_move_for_test(0, 1, "Toxic");
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    #[test]
    fn toxic_immune_vs_poison() {
        let mut battle = make_battle_with_moves("Blissey", [toxic_slot(), empty_slot(), empty_slot(), empty_slot()], "Toxapex");
        // Toxapex is Poison/Water — immune to Toxic
        battle.apply_status_move_for_test(0, 1, "Toxic");
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    #[test]
    fn wow_burns_neutral_target() {
        let mut battle = make_battle_with_moves("Blissey", [wow_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.apply_status_move_for_test(0, 1, "Will-O-Wisp");
        assert_eq!(battle.sides[1].active().status, Status::Burn);
        assert!(battle.protocol.iter().any(|l| l.contains("|-status|") && l.contains("brn")));
    }

    #[test]
    fn wow_immune_vs_fire() {
        let mut battle = make_battle_with_moves("Blissey", [wow_slot(), empty_slot(), empty_slot(), empty_slot()], "Arcanine");
        battle.apply_status_move_for_test(0, 1, "Will-O-Wisp");
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    #[test]
    fn thunder_wave_paralyzes_neutral() {
        let mut battle = make_battle_with_moves("Blissey", [twave_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        // Garchomp is Dragon/Ground — immune to Thunder Wave (Ground type)
        battle.apply_status_move_for_test(0, 1, "Thunder Wave");
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    #[test]
    fn thunder_wave_paralyzes_non_immune() {
        let mut battle = make_battle_with_moves("Blissey", [twave_slot(), empty_slot(), empty_slot(), empty_slot()], "Dragapult");
        // Dragapult is Dragon/Ghost — not immune
        battle.apply_status_move_for_test(0, 1, "Thunder Wave");
        assert_eq!(battle.sides[1].active().status, Status::Paralyze);
        assert!(battle.protocol.iter().any(|l| l.contains("|-status|") && l.contains("par")));
    }

    #[test]
    fn status_fails_if_already_statused() {
        let mut battle = make_battle_with_moves("Blissey", [toxic_slot(), empty_slot(), empty_slot(), empty_slot()], "Dragapult");
        battle.sides[1].active_mut().status = Status::Burn;
        battle.apply_status_move_for_test(0, 1, "Toxic");
        assert_eq!(battle.sides[1].active().status, Status::Burn); // unchanged
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|")));
    }

    // --- Heal tests ---

    #[test]
    fn recover_heals_half() {
        let mut battle = make_battle_with_moves("Blissey", [recover_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        let max_hp = battle.sides[0].active().max_hp;
        battle.sides[0].active_mut().hp = max_hp / 4;
        battle.apply_status_move_for_test(0, 1, "Recover");
        // Should heal max_hp/2
        let expected = (max_hp / 4) + (max_hp / 2);
        assert_eq!(battle.sides[0].active().hp, expected.min(max_hp));
        assert!(battle.protocol.iter().any(|l| l.contains("|-heal|")));
    }

    #[test]
    fn recover_fails_at_full_hp() {
        let mut battle = make_battle_with_moves("Blissey", [recover_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        // HP is already full
        battle.apply_status_move_for_test(0, 1, "Recover");
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|") && l.contains("heal")));
    }

    // --- Hazard tests ---

    #[test]
    fn spikes_sets_layer() {
        let mut battle = make_battle_with_moves("Blissey", [spikes_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.apply_status_move_for_test(0, 1, "Spikes");
        assert_eq!(battle.sides[1].side_conditions.spikes, 1);
        assert!(battle.protocol.iter().any(|l| l.contains("|-sidestart|") && l.contains("Spikes")));
    }

    #[test]
    fn spikes_stacks_to_three() {
        let mut battle = make_battle_with_moves("Blissey", [spikes_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.apply_status_move_for_test(0, 1, "Spikes");
        battle.protocol.clear();
        battle.apply_status_move_for_test(0, 1, "Spikes");
        assert_eq!(battle.sides[1].side_conditions.spikes, 2);
        battle.protocol.clear();
        battle.apply_status_move_for_test(0, 1, "Spikes");
        assert_eq!(battle.sides[1].side_conditions.spikes, 3);
    }

    #[test]
    fn spikes_fails_at_max() {
        let mut battle = make_battle_with_moves("Blissey", [spikes_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.sides[1].side_conditions.spikes = 3;
        battle.apply_status_move_for_test(0, 1, "Spikes");
        assert_eq!(battle.sides[1].side_conditions.spikes, 3);
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|")));
    }

    #[test]
    fn toxic_spikes_sets_layer() {
        let mut battle = make_battle_with_moves("Blissey", [tspikes_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.apply_status_move_for_test(0, 1, "Toxic Spikes");
        assert_eq!(battle.sides[1].side_conditions.toxic_spikes, 1);
        assert!(battle.protocol.iter().any(|l| l.contains("|-sidestart|") && l.contains("Toxic Spikes")));
    }

    #[test]
    fn toxic_spikes_fails_at_two() {
        let mut battle = make_battle_with_moves("Blissey", [tspikes_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.sides[1].side_conditions.toxic_spikes = 2;
        battle.apply_status_move_for_test(0, 1, "Toxic Spikes");
        assert_eq!(battle.sides[1].side_conditions.toxic_spikes, 2);
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|")));
    }

    #[test]
    fn sticky_web_sets() {
        let mut battle = make_battle_with_moves("Blissey", [sweb_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.apply_status_move_for_test(0, 1, "Sticky Web");
        assert!(battle.sides[1].side_conditions.sticky_web);
        assert!(battle.protocol.iter().any(|l| l.contains("|-sidestart|") && l.contains("Sticky Web")));
    }

    #[test]
    fn sticky_web_fails_if_already_set() {
        let mut battle = make_battle_with_moves("Blissey", [sweb_slot(), empty_slot(), empty_slot(), empty_slot()], "Garchomp");
        battle.sides[1].side_conditions.sticky_web = true;
        battle.apply_status_move_for_test(0, 1, "Sticky Web");
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|")));
    }
}

#[cfg(test)]
mod tests_volatiles_and_field {
    use crate::battle::Battle;
    use crate::choice::Choice;
    use crate::field::{Weather, Terrain};
    use crate::pokemon::{MoveSlot, Pokemon, Volatiles};
    use crate::side::Side;
    use pkmn_core::items::ItemId;
    use pkmn_core::nature::Nature;
    use pkmn_core::species::get_species;

    fn slot(id: u16, pp: u8) -> MoveSlot { MoveSlot { move_id: id, pp, max_pp: pp } }
    fn empty() -> MoveSlot { MoveSlot { move_id: 0, pp: 0, max_pp: 0 } }

    /// Build a 1v1 battle (no RNG init, no switch-in protocol) for unit testing.
    fn raw_battle(p1_moves: [MoveSlot; 4], p2_moves: [MoveSlot; 4]) -> Battle {
        let sp = get_species("Blissey").unwrap();
        let p1 = Pokemon::new(sp, 100, Nature::Hardy, p1_moves, [0; 6], [31; 6]);
        let p2 = Pokemon::new(sp, 100, Nature::Hardy, p2_moves, [0; 6], [31; 6]);
        Battle::new_raw(Side::new(vec![p1]), Side::new(vec![p2]))
    }

    /// Build a battle with a second mon on p2's team (for switch tests).
    fn raw_battle_with_switch(p1_moves: [MoveSlot; 4], p2_moves: [MoveSlot; 4]) -> Battle {
        let sp = get_species("Blissey").unwrap();
        let p1 = Pokemon::new(sp, 100, Nature::Hardy, p1_moves, [0; 6], [31; 6]);
        let p2 = Pokemon::new(sp, 100, Nature::Hardy, p2_moves, [0; 6], [31; 6]);
        let p2b = Pokemon::new(sp, 100, Nature::Hardy, [empty(); 4], [0; 6], [31; 6]);
        Battle::new_raw(Side::new(vec![p1]), Side::new(vec![p2, p2b]))
    }

    // Move IDs:
    // Taunt=269, Encore=227, Disable=50, Fire Spin=83, Rain Dance=240
    // Grassy Terrain=580, Swords Dance=14, Flamethrower=53, Tackle=33, Thunderbolt=85

    // ===== TAUNT TESTS =====

    #[test]
    fn taunt_blocks_status_moves_from_choices() {
        // P2 has: Swords Dance (status), Flamethrower (special)
        let mut battle = raw_battle(
            [slot(269, 20), empty(), empty(), empty()], // P1: Taunt
            [slot(14, 20), slot(53, 15), empty(), empty()], // P2: Swords Dance, Flamethrower
        );
        // Apply Taunt to P2
        battle.apply_status_move_for_test(0, 1, "Taunt");
        assert!(battle.sides[1].active().volatiles.contains(Volatiles::TAUNT));

        // P2's choices should NOT include Swords Dance (index 0, status)
        let choices = battle.choices(1);
        assert!(!choices.contains(&Choice::Move(0)), "Taunted mon should not have status move");
        assert!(choices.contains(&Choice::Move(1)), "Taunted mon should keep damaging move");
    }

    #[test]
    fn taunt_expires_after_3_turns() {
        let mut battle = raw_battle(
            [slot(269, 20), empty(), empty(), empty()],
            [slot(14, 20), slot(53, 15), empty(), empty()],
        );
        battle.apply_status_move_for_test(0, 1, "Taunt");
        assert_eq!(battle.sides[1].active().taunt_turns, 3);

        // Simulate 3 end-of-turns
        battle.end_of_turn();
        assert_eq!(battle.sides[1].active().taunt_turns, 2);
        battle.end_of_turn();
        assert_eq!(battle.sides[1].active().taunt_turns, 1);
        battle.end_of_turn();
        assert_eq!(battle.sides[1].active().taunt_turns, 0);
        assert!(!battle.sides[1].active().volatiles.contains(Volatiles::TAUNT));

        // Status moves should be available again
        let choices = battle.choices(1);
        assert!(choices.contains(&Choice::Move(0)), "After taunt expires, status moves return");
    }

    #[test]
    fn taunt_all_status_yields_struggle() {
        // P2 has ONLY status moves
        let mut battle = raw_battle(
            [slot(269, 20), empty(), empty(), empty()],
            [slot(14, 20), slot(240, 5), empty(), empty()], // Swords Dance, Rain Dance (both status)
        );
        battle.apply_status_move_for_test(0, 1, "Taunt");

        let choices = battle.choices(1);
        // Should only have Struggle (sentinel 255)
        let move_choices: Vec<_> = choices.iter().filter(|c| matches!(c, Choice::Move(_))).collect();
        assert_eq!(move_choices, vec![&Choice::Move(255)], "All-status taunted mon should Struggle");
    }

    // ===== ENCORE TESTS =====

    #[test]
    fn encore_forces_last_used_move() {
        // P2 has: Flamethrower, Thunderbolt
        let mut battle = raw_battle(
            [slot(227, 5), empty(), empty(), empty()], // P1: Encore
            [slot(53, 15), slot(85, 15), empty(), empty()], // P2: Flamethrower, Thunderbolt
        );
        // P2 uses Flamethrower (index 0) — set last_used_move_idx
        battle.sides[1].active_mut().last_used_move_idx = 0;

        // Apply Encore
        battle.apply_status_move_for_test(0, 1, "Encore");
        assert!(battle.sides[1].active().volatiles.contains(Volatiles::ENCORE));
        assert_eq!(battle.sides[1].active().encore_move_idx, 0);

        // P2's choices should be forced to move 0 only (plus switches if available)
        let choices = battle.choices(1);
        let move_choices: Vec<_> = choices.iter().filter(|c| matches!(c, Choice::Move(_))).collect();
        assert_eq!(move_choices, vec![&Choice::Move(0)], "Encored mon forced to last-used move");
    }

    #[test]
    fn encore_expires_after_3_turns() {
        let mut battle = raw_battle(
            [slot(227, 5), empty(), empty(), empty()],
            [slot(53, 15), slot(85, 15), empty(), empty()],
        );
        battle.sides[1].active_mut().last_used_move_idx = 0;
        battle.apply_status_move_for_test(0, 1, "Encore");

        battle.end_of_turn();
        battle.end_of_turn();
        battle.end_of_turn();
        assert!(!battle.sides[1].active().volatiles.contains(Volatiles::ENCORE));

        // Both moves available again
        let choices = battle.choices(1);
        assert!(choices.contains(&Choice::Move(0)));
        assert!(choices.contains(&Choice::Move(1)));
    }

    #[test]
    fn encore_fails_if_no_last_move() {
        let mut battle = raw_battle(
            [slot(227, 5), empty(), empty(), empty()],
            [slot(53, 15), empty(), empty(), empty()],
        );
        // last_used_move_idx is 255 (no last move)
        battle.apply_status_move_for_test(0, 1, "Encore");
        assert!(!battle.sides[1].active().volatiles.contains(Volatiles::ENCORE));
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|")));
    }

    // ===== DISABLE TESTS =====

    #[test]
    fn disable_removes_last_used_move_from_choices() {
        let mut battle = raw_battle(
            [slot(50, 20), empty(), empty(), empty()], // P1: Disable
            [slot(53, 15), slot(85, 15), empty(), empty()], // P2: Flamethrower, Thunderbolt
        );
        // P2 last used Flamethrower (index 0)
        battle.sides[1].active_mut().last_used_move_idx = 0;

        battle.apply_status_move_for_test(0, 1, "Disable");
        assert!(battle.sides[1].active().volatiles.contains(Volatiles::DISABLE));
        assert_eq!(battle.sides[1].active().disable_move_idx, 0);

        let choices = battle.choices(1);
        assert!(!choices.contains(&Choice::Move(0)), "Disabled move should be absent");
        assert!(choices.contains(&Choice::Move(1)), "Other moves should remain");
    }

    #[test]
    fn disable_expires_after_4_turns() {
        let mut battle = raw_battle(
            [slot(50, 20), empty(), empty(), empty()],
            [slot(53, 15), slot(85, 15), empty(), empty()],
        );
        battle.sides[1].active_mut().last_used_move_idx = 0;
        battle.apply_status_move_for_test(0, 1, "Disable");
        assert_eq!(battle.sides[1].active().disable_turns, 4);

        battle.end_of_turn();
        battle.end_of_turn();
        battle.end_of_turn();
        battle.end_of_turn();
        assert!(!battle.sides[1].active().volatiles.contains(Volatiles::DISABLE));

        let choices = battle.choices(1);
        assert!(choices.contains(&Choice::Move(0)), "Disabled move returns after expiry");
    }

    #[test]
    fn disable_fails_if_no_last_move() {
        let mut battle = raw_battle(
            [slot(50, 20), empty(), empty(), empty()],
            [slot(53, 15), empty(), empty(), empty()],
        );
        battle.apply_status_move_for_test(0, 1, "Disable");
        assert!(!battle.sides[1].active().volatiles.contains(Volatiles::DISABLE));
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|")));
    }

    // ===== PARTIAL TRAPPING TESTS =====

    #[test]
    fn partial_trap_blocks_switching() {
        let mut battle = raw_battle_with_switch(
            [slot(83, 15), empty(), empty(), empty()], // P1: Fire Spin
            [slot(53, 15), empty(), empty(), empty()], // P2: Flamethrower
        );
        // Manually apply trap (Fire Spin hit)
        let def_mon = battle.sides[1].active_mut();
        def_mon.volatiles.insert(Volatiles::TRAPPED);
        def_mon.trap_turns = 4;
        def_mon.trap_move_id = 83; // Fire Spin

        let choices = battle.choices(1);
        // Should have no Switch choices
        let switch_choices: Vec<_> = choices.iter().filter(|c| matches!(c, Choice::Switch(_))).collect();
        assert!(switch_choices.is_empty(), "Trapped mon cannot switch");
        // But should still have move choices
        assert!(choices.contains(&Choice::Move(0)));
    }

    #[test]
    fn partial_trap_deals_1_8_damage_eot() {
        let mut battle = raw_battle_with_switch(
            [slot(83, 15), empty(), empty(), empty()],
            [slot(53, 15), empty(), empty(), empty()],
        );
        let def_mon = battle.sides[1].active_mut();
        def_mon.volatiles.insert(Volatiles::TRAPPED);
        def_mon.trap_turns = 4;
        def_mon.trap_move_id = 83;

        let max_hp = battle.sides[1].active().max_hp;
        let expected_dmg = (max_hp / 8).max(1);
        let hp_before = battle.sides[1].active().hp;

        battle.protocol.clear();
        battle.end_of_turn();

        let hp_after = battle.sides[1].active().hp;
        assert_eq!(hp_before - hp_after, expected_dmg, "Trap should deal 1/8 max HP");

        // Check protocol emits correct [from] move: Fire Spin
        assert!(battle.protocol.iter().any(|l| l.contains("|-damage|") && l.contains("[from] move: Fire Spin")),
            "Trap damage should cite the trapping move");
    }

    #[test]
    fn partial_trap_releases_after_turns_expire() {
        let mut battle = raw_battle_with_switch(
            [slot(83, 15), empty(), empty(), empty()],
            [slot(53, 15), empty(), empty(), empty()],
        );
        let def_mon = battle.sides[1].active_mut();
        def_mon.volatiles.insert(Volatiles::TRAPPED);
        def_mon.trap_turns = 2; // Will expire after 2 EOTs
        def_mon.trap_move_id = 83;

        battle.end_of_turn();
        assert_eq!(battle.sides[1].active().trap_turns, 1);
        assert!(battle.sides[1].active().volatiles.contains(Volatiles::TRAPPED));

        battle.end_of_turn();
        assert_eq!(battle.sides[1].active().trap_turns, 0);
        assert!(!battle.sides[1].active().volatiles.contains(Volatiles::TRAPPED));

        // Switching should be available again
        let choices = battle.choices(1);
        let switch_choices: Vec<_> = choices.iter().filter(|c| matches!(c, Choice::Switch(_))).collect();
        assert!(!switch_choices.is_empty(), "Released mon can switch");
    }

    // ===== WEATHER SETTER TESTS =====

    #[test]
    fn rain_dance_sets_rain_5_turns() {
        let mut battle = raw_battle(
            [slot(240, 5), empty(), empty(), empty()], // Rain Dance
            [slot(53, 15), empty(), empty(), empty()],
        );
        battle.apply_status_move_for_test(0, 1, "Rain Dance");
        assert_eq!(battle.field.weather, Weather::Rain);
        assert_eq!(battle.field.weather_turns, 5);
    }

    #[test]
    fn rain_dance_with_damp_rock_sets_8_turns() {
        let mut battle = raw_battle(
            [slot(240, 5), empty(), empty(), empty()],
            [slot(53, 15), empty(), empty(), empty()],
        );
        battle.sides[0].active_mut().item_id = ItemId::DampRock;
        battle.apply_status_move_for_test(0, 1, "Rain Dance");
        assert_eq!(battle.field.weather, Weather::Rain);
        assert_eq!(battle.field.weather_turns, 8);
    }

    #[test]
    fn grassy_terrain_sets_terrain_5_turns() {
        let mut battle = raw_battle(
            [slot(580, 10), empty(), empty(), empty()], // Grassy Terrain
            [slot(53, 15), empty(), empty(), empty()],
        );
        battle.apply_status_move_for_test(0, 1, "Grassy Terrain");
        assert_eq!(battle.field.terrain, Terrain::Grassy);
        assert_eq!(battle.field.terrain_turns, 5);
    }

    #[test]
    fn grassy_terrain_with_terrain_extender_sets_8_turns() {
        let mut battle = raw_battle(
            [slot(580, 10), empty(), empty(), empty()],
            [slot(53, 15), empty(), empty(), empty()],
        );
        battle.sides[0].active_mut().item_id = ItemId::TerrainExtender;
        battle.apply_status_move_for_test(0, 1, "Grassy Terrain");
        assert_eq!(battle.field.terrain, Terrain::Grassy);
        assert_eq!(battle.field.terrain_turns, 8);
    }

    // ===== STRUGGLE TESTS =====

    #[test]
    fn struggle_when_no_pp() {
        // P2 has moves but all 0 PP
        let mut battle = raw_battle(
            [slot(53, 15), empty(), empty(), empty()],
            [slot(53, 0), slot(85, 0), empty(), empty()], // 0 PP on both
        );
        let choices = battle.choices(1);
        let move_choices: Vec<_> = choices.iter().filter(|c| matches!(c, Choice::Move(_))).collect();
        assert_eq!(move_choices, vec![&Choice::Move(255)], "No PP = Struggle");
    }

    #[test]
    fn struggle_deals_recoil_quarter_max_hp() {
        let mut battle = raw_battle(
            [slot(53, 15), empty(), empty(), empty()],
            [slot(53, 0), empty(), empty(), empty()],
        );
        let max_hp = battle.sides[1].active().max_hp;
        let hp_before = battle.sides[1].active().hp;

        // P2 uses Struggle (move_idx 255)
        battle.execute_choice(1, Choice::Move(255));

        // P2 should take 1/4 max HP recoil
        let expected_recoil = (max_hp / 4).max(1);
        let hp_after = battle.sides[1].active().hp;
        assert_eq!(hp_before - hp_after, expected_recoil, "Struggle recoil should be 1/4 max HP");

        // Protocol should mention Recoil
        assert!(battle.protocol.iter().any(|l| l.contains("[from] Recoil")));
    }
}

#[cfg(test)]
mod tests_multi_hit {
    use crate::battle::Battle;
    use crate::pokemon::{MoveSlot, Pokemon};
    use crate::side::Side;
    use crate::choice::Choice;
    use pkmn_core::nature::Nature;
    use pkmn_core::species::get_species;
    use pkmn_core::abilities::AbilityId;
    use pkmn_core::items::ItemId;

    fn slot(id: u16, pp: u8) -> MoveSlot { MoveSlot { move_id: id, pp, max_pp: pp } }
    fn empty() -> MoveSlot { MoveSlot { move_id: 0, pp: 0, max_pp: 0 } }

    fn raw_battle(p1_moves: [MoveSlot; 4], p2_moves: [MoveSlot; 4]) -> Battle {
        let sp = get_species("Blissey").unwrap();
        let p1 = Pokemon::new(sp, 100, Nature::Hardy, p1_moves, [0; 6], [31; 6]);
        let p2 = Pokemon::new(sp, 100, Nature::Hardy, p2_moves, [0; 6], [31; 6]);
        Battle::new_raw(Side::new(vec![p1]), Side::new(vec![p2]))
    }

    fn raw_battle_species(p1_species: &str, p1_moves: [MoveSlot; 4], p2_species: &str, p2_moves: [MoveSlot; 4]) -> Battle {
        let sp1 = get_species(p1_species).unwrap();
        let sp2 = get_species(p2_species).unwrap();
        let p1 = Pokemon::new(sp1, 100, Nature::Hardy, p1_moves, [252; 6], [31; 6]);
        let p2 = Pokemon::new(sp2, 100, Nature::Hardy, p2_moves, [252; 6], [31; 6]);
        Battle::new_raw(Side::new(vec![p1]), Side::new(vec![p2]))
    }

    // ===== 2-5 HIT MOVES =====

    #[test]
    fn bullet_seed_hits_2_to_5_times() {
        // Run with multiple seeds to verify hit range [2,5]
        let mut seen_hits = [false; 6]; // index 2..5
        for seed_val in 0u16..50 {
            let mut battle = raw_battle(
                [slot(331, 30), empty(), empty(), empty()], // Bullet Seed
                [empty(), empty(), empty(), empty()],
            );
            battle.rng_seed = [seed_val, seed_val + 1, seed_val + 2, seed_val + 3];
            let hp_before = battle.sides[1].active().hp;
            battle.execute_choice(0, Choice::Move(0));
            let hp_after = battle.sides[1].active().hp;
            let total_damage = hp_before.saturating_sub(hp_after);
            // Find hitcount from protocol
            let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|"));
            assert!(hitcount_line.is_some(), "Should emit hitcount");
            let line = hitcount_line.unwrap();
            let hits: u32 = line.split('|').last().unwrap().trim().parse().unwrap();
            assert!((2..=5).contains(&hits), "Hits should be 2-5, got {}", hits);
            seen_hits[hits as usize] = true;
            // Verify total damage = sum of individual hits
            let damage_lines: Vec<_> = battle.protocol.iter()
                .filter(|l| l.contains("|-damage|") && !l.contains("[from]"))
                .collect();
            assert_eq!(damage_lines.len(), hits as usize, "damage lines should equal hit count");
            assert!(total_damage > 0);
        }
        // Should have seen at least 2 different hit counts
        let unique = seen_hits[2..=5].iter().filter(|&&x| x).count();
        assert!(unique >= 2, "Should see multiple hit counts across seeds, got {}", unique);
    }

    // ===== SKILL LINK =====

    #[test]
    fn skill_link_forces_5_hits() {
        let mut battle = raw_battle(
            [slot(331, 30), empty(), empty(), empty()], // Bullet Seed
            [empty(), empty(), empty(), empty()],
        );
        battle.sides[0].active_mut().ability_id = AbilityId::SkillLink;
        battle.execute_choice(0, Choice::Move(0));
        let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|")).unwrap();
        let hits: u32 = hitcount_line.split('|').last().unwrap().trim().parse().unwrap();
        assert_eq!(hits, 5, "Skill Link should force 5 hits");
    }

    // ===== LOADED DICE =====

    #[test]
    fn loaded_dice_forces_at_least_4_hits() {
        for seed_val in 0u16..30 {
            let mut battle = raw_battle(
                [slot(331, 30), empty(), empty(), empty()], // Bullet Seed
                [empty(), empty(), empty(), empty()],
            );
            battle.sides[0].active_mut().item_id = ItemId::LoadedDice;
            battle.rng_seed = [seed_val, seed_val + 10, seed_val + 20, seed_val + 30];
            battle.execute_choice(0, Choice::Move(0));
            let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|")).unwrap();
            let hits: u32 = hitcount_line.split('|').last().unwrap().trim().parse().unwrap();
            assert!((4..=5).contains(&hits), "Loaded Dice should force 4-5 hits, got {}", hits);
        }
    }

    // ===== FIXED-COUNT MOVES =====

    #[test]
    fn double_kick_always_2_hits() {
        for seed_val in 0u16..10 {
            let mut battle = raw_battle(
                [slot(24, 30), empty(), empty(), empty()], // Double Kick
                [empty(), empty(), empty(), empty()],
            );
            battle.rng_seed = [seed_val, seed_val + 1, seed_val + 2, seed_val + 3];
            battle.execute_choice(0, Choice::Move(0));
            let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|")).unwrap();
            let hits: u32 = hitcount_line.split('|').last().unwrap().trim().parse().unwrap();
            assert_eq!(hits, 2, "Double Kick should always be 2 hits, got {}", hits);
        }
    }

    // ===== TRIPLE AXEL (ESCALATING) =====

    #[test]
    fn triple_axel_escalates_bp_and_stops_on_miss() {
        // Use enough seeds to see both full 3-hit and partial hit patterns
        let mut saw_full = false;
        let mut saw_partial = false;
        for seed_val in 0u16..100 {
            let mut battle = raw_battle_species(
                "Weavile", [slot(813, 10), empty(), empty(), empty()], // Triple Axel
                "Blissey", [empty(), empty(), empty(), empty()],
            );
            battle.rng_seed = [seed_val, seed_val + 5, seed_val + 10, seed_val + 15];
            let hp_before = battle.sides[1].active().hp;
            battle.execute_choice(0, Choice::Move(0));

            // Check if it missed entirely or hit
            let missed_all = battle.protocol.iter().any(|l| l.contains("[miss]"));
            if missed_all {
                // First hit missed
                assert_eq!(battle.sides[1].active().hp, hp_before);
                continue;
            }

            let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|"));
            if let Some(line) = hitcount_line {
                let hits: u32 = line.split('|').last().unwrap().trim().parse().unwrap();
                assert!((1..=3).contains(&hits), "Triple Axel should be 1-3 hits, got {}", hits);
                if hits == 3 { saw_full = true; }
                if hits < 3 { saw_partial = true; }
            }
        }
        // With 90% accuracy per hit, we should see both outcomes across 100 seeds
        assert!(saw_full, "Should see at least one full 3-hit Triple Axel");
        assert!(saw_partial, "Should see at least one partial-hit Triple Axel");
    }

    // ===== SCALE SHOT STAT CHANGES =====

    #[test]
    fn scale_shot_applies_speed_up_def_down_after() {
        let mut battle = raw_battle_species(
            "Garchomp", [slot(799, 20), empty(), empty(), empty()], // Scale Shot
            "Blissey", [empty(), empty(), empty(), empty()],
        );
        // Use a seed that gives a hit (Scale Shot acc=90 should mostly hit)
        battle.rng_seed = [100, 200, 300, 400];
        battle.execute_choice(0, Choice::Move(0));

        // If it didn't miss entirely, check secondaries
        let missed = battle.protocol.iter().any(|l| l.contains("[miss]"));
        if !missed {
            // Scale Shot secondaries: Def -1, Spe +1
            assert_eq!(battle.sides[0].active().boosts.spe, 1, "Scale Shot should +1 Spe");
            assert_eq!(battle.sides[0].active().boosts.def, -1, "Scale Shot should -1 Def");
        }
    }

    // ===== POPULATION BOMB WITH LOADED DICE =====

    #[test]
    fn population_bomb_loaded_dice_forces_4_or_5() {
        for seed_val in 0u16..20 {
            let mut battle = raw_battle(
                [slot(860, 10), empty(), empty(), empty()], // Population Bomb
                [empty(), empty(), empty(), empty()],
            );
            battle.sides[0].active_mut().item_id = ItemId::LoadedDice;
            battle.rng_seed = [seed_val, seed_val + 3, seed_val + 6, seed_val + 9];
            battle.execute_choice(0, Choice::Move(0));

            let missed = battle.protocol.iter().any(|l| l.contains("[miss]"));
            if missed { continue; }

            let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|")).unwrap();
            let hits: u32 = hitcount_line.split('|').last().unwrap().trim().parse().unwrap();
            assert!((4..=5).contains(&hits), "Pop Bomb + Loaded Dice = 4-5 hits, got {}", hits);
        }
    }

    // ===== SKILL LINK + POPULATION BOMB =====

    #[test]
    fn skill_link_population_bomb_gives_10_hits() {
        let mut battle = raw_battle(
            [slot(860, 10), empty(), empty(), empty()], // Population Bomb
            [empty(), empty(), empty(), empty()],
        );
        battle.sides[0].active_mut().ability_id = AbilityId::SkillLink;
        battle.rng_seed = [100, 200, 300, 400];
        battle.execute_choice(0, Choice::Move(0));

        let missed = battle.protocol.iter().any(|l| l.contains("[miss]"));
        if !missed {
            let hitcount_line = battle.protocol.iter().find(|l| l.contains("|-hitcount|")).unwrap();
            let hits: u32 = hitcount_line.split('|').last().unwrap().trim().parse().unwrap();
            assert_eq!(hits, 10, "Skill Link + Pop Bomb = 10 hits, got {}", hits);
        }
    }
}


#[cfg(test)]
mod tests_sleep {
    use crate::battle::Battle;
    use crate::pokemon::{MoveSlot, Pokemon, Status};
    use crate::side::Side;
    use crate::field::Terrain;
    use pkmn_core::nature::Nature;
    use pkmn_core::species::get_species;
    use pkmn_core::abilities::AbilityId;
    use pkmn_core::moves::get_move;

    fn slot(id: u16, pp: u8) -> MoveSlot { MoveSlot { move_id: id, pp, max_pp: pp } }
    fn empty() -> MoveSlot { MoveSlot { move_id: 0, pp: 0, max_pp: 0 } }

    fn make_battle(p1_species: &str, p1_moves: [MoveSlot; 4], p2_species: &str, p2_moves: [MoveSlot; 4]) -> Battle {
        let sp1 = get_species(p1_species).unwrap();
        let sp2 = get_species(p2_species).unwrap();
        let p1 = Pokemon::new(sp1, 100, Nature::Hardy, p1_moves, [0; 6], [31; 6]);
        let p2 = Pokemon::new(sp2, 100, Nature::Hardy, p2_moves, [0; 6], [31; 6]);
        Battle::new_raw(Side::new(vec![p1]), Side::new(vec![p2]))
    }

    // === 1. Sleep-inducing move sets Status::Sleep with counter 1..=3 ===
    #[test]
    fn test_spore_inflicts_sleep_with_valid_counter() {
        // Spore: id 147, 100% accuracy
        let spore_id = get_move("spore").unwrap().id;
        let mut battle = make_battle(
            "Breloom", [slot(spore_id, 15), empty(), empty(), empty()],
            "Dragonite", [empty(), empty(), empty(), empty()],
        );
        battle.execute_move(0, 0);
        let target = battle.sides[1].active();
        assert_eq!(target.status, Status::Sleep);
        assert!((1..=3).contains(&target.status_turns), "sleep counter should be 1-3, got {}", target.status_turns);
        assert!(battle.protocol.iter().any(|l| l.contains("|-status|") && l.contains("|slp")));
    }

    // === 2. Sleep can't-move + wake ===
    #[test]
    fn test_sleep_cant_move_then_wakes() {
        let spore_id = get_move("spore").unwrap().id;
        let tackle_id = get_move("tackle").unwrap().id;
        let mut battle = make_battle(
            "Breloom", [slot(spore_id, 15), empty(), empty(), empty()],
            "Dragonite", [slot(tackle_id, 35), empty(), empty(), empty()],
        );
        // Manually put p2 to sleep with 2 turns
        battle.sides[1].active_mut().status = Status::Sleep;
        battle.sides[1].active_mut().status_turns = 2;
        battle.sides[1].sleep_clause_count = 1;

        // Turn 1: p2 tries to move, counter 2→1, can't move
        battle.protocol.clear();
        battle.execute_move(1, 0);
        assert_eq!(battle.sides[1].active().status, Status::Sleep);
        assert_eq!(battle.sides[1].active().status_turns, 1);
        assert!(battle.protocol.iter().any(|l| l.contains("|cant|") && l.contains("|slp")));

        // Turn 2: p2 tries to move, counter 1→0, wakes and acts
        battle.protocol.clear();
        battle.execute_move(1, 0);
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert_eq!(battle.sides[1].active().status_turns, 0);
        assert!(battle.protocol.iter().any(|l| l.contains("|-curestatus|") && l.contains("|slp")));
        // Should have moved (Tackle) after waking
        assert!(battle.protocol.iter().any(|l| l.contains("|move|") && l.contains("Tackle")));
    }

    // === 3. Rest: 2-turn sleep + full HP ===
    #[test]
    fn test_rest_heals_fully_and_sleeps_2_turns() {
        let rest_id = get_move("rest").unwrap().id;
        let tackle_id = get_move("tackle").unwrap().id;
        let mut battle = make_battle(
            "Blissey", [slot(rest_id, 5), slot(tackle_id, 35), empty(), empty()],
            "Blissey", [slot(tackle_id, 35), empty(), empty(), empty()],
        );
        // Damage p1
        let max_hp = battle.sides[0].active().max_hp;
        battle.sides[0].active_mut().hp = max_hp / 2;

        battle.execute_move(0, 0);
        let mon = battle.sides[0].active();
        assert_eq!(mon.status, Status::Sleep);
        assert_eq!(mon.status_turns, 2);
        assert_eq!(mon.hp, mon.max_hp, "Rest should fully heal");
        assert!(battle.protocol.iter().any(|l| l.contains("|-status|") && l.contains("|slp|[from] move: Rest")));
        assert!(battle.protocol.iter().any(|l| l.contains("|-heal|")));
        // Rest-sleep does NOT increment sleep_clause_count
        assert_eq!(battle.sides[0].sleep_clause_count, 0);
    }

    // === 4a. Insomnia blocks sleep ===
    #[test]
    fn test_insomnia_blocks_sleep() {
        let spore_id = get_move("spore").unwrap().id;
        let mut battle = make_battle(
            "Breloom", [slot(spore_id, 15), empty(), empty(), empty()],
            "Blissey", [empty(), empty(), empty(), empty()],
        );
        battle.sides[1].active_mut().ability_id = AbilityId::Insomnia;

        battle.execute_move(0, 0);
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    // === 4b. Vital Spirit blocks sleep ===
    #[test]
    fn test_vital_spirit_blocks_sleep() {
        let spore_id = get_move("spore").unwrap().id;
        let mut battle = make_battle(
            "Breloom", [slot(spore_id, 15), empty(), empty(), empty()],
            "Blissey", [empty(), empty(), empty(), empty()],
        );
        battle.sides[1].active_mut().ability_id = AbilityId::VitalSpirit;

        battle.execute_move(0, 0);
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    // === 4c. Electric Terrain blocks sleep for grounded target ===
    #[test]
    fn test_electric_terrain_blocks_sleep_grounded() {
        let spore_id = get_move("spore").unwrap().id;
        let mut battle = make_battle(
            "Breloom", [slot(spore_id, 15), empty(), empty(), empty()],
            "Blissey", [empty(), empty(), empty(), empty()],
        );
        battle.field.terrain = Terrain::Electric;
        battle.field.terrain_turns = 5;

        battle.execute_move(0, 0);
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    // === 4d. Grass-type immunity to sleep powder (powder move) ===
    #[test]
    fn test_grass_type_immune_to_sleep_moves() {
        let sleep_powder_id = get_move("sleep powder").unwrap().id;
        let mut battle = make_battle(
            "Breloom", [slot(sleep_powder_id, 15), empty(), empty(), empty()],
            "Breloom", [empty(), empty(), empty(), empty()], // Grass/Fighting
        );

        battle.execute_move(0, 0);
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-immune|")));
    }

    // === 5. Sleep Clause Mod ===
    #[test]
    fn test_sleep_clause_prevents_second_sleep() {
        let spore_id = get_move("spore").unwrap().id;
        let tackle_id = get_move("tackle").unwrap().id;
        let sp = get_species("Blissey").unwrap();
        let p2a = Pokemon::new(sp, 100, Nature::Hardy, [slot(tackle_id, 35), empty(), empty(), empty()], [0; 6], [31; 6]);
        let p2b = Pokemon::new(sp, 100, Nature::Hardy, [slot(tackle_id, 35), empty(), empty(), empty()], [0; 6], [31; 6]);

        let sp1 = get_species("Breloom").unwrap();
        let p1 = Pokemon::new(sp1, 100, Nature::Hardy, [slot(spore_id, 15), empty(), empty(), empty()], [0; 6], [31; 6]);

        let mut battle = Battle::new_raw(
            Side::new(vec![p1]),
            Side::new(vec![p2a, p2b]),
        );

        // Put first mon to sleep
        battle.execute_move(0, 0);
        assert_eq!(battle.sides[1].active().status, Status::Sleep);
        assert_eq!(battle.sides[1].sleep_clause_count, 1);

        // Switch to second mon
        battle.sides[1].active_index = 1;
        battle.protocol.clear();

        // Try to sleep second mon — should fail due to Sleep Clause
        battle.execute_move(0, 0);
        assert_eq!(battle.sides[1].active().status, Status::None);
        assert!(battle.protocol.iter().any(|l| l.contains("|-fail|") && l.contains("|slp")));
    }
}
