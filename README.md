# pkmn-engine-rs

A high-performance Pokémon battle simulation engine in Rust, built for self-play reinforcement learning.

## Why?

Monte Carlo Tree Search and RL agents need to simulate **millions** of Pokemon battles to find optimal plays. JavaScript engines (`@pkmn/sim`) top out at ~500 games/sec. pkmn-engine-rs targets 100–1000x speedup with a compact, cloneable battle state — enough to search many turns deep in real time.

## Performance

**To be benchmarked.** Target: 100–1000x over `@pkmn/sim` (JS). The architecture is designed for:
- No heap allocation in the hot path
- Battle state <4KB, trivially cloneable via memcpy
- All data tables compiled as `const` (no runtime loading)

## Features

- ✅ Gen 9 damage formula with all modifiers
- ✅ 50+ abilities (Intimidate, Levitate, Technician, weather setters, …)
- ✅ 30+ items (Choice Band/Specs/Scarf, Life Orb, Focus Sash, …)
- ✅ Entry hazards, volatile statuses, multi-turn & recharge moves
- ✅ Terastallization (type change + STAB boost)
- ✅ Boost moves (Swords Dance, Dragon Dance, Shell Smash, …)
- ✅ Complete type chart (18 types), stat calc with natures
- ✅ WASM target (<500KB) for browser use

## Quick Start

```bash
git clone https://github.com/yourusername/pkmn-engine-rs
cd pkmn-engine-rs
cargo build --release
cargo test
```

## Architecture

```
crates/
├── pkmn-core/     # Types, stats, moves, damage formula
├── pkmn-engine/   # Battle simulation state machine
└── pkmn-wasm/     # WASM bindings for JS/TS
```

See [docs/plan.md](docs/plan.md) for the full development plan.

## Testing

Differential tested against [Pokémon Showdown](https://github.com/smogon/pokemon-showdown) replay fixtures. The type chart is 100% correct across 3126 damage events from 142 real replays.

The project is pivoting away from byte-for-byte protocol matching as the correctness metric. The new validation approach uses per-mechanic unit tests (hard gate), controlled damage comparisons against `@smogon/calc` (hard gate), and statistical differential testing against `@pkmn/sim` on outcome distributions (hard gate at ±2% win-rate).

## Status

**Early development — active refactor in progress.**

Core engine works: turn execution, damage, switching, hazards, weather, status, boosts, abilities, items, Terastallization. Currently refactoring the effect dispatch system from inline match blocks to a hook-based event system for extensibility (see [docs/plan.md](docs/plan.md)).

**Goal**: A rules-correct Gen 9 Random Battle singles engine for self-play RL, with policy transfer to the real Pokémon Showdown ladder.

## License

MIT
