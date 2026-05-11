# AGENTS.md

Guidelines for AI agents working on this repo.

## Build & Test

```bash
cargo build          # build all crates
cargo test           # run all tests
cargo test -p pkmn-core  # test specific crate
cargo clippy         # lint
cargo fmt --check    # format check
```

All tests must pass before committing. Run `cargo clippy` for lint warnings.

## Commit Convention

Conventional Commits:
```
<type>(<scope>): <summary>

<body>

Reconstruct: <one-shot prompt that would reproduce this commit's changes>
Verify: <1-2 sentence testing/verification steps>
```

Types: feat, fix, refactor, test, docs, chore, perf
Scopes: core, engine, wasm, data, bench

Every commit body MUST include:
- `Reconstruct:` — A single prompt/instruction that could reproduce the work
- `Verify:` — How to confirm the change works (e.g., "cargo test -p pkmn-engine passes 45 tests")

## Code Style

- No `unsafe` unless absolutely necessary (and documented why)
- No heap allocation in the hot path (battle simulation)
- All Pokemon/move data is `const` (compile-time)
- Use `#[derive(Debug, Clone, Copy)]` for small types
- Tests in `#[cfg(test)] mod tests` within each module
- Property: battle state is always valid (no invalid HP, no impossible boosts)

## Architecture Rules

- `pkmn-core` has ZERO dependencies (except serde for serialization)
- `pkmn-engine` depends only on `pkmn-core`
- `pkmn-wasm` is a thin wrapper, no logic
- All game data is in `pkmn-core` as const arrays
- The `Battle` struct must be `Clone` and < 4KB

## Testing Philosophy

- Unit tests: per-mechanic (damage, type chart, priority)
- Integration tests: full battles compared to Pokemon Showdown output
- Fixtures: real replay logs in `tests/fixtures/`
- Target: 100% accuracy against PS (no known divergences)

## Progress Tracking

After each major milestone or substantive progress:
1. Update `docs/plan.md` milestone checkboxes
2. Update `README.md` "Status" section to reflect what's built
3. Commit the doc updates alongside the code

## Full-Sim Differential Testing (Current Priority)

The goal is 100% protocol match against Pokemon Showdown on full battle simulations.

### Fixture Immutability Rule

**Fixtures are immutable once committed.** They represent "PS produced this output — we must match it."

Process:
1. **Never regenerate existing fixtures.** They are permanent regression tests.
2. **Engine changes can only INCREASE pass count.** If a code change reduces pass count, the change is wrong.
3. **Progress is monotonic.** Pass count only goes up, never down.
4. **To discover new gaps:** Generate NEW fixtures (new IDs, new seeds) in a separate batch. Run them. Commit both passing and failing ones as new targets.
5. **To verify a fix:** Run existing fixtures. More pass = fix is correct.

This ensures we never "move the goalposts" by changing tests to match broken code.

### Workflow
1. `cargo test -p pkmn-engine full_sim -- --nocapture` — see current pass/fail
2. Look at first divergence in a failing fixture — identify the missing mechanic
3. Fix the engine
4. Run tests again — pass count must increase (or stay same), never decrease
5. To find MORE gaps: `cd tools/generate-fixtures && npx tsx src/generate-random.ts 10` — generates new fixtures, commit them as new targets

### Current State
- `tests/fixtures/full-sim/` — curated (19) + random-1v1 (30) = 49 fixtures
- Target: 49/49 (100%)

## Accuracy Work (Legacy)

Workflow:
1. Run `cargo test -p pkmn-engine damage_matches -- --nocapture` to see current stats
2. Look at "Direction only" cases — these are the ones to fix
3. The fix is usually in `tests/damage_verification.rs` (the verification context needs to track more modifiers)
4. Sometimes the fix is in the engine itself (missing ability/item/move effect)
5. After fixing, run tests again to confirm improvement
6. To add more fixtures: `cd tools && python3 fetch-fixtures.py --count 50 --output ../tests/fixtures/replay_events/`

Common causes of non-exact matches:
- Missing ability modifier (check `ability_damage_modifier` in verification)
- Missing item modifier (check `item_damage_modifier` in verification)
- Variable base power move (Knock Off = 97.5 BP if target has item, not 65)
- Multi-hit move (total damage = per-hit × hits)
- Terrain boost (1.3x for matching terrain + type)
- Untracked stat changes (Intimidate, Download, etc.)
