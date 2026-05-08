use pkmn_engine::{Battle, Choice};

#[test]
fn protocol_emits_switch_on_lead() {
    let mut battle = Battle::default_test_battle([42, 0, 0, 0]);
    let proto = battle.drain_protocol();
    assert!(proto.iter().any(|e| e.starts_with("|switch|p1a: Garchomp")));
    assert!(proto.iter().any(|e| e.starts_with("|switch|p2a: Dragonite")));
}

#[test]
fn protocol_emits_turn_and_move() {
    let mut battle = Battle::default_test_battle([42, 0, 0, 0]);
    battle.drain_protocol();
    battle.apply(Choice::Move(0), Choice::Move(0));
    let proto = battle.drain_protocol();
    assert!(proto.iter().any(|e| e == "|turn|1"));
    assert!(proto.iter().any(|e| e.starts_with("|move|")));
    assert!(proto.iter().any(|e| e == "|upkeep"));
}

#[test]
fn protocol_emits_damage() {
    let mut battle = Battle::default_test_battle([42, 0, 0, 0]);
    battle.drain_protocol();
    battle.apply(Choice::Move(0), Choice::Move(0));
    let proto = battle.drain_protocol();
    assert!(proto.iter().any(|e| e.starts_with("|-damage|")));
}

#[test]
fn protocol_emits_faint_and_win() {
    let mut battle = Battle::default_test_battle([100, 0, 0, 0]);
    battle.drain_protocol();
    // Play until game ends
    for _ in 0..200 {
        if battle.result != pkmn_engine::BattleResult::Ongoing {
            break;
        }
        let p1_choices = battle.choices(0);
        let p2_choices = battle.choices(1);
        if p1_choices.is_empty() || p2_choices.is_empty() {
            break;
        }
        match battle.phase {
            pkmn_engine::BattlePhase::ForcedSwitch(p) => {
                let choices = battle.choices(p);
                if let Some(&c) = choices.first() {
                    if let Choice::Switch(t) = c {
                        battle.apply_switch(p, t);
                    }
                }
            }
            _ => {
                battle.apply(p1_choices[0], p2_choices[0]);
            }
        }
    }
    // Collect all protocol from a fresh game to check faint exists
    let mut battle2 = Battle::default_test_battle([100, 0, 0, 0]);
    let mut all_proto = battle2.drain_protocol();
    for _ in 0..200 {
        if battle2.result != pkmn_engine::BattleResult::Ongoing {
            break;
        }
        let p1_choices = battle2.choices(0);
        let p2_choices = battle2.choices(1);
        if p1_choices.is_empty() || p2_choices.is_empty() {
            break;
        }
        match battle2.phase {
            pkmn_engine::BattlePhase::ForcedSwitch(p) => {
                let choices = battle2.choices(p);
                if let Some(&c) = choices.first() {
                    if let Choice::Switch(t) = c {
                        battle2.apply_switch(p, t);
                    }
                }
            }
            _ => {
                battle2.apply(p1_choices[0], p2_choices[0]);
            }
        }
        all_proto.extend(battle2.drain_protocol());
    }
    assert!(all_proto.iter().any(|e| e.starts_with("|faint|")));
}
