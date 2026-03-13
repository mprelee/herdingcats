# HerdingCats Architecture

## Purpose

HerdingCats is a deterministic, input-driven, turn-based state transition engine for games or other software that evolve only when the caller submits a new `Input`.

The engine exists to bring order to complex rule interactions by letting the library user define an ordered set of **Behaviors** that are checked sequentially during dispatch.

This is not a real-time engine. It never advances on its own.

---

## High-Level Model

A caller submits an `Input` to the engine.

The engine then:

1. Creates a speculative **working state**
2. Checks all behaviors in deterministic order
3. Lets each behavior emit zero or more **Diffs**, or stop evaluation early
4. Applies diffs immediately to the working state
5. Appends **Trace** entries at the moment diffs are applied
6. If successful and non-empty, commits a **Frame**
7. Otherwise returns a non-committed outcome

The engine is atomic:

- either the dispatch commits fully
- or no state changes become visible

---

## Core Terms

### Input
A user-defined message sent to the engine API.

Examples:

- `Move`
- `Attack`
- `BeginTurn`
- `RollDice`
- `ChooseOption`

Every engine transition begins with an `Input`.

---

### State
A user-defined aggregate type representing the full engine state.

The engine assumes the user will typically structure it like:

```rust
State {
    domain: DomainState,
    behaviors: BehaviorState,
}
```

This is not mandatory, but it is the intended architecture.

Behavior-local state lives inside `State` as its own sub-object.

Examples of behavior-local state:

- booleans for enabled/disabled conditions
- counters
- cooldowns
- countdown timers
- remembered choices
- temporary modifiers such as “+1 draw for the next 3 turns”

---

### Behavior
A statically known, compile-time-defined participant in resolution.

A behavior is checked during dispatch and can:

- contribute zero or more diffs
- or stop evaluation early with a non-committed outcome

A behavior has:

- a stable `name`
- an `order_key`

Behaviors are not dynamically registered at runtime.

Default behaviors may live in `Default`.
Additional behaviors should be supplied through engine construction or type-level composition, not runtime registration.

---

### Diff
A user-defined description of a state mutation.

Diffs are both:

1. the ordered mutations emitted during dispatch
2. the mutations stored in a committed frame

A diff is expected to “look like” a small nested version of the overall state layout.

Example shape:

```rust
enum StateDiff {
    Domain(DomainDiff),
    Behaviors(BehaviorDiff),
}
```

Each state or substate should define its own diff enum.

Each diff enum should provide an associated `apply` operation that mutates the relevant working substate.

---

### Trace
A user-defined record of why state changed.

Trace is intended for:

- UI presentation
- replay explanation
- debugging
- isolating UI from direct state inspection

Example: the UI can look for “monster died” in the trace instead of re-deriving that fact from state.

Trace is only produced on successful committed transitions.

Failed dispatches do not produce a normal trace.

---

### Frame
A successful atomic transition record.

```rust
Frame {
    input: Input,
    diff: DiffCollection,
    trace: Trace,
}
```

A frame always represents one successful, non-empty state transition.

A frame is used in:

- forward commits
- undo
- redo
- history storage
- replay
- debugging

---

## Dispatch Model

The core API operation is `dispatch`.

`dispatch(input)` attempts to evaluate one input atomically against the current state.

### Dispatch algorithm

1. Receive `Input`
2. Create speculative **working state** using copy-on-write semantics
3. Order behaviors by `(order_key, behavior_name)`
4. For each behavior in order:
   1. evaluate the behavior against the current working state and input
   2. if it returns `Stop(outcome)`, halt immediately and discard working state
   3. if it returns `Continue(diffs)`, apply each diff immediately and in order
   4. as each diff is applied, append corresponding trace entries immediately
5. If the dispatch completes with no applied diffs, return `NoChange`
6. Otherwise construct `Frame { input, diff, trace }`
7. Commit atomically
8. Record history if appropriate
9. Return `Committed(frame)`

---

## Deterministic Ordering

Behaviors execute in:

```text
(order_key, behavior_name)
```

Properties:

- `order_key` is user-defined and ordered
- `behavior_name` is a stable deterministic tiebreaker
- alphabetical ordering of names is acceptable
- designers may assign unique order keys to every behavior for full explicit ordering

Equal order keys mean equal precedence, not nondeterminism.
The engine still runs behaviors in stable deterministic order.

Within a behavior, emitted diffs are applied in the order returned by that behavior.

So there are two levels of ordering:

1. between behaviors
2. within a behavior’s emitted diffs

---

## Why Ordered Behaviors Exist

The point of behavior ordering is to let the library user define a set of prioritized resolutions.

This makes it possible to model exceptions and rule-bending cleanly.

Example:

- a high-priority behavior lowers hit points below 80%
- a lower-priority behavior checks whether hit points are above 80%

The lower-priority behavior must see the updated working state and therefore may no longer apply.

This is intentional and central to the design.

---

## Working State and Copy-on-Write

Dispatch never mutates committed state directly.

