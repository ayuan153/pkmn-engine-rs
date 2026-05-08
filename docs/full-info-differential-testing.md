# Full-Information Differential Testing

## Goal
Zero-tolerance exact match verification against Pokemon Showdown. Every damage event must produce the EXACT observed damage — no ranges, no tolerance, no inference.

## Approach
Run battles locally using `@pkmn/sim` with:
- Known teams (full sets: species, ability, item, EVs, IVs, nature, moves)
- Known RNG seed (deterministic rolls)
- Omniscient stream (full information for both sides)

Extract per-damage-event data including the exact modifiers that fired, then verify our engine produces identical results.

## Architecture

```
tools/generate-fixtures/     # Node.js project
├── package.json             # @pkmn/sim, @pkmn/data, @pkmn/dex
├── tsconfig.json
├── src/
│   ├── generate.ts          # Main: run battles, capture logs
│   ├── teams.ts             # Curated teams covering key mechanics
│   └── parse-protocol.ts    # Parse omniscient log into structured events
└── output/                  # Generated fixtures land here

tests/fixtures/full-info/    # Committed fixtures (JSON)
├── battle-001.json
├── battle-002.json
└── ...

crates/pkmn-engine/tests/
└── strict_verification.rs   # Zero-tolerance test
```

## Fixture Format

```json
{
  "id": "battle-001",
  "seed": [1, 2, 3, 4],
  "format": "gen9customgame",
  "p1": {
    "name": "Bot 1",
    "team": [
      {
        "species": "Garchomp",
        "ability": "Rough Skin",
        "item": "Choice Scarf",
        "nature": "Jolly",
        "evs": { "hp": 0, "atk": 252, "def": 0, "spa": 0, "spd": 4, "spe": 252 },
        "ivs": { "hp": 31, "atk": 31, "def": 31, "spa": 31, "spd": 31, "spe": 31 },
        "moves": ["Earthquake", "Outrage", "Stone Edge", "Swords Dance"],
        "level": 100
      }
    ]
  },
  "p2": { ... },
  "turns": [
    {
      "turn": 1,
      "events": [
        {
          "type": "damage",
          "source": "p1a",
          "target": "p2a",
          "move": "Earthquake",
          "damage": 187,
          "crit": false,
          "effectiveness": 1.0,
          "attacker": {
            "species": "Garchomp",
            "level": 100,
            "stat_atk": 394,
            "ability": "Rough Skin",
            "item": "Choice Scarf",
            "boosts": { "atk": 0 },
            "status": null
          },
          "defender": {
            "species": "Tyranitar",
            "level": 100,
            "stat_def": 350,
            "ability": "Sand Stream",
            "item": "Leftovers",
            "boosts": { "def": 0 },
            "hp_before": 404,
            "hp_after": 217
          }
        }
      ]
    }
  ]
}
```

## Key Mechanics to Cover

Teams are curated to exercise:
1. Weather (Rain/Sun/Sand/Snow) + weather-boosted moves
2. Terrain (Electric/Grassy/Psychic/Misty) + terrain-boosted moves
3. Variable BP moves (Knock Off, Acrobatics, Facade, Low Kick, Weather Ball)
4. Damage-modifying abilities (Huge Power, Adaptability, Technician, Tinted Lens, Sheer Force)
5. Defensive abilities (Multiscale, Thick Fat, Ice Scales, Fur Coat)
6. Items (Choice Band/Specs, Life Orb, Expert Belt, type-boosting items)
7. Crits (use high-crit moves like Stone Edge, or Scope Lens)
8. Screens (Reflect, Light Screen, Aurora Veil)
9. Boosts (Swords Dance, Nasty Plot, Intimidate, Download)
10. Multi-hit moves (Bullet Seed, Population Bomb, Triple Axel)
11. Burn halving physical damage
12. STAB + Tera STAB
13. Type immunities + Tera type change

## Verification Logic (Rust)

For each damage event in a fixture:
1. Look up move base power (handle variable BP)
2. Use the EXACT attacker stat from fixture (no estimation)
3. Use the EXACT defender stat from fixture
4. Apply all modifiers: STAB, effectiveness, weather, terrain, ability, item, crit, burn, screens, boosts
5. The damage formula produces exactly 16 possible values (rolls 85-100)
6. The observed damage MUST equal one of those 16 values
7. If not: our engine has a bug. Fix it.

## Success Criteria
- 100% exact match on ALL generated fixtures
- Zero tolerance, zero skips, zero "close enough"
- Any failure = engine bug to fix

## Migration
- Old replay-based tests (`damage_verification.rs`) remain as a secondary smoke-test layer
- New strict tests (`strict_verification.rs`) are the source of truth
- Over time, expand strict fixtures to cover more mechanics
