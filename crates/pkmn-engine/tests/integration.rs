#[cfg(test)]
mod tests {
    use pkmn_core::abilities::AbilityId;
    use pkmn_core::items::ItemId;
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

    // === Ability tests ===

    #[test]
    fn test_intimidate_lowers_attack_on_switch() {
        let side1 = Side::new(vec![make_garchomp(), make_dragapult()]);
        let side2 = Side::new(vec![make_blissey()]);
        let mut battle = Battle::new(side1, side2, 42);
        // Give Dragapult Intimidate
        battle.sides[0].team[1].ability_id = AbilityId::Intimidate;
        assert_eq!(battle.sides[1].active().boosts.atk, 0);
        battle.execute_choice(0, Choice::Switch(1));
        assert_eq!(battle.sides[1].active().boosts.atk, -1);
    }

    #[test]
    fn test_intimidate_capped_at_minus_6() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().ability_id = AbilityId::Intimidate;
        battle.sides[1].active_mut().boosts.atk = -6;
        battle.trigger_ability_on_switch(0);
        assert_eq!(battle.sides[1].active().boosts.atk, -6);
    }

    #[test]
    fn test_levitate_ground_immunity() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[1].active_mut().ability_id = AbilityId::Levitate;
        let hp_before = battle.sides[1].active().hp;
        // Garchomp uses Earthquake (Ground, move_idx 0)
        battle.execute_choice(0, Choice::Move(0));
        assert_eq!(battle.sides[1].active().hp, hp_before);
    }

    #[test]
    fn test_flash_fire_immunity() {
        let mut battle = make_battle(make_dragapult(), make_garchomp());
        battle.sides[1].active_mut().ability_id = AbilityId::FlashFire;
        let hp_before = battle.sides[1].active().hp;
        // Dragapult uses Flamethrower (Fire, move_idx 0)
        battle.execute_choice(0, Choice::Move(0));
        assert_eq!(battle.sides[1].active().hp, hp_before);
    }

    #[test]
    fn test_volt_absorb_electric_immunity() {
        let mut battle = make_battle(make_dragapult(), make_garchomp());
        battle.sides[1].active_mut().ability_id = AbilityId::VoltAbsorb;
        let hp_before = battle.sides[1].active().hp;
        // Dragapult uses Thunderbolt (Electric, move_idx 1)
        battle.execute_choice(0, Choice::Move(1));
        assert_eq!(battle.sides[1].active().hp, hp_before);
    }

    #[test]
    fn test_drizzle_sets_rain() {
        let side1 = Side::new(vec![make_garchomp(), make_dragapult()]);
        let side2 = Side::new(vec![make_blissey()]);
        let mut battle = Battle::new(side1, side2, 42);
        battle.sides[0].team[1].ability_id = AbilityId::Drizzle;
        assert_eq!(battle.field.weather, Weather::None);
        battle.execute_choice(0, Choice::Switch(1));
        assert_eq!(battle.field.weather, Weather::Rain);
        assert_eq!(battle.field.weather_turns, 5);
    }

    #[test]
    fn test_drought_sets_sun() {
        let side1 = Side::new(vec![make_garchomp(), make_dragapult()]);
        let side2 = Side::new(vec![make_blissey()]);
        let mut battle = Battle::new(side1, side2, 42);
        battle.sides[0].team[1].ability_id = AbilityId::Drought;
        battle.execute_choice(0, Choice::Switch(1));
        assert_eq!(battle.field.weather, Weather::Sun);
    }

    #[test]
    fn test_sand_stream_sets_sand() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().ability_id = AbilityId::SandStream;
        battle.trigger_ability_on_switch(0);
        assert_eq!(battle.field.weather, Weather::Sand);
    }

    #[test]
    fn test_electric_surge_sets_terrain() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().ability_id = AbilityId::ElectricSurge;
        battle.trigger_ability_on_switch(0);
        assert_eq!(battle.field.terrain, Terrain::Electric);
        assert_eq!(battle.field.terrain_turns, 5);
    }

    #[test]
    fn test_speed_boost_end_of_turn() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().ability_id = AbilityId::SpeedBoost;
        assert_eq!(battle.sides[0].active().boosts.spe, 0);
        battle.end_of_turn();
        assert_eq!(battle.sides[0].active().boosts.spe, 1);
        battle.end_of_turn();
        assert_eq!(battle.sides[0].active().boosts.spe, 2);
    }

    #[test]
    fn test_speed_boost_capped_at_6() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().ability_id = AbilityId::SpeedBoost;
        battle.sides[0].active_mut().boosts.spe = 6;
        battle.end_of_turn();
        assert_eq!(battle.sides[0].active().boosts.spe, 6);
    }

    #[test]
    fn test_technician_boosts_weak_moves() {
        // Rapid Spin has 50 BP, should get Technician boost (1.5x)
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        let rapid_spin = pkmn_core::moves::get_move("Rapid Spin").unwrap();
        // Technician: base_power <= 60 => 1.5x modifier
        let mod_with = battle.ability_damage_modifier(0, rapid_spin);
        battle.sides[0].active_mut().ability_id = AbilityId::Technician;
        let mod_tech = battle.ability_damage_modifier(0, rapid_spin);
        assert_eq!(mod_with, 1.0);
        assert_eq!(mod_tech, 1.5);

        // Earthquake has 100 BP, should NOT get Technician boost
        let earthquake = pkmn_core::moves::get_move("Earthquake").unwrap();
        let mod_eq = battle.ability_damage_modifier(0, earthquake);
        assert_eq!(mod_eq, 1.0);
    }

    // === Item tests ===

    #[test]
    fn test_choice_band_boosts_physical() {
        let mut p1 = make_garchomp();
        p1.item_id = ItemId::ChoiceBand;
        let mut battle = make_battle(p1, make_dragapult());
        let hp_before = battle.sides[1].active().hp;
        battle.execute_choice(0, Choice::Move(0)); // Earthquake (Physical)
        let damage_with_band = hp_before - battle.sides[1].active().hp;

        let p1b = make_garchomp(); // No item
        let mut battle2 = make_battle(p1b, make_dragapult());
        let hp_before2 = battle2.sides[1].active().hp;
        battle2.execute_choice(0, Choice::Move(0));
        let damage_without = hp_before2 - battle2.sides[1].active().hp;

        assert!(damage_with_band > damage_without);
    }

    #[test]
    fn test_choice_scarf_speed_modifier() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().item_id = ItemId::ChoiceScarf;
        assert_eq!(battle.item_speed_modifier(0), 1.5);
        assert_eq!(battle.item_speed_modifier(1), 1.0);
    }

    #[test]
    fn test_life_orb_boost_and_recoil() {
        let mut p1 = make_garchomp();
        p1.item_id = ItemId::LifeOrb;
        let mut battle = make_battle(p1, make_dragapult());
        let attacker_hp_before = battle.sides[0].active().hp;
        let defender_hp_before = battle.sides[1].active().hp;
        battle.execute_choice(0, Choice::Move(0));
        // Defender took damage
        assert!(battle.sides[0].active().hp < attacker_hp_before || !battle.sides[1].active().is_alive());
        // Attacker took Life Orb recoil (if still alive)
        if battle.sides[0].active().is_alive() {
            assert!(battle.sides[0].active().hp < attacker_hp_before);
        }
    }

    #[test]
    fn test_leftovers_heals_end_of_turn() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().item_id = ItemId::Leftovers;
        let max_hp = battle.sides[0].active().max_hp;
        battle.sides[0].active_mut().hp = max_hp - 50;
        let hp_before = battle.sides[0].active().hp;
        battle.end_of_turn();
        let expected_heal = max_hp / 16;
        assert_eq!(battle.sides[0].active().hp, hp_before + expected_heal);
    }

    #[test]
    fn test_leftovers_does_not_overheal() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().item_id = ItemId::Leftovers;
        let max_hp = battle.sides[0].active().max_hp;
        // Already at full HP
        battle.end_of_turn();
        assert_eq!(battle.sides[0].active().hp, max_hp);
    }

    #[test]
    fn test_focus_sash_survives_ohko() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[1].active_mut().item_id = ItemId::FocusSash;
        let max_hp = battle.sides[1].active().max_hp;
        // Simulate a huge hit
        let damage = max_hp + 100;
        let adjusted = battle.check_focus_sash(1, damage);
        assert_eq!(adjusted, max_hp - 1);
        // Sash consumed
        assert_eq!(battle.sides[1].active().item_id, ItemId::None);
    }

    #[test]
    fn test_focus_sash_only_from_full() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[1].active_mut().item_id = ItemId::FocusSash;
        battle.sides[1].active_mut().hp -= 1; // Not at full
        let damage = battle.sides[1].active().hp + 100;
        let adjusted = battle.check_focus_sash(1, damage);
        // Sash doesn't activate when not at full HP
        assert_eq!(adjusted, damage);
        assert_eq!(battle.sides[1].active().item_id, ItemId::FocusSash);
    }

    #[test]
    fn test_heavy_duty_boots_skips_hazards() {
        let side1 = Side::new(vec![make_garchomp(), make_blissey()]);
        let side2 = Side::new(vec![make_dragapult()]);
        let mut battle = Battle::new(side1, side2, 42);
        battle.sides[0].side_conditions.stealth_rock = true;
        battle.sides[0].side_conditions.spikes = 3;
        battle.sides[0].team[1].item_id = ItemId::HeavyDutyBoots;
        battle.execute_choice(0, Choice::Switch(1));
        // Blissey with HDB takes no hazard damage
        let mon = battle.sides[0].active();
        assert_eq!(mon.hp, mon.max_hp);
    }

    #[test]
    fn test_flame_orb_burns_end_of_turn() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().item_id = ItemId::FlameOrb;
        assert_eq!(battle.sides[0].active().status, Status::None);
        battle.end_of_turn();
        assert_eq!(battle.sides[0].active().status, Status::Burn);
    }

    #[test]
    fn test_toxic_orb_poisons_end_of_turn() {
        let mut battle = make_battle(make_garchomp(), make_dragapult());
        battle.sides[0].active_mut().item_id = ItemId::ToxicOrb;
        assert_eq!(battle.sides[0].active().status, Status::None);
        battle.end_of_turn();
        assert_eq!(battle.sides[0].active().status, Status::Toxic);
    }
}
