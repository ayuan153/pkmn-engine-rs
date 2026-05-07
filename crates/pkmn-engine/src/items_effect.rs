use crate::battle::Battle;
use crate::pokemon::Status;
use pkmn_core::items::ItemId;
use pkmn_core::moves::{MoveCategory, MoveData};

impl Battle {
    /// Get damage modifier from attacker's item
    pub fn item_damage_modifier(&self, attacker_player: u8, move_data: &MoveData) -> f32 {
        let item = self.sides[attacker_player as usize].active().item_id;
        match item {
            ItemId::ChoiceBand => {
                if move_data.category == MoveCategory::Physical {
                    1.5
                } else {
                    1.0
                }
            }
            ItemId::ChoiceSpecs => {
                if move_data.category == MoveCategory::Special {
                    1.5
                } else {
                    1.0
                }
            }
            ItemId::LifeOrb => 1.3,
            ItemId::ExpertBelt => 1.2,
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

    /// End-of-turn item effects
    pub fn trigger_item_end_of_turn(&mut self, player: u8) {
        let mon = self.sides[player as usize].active_mut();
        if !mon.is_alive() {
            return;
        }
        match mon.item_id {
            ItemId::Leftovers | ItemId::BlackSludge => {
                let heal = mon.max_hp / 16;
                mon.hp = (mon.hp + heal).min(mon.max_hp);
            }
            ItemId::FlameOrb => {
                if mon.status == Status::None {
                    mon.status = Status::Burn;
                }
            }
            ItemId::ToxicOrb => {
                if mon.status == Status::None {
                    mon.status = Status::Toxic;
                }
            }
            _ => {}
        }
    }

    /// Life Orb recoil after dealing damage
    pub fn apply_life_orb_recoil(&mut self, player: u8) {
        let mon = self.sides[player as usize].active_mut();
        if mon.item_id == ItemId::LifeOrb {
            let recoil = (mon.max_hp / 10).max(1);
            mon.hp = mon.hp.saturating_sub(recoil);
            if mon.hp == 0 {
                mon.is_fainted = true;
            }
        }
    }

    /// Focus Sash: survive at 1 HP from full
    pub fn check_focus_sash(&mut self, player: u8, damage: u16) -> u16 {
        let mon = self.sides[player as usize].active();
        if mon.item_id == ItemId::FocusSash && mon.hp == mon.max_hp && damage >= mon.hp {
            let adjusted = mon.hp - 1;
            self.sides[player as usize].active_mut().item_id = ItemId::None;
            return adjusted;
        }
        damage
    }

    /// Heavy-Duty Boots: skip entry hazards
    pub fn has_heavy_duty_boots(&self, player: u8) -> bool {
        self.sides[player as usize].active().item_id == ItemId::HeavyDutyBoots
    }
}
