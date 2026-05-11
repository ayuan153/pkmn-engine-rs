# Full-Sim Differential Testing

## Goal

Run identical battles in Pokemon Showdown and our engine. Compare the complete protocol output line-by-line. Any divergence = engine bug.

This tests EVERYTHING: turn order, damage, status, abilities, items, switching, fainting, field conditions, win conditions.

## Fixture Immutability Rule (CRITICAL)

**Fixtures are immutable once committed.** They are the source of truth.

| Rule | Rationale |
|------|-----------|
| Never regenerate existing fixtures | They represent "PS said this" — that doesn't change |
| Engine changes can only INCREASE pass count | If pass count drops, the change is wrong |
| Progress is monotonic | Pass count only goes up |
| Generate NEW fixtures to find NEW gaps | Use new IDs/seeds, commit as new targets |
| Both passing and failing fixtures are committed | Failing ones are targets to fix |

**Anti-pattern:** Regenerating fixtures after a code change to "make tests pass" is cheating. It moves the goalposts instead of fixing the engine.

**Correct workflow:**
1. Run tests → see failures
2. Fix engine code
3. Run tests → pass count increases
4. Commit engine fix (fixtures unchanged)

**To expand coverage:**
1. Generate new fixtures: `npx tsx src/generate-random.ts 10` (new seeds)
2. Run them through our engine
3. Commit ALL of them (passing + failing) with new IDs
4. Fix engine until new fixtures pass too

## Architecture

```
tools/generate-fixtures/
└── src/
    ├── generate-full-sim.ts   # Runs battles in PS, captures full protocol
    ├── teams.ts               # Battle scenarios (teams + choices)
    └── scenarios-full-sim.ts  # Full-sim scenarios with predetermined choices

tests/fixtures/full-sim/
├── battle-001.json         # Full protocol fixture
└── ...

crates/pkmn-engine/
├── src/protocol.rs         # Protocol emitter (PS-compatible output)
└── tests/full_sim_test.rs  # Comparison test
```

## Fixture Format

```json
{
  "id": "weather-hazards-switching",
  "description": "Sand + SR + switching + Leftovers + Toxic",
  "seed": [1, 2, 3, 4],
  "p1": {
    "team": [
      { "species": "Tyranitar", "ability": "Sand Stream", "item": "Leftovers", ... },
      { "species": "Excadrill", "ability": "Sand Rush", "item": "Choice Band", ... }
    ]
  },
  "p2": {
    "team": [
      { "species": "Corviknight", "ability": "Pressure", "item": "Leftovers", ... },
      { "species": "Toxapex", "ability": "Regenerator", "item": "Rocky Helmet", ... }
    ]
  },
  "choices": [
    ["move 1", "move 1"],
    ["move 2", "switch 2"],
    ["switch 2", "move 1"],
    ["move 1", "move 2"]
  ],
  "protocol": [
    "|turn|1",
    "|move|p1a: Tyranitar|Stealth Rock|p2a: Corviknight",
    "|-sidestart|p2: Player 2|move: Stealth Rock",
    "|move|p2a: Corviknight|Brave Bird|p1a: Tyranitar",
    "|-damage|p1a: Tyranitar|312/404",
    "|-damage|p2a: Corviknight|330/369|[from] recoil",
    "|-weather|Sandstorm|[upkeep]",
    "|-damage|p2a: Corviknight|307/369|[from] Sandstorm",
    "|-heal|p1a: Tyranitar|337/404|[from] item: Leftovers",
    "|-heal|p2a: Corviknight|330/369|[from] item: Leftovers",
    "|turn|2",
    "..."
  ]
}
```

## Protocol Events We Compare

