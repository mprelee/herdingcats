# herdingcats — Rename & Reversibility

## Current Milestone: v1.1 Rename & Reversibility

**Goal:** Refine the public API naming and reversibility model to better reflect the Mealy/Moore state machine design intent for turn-based games.

**Target features:**
- Rename `Operation→Mutation`, `Rule→Behavior`, `Transaction→Action`; remove `RuleLifetime`
- Per-mutation reversibility (`is_reversible()`) with undo barrier on irreversible commits
- Self-managing behavior lifecycle (`is_active`, `on_dispatch`, `on_undo`) replacing external `RuleLifetime` enum

## What This Is

`herdingcats` is a published Rust library crate providing a generic, trait-driven state machine engine for turn-based games. It exposes `Engine<S, M, I, P>`, which manages state mutation via `Behavior`s, undo/redo with reversibility barriers, behavior lifecycle, and replay hashing. The codebase is a properly structured multi-module crate with full rustdoc, unit tests, proptest property tests, and two runnable examples (tictactoe and backgammon).

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

- [ ] `Mutation<S>` trait — `apply`, `undo`, `hash_bytes`, `is_reversible()` — v1.1
- [ ] `Behavior<S,M,I,P>` trait — `before`/`after` hooks, `is_active`, `on_dispatch`, `on_undo` — v1.1
- [ ] `Action<M>` — batches mutations, derived reversibility, `deterministic`, `cancelled` — v1.1
- [ ] Undo barrier: irreversible `Action` clears undo stack — v1.1
- [ ] Self-managing behavior lifecycle replaces `RuleLifetime` enum — v1.1

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
- **API stability**: v1.1 is a intentional breaking rename — semver minor bump; all public names change
- **Existing examples**: `examples/tictactoe.rs` and `examples/backgammon.rs` must compile and run under new names

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
| Rename Operation→Mutation, Rule→Behavior, Transaction→Action | `Transaction`/`Operation` semantically overlap; `Rule` conflicts with PEG parser terminology (future `pest` integration planned) | — Pending |
| Remove `RuleLifetime` enum; behaviors self-manage lifecycle | Allows arbitrary state (charges, toggles, counters) without engine coupling; `on_dispatch`/`on_undo`/`is_active` default methods keep simple cases zero-cost | — Pending |
| Undo barrier: irreversible action clears undo stack | Matches Mealy machine semantics — irreversible inputs (drawing a card) commit prior history; models "publicly visible information" boundary cleanly | — Pending |

---
*Last updated: 2026-03-10 after v1.1 milestone started*
