use crate::battle::Battle;
use crate::choice::Choice;
use crate::field::{Terrain, Weather};
use crate::pokemon::Volatiles;
use pkmn_core::abilities::AbilityId;
use pkmn_core::moves::MoveCategory;

impl Battle {
    /// Determine action order based on priority bracket, then effective speed.
    /// Within the same priority bracket, Trick Room reverses speed comparison.
    pub fn determine_order(&mut self, p1: Choice, p2: Choice) -> Vec<(u8, Choice)> {
        let p1_priority = self.get_priority(0, p1);
        let p2_priority = self.get_priority(1, p2);

        if p1_priority != p2_priority {
            if p1_priority > p2_priority {
                vec![(0, p1), (1, p2)]
            } else {
                vec![(1, p2), (0, p1)]
            }
        } else {
            // Same priority bracket: compare effective speed
            let p1_speed = self.ordering_speed(0);
            let p2_speed = self.ordering_speed(1);

            if p1_speed == p2_speed {
                // Speed tie: random coin flip (PS behavior)
                if self.random_chance(1, 2) {
                    return vec![(0, p1), (1, p2)];
                } else {
                    return vec![(1, p2), (0, p1)];
                }
            }

            // Trick Room: slower acts first within same priority bracket
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

    /// Compute effective speed for ordering, applying ability and item modifiers.
    /// Does NOT change stored stats.
    pub(crate) fn ordering_speed(&self, player: u8) -> u16 {
        let base = self.sides[player as usize].active().effective_speed() as f32;
        let ability_mod = self.ability_speed_modifier(player);
        let item_mod = self.item_speed_modifier(player);
        (base * ability_mod * item_mod) as u16
    }

    /// Speed multiplier from abilities (weather/terrain doublers + Unburden).
    fn ability_speed_modifier(&self, player: u8) -> f32 {
        let mon = self.sides[player as usize].active();
        match mon.ability_id {
            AbilityId::SwiftSwim if self.field.weather == Weather::Rain => 2.0,
            AbilityId::Chlorophyll if self.field.weather == Weather::Sun => 2.0,
            AbilityId::SandRush if self.field.weather == Weather::Sand => 2.0,
            AbilityId::SlushRush if self.field.weather == Weather::Snow => 2.0,
            AbilityId::SurgeSurfer if self.field.terrain == Terrain::Electric => 2.0,
            AbilityId::Unburden if mon.volatiles.contains(Volatiles::UNBURDEN) => 2.0,
            _ => 1.0,
        }
    }

    /// Get effective priority for a choice, accounting for priority-modifying abilities.
    fn get_priority(&self, player: u8, choice: Choice) -> i8 {
        match choice {
            Choice::Switch(_) => 6, // Switches always have +6 priority
            Choice::Move(idx) | Choice::Tera(idx) => {
                if idx == 255 {
                    return 0; // Struggle has priority 0
                }
                let mon = self.sides[player as usize].active();
                let move_id = mon.moves[idx as usize].move_id;
                let move_data = match pkmn_core::moves::get_move_by_id(move_id) {
                    Some(m) => m,
                    None => return 0,
                };
                let mut priority = move_data.priority;

                // Prankster: +1 priority to Status-category moves
                if mon.ability_id == AbilityId::Prankster
                    && move_data.category == MoveCategory::Status
                {
                    priority += 1;
                }

                // Gale Wings: +1 priority to Flying-type moves at full HP
                if mon.ability_id == AbilityId::GaleWings
                    && move_data.move_type == pkmn_core::types::Type::Flying
                    && mon.hp == mon.max_hp
                {
                    priority += 1;
                }

                // Triage: +3 priority to healing/drain moves
                if mon.ability_id == AbilityId::Triage && Self::is_triage_move(move_data) {
                    priority += 3;
                }

                priority
            }
        }
    }

    /// Returns true if this move is boosted by Triage (drain moves + recovery moves).
    fn is_triage_move(move_data: &pkmn_core::moves::MoveData) -> bool {
        let name = move_data.name.to_lowercase();
        matches!(name.as_str(),
            "drain punch" | "giga drain" | "horn leech" | "leech life"
            | "oblivion wing" | "parabolic charge" | "draining kiss" | "mega drain"
            | "absorb" | "recover" | "soft-boiled" | "slack off" | "milk drink"
            | "roost" | "synthesis" | "morning sun" | "moonlight"
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::battle::Battle;
    use crate::choice::Choice;
    use crate::field::{Terrain, Weather};
    use crate::pokemon::{MoveSlot, Pokemon, Volatiles};
    use crate::side::Side;
    use pkmn_core::abilities::AbilityId;
    use pkmn_core::nature::Nature;
    use pkmn_core::species::get_species;

    fn slot(id: u16, pp: u8) -> MoveSlot { MoveSlot { move_id: id, pp, max_pp: pp } }
    fn empty() -> MoveSlot { MoveSlot { move_id: 0, pp: 0, max_pp: 0 } }

    /// Build a raw battle with custom species and speed stats for ordering tests.
    fn ordering_battle(
        p1_species: &str, p1_moves: [MoveSlot; 4], p1_ability: AbilityId,
        p2_species: &str, p2_moves: [MoveSlot; 4], p2_ability: AbilityId,
    ) -> Battle {
        let sp1 = get_species(p1_species).unwrap();
        let sp2 = get_species(p2_species).unwrap();
        let mut p1 = Pokemon::new(sp1, 100, Nature::Hardy, p1_moves, [0; 6], [31; 6]);
        let mut p2 = Pokemon::new(sp2, 100, Nature::Hardy, p2_moves, [0; 6], [31; 6]);
        p1.ability_id = p1_ability;
        p2.ability_id = p2_ability;
        Battle::new_raw(Side::new(vec![p1]), Side::new(vec![p2]))
    }

    // (a) A +priority move (Quick Attack, priority +1) user acts before a faster foe
    #[test]
    fn priority_move_goes_first_vs_faster_foe() {
        // Quick Attack (id=98) has priority +1. Use slow P1 vs fast P2.
        // Blissey (base spe 55) vs Dragapult (base spe 142)
        let mut battle = ordering_battle(
            "Blissey", [slot(98, 30), empty(), empty(), empty()], AbilityId::None,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        // P1 (Blissey with Quick Attack +1) should go first
        assert_eq!(order[0].0, 0, "Quick Attack user should go first vs faster foe");
    }

    // (b) Prankster makes a status move go first vs a faster foe
    #[test]
    fn prankster_status_move_goes_first() {
        // Thunder Wave (id=86, Status, priority 0). Prankster makes it +1.
        // Slow Blissey vs fast Dragapult
        let mut battle = ordering_battle(
            "Blissey", [slot(86, 20), empty(), empty(), empty()], AbilityId::Prankster,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "Prankster T-Wave should outprioritize faster foe");
    }

    // Prankster does NOT boost damaging moves
    #[test]
    fn prankster_no_boost_on_damaging_move() {
        // Flamethrower (id=53, Special, priority 0) — Prankster shouldn't boost it
        let mut battle = ordering_battle(
            "Blissey", [slot(53, 15), empty(), empty(), empty()], AbilityId::Prankster,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        // P2 (Dragapult) is faster, should go first since Prankster doesn't boost special moves
        assert_eq!(order[0].0, 1, "Prankster should not boost damaging moves");
    }

    // (c) Gale Wings gives priority to Flying moves only at full HP
    #[test]
    fn gale_wings_full_hp_gives_priority() {
        // Brave Bird (id=413, Flying, priority 0). Gale Wings makes it +1 at full HP.
        let mut battle = ordering_battle(
            "Blissey", [slot(413, 15), empty(), empty(), empty()], AbilityId::GaleWings,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "Gale Wings at full HP should give +1 priority to Flying move");
    }

    #[test]
    fn gale_wings_not_at_full_hp() {
        // Not full HP: Gale Wings does NOT apply
        let mut battle = ordering_battle(
            "Blissey", [slot(413, 15), empty(), empty(), empty()], AbilityId::GaleWings,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.sides[0].active_mut().hp -= 1; // Not full HP
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        // Dragapult is faster, so it goes first
        assert_eq!(order[0].0, 1, "Gale Wings should not boost if not at full HP");
    }

    #[test]
    fn gale_wings_non_flying_no_boost() {
        // Earthquake (id=89, Ground) — Gale Wings only boosts Flying
        let mut battle = ordering_battle(
            "Blissey", [slot(89, 10), empty(), empty(), empty()], AbilityId::GaleWings,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 1, "Gale Wings should not boost non-Flying moves");
    }

    // (d) Swift Swim user outspeeds a faster foe in Rain but not in no-weather
    #[test]
    fn swift_swim_doubles_speed_in_rain() {
        // Blissey (base spe 55) with Swift Swim vs Dragapult (base spe 142)
        // In Rain, Blissey effective speed ~ 55*2 = ~200+ (after stat calc) vs Dragapult ~284
        // Actually we need a mon that's fast enough with 2x to beat Dragapult.
        // Use custom: Garchomp (base spe 102) vs Dragapult (base spe 142)
        // Garchomp effective speed @ lv100: (2*100/5+2)*102/50+5 -> roughly 236
        // With 2x = 472. Dragapult ~ 284+. That works.
        let mut battle = ordering_battle(
            "Garchomp", [slot(89, 10), empty(), empty(), empty()], AbilityId::SwiftSwim,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        // No weather: Dragapult faster
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 1, "Without Rain, Dragapult is faster");

        // Set Rain
        battle.field.weather = Weather::Rain;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "With Rain + Swift Swim, Garchomp outspeeds Dragapult");
    }

    #[test]
    fn chlorophyll_doubles_speed_in_sun() {
        let mut battle = ordering_battle(
            "Garchomp", [slot(89, 10), empty(), empty(), empty()], AbilityId::Chlorophyll,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.field.weather = Weather::Sun;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "With Sun + Chlorophyll, Garchomp outspeeds");
    }

    #[test]
    fn sand_rush_doubles_speed_in_sand() {
        let mut battle = ordering_battle(
            "Garchomp", [slot(89, 10), empty(), empty(), empty()], AbilityId::SandRush,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.field.weather = Weather::Sand;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "With Sand + Sand Rush, Garchomp outspeeds");
    }

    #[test]
    fn slush_rush_doubles_speed_in_snow() {
        let mut battle = ordering_battle(
            "Garchomp", [slot(89, 10), empty(), empty(), empty()], AbilityId::SlushRush,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.field.weather = Weather::Snow;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "With Snow + Slush Rush, Garchomp outspeeds");
    }

    #[test]
    fn surge_surfer_doubles_speed_in_electric_terrain() {
        let mut battle = ordering_battle(
            "Garchomp", [slot(89, 10), empty(), empty(), empty()], AbilityId::SurgeSurfer,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.field.terrain = Terrain::Electric;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "With Electric Terrain + Surge Surfer, Garchomp outspeeds");
    }

    #[test]
    fn unburden_doubles_speed_after_item_consumed() {
        let mut battle = ordering_battle(
            "Garchomp", [slot(89, 10), empty(), empty(), empty()], AbilityId::Unburden,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        // Without UNBURDEN volatile: Dragapult faster
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 1, "Without Unburden active, Dragapult is faster");

        // Set UNBURDEN volatile (item was consumed)
        battle.sides[0].active_mut().volatiles.insert(Volatiles::UNBURDEN);
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "With Unburden active, Garchomp outspeeds");
    }

    // (e) Under Trick Room the slower Pokemon acts first
    #[test]
    fn trick_room_slower_goes_first() {
        // Blissey (spe 55) vs Dragapult (spe 142), same priority moves
        let mut battle = ordering_battle(
            "Blissey", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        // Normal: Dragapult first
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 1, "Without Trick Room, faster Dragapult goes first");

        // Under Trick Room: Blissey first
        battle.field.trick_room = 5;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0, "Under Trick Room, slower Blissey goes first");
    }

    // (f) Trick Room expires after 5 turns
    #[test]
    fn trick_room_expires_after_5_turns() {
        let mut battle = ordering_battle(
            "Blissey", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.field.trick_room = 5;

        // Run 5 end-of-turns
        for i in 0..5 {
            assert!(battle.field.trick_room > 0, "Trick Room should be active on turn {}", i);
            battle.end_of_turn();
        }
        assert_eq!(battle.field.trick_room, 0, "Trick Room should expire after 5 EOTs");
        // Protocol should contain fieldend
        assert!(battle.protocol.iter().any(|l| l.contains("|-fieldend|move: Trick Room")),
            "Should emit fieldend on expiry");
    }

    // Trick Room move toggles the field state
    #[test]
    fn trick_room_move_sets_field() {
        // Trick Room (id=433)
        let mut battle = ordering_battle(
            "Blissey", [slot(433, 5), empty(), empty(), empty()], AbilityId::None,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        assert_eq!(battle.field.trick_room, 0);
        battle.apply_status_move_for_test(0, 1, "Trick Room");
        assert_eq!(battle.field.trick_room, 5);
        assert!(battle.protocol.iter().any(|l| l.contains("|-fieldstart|move: Trick Room")));
    }

    #[test]
    fn trick_room_move_toggles_off() {
        let mut battle = ordering_battle(
            "Blissey", [slot(433, 5), empty(), empty(), empty()], AbilityId::None,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        battle.field.trick_room = 3; // Already active
        battle.protocol.clear();
        battle.apply_status_move_for_test(0, 1, "Trick Room");
        assert_eq!(battle.field.trick_room, 0, "Trick Room should toggle off");
        assert!(battle.protocol.iter().any(|l| l.contains("|-fieldend|move: Trick Room")));
    }

    // Triage: +3 priority to drain/heal moves
    #[test]
    fn triage_boosts_drain_punch_priority() {
        // Drain Punch (id=409). Blissey slower than Dragapult normally.
        let mut battle = ordering_battle(
            "Blissey", [slot(409, 10), empty(), empty(), empty()], AbilityId::Triage,
            "Dragapult", [slot(53, 15), empty(), empty(), empty()], AbilityId::None,
        );
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        // Drain Punch base priority 0, +3 from Triage = 3. Flamethrower has 0. P1 goes first.
        assert_eq!(order[0].0, 0, "Triage should give +3 priority to Drain Punch");
    }
}
