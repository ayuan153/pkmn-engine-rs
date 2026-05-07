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
```

Types: feat, fix, refactor, test, docs, chore, perf
Scopes: core, engine, wasm, data, bench

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

After substantive changes, update `docs/plan.md` milestone checkboxes.
