use crate::battle::Battle;
use crate::pokemon::Status;
use pkmn_core::items::ItemId;
use pkmn_core::moves::MoveData;
use pkmn_core::types::Type;

impl Battle {
    /// Get damage modifier from attacker's item
    pub fn item_damage_modifier(&self, attacker_player: u8, move_data: &MoveData) -> f32 {
        let item = self.sides[attacker_player as usize].active().item_id;
        let defender_player = 1 - attacker_player;
        let defender = self.sides[defender_player as usize].active();

        // Generic hook dispatch: if the item has on_source_modify_damage, use it directly
        let hooks = crate::events::item_hooks(item);
        if let Some(hook_fn) = hooks.on_source_modify_damage {
            return hook_fn(self, attacker_player, move_data);
        }

        // Non-migrated item modifiers (no hook yet)
        match item {
            ItemId::ChoiceBand => 1.0,
            ItemId::ChoiceSpecs => 1.0,
            ItemId::ExpertBelt => {
                let effectiveness = Type::effectiveness(move_data.move_type, &defender.types);
                if effectiveness > 1.0 { 1.2 } else { 1.0 }
            }
            // Type-boosting items and Muscle Band/Wise Glasses: handled in apply_bp_modifiers
            _ => 1.0,
        }
    }

    /// Apply item speed modifier
    pub fn item_speed_modifier(&self, player: u8) -> f32 {
        match self.sides[player as usize].active().item_id {
            ItemId::ChoiceScarf => 1.5,
            _ => 1.0,
        }
    }

    /// End-of-turn item effects: healing (Leftovers, Black Sludge)
    pub fn trigger_item_heal_eot(&mut self, player: u8) {
        let mon = self.sides[player as usize].active_mut();
        if !mon.is_alive() {
            return;
        }
        match mon.item_id {
            ItemId::Leftovers | ItemId::BlackSludge => {
                let heal = mon.max_hp / 16;
                if mon.hp < mon.max_hp {
                    mon.hp = (mon.hp + heal).min(mon.max_hp);
                    let name = self.species_name(player);
                    let hp = self.sides[player as usize].active().hp;
                    let max_hp = self.sides[player as usize].active().max_hp;
                    let item_name = if self.sides[player as usize].active().item_id == ItemId::Leftovers { "Leftovers" } else { "Black Sludge" };
                    self.emit(format!("|-heal|p{}a: {}|{}/{}|[from] item: {}", player+1, name, hp, max_hp, item_name));
                }
            }
            _ => {}
        }
    }

    /// End-of-turn item effects: status orbs (Flame Orb, Toxic Orb)
    pub fn trigger_item_orb_eot(&mut self, player: u8) {
        let mon = self.sides[player as usize].active_mut();
        if !mon.is_alive() {
            return;
        }
        match mon.item_id {
            ItemId::FlameOrb => {
                if mon.status == Status::None {
                    mon.status = Status::Burn;
                    let name = self.species_name(player);
                    self.emit(format!("|-status|p{}a: {}|brn|[from] item: Flame Orb", player+1, name));
                }
            }
            ItemId::ToxicOrb => {
                if mon.status == Status::None {
                    mon.status = Status::Toxic;
                    let name = self.species_name(player);
                    self.emit(format!("|-status|p{}a: {}|tox|[from] item: Toxic Orb", player+1, name));
                }
            }
            _ => {}
        }
    }

    /// End-of-turn item effects (legacy, calls both)
    pub fn trigger_item_end_of_turn(&mut self, player: u8) {
        self.trigger_item_heal_eot(player);
        self.trigger_item_orb_eot(player);
    }

    /// Life Orb recoil after dealing damage
    pub fn apply_life_orb_recoil(&mut self, player: u8) {
        let mon = self.sides[player as usize].active_mut();
        if mon.item_id == ItemId::LifeOrb && mon.is_alive()
            && mon.ability_id != pkmn_core::abilities::AbilityId::MagicGuard
        {
            let recoil = (mon.max_hp / 10).max(1);
            mon.hp = mon.hp.saturating_sub(recoil);
            if mon.hp == 0 {
                mon.is_fainted = true;
            }
            let name = self.species_name(player);
            let hp = self.sides[player as usize].active().hp;
            let max_hp = self.sides[player as usize].active().max_hp;
            if hp == 0 {
                self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] item: Life Orb", player+1, name));
            } else {
                self.emit(format!("|-damage|p{}a: {}|{}/{}|[from] item: Life Orb", player+1, name, hp, max_hp));
            }
        }
    }

    /// Focus Sash: survive at 1 HP from full
    pub fn check_focus_sash(&mut self, player: u8, damage: u16) -> u16 {
        let mon = self.sides[player as usize].active();
        if mon.item_id == ItemId::FocusSash && mon.hp == mon.max_hp && damage >= mon.hp {
            let adjusted = mon.hp - 1;
            self.sides[player as usize].active_mut().item_id = ItemId::None;
            let name = self.species_name(player);
            self.emit(format!("|-enditem|p{}a: {}|Focus Sash", player+1, name));
            return adjusted;
        }
        damage
    }

    /// Heavy-Duty Boots: skip entry hazards
    pub fn has_heavy_duty_boots(&self, player: u8) -> bool {
        self.sides[player as usize].active().item_id == ItemId::HeavyDutyBoots
    }
}
