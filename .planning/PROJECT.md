# herdingcats — Refactor & Test

## What This Is

`herdingcats` is a published Rust library crate that provides a generic, trait-driven rule orchestration engine for turn-based games. It exposes `Engine<S, O, E, P>`, which manages state mutation, undo/redo, rule lifecycle, and replay hashing. The current codebase is a single `src/lib.rs` that needs to be split into focused modules with substantial test coverage.

## Core Value

The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest should make this machine-verifiable, not just manually checked.

## Requirements

### Validated

- ✓ `Operation<S>` trait — apply, undo, hash_bytes contract — existing
- ✓ `Transaction<O>` — batches ops, cancellable, irreversible flag — existing
- ✓ `RuleLifetime` — Permanent, Turns(u32), Triggers(u32) — existing
- ✓ `Rule<S, O, E, P>` trait — before/after hooks, priority ordering — existing
- ✓ `Engine<S, O, E, P>` — dispatch, dispatch_preview, undo, redo, read, write — existing
- ✓ FNV-1a 64-bit replay hash — existing
- ✓ `CommitFrame` for snapshot/restore on undo — existing
- ✓ Tic-tac-toe example (`examples/tictactoe.rs`) — existing

### Active

- [ ] Split `src/lib.rs` into concept-focused modules: `hash.rs`, `operation.rs`, `transaction.rs`, `rule.rs`, `engine.rs` — re-exported from `lib.rs`
- [ ] Add `proptest` as a dev-dependency
- [ ] Inline `#[cfg(test)]` unit tests in every module file
- [ ] Property-based tests where applicable (hash determinism, undo/redo roundtrips, transaction isolation)
- [ ] Backgammon example (`examples/backgammon.rs`) — runnable, minimal-but-correct implementation
- [ ] Backgammon used as test harness: exercises non-determinism (dice rolls), partial moves, undo of single-die usage
- [ ] All existing behavior preserved — no public API changes, `cargo test` passes

### Out of Scope

- Doubling cube, Crawford rule, Jacoby rule — not needed for engine testing
- Full tournament/match scoring — v2 if ever
- Async or networked gameplay — not part of this library's purpose
- New Engine API features — this refactor is test/structure only

## Context

- Published on crates.io as `herdingcats` (MIT OR Apache-2.0)
- Zero external runtime dependencies — proptest goes in `[dev-dependencies]` only
- Edition 2024, Rust 1.85+
- Backgammon chosen because it has: non-deterministic events (dice), reversible partial moves (use one die, want to undo), and board state complex enough to stress the engine

## Constraints

- **Tech stack**: Rust only — no new runtime dependencies beyond proptest dev-dep
- **API stability**: Public API surface must remain identical (lib.rs re-exports everything currently public)
- **Existing example**: `examples/tictactoe.rs` must continue to compile and run unchanged

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Split by concept (hash/operation/transaction/rule/engine) | Matches mental model of the library's abstractions | — Pending |
| proptest over quickcheck | Better shrinking via value trees, larger community | — Pending |
| Backgammon as example + test harness | Non-determinism and partial-move undo stress engine in ways tictactoe cannot | — Pending |
| Inline #[cfg(test)] | Standard Rust convention, keeps tests close to code | — Pending |

---
*Last updated: 2026-03-08 after initialization*
