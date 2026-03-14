# HerdingCats

A deterministic, turn-based game engine for Rust.

## Overview

HerdingCats brings order to complex rule interactions by evaluating an input through a
statically known, ordered set of Behaviors. The engine dispatches each Input atomically:
behaviors run in deterministic `(order_key, name)` order, each contributing Diffs to a
speculative working State. If dispatch succeeds and produces at least one Diff, the engine
commits a Frame and returns `Outcome::Committed`. If anything stops dispatch early, the
working State is discarded and no committed state changes occur.

The engine never advances on its own. It only transitions when the caller submits an Input
or explicitly calls undo/redo.

## Core Concepts

### Input

A user-defined message submitted to the engine. Every engine transition begins with an
Input. Examples: `Move`, `Attack`, `BeginTurn`, `RollDice`.

### State

The full game state, user-defined. Provided to `Engine::new()` — the engine does not
require a `Default` implementation. Behavior-local state lives inside the main State tree
(not hidden inside the engine), ensuring undo/redo correctness and serializability.

### Behavior

A single game rule expressed as a `BehaviorDef` struct with a `name`, `order_key`, and
`evaluate` fn pointer. During dispatch the engine calls each entry's `evaluate` fn pointer
with an immutable borrow of the current working State and the Input, then either:

- returns `Continue(diffs)` — zero or more Diffs to apply, dispatch continues
- returns `Stop(outcome)` — halts dispatch immediately with a non-committed Outcome

`BehaviorDef` entries are sorted once at construction by `(order_key, name)` and run in
that order for every dispatch call. The shared borrow prevents mutation — behaviors cannot
mutate State directly.

### Diff

An atomic, user-defined description of a state mutation. Behaviors emit Diffs; the engine
applies them. Each Diff implements the `Apply` trait, which mutates the working State and
returns at least one Trace entry. Applied immediately and sequentially — later Behaviors
see earlier Diffs.

### Trace

Per-Diff metadata generated at the exact moment each Diff is applied. Trace exists only
for successful committed transitions and records what happened in execution order. Useful
for UI presentation, replay explanation, and isolating UI logic from direct State
inspection.

### Frame

The canonical record of a successful, non-empty state transition:

```rust
Frame { input, diffs, traces }
```

Frames are immutable records. The same Frame struct is returned by `Committed`, `Undone`,
and `Redone` outcomes — the variant carries the meaning, not the frame.

### Outcome

The result of every `dispatch`, `undo`, or `redo` call:

```rust
pub enum Outcome<F, N> {
    Committed(F),      // dispatch succeeded, frame committed
    Undone(F),         // undo succeeded, frame returned
    Redone(F),         // redo succeeded, frame returned
    NoChange,          // dispatch completed with zero diffs
    InvalidInput(N),   // input does not make sense in current context
    Disallowed(N),     // input meaningful but an active rule forbids it
    Aborted(N),        // dispatch started but could not safely complete
}
```

### Engine

The central coordinator. Holds the committed State, the ordered list of `BehaviorDef`
entries, and the undo/redo history stacks. Exposes `dispatch`, `undo`, and `redo`.

## How Dispatch Works

1. Caller submits an Input with a `Reversibility` declaration (`Reversible` or
   `Irreversible`).
2. Engine creates a copy-on-write working State — no clone until the first Diff is
   applied.
3. Behaviors are evaluated in deterministic `(order_key, behavior_name)` order.
4. Each Behavior either emits Diffs (`Continue`) or halts dispatch (`Stop` with a
   `NonCommittedOutcome`).
5. Diffs are applied immediately to the working State; Trace entries are appended at
   application time.
6. If any Behavior returns `Stop`, the working State is discarded and the corresponding
   non-committed Outcome is returned — no state change is visible.
7. If dispatch completes with no Diffs, `Outcome::NoChange` is returned.
8. If one or more Diffs were applied, the working State is committed atomically, a Frame
   is created, and `Outcome::Committed(frame)` is returned.

## Undo / Redo

Undo and redo operate on stored Frames, not reverse-diff computations. No `Reversible`
trait is required on your types — the engine stores a prior-State snapshot alongside each
Frame. Irreversible dispatches erase undo/redo history across that boundary (once dice
are rolled, you cannot undo before the roll).

## Quick Start

```rust
use herdingcats::{Engine, EngineSpec, Apply, BehaviorDef, BehaviorResult, Reversibility};

// 1. Define your spec (bundles all associated types)
struct MySpec;

#[derive(Debug, Clone)]
struct AppendDiff(u8);

// 2. Implement Apply so each Diff can mutate State and emit Trace
impl Apply<MySpec> for AppendDiff {
    fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
        state.push(self.0);
        vec![format!("appended {}", self.0)]
    }
}

impl EngineSpec for MySpec {
    type State = Vec<u8>;
    type Input = u8;
    type Diff = AppendDiff;
    type Trace = String;
    type NonCommittedInfo = String;
    type OrderKey = u32;
}

// 3. Define a BehaviorDef with an evaluate fn pointer
fn append_eval(input: &u8, _state: &Vec<u8>) -> BehaviorResult<AppendDiff, String> {
    BehaviorResult::Continue(vec![AppendDiff(*input)])
}

// 4. Construct the Engine with initial State and BehaviorDef entries
let behaviors = vec![
    BehaviorDef::<MySpec> { name: "append", order_key: 0, evaluate: append_eval },
];
let engine = Engine::<MySpec>::new(vec![], behaviors);

// 5. Dispatch an Input
let outcome = engine.dispatch(42u8, Reversibility::Reversible).unwrap();
```

## Architecture Status

v0.5.0 implements the full architecture described in `ARCHITECTURE.md`:

- Static behavior set via `BehaviorDef` structs (fn pointers, no trait objects)
- Copy-on-write working state (zero-clone until first diff)
- Snapshot-based undo/redo (no `Reversible` trait burden)
- Deterministic `(order_key, name)` behavior ordering
- `Apply` trace contract enforced by `debug_assert!` in dispatch

## Zero Dependencies

HerdingCats has no runtime dependencies. `proptest` is a dev-only dependency used for
property-based testing.
