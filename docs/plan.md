# pkmn-engine-rs — Development Plan

## Vision
A 100% accurate, high-performance Pokemon battle simulator in Rust. Targets 100x speedup over @pkmn/sim (JS) while maintaining byte-for-byte protocol compatibility with Pokemon Showdown.

## Scope
- **Primary**: Gen 9 Random Battle (singles)
- **Secondary**: Gen 4-8 (incremental, architecture supports multi-gen)
- **Non-goals (initially)**: Doubles, custom formats, team validation

## Architecture

```
pkmn-engine-rs/
├── crates/
│   ├── pkmn-core/       # Types, type chart, base stats, move/ability/item data
│   ├── pkmn-engine/     # Battle simulation: turn execution, state machine
│   └── pkmn-wasm/       # WASM bindings for browser/Node.js
├── data/
│   └── gen9/            # Generated data tables (from PS source)
├── docs/                # This plan, architecture notes
├── tests/
│   ├── fixtures/        # PS replay logs as ground truth
│   └── integration/     # Full battle differential tests
└── tools/
    └── gen-data/        # Script to extract data from pokemon-showdown
```

### Crate Responsibilities

**pkmn-core**: Zero-dependency data layer
- Pokemon species data (base stats, types, abilities, weight)
- Move data (base power, type, category, accuracy, priority, flags, effect)
- Ability data (ID, effect hooks)
- Item data (ID, effect hooks)
- Type effectiveness chart (18x18 matrix)
- Nature stat modifiers
- All data is compile-time constant (no runtime loading)

**pkmn-engine**: Battle simulation
- `Battle` struct: full game state (~2KB, trivially cloneable)
- `Battle::new(teams, seed)` — create from teams
- `Battle::choices(player)` — legal actions
- `Battle::apply(p1_choice, p2_choice)` — advance one turn
- Turn execution: priority calc, damage, effects, fainting, end-of-turn
- State machine: handles forced switches, multi-turn moves, etc.
- RNG: seeded PRNG for reproducibility
- Generation trait for multi-gen support

**pkmn-wasm**: Thin WASM wrapper
- Exposes Battle API to JavaScript
- Serialization of state for external consumption
- Target: <500KB WASM bundle

## State Representation

```rust
struct Battle {
    sides: [Side; 2],       // 2 players
    field: Field,           // weather, terrain, trick room, gravity
    turn: u16,
    rng: PseudoRng,
    result: Option<BattleResult>,
}

struct Side {
    active: u8,             // index into team
    team: [Pokemon; 6],
    side_conditions: SideConditions,  // hazards, screens, tailwind
}

struct Pokemon {
    species: SpeciesId,     // u16 index
    level: u8,
    hp: u16,
    max_hp: u16,
    status: Status,         // u8 bitflags
    boosts: Boosts,         // [i8; 7]
    moves: [MoveSlot; 4],
    ability: AbilityId,     // u16
    item: ItemId,           // u16
    types: [Type; 2],
    stats: Stats,           // computed at level
    // Volatile state
    volatiles: Volatiles,   // bitflags for confusion, substitute, etc.
    // ~64 bytes total
}
```

Total battle state: ~1.5KB. Clone cost: ~1.5KB memcpy (trivial for MCTS).

## Performance Targets

| Metric | @pkmn/sim (JS) | Target | Method |
|--------|----------------|--------|--------|
| Battles/sec (1 core) | ~500 | 50,000+ | Compact state, no alloc |
| Clone cost | ~500KB alloc | ~1.5KB memcpy | Bitpacked state |
| Memory per battle | ~500KB | ~2KB | No heap allocation in hot path |
| WASM bundle | N/A | <500KB | Minimal dependencies |

## Testing Strategy

### 1. Unit Tests (per mechanic)
- Damage formula: compare against @smogon/calc for known inputs
- Type effectiveness: exhaustive 18x18 matrix
- Priority: verify correct ordering for all priority brackets
- Each ability/item: isolated test of its effect

### 2. Integration Tests (full battles)
- Download 50K+ replays from replay.pokemonshowdown.com
- For each replay: feed same teams + choices into our engine
- Compare output protocol line-by-line
- CI gate: 100% of fixtures must pass

### 3. Differential Fuzzing
- Generate random teams + random choices
- Run in both our engine and @pkmn/sim (via Node.js subprocess)
- Compare results
- Find edge cases automatically

### 4. Property Testing
- HP never exceeds max_hp
- Fainted Pokemon can't act
- Turn count always increases
- Legal moves are always a subset of known moves

## Milestones

### M1: Core Data + Damage Formula (Week 1)
- [ ] Type chart (18 types, effectiveness matrix)
- [ ] Species data (top 200 randbats Pokemon)
- [ ] Move data (top 300 randbats moves)
- [ ] Gen 9 damage formula (matching @smogon/calc)
- [ ] 50+ unit tests

### M2: Turn Execution (Week 2-3)
- [ ] Priority calculation (brackets -7 to +5)
- [ ] Move execution (damage, accuracy, secondary effects)
- [ ] Switching (in-turn and end-of-turn)
- [ ] Fainting + forced switch
- [ ] Weather/terrain application and end-of-turn
- [ ] Entry hazards (Stealth Rock, Spikes, Toxic Spikes)

### M3: Abilities + Items (Week 3-4)
- [ ] Top 50 abilities (Intimidate, Levitate, Mold Breaker, etc.)
- [ ] Top 30 items (Choice Band/Specs/Scarf, Life Orb, Leftovers, etc.)
- [ ] Ability triggers (on-switch, on-hit, on-damage, end-of-turn)

### M4: Full Battle Loop (Week 4-5)
- [ ] Complete game state machine (team preview → battle → end)
- [ ] All volatile statuses (confusion, substitute, encore, etc.)
- [ ] Multi-turn moves (Outrage, Petal Dance)
- [ ] Terastallization
- [ ] 100+ integration tests against replay fixtures

### M5: WASM + Benchmarks (Week 5-6)
- [ ] WASM compilation via wasm-pack
- [ ] JavaScript API bindings
- [ ] Benchmark suite (battles/sec, clone cost)
- [ ] Integration with randbats-bot self-play harness

## Data Generation

Pokemon Showdown's data lives in `pokemon-showdown/data/`. We extract:
- `pokedex.ts` → species base stats, types, abilities, weight
- `moves.ts` → move data (BP, type, category, accuracy, flags, effects)
- `abilities.ts` → ability effects
- `items.ts` → item effects

A `tools/gen-data/` script parses these into Rust source (compile-time constants).

## Multi-Gen Architecture

```rust
pub trait Generation {
    fn damage_formula(&self, ctx: &DamageContext) -> DamageRoll;
    fn speed_order(&self, battle: &Battle) -> Vec<ActionOrder>;
    fn end_of_turn(&self, battle: &mut Battle);
    fn apply_weather(&self, battle: &mut Battle);
    // ~20 methods covering gen-specific behavior
}

pub struct Gen9;
impl Generation for Gen9 { ... }
```

Adding a new gen = implement the trait + add gen-specific data tables.

## Key Design Principles

1. **No heap allocation in the hot path** — Battle state is stack-allocated, cloning is memcpy
2. **Data-oriented design** — Arrays of structs, cache-friendly layout
3. **Correctness first, optimize second** — Get it right, then make it fast
4. **Test against ground truth** — PS replays are the oracle
5. **Compile-time data** — All Pokemon/move/ability data is `const`
