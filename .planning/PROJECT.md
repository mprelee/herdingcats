# herdingcats — Refactor & Test

## What This Is

`herdingcats` is a published Rust library crate providing a generic, trait-driven rule orchestration engine for turn-based games. It exposes `Engine<S, O, E, P>`, which manages state mutation, undo/redo, rule lifecycle, and replay hashing. The codebase is now a properly structured multi-module crate (`hash`, `operation`, `transaction`, `rule`, `engine`) with full rustdoc, 19 unit tests, 5 proptest property tests, and two runnable examples (tictactoe and backgammon).

## Core Value

The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest make this machine-verifiable, not just manually checked.

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
- ✓ Split `src/lib.rs` into concept-focused modules (`hash`, `operation`, `transaction`, `rule`, `engine`) — v1.0
- ✓ `proptest = "1.10"` as dev-dependency — v1.0
- ✓ Inline `#[cfg(test)]` unit tests in every module file — v1.0
- ✓ Property-based tests: undo/redo roundtrips, preview isolation, rule lifetime lifecycle, cancelled tx invariant — v1.0
- ✓ Backgammon example (`examples/backgammon.rs`) — runnable demo + proptest harness — v1.0
- ✓ Backgammon exercises non-determinism (dice rolls via `tx.irreversible=false`), per-die undo — v1.0
- ✓ All existing behavior preserved — no public API changes — v1.0

### Active

(None — v1.0 complete. Use `/gsd:new-milestone` to define next milestone goals.)

### Out of Scope

- Doubling cube, Crawford rule, Jacoby rule — not needed for engine testing
- Full tournament/match scoring — v2 if ever
- Async or networked gameplay — not part of this library's purpose
- New Engine API features — v1.0 was refactor/test only

## Context

- **Published:** crates.io as `herdingcats` (MIT OR Apache-2.0)
- **Zero external runtime dependencies** — proptest in `[dev-dependencies]` only
- **Edition 2024, Rust 1.85+**
- **Shipped v1.0:** 3 phases, 6 plans, 2,647 lines Rust across 7 files (+2,367 net insertions in 1 day)
- **Test suite:** 19 unit tests + 15 doctests + 5 proptest property tests (+ 2 integration property tests in backgammon example) — all green
- **Key architectural pattern established:** `tx.irreversible` flag gates CommitFrame push — `false` = non-undoable dispatch (dice roll), `true` (default) = per-CommitFrame undo target (each move)
- **Backgammon chosen** because it has non-deterministic events (dice), reversible partial moves (per-die undo), and board state complex enough to stress the engine

## Constraints

- **Tech stack**: Rust only — no new runtime dependencies beyond proptest dev-dep
- **API stability**: Public API surface must remain identical (lib.rs re-exports everything currently public)
- **Existing example**: `examples/tictactoe.rs` must continue to compile and run unchanged

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Split by concept (hash/operation/transaction/rule/engine) | Matches mental model of the library's abstractions | ✓ Good — clean DAG dependency order, each file self-contained |
| proptest over quickcheck | Better shrinking via value trees, larger community | ✓ Good — shrinking worked well in practice |
| Backgammon as example + test harness | Non-determinism and partial-move undo stress engine in ways tictactoe cannot | ✓ Good — `tx.irreversible=false` architecture demonstrated clearly |
| Inline `#[cfg(test)]` | Standard Rust convention, keeps tests close to code | ✓ Good — consistent across all 5 modules + both examples |
| Separate `mod tests` (unit) + `mod props` (proptest) | Clear separation between deterministic and property-based tests | ✓ Good — established in Phase 2, reused in Phase 3 |
| BearOffOp as dedicated variant vs sentinel encoding in `to` field | Avoid out-of-bounds panic on `board[26]` | ✓ Good — explicit variant makes undo logic unambiguous |
| Op-level proptest generation (BACK-05) | Bypass rule system for board conservation test | ✓ Good — simpler strategies, conservation doesn't require game-legality |
| `tx.irreversible` naming | Pre-existing: `true` = "put on undo stack", `false` = "skip stack" | ⚠️ Revisit — name reads opposite to behavioral effect; candidate for rename in v2 |

---
*Last updated: 2026-03-09 after v1.0 milestone*
