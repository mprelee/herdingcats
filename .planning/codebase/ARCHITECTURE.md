# Architecture

**Analysis Date:** 2026-03-08

## Pattern Overview

**Overall:** Generic, trait-driven rule orchestration engine (library crate)

**Key Characteristics:**
- Single-file library (`src/lib.rs`) exposing a set of generic types and traits
- Fully parameterized over four type variables: game state `S`, operation `O`, event `E`, priority `P`
- No framework dependencies — zero external crates; pure Rust standard library
- Strict determinism invariant: all state mutation flows through `Operation`, never bypassed
- Consumer (game) code lives entirely outside the library (e.g., `examples/`)

## Layers

**Engine Core:**
- Purpose: Orchestrates rule execution, manages undo/redo stacks, hashing, and rule lifetimes
- Location: `src/lib.rs` — `Engine<S, O, E, P>` struct
- Contains: `Engine`, `CommitFrame`, `Transaction`, `RuleLifetime`, `fnv1a_hash`
- Depends on: `Operation` trait, `Rule` trait (via dynamic dispatch `Box<dyn Rule>`)
- Used by: Consumer game code (the library user)

**Operation Abstraction:**
- Purpose: Represents a single, reversible, hashable state mutation
- Location: `src/lib.rs` — `Operation<S>` trait
- Contains: `apply()`, `undo()`, `hash_bytes()` contract
- Depends on: Game state type `S`
- Used by: `Transaction`, `Engine` dispatch/undo/redo, `CommitFrame`

**Rule Abstraction:**
- Purpose: Encapsulates game logic that responds to events before and after state mutations
- Location: `src/lib.rs` — `Rule<S, O, E, P>` trait
- Contains: `id()`, `priority()`, optional `before()`, optional `after()` hooks
- Depends on: `Operation<S>`, game state `S`, event type `E`, priority type `P`
- Used by: `Engine` (stored as `Vec<Box<dyn Rule<S, O, E, P>>>`)

**Transaction:**
- Purpose: Batches a sequence of operations for atomic application or cancellation
- Location: `src/lib.rs` — `Transaction<O>` struct
- Contains: `ops: Vec<O>`, `irreversible: bool`, `deterministic: bool`, `cancelled: bool`
- Depends on: Operation type `O`
- Used by: `Engine::dispatch()`, `Engine::dispatch_preview()`, all Rule hooks

**Consumer Game Code (examples):**
- Purpose: Implements game-specific state, operations, events, rules, and entry point
- Location: `examples/tictactoe.rs`
- Contains: Game state structs, `Op` enum implementing `Operation`, `GameEvent` enum, `Rule` impls, `main()`
- Depends on: `herdingcats::*` (Engine, Operation, Rule, Transaction, RuleLifetime)
- Used by: Cargo example runner (`cargo run --example tictactoe`)

## Data Flow

**Commit Dispatch (`Engine::dispatch`):**

1. Caller constructs a `Transaction<O>` and an event value `E`
2. `Engine::dispatch()` iterates rules in priority order (low → high), calling `rule.before(state, event, tx)` for each enabled rule
3. Each `before` hook may push `Operation` values onto `tx.ops` or set `tx.cancelled = true`
4. If `tx.cancelled` is false, engine applies all ops via `op.apply(&mut state)` in order
5. Engine iterates rules in reverse priority order (high → low), calling `rule.after(state, event, tx)` for each enabled rule
6. `after` hooks may push additional operations (e.g., win detection appending `SetWinner`)
7. Turn-based (`RuleLifetime::Turns`) lifetimes are decremented; exhausted rules are disabled
8. If `tx.irreversible` and not cancelled: each op's `hash_bytes()` is FNV-1a hashed into `replay_hash`; a `CommitFrame` is pushed onto `undo_stack`; `redo_stack` is cleared

**Preview Dispatch (`Engine::dispatch_preview`):**

1. Full snapshots taken: state, lifetimes, enabled set, replay hash
2. Same before/apply/after pipeline runs as in commit dispatch
3. All snapshots restored — no commit frame pushed, no hash update, no lifetime decrement

**Undo/Redo:**

1. `undo()` pops from `undo_stack`, applies ops in reverse order via `op.undo()`, restores replay hash, lifetimes, and enabled set from `CommitFrame`
2. `redo()` pops from `redo_stack`, re-applies ops forward, restores hash/lifetimes/enabled from frame

**State Management:**
- State `S` is owned by `Engine::state` (public field, direct access)
- `Engine::read()` returns a clone; `Engine::write(snapshot)` replaces state and clears all history

## Key Abstractions

**`Operation<S>` trait:**
- Purpose: Atomic, reversible, hashable state mutation unit
- Examples: `Op::Place`, `Op::SetWinner`, `Op::SwitchTurn` in `examples/tictactoe.rs`
- Pattern: Enum with variants per mutation type; each variant implements `apply`, `undo`, `hash_bytes`

**`Rule<S, O, E, P>` trait:**
- Purpose: Stateless or stateful game logic responding to events
- Examples: `PlayRule`, `WinRule` in `examples/tictactoe.rs`
- Pattern: Zero-size or data-carrying structs; override only relevant hooks (`before` or `after`)

**`Transaction<O>`:**
- Purpose: Mutable bag of ops passed through the rule pipeline; supports cancellation
- Pattern: Created empty by caller (`Transaction::new()`), populated by `before` hooks, consumed by engine

**`RuleLifetime`:**
- Purpose: Controls how long a rule stays active: forever, N turns, or N triggers
- Pattern: Enum (`Permanent`, `Turns(u32)`, `Triggers(u32)`); stored and restored in commit frames

**`CommitFrame<S, O>`:**
- Purpose: Full snapshot of engine metadata at commit time; enables precise undo
- Pattern: Internal struct stored on `undo_stack`; holds tx, hashes before/after, lifetime and enabled snapshots

## Entry Points

**Library Public API:**
- Location: `src/lib.rs`
- Triggers: Consumed by downstream Rust crates via `use herdingcats::*`
- Responsibilities: Exposes `Engine`, `Transaction`, `RuleLifetime`, `Operation` trait, `Rule` trait

**Example Binary (`tictactoe`):**
- Location: `examples/tictactoe.rs` — `fn main()`
- Triggers: `cargo run --example tictactoe`
- Responsibilities: Constructs `Engine`, registers rules, drives the game loop, prints board state

## Error Handling

**Strategy:** No `Result`/`Error` types in the engine. Invalid moves are modeled as transaction cancellation.

**Patterns:**
- Rules set `tx.cancelled = true` in `before` hooks to reject invalid events (e.g., occupied cell, game already over)
- No panics in engine core; game code may panic (e.g., `unreachable!()` in exhaustive match arms)
- No error propagation — all logic is pure state transformation

## Cross-Cutting Concerns

**Logging:** None — no logging framework used
**Validation:** Handled inside `Rule::before()` hooks by inspecting state and cancelling the transaction
**Authentication:** Not applicable (library crate, no I/O)
**Determinism:** Enforced architecturally — all mutation through `Operation`, hash updated only on irreversible commit, rule order fixed by priority sort

---

*Architecture analysis: 2026-03-08*
