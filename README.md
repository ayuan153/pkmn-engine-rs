# pkmn-engine-rs

A high-performance Pokemon battle simulation engine in Rust. 100% accurate, 100x faster than reference JS implementations.

## Status: Early Development

Currently implements:
- Complete Gen 6+ type chart (18 types)
- Gen 9 damage formula with all modifiers
- Species data (50 Pokemon), move data (30 moves)
- Stat calculation with natures

## Goals

- **100% accuracy**: Differential tested against Pokemon Showdown replay fixtures
- **100x speed**: Target 50,000+ battles/sec (vs ~500 for @pkmn/sim)
- **Tiny state**: ~2KB per battle, trivially cloneable for MCTS/search
- **Multi-gen**: Architecture supports Gen 4-9 via trait-based dispatch
- **WASM ready**: Compiles to <500KB WASM for browser use

## Build

```bash
cargo build
cargo test
cargo bench  # (coming soon)
```

## Architecture

```
crates/
├── pkmn-core/     # Types, stats, moves, damage formula
├── pkmn-engine/   # Battle simulation state machine
└── pkmn-wasm/     # WASM bindings for JS/TS
```

See [docs/plan.md](docs/plan.md) for the full development plan.

## License

MIT
