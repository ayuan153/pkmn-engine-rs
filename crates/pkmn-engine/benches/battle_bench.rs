use criterion::{criterion_group, criterion_main, Criterion};
use pkmn_engine::{Battle, BattleResult};

fn bench_random_battle(c: &mut Criterion) {
    c.bench_function("play_random_game", |b| {
        b.iter(|| {
            let mut battle = Battle::default_test_battle(42);
            let mut turn = 0;
            while battle.result == BattleResult::Ongoing && turn < 200 {
                let c1 = battle.choices(0);
                let c2 = battle.choices(1);
                if c1.is_empty() || c2.is_empty() { break; }
                battle.apply(c1[0], c2[0]);
                turn += 1;
            }
        })
    });

    c.bench_function("clone_battle", |b| {
        let battle = Battle::default_test_battle(42);
        b.iter(|| battle.clone())
    });

    c.bench_function("get_choices", |b| {
        let battle = Battle::default_test_battle(42);
        b.iter(|| {
            battle.choices(0);
            battle.choices(1);
        })
    });
}

criterion_group!(benches, bench_random_battle);
criterion_main!(benches);
