use crate::battle::Battle;
use crate::pokemon::Status;
use pkmn_core::items::ItemId;
use pkmn_core::moves::{MoveCategory, MoveData};
use pkmn_core::types::Type;

impl Battle {
    /// Get damage modifier from attacker's item
    pub fn item_damage_modifier(&self, attacker_player: u8, move_data: &MoveData) -> f32 {
        let item = self.sides[attacker_player as usize].active().item_id;
        let defender_player = 1 - attacker_player;
        let defender = self.sides[defender_player as usize].active();
        let modifier = match item {
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
            ItemId::ExpertBelt => {
                let effectiveness = Type::effectiveness(move_data.move_type, &defender.types);
                if effectiveness > 1.0 { 1.2 } else { 1.0 }
            }
            // Type-boosting items (1.2x)
            ItemId::MysticWater => if move_data.move_type == Type::Water { 1.2 } else { 1.0 },
            ItemId::Charcoal => if move_data.move_type == Type::Fire { 1.2 } else { 1.0 },
            ItemId::Magnet => if move_data.move_type == Type::Electric { 1.2 } else { 1.0 },
            ItemId::MiracleSeed => if move_data.move_type == Type::Grass { 1.2 } else { 1.0 },
            ItemId::NeverMeltIce => if move_data.move_type == Type::Ice { 1.2 } else { 1.0 },
            ItemId::BlackBelt => if move_data.move_type == Type::Fighting { 1.2 } else { 1.0 },
            ItemId::PoisonBarb => if move_data.move_type == Type::Poison { 1.2 } else { 1.0 },
            ItemId::SoftSand => if move_data.move_type == Type::Ground { 1.2 } else { 1.0 },
            ItemId::SharpBeak => if move_data.move_type == Type::Flying { 1.2 } else { 1.0 },
            ItemId::TwistedSpoon => if move_data.move_type == Type::Psychic { 1.2 } else { 1.0 },
            ItemId::SilverPowder => if move_data.move_type == Type::Bug { 1.2 } else { 1.0 },
            ItemId::HardStone => if move_data.move_type == Type::Rock { 1.2 } else { 1.0 },
            ItemId::SpellTag => if move_data.move_type == Type::Ghost { 1.2 } else { 1.0 },
            ItemId::DragonFang => if move_data.move_type == Type::Dragon { 1.2 } else { 1.0 },
            ItemId::BlackGlasses => if move_data.move_type == Type::Dark { 1.2 } else { 1.0 },
            ItemId::MetalCoat => if move_data.move_type == Type::Steel { 1.2 } else { 1.0 },
            ItemId::SilkScarf => if move_data.move_type == Type::Normal { 1.2 } else { 1.0 },
            ItemId::FairyFeather => if move_data.move_type == Type::Fairy { 1.2 } else { 1.0 },
            _ => 1.0,
        };
        modifier
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
                if mon.hp < mon.max_hp {
                    mon.hp = (mon.hp + heal).min(mon.max_hp);
                    let name = self.species_name(player);
                    let hp = self.sides[player as usize].active().hp;
                    let max_hp = self.sides[player as usize].active().max_hp;
                    let item_name = if self.sides[player as usize].active().item_id == ItemId::Leftovers { "Leftovers" } else { "Black Sludge" };
                    self.emit(format!("|-heal|p{}a: {}|{}/{}|[from] item: {}", player+1, name, hp, max_hp, item_name));
                }
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
        if mon.item_id == ItemId::LifeOrb && mon.is_alive() {
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