Instead it evaluates against a speculative **working state**.

### Requirements

- working state begins from current committed state
- later behaviors see changes made by earlier diffs
- if dispatch fails, working state is discarded
- if dispatch succeeds, the final result is committed atomically

### Implementation guidance

Use copy-on-write (CoW) semantics.

Do not clone the entire state eagerly unless necessary.

The intended behavior is:

- read from committed substate until first write
- clone a substate only when it is first written
- subsequent reads see the working cloned substate

Substate granularity is chosen by the library user.

The engine should support statically defined substates.
Avoid dynamic substates and avoid dynamic dispatch where possible.

---

## Behavior Evaluation Contract

Each behavior is checked for every dispatch.

A behavior may be effectively “off” by inspecting its own state and returning no diffs.

There is no separate “not applicable” outcome.

A behavior is stateful if needed, but it does not mutate its own state eagerly during evaluation.
All behavior-state changes happen through diffs applied to the working state.

### Behavior return shape

Conceptually:

```rust
enum BehaviorResult<D, O> {
    Continue(Vec<D>),
    Stop(O),
}
```

Where:

- `D` is the diff type emitted by the behavior
- `O` is a non-committed outcome

Interpretation:

- `Continue(vec![])` means “no contribution; continue”
- `Continue(diffs)` means “apply these diffs immediately, in order”
- `Stop(outcome)` means “halt dispatch immediately, discard working state, return this outcome”

---

## No Separate Validation Pass

There is no global early validation pass before ordered behavior resolution.

Reason:

A separate pre-validation pass would circumvent the meaning of ordering and priority.

Instead, validation happens naturally during ordered evaluation by inspecting the current working state.

This keeps resolution path-dependent in the intended way.

---

## Diff Application

The engine owns state mutation.

Behaviors do not mutate the working state directly.

Instead, behaviors emit diffs, and the engine applies them centrally.

### Diff application rules

- diffs are applied immediately
- diffs are applied sequentially
- each applied diff mutates the working state
- each applied diff appends trace entries immediately
- later behaviors see the updated working state

### Trace coupling rule

Any diff that mutates state must append at least one trace entry describing that mutation.

This keeps trace synchronized with actual execution.

---

## Trace Rules

Trace is generated during dispatch, at the exact moment diffs are applied.

This guarantees that trace order matches real execution order.

Trace is not reconstructed later.

Trace exists only for successful committed transitions.

There is no successful trace-only commit.

If no diffs are applied, dispatch returns `NoChange`.

If dispatch fails partway through, no normal trace is returned.

If additional debugging data for failed dispatches is desired, it should live inside the relevant failure outcome, such as `Aborted`.

---

## Diff Accumulation

The engine should accumulate the applied diffs during dispatch in order.

At the end of successful dispatch:

- if the accumulated diff set is empty, return `NoChange`
- otherwise package the accumulated diffs into the committed `Frame`

The exact collection type for frame diffs is user-defined.
It may be:

- a flat `Vec<StateDiff>`
- a nested patch object
- another strongly typed aggregate

The only hard requirement is that it be sufficient to describe the successful state transition.

---

## Outcome Model

The engine should return:

```rust
Result<Outcome, EngineError>
```

Where `Outcome` describes successful or non-committed domain results, and `EngineError` is reserved for actual engine/library/program failures.

### Successful outcomes

```rust
Committed(Frame)
Undone(Frame)
Redone(Frame)
```

### Non-committed outcomes

```rust
NoChange
InvalidInput(...)
Disallowed(...)
Aborted(...)
```

`NeedsChoice` is intentionally omitted from the MVP.

### Meaning of non-committed outcomes

#### NoChange
Dispatch completed without producing any state mutation.

No frame is committed.

#### InvalidInput
The input does not make sense in the current context.

Example:

- sending `Move` while still in a menu phase

#### Disallowed
The input is meaningful, but some active behavior or rule forbids it.

May include:

- a specific violated rule
- or the first encountered disallowing rule

#### Aborted
Dispatch began, but applying a diff or continuing resolution could not safely proceed.

Examples:

- invalid intermediate domain state
- missing entity encountered during controlled application
- arithmetic condition that the user chooses to treat as safe abort

`Aborted` is for controlled failure, not engine bugs.

---

## EngineError

`EngineError` is outside the normal dispatch semantics.

It is for genuine implementation/runtime failures, such as:

- impossible internal engine state
- bug in engine orchestration
- panic-like conditions elevated to error
- malformed impossible configuration that should have been prevented earlier

The library user should be able to distinguish:

- “dispatch was rejected or safely aborted”
from
- “the engine malfunctioned”

---

## History Model

The engine maintains a history of committed frames.

A frame is the canonical successful transition record.

Undo/redo operate on frames already present in history.

### Frame contents

```rust
Frame {
    input: Input,
    diff: DiffCollection,
    trace: Trace,
}
```

A frame is unchanged regardless of whether it is being returned as:

- `Committed(frame)`
- `Undone(frame)`
- `Redone(frame)`

