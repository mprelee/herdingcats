# herdingcats — Project State

## What This Is

`herdingcats` is a published Rust library crate providing a generic, trait-driven state machine engine for turn-based games. It exposes `Engine<S, M, I, P>`, which manages state mutation via `Behavior`s, undo/redo with reversibility barriers, per-mutation reversibility, behavior lifecycle hooks, and replay hashing. The codebase is a properly structured multi-module crate with full rustdoc (Mealy/Moore framing), unit tests, proptest property tests, and two runnable examples (tictactoe and backgammon).

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
- ✓ Split `src/lib.rs` into concept-focused modules — v1.0
- ✓ `proptest = "1.10"` as dev-dependency — v1.0
- ✓ Inline `#[cfg(test)]` unit tests in every module file — v1.0
- ✓ Property-based tests: undo/redo roundtrips, preview isolation, rule lifetime lifecycle, cancelled tx invariant — v1.0
- ✓ Backgammon example (`examples/backgammon.rs`) — runnable demo + proptest harness — v1.0
- ✓ `Mutation<S>` trait — `apply`, `undo`, `hash_bytes`, `is_reversible()` — v1.1
- ✓ `Behavior<S,M,I,P>` trait — `before`/`after` hooks, `is_active`, `on_dispatch`, `on_undo` — v1.1
- ✓ `Action<M>` — batches mutations, derived reversibility, `deterministic`, `cancelled` — v1.1
- ✓ Undo barrier: irreversible `Action` clears undo stack — v1.1
- ✓ Self-managing behavior lifecycle replaces `RuleLifetime` enum — v1.1
- ✓ Full rustdoc with Mealy/Moore framing, runnable doctests — v1.1
- ✓ Property tests `prop_05`/`prop_06` verifying reversibility model — v1.1
- ✓ Edge-case unit tests for reversibility and lifecycle — v1.1

### Active

(None — planning next milestone)

### Out of Scope

- Doubling cube, Crawford rule, Jacoby rule — not needed for engine testing
- Full tournament/match scoring — v2 if ever
- Async or networked gameplay — not part of this library's purpose
- Output/event emission — deferred to v1.2 or v2.0 (Mealy output layer)
- `pest` grammar integration — future milestone; `Input<I>` naming lays groundwork

## Context

- **Published:** crates.io as `herdingcats` (MIT OR Apache-2.0)
- **Zero external runtime dependencies** — proptest in `[dev-dependencies]` only
- **Edition 2024, Rust 1.85+, version 0.3.0**
- **Shipped v1.0:** 3 phases, 6 plans; module split + proptest foundation (1 day)
- **Shipped v1.1:** 4 phases, 10 plans; 3,476 total Rust LOC, 43 files changed (2 days)
- **Test suite:** 35 unit tests + 21 doctests + property tests (prop_01–06) — all green; backgammon has 13 additional tests
- **Key architectural patterns:**
  - `tx.mutations.iter().all(|m| m.is_reversible())` gates undo stack push vs clear
  - Separate `iter_mut()` lifecycle passes after commit/undo/redo avoid borrow conflicts
  - `mod tests` (unit) + `mod props` (proptest) separation in `engine.rs`
  - `Rc<Cell<u32>>` pattern for observing boxed behavior state from test scope

## Constraints

- **Tech stack**: Rust only — no new runtime dependencies beyond proptest dev-dep
- **API stability**: v1.1 is complete breaking rename (v0.3.0); next version may add output emission
- **Examples**: `examples/tictactoe.rs` and `examples/backgammon.rs` must compile and run under any API changes

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Split by concept (hash/operation/transaction/rule/engine) | Matches mental model of the library's abstractions | ✓ Good — clean DAG dependency order, each file self-contained |
| proptest over quickcheck | Better shrinking via value trees, larger community | ✓ Good — shrinking worked well in practice |
| Backgammon as example + test harness | Non-determinism and partial-move undo stress engine in ways tictactoe cannot | ✓ Good — irreversible dice architecture demonstrated clearly |
| Inline `#[cfg(test)]` | Standard Rust convention, keeps tests close to code | ✓ Good — consistent across all modules + both examples |
| Separate `mod tests` (unit) + `mod props` (proptest) | Clear separation between deterministic and property-based tests | ✓ Good — established in Phase 2, reused through Phase 7 |
| BearOffOp as dedicated variant vs sentinel encoding | Avoid out-of-bounds panic on `board[26]` | ✓ Good — explicit variant makes undo logic unambiguous |
| Op-level proptest generation (BACK-05) | Bypass rule system for board conservation test | ✓ Good — simpler strategies, conservation doesn't require game-legality |
| Rename Operation→Mutation, Rule→Behavior, Transaction→Action | `Transaction`/`Operation` semantically overlap; `Rule` conflicts with PEG parser terminology | ✓ Good — names now accurately reflect Mealy/Moore model |
| Remove `RuleLifetime` enum; behaviors self-manage lifecycle | Allows arbitrary state (charges, toggles, counters) without engine coupling | ✓ Good — `is_active`/`on_dispatch`/`on_undo` are clean zero-cost defaults |
| Undo barrier: irreversible action clears undo stack | Matches Mealy machine semantics — irreversible inputs commit prior history | ✓ Good — `tx.mutations.iter().all(|m| m.is_reversible())` is clean |
| Lifecycle passes unconditional (all behaviors, regardless of is_active) | Sleeping behaviors still track history | ✓ Good — avoids surprising gaps in on_dispatch/on_undo call sequence |
| Empty action guard (!tx.mutations.is_empty()) | Prevents spurious on_dispatch calls | ✓ Good — reduces noise for dry-run dispatches |
| `can_undo()`/`can_redo()` public methods | External crates can't access private stack fields | ✓ Good — clean API boundary; internal tests still use direct field access |
| `tx.irreversible` naming (pre-v1.1) | Pre-existing: `true` = put on undo stack, `false` = skip stack | ⚠️ Resolved — replaced by `is_reversible()` opt-out model in v1.1 |

---
*Last updated: 2026-03-11 after v1.1 milestone shipped*
