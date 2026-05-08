use crate::battle::Battle;
use crate::pokemon::Status;
use pkmn_core::types::Type;

impl Battle {
    pub fn apply_entry_hazards(&mut self, player: u8) {
        let conditions = self.sides[player as usize].side_conditions;

        // Stealth Rock
        if conditions.stealth_rock {
            let mon = self.sides[player as usize].active_mut();
            let eff = Type::effectiveness(Type::Rock, &mon.types);
            let dmg = ((mon.max_hp as f32 * eff) / 8.0) as u16;
            mon.hp = mon.hp.saturating_sub(dmg.max(1));
            let name = self.species_name(player);
            let hp = self.sides[player as usize].active().hp;
            let max_hp = self.sides[player as usize].active().max_hp;
            if hp == 0 {
                self.emit(format!("|-damage|p{}a: {}|0 fnt|[from] Stealth Rock", player+1, name));
            } else {
                self.emit(format!("|-damage|p{}a: {}|{}/{}|[from] Stealth Rock", player+1, name, hp, max_hp));
            }
        }

        // Spikes (grounded only)
        if conditions.spikes > 0 && !self.sides[player as usize].active().types.contains(&Type::Flying) {
            let mon = self.sides[player as usize].active_mut();
            let fraction = match conditions.spikes {
                1 => 8,
                2 => 6,
                _ => 4,
            };
            let dmg = (mon.max_hp / fraction).max(1);
            mon.hp = mon.hp.saturating_sub(dmg);
            let name = self.species_name(player);
            let hp = self.sides[player as usize].active().hp;
            let max_hp = self.sides[player as usize].active().max_hp;
            self.emit(format!("|-damage|p{}a: {}|{}/{}|[from] Spikes", player+1, name, hp, max_hp));
        }

        // Toxic Spikes (grounded only)
        if conditions.toxic_spikes > 0
            && !self.sides[player as usize].active().types.contains(&Type::Flying)
            && self.sides[player as usize].active().ability_id != pkmn_core::abilities::AbilityId::Levitate
            && self.sides[player as usize].active().status == Status::None
        {
            let mon = self.sides[player as usize].active_mut();
            if mon.types.contains(&Type::Poison) {
                // Poison types absorb toxic spikes
                self.sides[player as usize].side_conditions.toxic_spikes = 0;
                let name = self.species_name(player);
                self.emit(format!("|-sideend|p{}: Player {}|move: Toxic Spikes|[of] p{}a: {}", player+1, player+1, player+1, name));
            } else if conditions.toxic_spikes >= 2 {
                let mon = self.sides[player as usize].active_mut();
                mon.status = Status::Toxic;
                let name = self.species_name(player);
                self.emit(format!("|-status|p{}a: {}|tox", player+1, name));
            } else {
                let mon = self.sides[player as usize].active_mut();
                mon.status = Status::Poison;
                let name = self.species_name(player);
                self.emit(format!("|-status|p{}a: {}|psn", player+1, name));
            }
        }

        // Sticky Web (grounded only)
        if conditions.sticky_web
            && !self.sides[player as usize].active().types.contains(&Type::Flying)
            && self.sides[player as usize].active().ability_id != pkmn_core::abilities::AbilityId::Levitate
            && self.sides[player as usize].active().item_id != pkmn_core::items::ItemId::AirBalloon
        {
            let mon = self.sides[player as usize].active_mut();
            mon.boosts.spe = (mon.boosts.spe - 1).max(-6);
            let name = self.species_name(player);
            self.emit(format!("|-activate|p{}a: {}|move: Sticky Web", player+1, name));
            self.emit(format!("|-unboost|p{}a: {}|spe|1", player+1, name));
        }

        if self.sides[player as usize].active().hp == 0 {
            self.sides[player as usize].active_mut().is_fainted = true;
        }
    }
}
