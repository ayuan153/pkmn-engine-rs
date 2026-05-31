use crate::battle::Battle;
use crate::field::Terrain;
use pkmn_core::abilities::AbilityId;
use pkmn_core::moves::{MoveCategory, MoveData, MoveFlags};
use pkmn_core::types::Type;

impl Battle {
    /// Called when a Pokemon switches in
    pub fn trigger_ability_on_switch(&mut self, player: u8) {
        let ability = self.sides[player as usize].active().ability_id;
        let name = self.species_name(player);

        // Hook dispatch: migrated abilities use the event system
        let hooks = crate::events::ability_hooks(ability);
        if let Some(hook) = hooks.on_switch_in {
            hook(self, player);
            return;
        }

        match ability {
            // Intimidate, Drizzle, Drought, SandStream, SnowWarning
            // are now dispatched via EventHooks (see events.rs)
            AbilityId::ElectricSurge => {
                self.field.terrain = Terrain::Electric;
                self.field.terrain_turns = 5;
                self.emit(format!("|-fieldstart|move: Electric Terrain|[from] ability: Electric Surge|[of] p{}a: {}", player+1, name));
            }
            AbilityId::GrassySurge => {
                self.field.terrain = Terrain::Grassy;
                self.field.terrain_turns = 5;
                self.emit(format!("|-fieldstart|move: Grassy Terrain|[from] ability: Grassy Surge|[of] p{}a: {}", player+1, name));
            }
            AbilityId::MistySurge => {
                self.field.terrain = Terrain::Misty;
                self.field.terrain_turns = 5;
                self.emit(format!("|-fieldstart|move: Misty Terrain|[from] ability: Misty Surge|[of] p{}a: {}", player+1, name));
            }
            AbilityId::PsychicSurge => {
                self.field.terrain = Terrain::Psychic;
                self.field.terrain_turns = 5;
                self.emit(format!("|-fieldstart|move: Psychic Terrain|[from] ability: Psychic Surge|[of] p{}a: {}", player+1, name));
            }
            AbilityId::Pressure => {
                self.emit(format!("|-ability|p{}a: {}|Pressure", player+1, name));
            }
            AbilityId::Unnerve => {
                self.emit(format!("|-ability|p{}a: {}|Unnerve", player+1, name));
            }
            AbilityId::CloudNine => {
                self.emit(format!("|-ability|p{}a: {}|Cloud Nine", player+1, name));
            }
            AbilityId::MoldBreaker => {
                self.emit(format!("|-ability|p{}a: {}|Mold Breaker", player+1, name));
            }
            AbilityId::Turboblaze => {
                self.emit(format!("|-ability|p{}a: {}|Turboblaze", player+1, name));
            }
            AbilityId::Teravolt => {
                self.emit(format!("|-ability|p{}a: {}|Teravolt", player+1, name));
            }
            _ => {}
        }
    }

    /// Check if a move is immune due to ability
    pub fn check_ability_immunity(&self, defender_player: u8, move_type: Type) -> bool {
        let ability = self.sides[defender_player as usize].active().ability_id;
        matches!(
            (ability, move_type),
            (AbilityId::Levitate, Type::Ground)
                | (AbilityId::FlashFire, Type::Fire)
                | (AbilityId::VoltAbsorb, Type::Electric)
                | (AbilityId::WaterAbsorb, Type::Water)
                | (AbilityId::LightningRod, Type::Electric)
                | (AbilityId::StormDrain, Type::Water)
        )
    }

    /// Get damage modifier from attacker's ability
    pub fn ability_damage_modifier(&self, attacker_player: u8, move_data: &MoveData) -> f32 {
        let mon = self.sides[attacker_player as usize].active();
        let defender_player = 1 - attacker_player;
        let defender = self.sides[defender_player as usize].active();
        let mut modifier = match mon.ability_id {
            AbilityId::HugePower | AbilityId::PurePower => 1.0,
            AbilityId::Technician => {
                // Dispatched via EventHooks
                let hooks = crate::events::ability_hooks(AbilityId::Technician);
                hooks.on_source_modify_damage.unwrap()(self, attacker_player, move_data)
            }
            AbilityId::SheerForce => {
                if !pkmn_core::moves::get_secondaries(move_data.id).is_empty() {
                    1.3
                } else {
                    1.0
                }
            }
            AbilityId::ToughClaws => {
                if move_data.flags.has(MoveFlags::CONTACT) {
                    1.3
                } else {
                    1.0
                }
            }
            AbilityId::IronFist => {
                if move_data.flags.has(MoveFlags::PUNCH) {
                    1.2
                } else {
                    1.0
                }
            }
            AbilityId::StrongJaw => {
                if move_data.flags.has(MoveFlags::BITE) {
                    1.5
                } else {
                    1.0
                }
            }
            AbilityId::Adaptability => {
                // Dispatched via EventHooks
                let hooks = crate::events::ability_hooks(AbilityId::Adaptability);
                hooks.on_source_modify_damage.unwrap()(self, attacker_player, move_data)
            }
            AbilityId::Guts => {
                if mon.status != crate::pokemon::Status::None
                    && move_data.category == MoveCategory::Physical
                {
                    1.5
                } else {
                    1.0
                }
            }
            AbilityId::FlashFire => {
                if mon.volatiles.contains(crate::pokemon::Volatiles::FLASH_FIRE)
                    && move_data.move_type == Type::Fire
                {
                    1.5
                } else {
                    1.0
                }
            }
            AbilityId::TintedLens => {
                let def_types = self.defender_types(defender_player);
                let effectiveness =
                    Type::effectiveness(move_data.move_type, &def_types);
                if effectiveness < 1.0 && effectiveness > 0.0 {
                    2.0
                } else {
                    1.0
                }
            }
            AbilityId::SwordOfRuin => {
                if move_data.category == MoveCategory::Physical {
                    1.33
                } else {
                    1.0
                }
            }
            _ => 1.0,
        };

        // Defender's abilities
        match defender.ability_id {
            AbilityId::Multiscale => {
                if defender.hp == defender.max_hp {
                    modifier *= 0.5;
                }
            }
            AbilityId::TabletsOfRuin => {
                if move_data.category == MoveCategory::Physical {
                    modifier *= 0.75;
                }
            }
            AbilityId::VesselOfRuin => {
                if move_data.category == MoveCategory::Special {
                    modifier *= 0.75;
                }
            }
            _ => {}
        }
        // Attacker's Beads of Ruin boosts special damage (reduces opponent SpD)
        if mon.ability_id == AbilityId::BeadsOfRuin
            && move_data.category == MoveCategory::Special
        {
            modifier *= 1.33;
        }

        modifier
    }

    /// End-of-turn ability effects
    pub fn trigger_ability_end_of_turn(&mut self, player: u8) {
        let ability = self.sides[player as usize].active().ability_id;
        match ability {
            AbilityId::SpeedBoost => {
                let mon = self.sides[player as usize].active_mut();
                mon.boosts.spe = (mon.boosts.spe + 1).min(6);
            }
            _ => {}
        }
    }
}
