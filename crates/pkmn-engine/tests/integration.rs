#[cfg(test)]
mod tests {
    use pkmn_core::nature::Nature;
    use pkmn_core::species::get_species;
    use pkmn_engine::*;

    fn eq_slot() -> MoveSlot {
        MoveSlot { move_id: 89, pp: 10, max_pp: 10 } // Earthquake
    }

    fn flamethrower_slot() -> MoveSlot {
        MoveSlot { move_id: 53, pp: 15, max_pp: 15 }
    }

    fn ice_beam_slot() -> MoveSlot {
        MoveSlot { move_id: 58, pp: 10, max_pp: 10 }
    }

    fn tbolt_slot() -> MoveSlot {
        MoveSlot { move_id: 85, pp: 15, max_pp: 15 }
    }

    fn toxic_slot() -> MoveSlot {
        MoveSlot { move_id: 92, pp: 10, max_pp: 10 }
    }

    fn empty_slot() -> MoveSlot {
        MoveSlot { move_id: 0, pp: 0, max_pp: 0 }
    }

    fn make_pokemon(name: &str, nature: Nature, moves: [MoveSlot; 4]) -> Pokemon {
        let species = get_species(name).unwrap();
        Pokemon::new(species, 100, nature, moves, [0, 252, 0, 0, 0, 252], [31; 6])
    }

    fn make_garchomp() -> Pokemon {
        make_pokemon("Garchomp", Nature::Adamant, [eq_slot(), flamethrower_slot(), empty_slot(), empty_slot()])
    }

    fn make_dragapult() -> Pokemon {
        make_pokemon("Dragapult", Nature::Timid, [flamethrower_slot(), tbolt_slot(), ice_beam_slot(), empty_slot()])
    }

    fn make_blissey() -> Pokemon {
        make_pokemon("Blissey", Nature::Bold, [toxic_slot(), empty_slot(), empty_slot(), empty_slot()])
    }

    fn make_battle(p1: Pokemon, p2: Pokemon) -> Battle {
        let side1 = Side::new(vec![p1]);
        let side2 = Side::new(vec![p2]);
        Battle::new(side1, side2, 12345)
    }

    // === Pokemon tests ===

    #[test]
    fn test_pokemon_creation() {
        let mon = make_garchomp();
        assert_eq!(mon.level, 100);
        assert!(mon.hp > 0);
        assert!(mon.is_alive());
        assert_eq!(mon.status, Status::None);
    }

    #[test]
    fn test_effective_speed_paralysis() {
        let mut mon = make_garchomp();
        let normal = mon.effective_speed();
        mon.status = Status::Paralyze;
        assert_eq!(mon.effective_speed(), normal / 2);
    }

    #[test]
    fn test_boost_multiplier() {
        assert_eq!(Boosts::multiplier(0), 1.0);
        assert_eq!(Boosts::multiplier(2), 2.0);
        assert_eq!(Boosts::multiplier(-2), 0.5);
    }

    #[test]
    fn test_fainted_not_alive() {
        let mut mon = make_garchomp();
        mon.is_fainted = true;
        assert!(!mon.is_alive());
    }

    #[test]
    fn test_zero_hp_not_alive() {
        let mut mon = make_garchomp();
        mon.hp = 0;
        assert!(!mon.is_alive());
    }

    // === Side tests ===

    #[test]
    fn test_side_alive_count() {
        let side = Side::new(vec![make_garchomp(), make_dragapult()]);
        assert_eq!(side.alive_count(), 2);
    }

    #[test]
    fn test_side_has_alive_switch() {
        let side = Side::new(vec![make_garchomp(), make_dragapult()]);
        assert!(side.has_alive_switch());
    }

    #[test]
    fn test_side_no_switch_single() {
        let side = Side::new(vec![make_garchomp()]);
        assert!(!side.has_alive_switch());
    }

    // === Choice tests ===

    #[test]
    fn test_choices_moves_available() {
        let battle = make_battle(make_garchomp(), make_dragapult());
        let choices = battle.choices(0);
        // Garchomp has 2 moves with PP
        assert!(choices.contains(&Choice::Move(0)));
        assert!(choices.contains(&Choice::Move(1)));
        assert!(!choices.contains(&Choice::Move(2)));
    }

