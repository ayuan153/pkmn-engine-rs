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
            && self.sides[player as usize].active().status == Status::None
        {
            let mon = self.sides[player as usize].active_mut();
            if mon.types.contains(&Type::Poison) {
                // Poison types absorb toxic spikes
            } else if conditions.toxic_spikes >= 2 {
                mon.status = Status::Toxic;
            } else {
                mon.status = Status::Poison;
            }
        }

        if self.sides[player as usize].active().hp == 0 {
            self.sides[player as usize].active_mut().is_fainted = true;
        }
    }
}
