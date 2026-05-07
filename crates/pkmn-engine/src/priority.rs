use crate::battle::Battle;
use crate::choice::Choice;

impl Battle {
    /// Determine action order based on priority and speed
    pub fn determine_order(&self, p1: Choice, p2: Choice) -> Vec<(u8, Choice)> {
        let p1_priority = self.get_priority(0, p1);
        let p2_priority = self.get_priority(1, p2);

        if p1_priority != p2_priority {
            if p1_priority > p2_priority {
                vec![(0, p1), (1, p2)]
            } else {
                vec![(1, p2), (0, p1)]
            }
        } else {
            let p1_speed = self.sides[0].active().effective_speed();
            let p2_speed = self.sides[1].active().effective_speed();

            if p1_speed == p2_speed {
                return vec![(0, p1), (1, p2)];
            }

            let (faster, slower) = if self.field.trick_room > 0 {
                if p1_speed <= p2_speed { (0u8, 1u8) } else { (1u8, 0u8) }
            } else {
                if p1_speed >= p2_speed { (0u8, 1u8) } else { (1u8, 0u8) }
            };

            vec![
                (faster, if faster == 0 { p1 } else { p2 }),
                (slower, if slower == 0 { p1 } else { p2 }),
            ]
        }
    }

    fn get_priority(&self, player: u8, choice: Choice) -> i8 {
        match choice {
            Choice::Switch(_) => 6,
            Choice::Move(idx) => {
                let mon = self.sides[player as usize].active();
                let move_id = mon.moves[idx as usize].move_id;
                pkmn_core::moves::get_move_by_id(move_id)
                    .map(|m| m.priority)
                    .unwrap_or(0)
            }
        }
    }
}