### Must Match Exactly
- `|turn|N` — turn boundaries
- `|move|POKEMON|MOVE|TARGET` — move execution + order
- `|switch|POKEMON|DETAILS|HP` — switches
- `|drag|POKEMON|DETAILS|HP` — forced switches
- `|-damage|POKEMON|HP STATUS` — all HP loss
- `|-heal|POKEMON|HP STATUS` — all HP gain
- `|-status|POKEMON|STATUS` — status infliction
- `|-curestatus|POKEMON|STATUS` — status cure
- `|-boost|POKEMON|STAT|AMOUNT` — stat raises
- `|-unboost|POKEMON|STAT|AMOUNT` — stat drops
- `|-weather|WEATHER` — weather changes
- `|-fieldstart|FIELD` — terrain/field start
- `|-fieldend|FIELD` — terrain/field end
- `|-sidestart|SIDE|CONDITION` — hazards/screens set
- `|-sideend|SIDE|CONDITION` — hazards/screens removed
- `|-start|POKEMON|EFFECT` — volatile start (confusion, sub, etc.)
- `|-end|POKEMON|EFFECT` — volatile end
- `|-item|POKEMON|ITEM` — item revealed
- `|-enditem|POKEMON|ITEM` — item consumed
- `|-ability|POKEMON|ABILITY` — ability revealed/triggered
- `|-activate|POKEMON|EFFECT` — ability/item activation
- `|-crit|POKEMON` — critical hit
- `|-supereffective|POKEMON` — super effective
- `|-resisted|POKEMON` — not very effective
- `|-immune|POKEMON` — immune
- `|-miss|SOURCE|TARGET` — move missed
- `|-hitcount|POKEMON|NUM` — multi-hit count
- `|cant|POKEMON|REASON` — can't move
- `|faint|POKEMON` — fainted
- `|win|PLAYER` — game over
- `|upkeep` — end-of-turn marker

### Filtered Out (not compared)
- `|t:|TIMESTAMP` — real-time clock
- `|request|...` — client UI data
- `|split|` — formatting wrapper
- `|-hint|` — cosmetic explanation
- `|inactive|` — timer
- `|player|` — player metadata
- `|teamsize|` — team preview
- `|gen|` — generation info
- `|tier|` — format info
- `|rule|` — clauses
- `|start` — battle start marker
- `|clearpoke` / `|poke|` — team preview

## Comparison Algorithm

```rust
fn compare_protocols(expected: &[String], actual: &[String]) -> Result<(), Divergence> {
    let expected_filtered = filter_semantic(expected);
    let actual_filtered = filter_semantic(actual);
    
    for (i, (exp, act)) in expected_filtered.iter().zip(actual_filtered.iter()).enumerate() {
        if exp != act {
            return Err(Divergence {
                line: i,
                expected: exp.clone(),
                actual: act.clone(),
                context: &expected_filtered[i.saturating_sub(3)..i+3],
            });
        }
    }
    
    if expected_filtered.len() != actual_filtered.len() {
        return Err(Divergence::LengthMismatch { ... });
    }
    
    Ok(())
}
```

## What Our Engine Needs

1. **Protocol emitter** — `protocol.rs` module that emits PS-format strings
2. **Same PRNG** — implement PS's 4-seed LCG (already done in our engine)
3. **Deterministic execution** — given same inputs, produce same outputs
4. **Complete event coverage** — emit events for ALL state changes

## PRNG Implementation

PS uses a custom PRNG from `sim/prng.ts`:
```
next(from, to) {
  // 4-word state, multiply-with-carry
  this.seed[0] = (0x5D588B65 * this.seed[0] + 0x269EC3) >>> 0;
  // ... returns value in [from, to)
}
```

Our engine must use the EXACT same PRNG to produce identical random outcomes.

## Success Criteria

- Run 50+ full battles through both PS and our engine
- 100% protocol match on all semantic events
- Zero divergences = full fidelity
- Any new mechanic we add: generate a fixture that exercises it, verify match

## Incremental Approach

1. Start with simple 1v1 battles (no switching)
2. Add switching, hazards, weather
3. Add abilities that trigger on events
4. Add items that trigger on events
5. Add multi-turn moves, volatiles
6. Scale to full 6v6 battles

Each step: generate fixtures, run comparison, fix divergences, repeat.
