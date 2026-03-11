# `herdingcats`

Deterministic rule orchestration for turn-based games.

Author:  Madeline Prelee (mprelee@gmail.com)


## Overview

`herdingcats` is a generic engine for implementing deterministic, behavior-driven turn-based systems.

It provides:

- Explicit priority-ordered behavior execution
- Atomic state mutation via `Action<M>`
- Undo / redo
- Replay-safe hashing (FNV‑1a)
- Behavior enable / disable
- Event cancellation
- Preview execution without committing state
- Static, enum-based event dispatch
- Compile-time enforced integer-backed priority

The engine is application-agnostic and does not prescribe game semantics.

---

## Core Model

The engine is parameterized over:

- `S` — game state
- `M` — mutation type (`Mutation<S>`)
- `I` — input / event enum
- `P` — priority type (`Copy + Ord`, e.g. `u8` or `#[repr(i32)]` enum)

State mutation occurs exclusively through `Mutation`.

All mutations:

- Are logged
- Are undoable
- Update the replay hash
- Produce commit frames

---

## Dispatch API

Three methods cover the main dispatch use-cases:

| Method | Description | Returns |
|---|---|---|
| `dispatch(event)` | Simple path. Behaviors inject mutations via `before` hooks. | `Option<Action<M>>` — `Some` if mutations were committed, `None` if cancelled or no mutations produced |
| `dispatch_with(event, tx)` | Pre-built action path. Pass an `Action<M>` with mutations already populated; `dispatch` delegates to this internally. | `Option<Action<M>>` |
| `dispatch_preview(event, tx)` | Dry-run. Same pipeline as `dispatch_with` but all state changes are rolled back. Useful for AI look-ahead and UI preview. | `Action<M>` (never `None`) |

---

## Determinism Guarantees

- No hidden state mutation
- No dynamic event dispatch
- No runtime type inspection
- No unordered behavior execution
- Replay hash restored on undo / redo
- Preview execution does not mutate history

All ordering is explicit via priority.

---

## Intended Use

Designed for discrete, turn-based systems such as:

- Roguelikes
- Tactical strategy games
- Card systems
- Deterministic multiplayer simulations
- Digital board games

Not intended for:

- Real-time systems
- Physics simulations
- Dynamic scripting engines
- Freeform narrative interpretation

---

## Status

Pre‑1.0. API may change.

---

## License

MIT OR Apache-2.0

# AI Usage Policy (subject to change)

This codebase is being (co)written using AI frameworks and committed to
enforcing idiomatic rust patterns where-possible.  It's more important
that the code adheres to good standards to the point where it is unclear
whether it was written by a knowledgeable Rust expert or a very good LLM.