The outcome variant carries the meaning.
The frame itself remains the original transition record.

---

## Undo / Redo

Undo and redo are separate engine operations, not special forms of dispatch.

They should still use the same overall result shape:

```rust
Result<Outcome, EngineError>
```

and return:

- `Undone(frame)`
- `Redone(frame)`

or appropriate non-committed outcomes such as:

- `Disallowed(NothingToUndo)`
- `Disallowed(NothingToRedo)`
- `Disallowed(UndoBlockedByIrreversibleBoundary)`

### Irreversibility

Some committed transitions are irreversible.

This is tightly coupled to domain-level concepts such as publicly revealed information or resolved randomness.

Examples:

- once dice are rolled and revealed
- once hidden information becomes public
- once some irreversible domain event occurs

The library user, not the engine, is responsible for identifying irreversible transitions.

Recommended policy for MVP:

- when an irreversible transition is committed, erase undo/redo history across that boundary

This is the safest and cleanest default.

---

## Static Behavior Set

The intended design is that all behaviors are statically known at compile time.

This preserves:

- tight typing
- deterministic structure
- minimal dynamic dispatch
- easier compiler assistance
- easier future DSL compilation

Default behaviors may come from `Default`.
Additional behaviors should be supplied during engine construction or through compile-time composition.

Do not design the MVP around runtime behavior registration.

---

## State and Substate Layout

Substate granularity is deliberately chosen by the library user.

The library should not force one fixed partitioning strategy.

Examples of coarse-grained layout:

- `domain`
- `behaviors`

Examples of finer-grained layout:

- `board`
- `players`
- `monsters`
- `effects`
- `rng`
- `behavior_state`

The architecture should support either, provided the structure is static and strongly typed.

If a common pattern emerges later, the library can provide helpers.

---

## Behavior State

Behavior state is part of the main committed state.

It must not live as hidden mutable engine internals outside `State`.

This ensures:

- undo/redo correctness
- serializability
- UI inspectability
- deterministic history
- consistent copy-on-write semantics

Behaviors may be stateful, but only through state changes expressed as diffs.

---

## MVP Scope

The MVP should include:

- statically known behaviors
- input-driven dispatch
- deterministic behavior ordering
- CoW working state
- immediate diff application
- immediate trace generation
- non-empty committed frames
- `dispatch`
- `undo`
- `redo`
- `NoChange`
- `InvalidInput`
- `Disallowed`
- `Aborted`
- `EngineError`

The MVP should omit:

- runtime behavior registration
- dynamic substates
- dynamic dispatch as a design center
- `NeedsChoice`
- separate validation phases
- autonomous time advancement
- real-time scheduling

---

## Core Invariants

1. The engine never advances without a new input or explicit undo/redo call.
2. Dispatch is atomic.
3. Behaviors are evaluated in deterministic `(order_key, name)` order.
4. Each behavior sees only the current working state.
5. Later behaviors see earlier applied diffs.
6. Behaviors do not mutate state directly.
7. The engine applies diffs centrally.
8. Any diff that mutates state must append at least one trace entry.
9. Trace is generated in execution order, not reconstructed later.
10. Only successful, non-empty transitions produce frames.
11. Non-committed outcomes do not modify visible committed state.
12. Behavior state lives inside the main state tree.
13. Undo/redo operate on canonical stored frames.
14. Irreversible transitions are designated by the library user.
15. Engine errors are distinct from normal rejected or aborted dispatch outcomes.

---

## Suggested Conceptual API

This is illustrative, not final:

```rust
pub enum Outcome<F, N> {
    Committed(F),
    Undone(F),
    Redone(F),
    NoChange,
    InvalidInput(N),
    Disallowed(N),
    Aborted(N),
}

pub enum BehaviorResult<D, O> {
    Continue(Vec<D>),
    Stop(O),
}

pub trait Behavior<S, I, D, O, K> {
    fn name(&self) -> &'static str;
    fn order_key(&self) -> K;
    fn evaluate(&self, input: &I, state: &S) -> BehaviorResult<D, O>;
}
```

The actual API may differ, but the semantics should remain aligned with the architecture above.

---

## Intended Long-Term Direction

The long-term goal is to support writing behaviors in an English-like language that can also be printed on cards or other game objects, then compiled into behavior logic.

This is one reason the architecture emphasizes:

- deterministic ordered behavior resolution
- strongly typed state and diff structures
- centralized mutation in the engine
- trace generation aligned with execution
- compile-time structure over dynamic dispatch

The engine is not restricted to card games, but this design should make card-text-like rule systems far less ambiguous.

---

## One-Paragraph Summary

HerdingCats is a deterministic, input-driven, turn-based state engine in which a statically known set of ordered behaviors is checked sequentially during `dispatch`. Each behavior sees only the current speculative working state and either contributes an ordered sequence of diffs or halts evaluation with a non-committed outcome. The engine applies diffs immediately, updates trace at the same moment, and commits a `Frame { input, diff, trace }` only if the entire dispatch succeeds and produces a non-empty state change.
