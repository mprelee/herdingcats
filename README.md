# `herdingcats`

Deterministic rule orchestration for turn-based games.

## Overview

`herdingcats` is a generic engine for implementing deterministic, rule-driven turn-based systems.

It provides:

- Explicit priority-ordered rule execution  
- Transactional state mutation  
- Undo / redo  
- Replay-safe hashing (FNV‑1a)  
- Rule enable / disable  
- Rule lifetimes (per-turn / per-trigger)  
- Event cancellation  
- Preview execution without committing state  
- Static, enum-based event dispatch  
- Compile-time enforced integer-backed priority  

The engine is application-agnostic and does not prescribe game semantics.

---

## Core Model

The engine is parameterized over:

- `S` — game state  
- `O` — operation type (`Operation<S>`)  
- `E` — event enum  
- `P` — priority enum (`#[repr(i32)]`, sealed)  

State mutation occurs exclusively through `Operation`.

All irreversible transactions:

- Are logged  
- Are undoable  
- Update the replay hash  
- Produce commit frames  

---

## Determinism Guarantees

- No hidden state mutation  
- No dynamic event dispatch  
- No runtime type inspection  
- No unordered rule execution  
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
