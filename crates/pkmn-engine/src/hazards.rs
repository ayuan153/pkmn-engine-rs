use crate::battle::Battle;
use crate::pokemon::Status;
use pkmn_core::types::Type;

impl Battle {
    pub fn apply_entry_hazards(&mut self, player: u8) {
        let conditions = self.sides[player as usize].side_conditions;
        let mon = self.sides[player as usize].active_mut();

        // Stealth Rock
        if conditions.stealth_rock {
            let eff = Type::effectiveness(Type::Rock, &mon.types);
            let dmg = ((mon.max_hp as f32 * eff) / 8.0) as u16;
            mon.hp = mon.hp.saturating_sub(dmg.max(1));
        }

        // Spikes (grounded only)
        if conditions.spikes > 0 && !mon.types.contains(&Type::Flying) {
            let fraction = match conditions.spikes {
                1 => 8,
                2 => 6,
                _ => 4,
            };
            let dmg = (mon.max_hp / fraction).max(1);
            mon.hp = mon.hp.saturating_sub(dmg);
        }

        // Toxic Spikes (grounded only)
        if conditions.toxic_spikes > 0
            && !mon.types.contains(&Type::Flying)
            && mon.status == Status::None
        {
            if mon.types.contains(&Type::Poison) {
                // Poison types absorb toxic spikes
            } else if conditions.toxic_spikes >= 2 {
                mon.status = Status::Toxic;
            } else {
                mon.status = Status::Poison;
            }
        }

        if mon.hp == 0 {
            mon.is_fainted = true;
        }
    }
}