    #[test]
    fn test_choices_with_switch() {
        let side1 = Side::new(vec![make_garchomp(), make_dragapult()]);
        let side2 = Side::new(vec![make_blissey()]);
        let battle = Battle::new(side1, side2, 42);
        let choices = battle.choices(0);
        assert!(choices.contains(&Choice::Switch(1)));
    }

    // === Priority tests ===

    #[test]
    fn test_switch_before_move() {
        let battle = make_battle(make_garchomp(), make_dragapult());
        let order = battle.determine_order(Choice::Move(0), Choice::Switch(0));
        assert_eq!(order[0].0, 1); // p2 switch goes first
    }

    #[test]
    fn test_faster_pokemon_moves_first() {
        // Dragapult (142 base spe) vs Garchomp (102 base spe)
        let battle = make_battle(make_dragapult(), make_garchomp());
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 0); // Dragapult is faster
    }

    #[test]
    fn test_trick_room_reverses_speed() {
        let mut battle = make_battle(make_dragapult(), make_garchomp());
        battle.field.trick_room = 5;
        let order = battle.determine_order(Choice::Move(0), Choice::Move(0));
        assert_eq!(order[0].0, 1); // Garchomp (slower) goes first in trick room
    }

    // === Damage tests ===

    #[test]
    fn test_move_deals_damage() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        let initial_hp = battle.sides[1].active().hp;
        battle.apply(Choice::Move(0), Choice::Move(0));
        // Both should have taken damage (unless one fainted first)
        assert!(battle.sides[1].active().hp < initial_hp || !battle.sides[1].active().is_alive());
    }

    #[test]
    fn test_immune_no_damage() {
        // Earthquake (Ground) vs Dragapult (Dragon/Ghost) - Ground doesn't hit Ghost? No, Ghost is neutral to Ground.
        // Let's use Thunderbolt (Electric) vs Garchomp (Dragon/Ground) - Ground is immune to Electric
        let mut battle = make_battle(make_dragapult(), make_garchomp());
        let initial_hp = battle.sides[1].active().hp;
        // Dragapult uses Thunderbolt (index 1) vs Garchomp
        battle.execute_choice(0, Choice::Move(1));
        assert_eq!(battle.sides[1].active().hp, initial_hp); // Immune
    }

    #[test]
    fn test_pp_deducted() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        assert_eq!(battle.sides[0].active().moves[0].pp, 10);
        battle.execute_choice(0, Choice::Move(0));
        assert_eq!(battle.sides[0].active().moves[0].pp, 9);
    }

    // === Status tests ===

    #[test]
    fn test_burn_end_of_turn_damage() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().status = Status::Burn;
        let hp_before = battle.sides[0].active().hp;
        battle.end_of_turn();
        let expected_dmg = (battle.sides[0].active().max_hp / 16).max(1);
        assert_eq!(battle.sides[0].active().hp, hp_before - expected_dmg);
    }

    #[test]
    fn test_poison_end_of_turn_damage() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().status = Status::Poison;
        let hp_before = battle.sides[0].active().hp;
        battle.end_of_turn();
        let expected_dmg = (battle.sides[0].active().max_hp / 8).max(1);
        assert_eq!(battle.sides[0].active().hp, hp_before - expected_dmg);
    }

    #[test]
    fn test_toxic_escalating_damage() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().status = Status::Toxic;
        let max_hp = battle.sides[0].active().max_hp;

        battle.end_of_turn();
        let dmg1 = max_hp - battle.sides[0].active().hp;

        let hp_after_1 = battle.sides[0].active().hp;
        battle.end_of_turn();
        let dmg2 = hp_after_1 - battle.sides[0].active().hp;

        assert!(dmg2 > dmg1); // Toxic damage escalates
    }

    // === Weather tests ===

    #[test]
    fn test_sandstorm_damage() {
        let mut battle = make_battle(make_dragapult(), make_blissey());
        battle.field.weather = Weather::Sand;
        battle.field.weather_turns = 5;
        let hp_before = battle.sides[0].active().hp;
        battle.end_of_turn();
        // Dragapult (Dragon/Ghost) takes sand damage
        assert!(battle.sides[0].active().hp < hp_before);
    }

    #[test]
    fn test_sandstorm_immune_ground() {
        let mut battle = make_battle(make_garchomp(), make_blissey());
        battle.field.weather = Weather::Sand;
        battle.field.weather_turns = 5;
        let hp_before = battle.sides[0].active().hp;
        battle.end_of_turn();
        // Garchomp (Dragon/Ground) is immune to sand
        assert_eq!(battle.sides[0].active().hp, hp_before);
    }

    #[test]
    fn test_weather_turns_decrement() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.field.weather = Weather::Rain;
        battle.field.weather_turns = 1;
        battle.end_of_turn();
        assert_eq!(battle.field.weather, Weather::None);
    }

    // === Hazards tests ===

    #[test]
    fn test_stealth_rock_damage() {
        let side1 = Side::new(vec![make_garchomp(), make_dragapult()]);
        let side2 = Side::new(vec![make_blissey()]);
        let mut battle = Battle::new(side1, side2, 42);
        battle.sides[0].side_conditions.stealth_rock = true;
        // Switch to Dragapult
        battle.execute_choice(0, Choice::Switch(1));
        // Dragapult is Dragon/Ghost, Rock is 1x to both = 1.0 effectiveness
        // Damage = max_hp * 1.0 / 8
        let mon = battle.sides[0].active();
        assert!(mon.hp < mon.max_hp);
    }

    #[test]
    fn test_spikes_damage() {
        let side1 = Side::new(vec![make_garchomp(), make_blissey()]);
        let side2 = Side::new(vec![make_dragapult()]);
        let mut battle = Battle::new(side1, side2, 42);
        battle.sides[0].side_conditions.spikes = 1;
        battle.execute_choice(0, Choice::Switch(1));
        // Blissey is Normal, grounded, takes 1/8 from 1 layer
        let mon = battle.sides[0].active();
        assert!(mon.hp < mon.max_hp);
    }

    #[test]
    fn test_spikes_flying_immune() {
        // Dragapult is Ghost/Dragon, not Flying - let's use a Flying type
        let corviknight = make_pokemon("Corviknight", Nature::Impish, [eq_slot(), empty_slot(), empty_slot(), empty_slot()]);
        let side1 = Side::new(vec![make_garchomp(), corviknight]);
        let side2 = Side::new(vec![make_blissey()]);
        let mut battle = Battle::new(side1, side2, 42);
        battle.sides[0].side_conditions.spikes = 3;
        battle.execute_choice(0, Choice::Switch(1));
        // Corviknight is Flying/Steel, immune to spikes
        let mon = battle.sides[0].active();
        assert_eq!(mon.hp, mon.max_hp);
    }

    // === Win condition tests ===

    #[test]
    fn test_win_when_all_fainted() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[1].active_mut().hp = 0;
        battle.sides[1].active_mut().is_fainted = true;
        battle.check_win();
        assert_eq!(battle.result, BattleResult::Win(0));
    }

    #[test]
    fn test_tie_both_fainted() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().hp = 0;
        battle.sides[0].active_mut().is_fainted = true;
        battle.sides[1].active_mut().hp = 0;
        battle.sides[1].active_mut().is_fainted = true;
        battle.check_win();
        assert_eq!(battle.result, BattleResult::Tie);
    }

    // === Full battle test ===

    #[test]
    fn test_full_battle_to_completion() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        let mut turns = 0;
        while battle.result == BattleResult::Ongoing && turns < 100 {
            let result = battle.apply(Choice::Move(0), Choice::Move(0));
            turns += 1;
            if result != BattleResult::Ongoing {
                break;
            }
        }
        assert_ne!(battle.result, BattleResult::Ongoing);
        assert!(turns < 100);
    }

    #[test]
    fn test_battle_clone_independent() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        let clone = battle.clone();
        battle.apply(Choice::Move(0), Choice::Move(0));
        // Clone should be unaffected
        assert_eq!(clone.turn, 0);
        assert_eq!(clone.sides[0].active().hp, clone.sides[0].active().max_hp);
    }

    // === Field tests ===

    #[test]
    fn test_terrain_turns_decrement() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.field.terrain = Terrain::Electric;
        battle.field.terrain_turns = 1;
        battle.end_of_turn();
        assert_eq!(battle.field.terrain, Terrain::None);
    }

    #[test]
    fn test_trick_room_decrements() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.field.trick_room = 3;
        battle.end_of_turn();
        assert_eq!(battle.field.trick_room, 2);
    }

    // === Side condition tests ===

    #[test]
    fn test_reflect_decrements() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].side_conditions.reflect = 1;
        battle.end_of_turn();
        assert_eq!(battle.sides[0].side_conditions.reflect, 0);
    }
}
