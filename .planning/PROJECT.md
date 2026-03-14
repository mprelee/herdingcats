# HerdingCats

## What This Is

HerdingCats is a deterministic, input-driven, turn-based state transition engine for Rust. It lets game developers define an ordered set of **Behaviors** (via `BehaviorDef` structs with fn pointers) that are evaluated sequentially during dispatch, bringing clarity and order to complex rule interactions. The engine never advances on its own — it only moves when the caller submits a new `Input`.

## Core Value

An ordered set of statically known behaviors resolves every input deterministically, so complex rule interactions are never ambiguous.

## Requirements

### Validated

- ✓ Behavioral dispatch (ordered behaviors checked sequentially) — v0.4.0
- ✓ Undo/redo history — v0.4.0
- ✓ Deterministic behavior resolution — v0.4.0
- ✓ Turn-based (input-driven, never autonomous) — v0.4.0
- ✓ `EngineSpec` trait bundles all type params (S, I, D, T, N, K) — v0.5
- ✓ `BehaviorDef<E>` with fn pointers (no trait objects) — v0.5
- ✓ `BehaviorResult::Continue(Vec<D>)` / `Stop(NonCommittedOutcome<N>)` — v0.5
- ✓ CoW working state (lazy clone on first diff) — v0.5
- ✓ Deterministic `(order_key, name)` dispatch ordering — v0.5
- ✓ `Outcome` enum with 7 variants (Committed, Undone, Redone, NoChange, InvalidInput, Disallowed, Aborted) — v0.5
- ✓ `EngineError` distinct from domain `Outcome` — v0.5
- ✓ `Frame { input, diffs, traces }` as canonical transition record — v0.5
- ✓ Snapshot-based undo/redo (no Reversible trait burden) — v0.5
- ✓ Irreversibility boundary clears undo/redo history — v0.5
- ✓ Behavior state in main state tree (not engine-internal) — v0.5
- ✓ Static behavior set — no runtime registration — v0.5
- ✓ Tic-tac-toe example demonstrating full API — v0.5
- ✓ Backgammon example demonstrating irreversibility — v0.5
- ✓ 72 unit tests + 2 proptest suites — v0.5

### Active

(None — next milestone requirements TBD via `/gsd:new-milestone`)

### Out of Scope

- `NeedsChoice` — requires suspending/resuming dispatch state; needs concrete use case
- Runtime behavior registration — static only, core architectural invariant
- Dynamic substates / dynamic dispatch as design center
- Separate validation pass — validation happens through ordered behavior evaluation
- Autonomous time advancement / real-time scheduling
- DSL / card-text compilation — long-term direction, requires mature behavior model first

## Context

v0.5 shipped as a clean reimplementation on `maddie-edits` branch, building from scratch to match ARCHITECTURE.md exactly.

**Current codebase:**
- 3,380 LOC Rust, zero external dependencies
- 72 unit tests + 7 doctests + 2 proptest suites, all passing
- `cargo clippy` clean, `cargo doc` zero warnings
- Two working examples: `examples/tictactoe.rs`, `examples/backgammon.rs`

**v0.4.0 issues resolved in v0.5:**
- Behavior lifetimes no longer engine-internal (fixed undo correctness)
- Ordering uses `(order_key, name)` not memory address (deterministic)
- API returns `Outcome` enum (not `Option<Action<M>>`)
- No `dispatch_preview()` side effects (CoW working state)
- Undo/redo stacks private with `undo_depth()`/`redo_depth()` queries
- True copy-on-write (no eager clone)

## Constraints

- **Tech stack**: Rust (edition 2024), zero runtime dependencies
- **Build**: Cargo, rustfmt, clippy
- **API alignment**: All implementations must match the semantics in `ARCHITECTURE.md`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build from scratch on maddie-edits | Prior implementation had fundamental design mismatches; cleaner to start fresh | ✓ Good — clean slate avoided incremental hacks |
| Static behavior set only | Preserves tight typing, determinism, easier DSL compilation path | ✓ Good — `BehaviorDef` fn pointers work well |
| CoW working state (not eager clone) | Avoids performance penalty for large state AI look-ahead | ✓ Good — confirmed via pointer test |
| Behavior state in main state tree | Ensures undo/redo correctness and serializability | ✓ Good — undo restores full state |
| `(order_key, behavior_name)` ordering | Deterministic tiebreaker without relying on memory address | ✓ Good — verified by invariant tests |
| `BehaviorDef` struct over `Behavior` trait | Eliminates Box<dyn>, simpler construction, fn pointer call syntax | ✓ Good — added in Phase 6 |
| `NonCommittedOutcome` wrapper | Behaviors choose InvalidInput/Disallowed/Aborted semantically | ✓ Good — added in Phase 5 |
| Snapshot undo (no Reversible trait) | Simplifies user API at cost of memory; acceptable for MVP | ⚠️ Revisit — memory for long AI sessions |
| `Apply` trace contract as trusted invariant | `debug_assert!` in dispatch catches violations in dev/test builds | ✓ Good — not enforced by type system |

---
*Last updated: 2026-03-14 after v0.5 milestone*
