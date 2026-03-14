# HerdingCats

## What This Is

HerdingCats is a deterministic, input-driven, turn-based state transition engine for Rust. It lets game developers define an ordered set of **Behaviors** that are checked sequentially during dispatch, bringing clarity and order to complex rule interactions. The engine never advances on its own — it only moves when the caller submits a new `Input`.

## Core Value

An ordered set of statically known behaviors resolves every input deterministically, so complex rule interactions are never ambiguous.

## Requirements

### Validated

- ✓ Behavioral dispatch (ordered behaviors checked sequentially) — v0.4.0
- ✓ Undo/redo history — v0.4.0
- ✓ Deterministic behavior resolution — v0.4.0
- ✓ Turn-based (input-driven, never autonomous) — v0.4.0

### Active

- [ ] Implement `Behavior` trait with `name()`, `order_key()`, `evaluate()` per ARCHITECTURE.md
- [ ] Implement `BehaviorResult<D, O>` with `Continue(Vec<D>)` and `Stop(O)` variants
- [ ] Implement CoW working state: read from committed until first write, clone substate on first write
- [ ] Implement dispatch algorithm: deterministic `(order_key, name)` ordering, immediate diff application, immediate trace generation
- [ ] Implement `Outcome` enum: `Committed`, `Undone`, `Redone`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted`
- [ ] Implement `EngineError` for genuine engine/library failures (distinct from domain outcomes)
- [ ] Implement `Frame { input, diff, trace }` as canonical transition record
- [ ] Implement `undo()` and `redo()` returning `Result<Outcome, EngineError>`
- [ ] Implement irreversibility boundary: library user designates, erases undo/redo history on commit
- [ ] Behavior state lives in main state tree (not engine-internal)
- [ ] Static behavior set — no runtime registration
- [ ] Tic-tac-toe example demonstrating the full API
- [ ] Backgammon example demonstrating the full API
- [ ] Basic unit tests covering dispatch, undo/redo, outcomes, and core invariants

### Out of Scope

- `NeedsChoice` — intentionally deferred per ARCHITECTURE.md
- Runtime behavior registration — static only
- Dynamic substates / dynamic dispatch as design center
- Separate validation pass — validation happens through ordered behavior evaluation
- Autonomous time advancement / real-time scheduling
- DSL / card-text compilation — long-term direction, not v0.5.0

## Context

v0.4.0 had several issues identified during review:
- Behavior lifetimes lived as engine-internal mutable state (broke undo correctness, serializability)
- Ordering tiebreaker was memory address instead of deterministic `behavior_name`
- API returned `Option<Action<M>>` — diverged from the intended `Outcome` enum design
- `dispatch_preview()` had side effects on behavior state, creating confusing semantics
- Undo/redo stacks were public fields rather than encapsulated
- Full state clone on every preview dispatch (not true CoW)

v0.5.0 is a clean reimplementation on `maddie-edits` branch, building from scratch to match ARCHITECTURE.md exactly. Reference document: `ARCHITECTURE.md` in repository root.

Two examples exist as empty placeholders: `examples/tictactoe.rs` and `examples/backgammon.rs`.

## Constraints

- **Tech stack**: Rust (edition 2024), zero runtime dependencies
- **Build**: Cargo, rustfmt, clippy
- **Starting point**: Build from scratch on `maddie-edits` branch (not a refactor of main)
- **API alignment**: All implementations must match the semantics in `ARCHITECTURE.md`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build from scratch on maddie-edits | Prior implementation had fundamental design mismatches; cleaner to start fresh | — Pending |
| Static behavior set only | Preserves tight typing, determinism, easier DSL compilation path | — Pending |
| CoW working state (not eager clone) | Avoids performance penalty for large state AI look-ahead | — Pending |
| Behavior state in main state tree | Ensures undo/redo correctness and serializability | — Pending |
| `(order_key, behavior_name)` ordering | Deterministic tiebreaker without relying on memory address | — Pending |

---
*Last updated: 2026-03-13 after initialization*
