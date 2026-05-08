use crate::battle::Battle;
use crate::field::{Terrain, Weather};
use pkmn_core::abilities::AbilityId;
use pkmn_core::moves::{MoveCategory, MoveData, MoveFlags};
use pkmn_core::types::Type;

impl Battle {
    /// Called when a Pokemon switches in
    pub fn trigger_ability_on_switch(&mut self, player: u8) {
        let ability = self.sides[player as usize].active().ability_id;
        match ability {
            AbilityId::Intimidate => {
                let opp = 1 - player;
                let cur = self.sides[opp as usize].active().boosts.atk;
                self.sides[opp as usize].active_mut().boosts.atk = (cur - 1).max(-6);
            }
            AbilityId::Drizzle => {
                self.field.weather = Weather::Rain;
                self.field.weather_turns = 5;
            }
            AbilityId::Drought => {
                self.field.weather = Weather::Sun;
                self.field.weather_turns = 5;
            }
            AbilityId::SandStream => {
                self.field.weather = Weather::Sand;
                self.field.weather_turns = 5;
            }
            AbilityId::SnowWarning => {
                self.field.weather = Weather::Snow;
                self.field.weather_turns = 5;
            }
            AbilityId::ElectricSurge => {
                self.field.terrain = Terrain::Electric;
                self.field.terrain_turns = 5;
            }
            AbilityId::GrassySurge => {
                self.field.terrain = Terrain::Grassy;
                self.field.terrain_turns = 5;
            }
            AbilityId::MistySurge => {
                self.field.terrain = Terrain::Misty;
                self.field.terrain_turns = 5;
            }
            AbilityId::PsychicSurge => {
                self.field.terrain = Terrain::Psychic;
                self.field.terrain_turns = 5;
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
            AbilityId::HugePower | AbilityId::PurePower => {
                if move_data.category == MoveCategory::Physical {
                    2.0
                } else {
                    1.0
                }
            }
            AbilityId::Technician => {
                if move_data.base_power <= 60 {
                    1.5
                } else {
                    1.0
                }
            }
            AbilityId::SheerForce => 1.3,
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
                // STAB becomes 2.0x; since STAB 1.5x is applied separately, multiply by 4/3
                let species = pkmn_core::species::get_species_by_id(mon.species_id);
                let has_stab = species
                    .map(|s| s.types.contains(&move_data.move_type))
                    .unwrap_or(false);
                if has_stab {
                    4.0 / 3.0
                } else {
                    1.0
                }
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
            AbilityId::TintedLens => {
                let effectiveness =
                    Type::effectiveness(move_data.move_type, &defender.types);
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

        // Defender's Ruin abilities
        match defender.ability_id {
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
