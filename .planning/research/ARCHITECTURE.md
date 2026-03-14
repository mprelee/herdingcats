# Architecture Research

**Domain:** Rust deterministic turn-based game engine library
**Researched:** 2026-03-13
**Confidence:** HIGH — derived directly from the authoritative ARCHITECTURE.md spec; Rust type system patterns verified against standard library and community consensus.

---

## Standard Architecture

### System Overview

```
┌────────────────────────────────────────────────────────────┐
│                     Public API Layer                        │
│   dispatch(input) → Result<Outcome, EngineError>           │
│   undo()          → Result<Outcome, EngineError>           │
│   redo()          → Result<Outcome, EngineError>           │
├────────────────────────────────────────────────────────────┤
│                    Engine Core Layer                        │
│  ┌──────────────┐  ┌───────────────┐  ┌────────────────┐  │
│  │ Behavior     │  │ WorkingState  │  │ History        │  │
│  │ Ordering &   │  │ (CoW wrapper) │  │ (undo/redo     │  │
│  │ Evaluation   │  │               │  │  stacks)       │  │
│  └──────┬───────┘  └──────┬────────┘  └───────┬────────┘  │
│         │                 │                    │           │
├─────────┴─────────────────┴────────────────────┴──────────┤
│                   User-Supplied Types                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ State    │  │ Diff     │  │ Trace    │  │ Outcome  │  │
│  │ (domain) │  │ (enum)   │  │ (enum)   │  │ payload  │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└────────────────────────────────────────────────────────────┘
```

The engine library is a thin orchestrator: it owns only dispatch logic, behavior ordering, CoW working state coordination, and history stacks. All domain types (State, Diff, Trace, Outcome payloads) are supplied by the library user through generic type parameters.

### Component Responsibilities

| Component | Responsibility | Notes |
|-----------|----------------|-------|
| `Engine<S, I, D, T, O, K, B>` | Owns committed state + history stacks, exposes `dispatch`/`undo`/`redo` | Top-level struct the user constructs |
| `WorkingState<S>` | Wraps committed state reference + optional owned working clone; implements CoW access | Internal to engine; never exposed directly |
| `Behavior` trait | Defines `name()`, `order_key()`, `evaluate()` contract | User implements; engine calls |
| `BehaviorResult<D, O>` | `Continue(Vec<D>)` or `Stop(O)` — behavior's decision per input | Engine-defined; user returns |
| `Frame<I, D, T>` | Canonical committed transition: `{ input, diff, trace }` | Stored in history; returned in `Committed`/`Undone`/`Redone` |
| `Outcome<F, N>` | `Committed(F)` / `Undone(F)` / `Redone(F)` / `NoChange` / `InvalidInput(N)` / `Disallowed(N)` / `Aborted(N)` | Engine-defined enum; F = Frame, N = user payload |
| `EngineError` | Engine implementation failures distinct from domain outcomes | Engine-defined enum |
| History | Two `Vec<Frame>` stacks (undo stack + redo stack), encapsulated inside Engine | Never public fields |

---

## Recommended Module Structure

```
src/
├── lib.rs                  # Public re-exports only; no implementation
├── behavior.rs             # Behavior trait + BehaviorResult enum
├── outcome.rs              # Outcome enum + EngineError enum
├── frame.rs                # Frame struct
├── working_state.rs        # WorkingState<S> CoW wrapper
├── history.rs              # History struct (undo/redo stacks)
└── engine.rs               # Engine<...> struct — dispatch, undo, redo
```

### Structure Rationale

- **`lib.rs` re-exports only:** Keeps the public surface clean and decoupled from internal layout. Users import `herdingcats::{Behavior, BehaviorResult, Engine, Frame, Outcome, EngineError}`.
- **`behavior.rs` first:** It has no internal dependencies. It is the base of the dependency graph.
- **`outcome.rs` second:** `Outcome` and `EngineError` depend only on `Frame`, or can be defined before `Frame` with a type parameter.
- **`frame.rs`:** Depends on nothing engine-internal; purely a data carrier.
- **`working_state.rs`:** Depends only on user-supplied `S`. Can be implemented immediately after understanding the CoW contract.
- **`history.rs`:** Depends on `Frame`. Can be a simple wrapper around two `Vec<Frame>`.
- **`engine.rs`:** Depends on all of the above. Implemented last. This is where dispatch, undo, redo live.

No subfolders needed at MVP scale. A flat module structure keeps navigation simple and avoids artificial grouping.

---

## Type Dependency Graph

The build order follows this dependency chain. Each item can only be implemented after the items it depends on exist.

```
(nothing)
    │
    ▼
BehaviorResult<D, O>          — depends on: nothing engine-internal
    │
    ▼
Behavior<S, I, D, O, K> trait — depends on: BehaviorResult
    │
    ▼
Frame<I, D, T>                — depends on: nothing engine-internal
    │
    ▼
Outcome<F, N>                 — depends on: Frame (used as F)
EngineError                   — depends on: nothing
    │
    ▼
WorkingState<S>               — depends on: nothing engine-internal (wraps user S)
    │
    ▼
History<I, D, T>              — depends on: Frame
    │
    ▼
Engine<S, I, D, T, O, K, B>  — depends on: all of the above
```

This graph has no cycles. Each layer is independently testable.

---

## Suggested Build Order for Incremental Implementation

Implement in this sequence. Each step is compilable and partially testable before moving on.

### Step 1: `BehaviorResult<D, O>`

```rust
pub enum BehaviorResult<D, O> {
    Continue(Vec<D>),
    Stop(O),
}
```

No dependencies. Zero complexity. Defines the contract that every behavior returns.

### Step 2: `Behavior<S, I, D, O, K>` trait

```rust
pub trait Behavior<S, I, D, O, K> {
    fn name(&self) -> &'static str;
    fn order_key(&self) -> K;
    fn evaluate(&self, input: &I, state: &S) -> BehaviorResult<D, O>;
}
```

`S` is read-only (`&S`) in `evaluate`. Behaviors never mutate working state directly. `K: Ord` enables sorting by `(order_key, name)`.

### Step 3: `Frame<I, D, T>`

```rust
pub struct Frame<I, D, T> {
    pub input: I,
    pub diff: Vec<D>,   // or user-defined DiffCollection
    pub trace: Vec<T>,  // or user-defined Trace
}
```

Pure data carrier. No logic. Can be cloned (require `I: Clone, D: Clone, T: Clone`).

### Step 4: `Outcome<F, N>` and `EngineError`

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

pub enum EngineError {
    EmptyBehaviorSet,
    // Add as genuine engine bugs are discovered
}
```

`F` will be `Frame<I, D, T>`. `N` is a user-defined non-committed payload type. `EngineError` starts minimal — do not pre-populate with speculative variants.

### Step 5: `WorkingState<S>` (CoW wrapper)

This is the most mechanically interesting step. See the CoW section below for the full implementation strategy.

### Step 6: `History<I, D, T>`

```rust
struct History<I, D, T> {
    undo_stack: Vec<Frame<I, D, T>>,
    redo_stack: Vec<Frame<I, D, T>>,
}
```

Simple wrapper. Methods: `push_committed(frame)`, `pop_undo() -> Option<Frame>`, `pop_redo() -> Option<Frame>`, `clear()` (for irreversibility boundary). Redo stack is cleared on every new commit (standard undo/redo contract).

### Step 7: `Engine<...>` with dispatch, undo, redo

Implement dispatch first, then undo, then redo. Dispatch is the core; undo/redo are mechanical inversions using stored frames.

---

## CoW Working State: Implementation Strategy

### The Problem

The ARCHITECTURE.md spec requires:
- read from committed state until first write
- clone a substate only when first written
- later behaviors see the updated working substate
- if dispatch fails, discard the working copy with no committed mutation

### Approach: Reference + Optional Owned Clone

The engine holds a reference to the committed state during dispatch. On first write (first diff application), it clones the committed state into an owned working copy. All subsequent reads and writes go to the working copy.

```rust
// Internal to engine dispatch — not a public struct
enum WorkingState<'a, S: Clone> {
    Borrowed(&'a S),
    Owned(S),
}

impl<'a, S: Clone> WorkingState<'a, S> {
    fn new(committed: &'a S) -> Self {
        WorkingState::Borrowed(committed)
    }

    fn read(&self) -> &S {
        match self {
            WorkingState::Borrowed(s) => s,
            WorkingState::Owned(s) => s,
        }
    }

    fn write(&mut self) -> &mut S {
        if let WorkingState::Borrowed(s) = self {
            *self = WorkingState::Owned((*s).clone());
        }
        match self {
            WorkingState::Owned(s) => s,
            WorkingState::Borrowed(_) => unreachable!(),
        }
    }

    fn into_committed(self) -> Option<S> {
        match self {
            WorkingState::Owned(s) => Some(s),
            WorkingState::Borrowed(_) => None, // no writes occurred
        }
    }
}
```

The dispatch loop calls `working.read()` for all behavior evaluations and `working.write()` only when applying a diff. `into_committed()` returns `None` for no-diff dispatches (maps to `NoChange`) and `Some(new_state)` for successful commits.

### Why Not `std::borrow::Cow`?

`std::borrow::Cow<'a, S>` requires `S: ToOwned` and is designed primarily for string/slice scenarios. For arbitrary user State types, implementing `ToOwned` adds friction without benefit. A hand-rolled enum variant as shown above is clearer, more idiomatic for this use case, and does not require any trait bounds beyond `Clone`. Confidence: HIGH (standard library docs confirm `Cow`'s `ToOwned` requirement; this pattern is well-established in Rust).

### Substate Granularity

The ARCHITECTURE.md spec says substate granularity is user-chosen. The library does not enforce sub-substate CoW. The single `WorkingState<S>` wrapping the entire user State is sufficient for the MVP. If the user wants substate-level lazy cloning, they can implement it inside their own `State` type. The engine's contract is just: "read from committed, write to working copy, commit atomically on success."

### State Bound Requirements

The engine requires `S: Clone` to enable the working-copy clone on first write. No other bounds on `S` are needed by the engine. Behaviors receive `&S` (read-only), so no mutable borrow of state ever escapes the engine.

---

## Dispatch Algorithm: Concrete Implementation Flow

```
dispatch(input: I) -> Result<Outcome<Frame<I,D,T>, N>, EngineError>

1. Create WorkingState::Borrowed(&self.committed_state)
2. Sort behaviors by (order_key, name) — sort once at construction, not per dispatch
3. accumulated_diffs: Vec<D> = vec![]
4. accumulated_trace: Vec<T> = vec![]
5. For each behavior in sorted order:
   a. let result = behavior.evaluate(&input, working.read())
   b. match result:
      - Stop(outcome_payload) =>
          drop(working)            // discard — borrowed or owned
          return Ok(Outcome::Disallowed(outcome_payload))  // or InvalidInput/Aborted per user convention
      - Continue(diffs) =>
          for diff in diffs:
            diff.apply(working.write())   // triggers clone on first write
            let trace_entries = diff.trace_entries()
            accumulated_trace.extend(trace_entries)
            accumulated_diffs.push(diff)
6. If accumulated_diffs is empty:
   return Ok(Outcome::NoChange)
7. new_state = working.into_committed().unwrap()  // safe: diffs were applied
8. frame = Frame { input, diff: accumulated_diffs, trace: accumulated_trace }
9. self.committed_state = new_state
10. self.history.push_committed(frame.clone())
11. return Ok(Outcome::Committed(frame))
```

### Behavior Ordering at Construction Time

Sort behaviors once when `Engine` is constructed, not on every dispatch. Store behaviors in sorted order. This eliminates per-dispatch allocation and sorting overhead while guaranteeing determinism.

```rust
// Sort during Engine::new()
behaviors.sort_by(|a, b| {
    a.order_key().cmp(&b.order_key())
        .then_with(|| a.name().cmp(&b.name()))
});
```

### Diff Application Contract

The `apply` method must live on the user's Diff type, not on the engine. The engine calls it but does not own the logic. This requires a trait bound:

```rust
pub trait Apply<S> {
    fn apply(&self, state: &mut S);
}
```

Similarly, trace entries are extracted from diffs. One clean approach: a `Traced<T>` trait:

```rust
pub trait Traced<T> {
    fn trace_entries(&self) -> Vec<T>;
}
```

This keeps the engine generic: `D: Apply<S> + Traced<T>`. Users implement these two traits on their diff type.

---

## Static Behavior Set: How to Represent It

The ARCHITECTURE.md spec says behaviors are statically known at compile time. There are two clean options in Rust:

### Option A: `Vec<Box<dyn Behavior<...>>>` (Recommended for MVP)

```rust
pub struct Engine<S, I, D, T, O, K> {
    committed_state: S,
    behaviors: Vec<Box<dyn Behavior<S, I, D, O, K>>>,
    history: History<I, D, T>,
}
```

Users pass a `Vec<Box<dyn Behavior<...>>>` at construction. The set is static (fixed at construction, never modified at runtime). Dynamic dispatch here is a vtable call per behavior per dispatch — negligible cost for game turn evaluation. This avoids HList complexity and makes the API ergonomic.

Confidence: HIGH — this is the standard Rust pattern for this kind of plugin slot. The ARCHITECTURE.md says "avoid dynamic dispatch as a design center," which is satisfied: dynamic dispatch is used only in the behavior collection, not throughout the system.

### Option B: Type-parameter HList (not recommended for MVP)

Using `frunk` HLists or a hand-rolled type-level list makes the behavior set zero-cost but creates combinatorial generic complexity on `Engine`. This makes the API nearly unusable without complex type inference gymnastics. Defer to post-MVP if benchmark evidence demands it.

---

## Undo / Redo: Implementation Details

```
undo() -> Result<Outcome<Frame<I,D,T>, N>, EngineError>

1. Check history.undo_stack.last() — if empty, return Ok(Outcome::Disallowed(NothingToUndo))
2. frame = history.undo_stack.pop().unwrap()
3. Apply frame.diff in REVERSE order, each diff reversed, to self.committed_state
   OR: store snapshots of committed state alongside frames
4. history.redo_stack.push(frame.clone())
5. return Ok(Outcome::Undone(frame))
```

### Undo Strategy: Reverse Diff vs. Snapshot

There are two implementable approaches:

**Reverse diff:** Each diff type implements a `reverse()` method returning an inverse diff. Applied in reverse order. Zero extra storage per frame. Requires the user to implement `Reverse` on their diff type. This is elegant but adds a contract burden.

**Snapshot:** Store `committed_state.clone()` alongside each frame in history. Undo = replace committed state with the snapshot. O(1) undo logic, O(N * state_size) memory.

**Recommendation for MVP:** Snapshot approach. It has no additional trait requirements on user diff types, is trivially correct, and state size for turn-based games is small. If memory becomes an issue post-MVP, introduce `Reversible` trait.

```rust
struct HistoryEntry<I, D, T, S> {
    frame: Frame<I, D, T>,
    prior_state: S,   // state BEFORE this frame was committed
}
```

Undo: `committed_state = entry.prior_state`. Return `Undone(entry.frame)`.

### Irreversibility Boundary

The library user calls an `Engine::mark_irreversible()` method (or a flag on dispatch). When an irreversible transition commits:

1. Commit the frame normally
2. Call `history.clear()` — erases both undo and redo stacks
3. No further undo is possible across that boundary

This matches the ARCHITECTURE.md recommended policy exactly.

---

## Architectural Patterns

### Pattern 1: Opaque Engine with Generic Type Parameters

**What:** The engine is a single struct with all user-supplied types as generic parameters. The engine's internals are fully hidden; only `dispatch`, `undo`, `redo`, and a constructor are public.

**When to use:** Always — this is the intended design.

**Trade-offs:** Complex generic signatures at construction sites. Mitigated by type aliases in user code (`type MyEngine = Engine<MyState, MyInput, MyDiff, MyTrace, MyOutcomePayload, u32>`).

### Pattern 2: Atomic Commit via Move Semantics

**What:** The working state is a local variable inside `dispatch`. It is never stored inside `Engine`. On success, move the owned working copy into `self.committed_state`. On failure, the local variable is dropped.

**When to use:** Always. This is how atomicity is guaranteed without locks or transactions.

**Trade-offs:** None — this is idiomatic Rust. The borrow checker enforces correctness.

### Pattern 3: Behaviors Sorted Once at Construction

**What:** Sort the behavior `Vec` by `(order_key, name)` when `Engine::new()` is called, not on each `dispatch()`.

**When to use:** Always.

**Trade-offs:** None at MVP. Behaviors cannot be added after construction, which the spec already requires.

---

## Data Flow

### Dispatch Flow

```
User calls dispatch(input)
    │
    ▼
WorkingState::Borrowed(&committed_state) created
    │
    ▼
For each Behavior (pre-sorted by order_key, name):
    behavior.evaluate(&input, working.read())
        │
        ├─ Stop(N)  ──► drop working, return Disallowed/InvalidInput/Aborted(N)
        │
        └─ Continue(diffs):
               for each diff:
                   diff.apply(working.write())  ← triggers clone on first write
                   accumulated_trace += diff.trace_entries()
                   accumulated_diffs.push(diff)
    │
    ▼
accumulated_diffs empty? ──► return NoChange
    │
    ▼
new_state = working.into_committed()
frame = Frame { input, diff: accumulated_diffs, trace: accumulated_trace }
history.push(HistoryEntry { frame, prior_state: old_committed })
committed_state = new_state
return Committed(frame)
```

### Undo Flow

```
User calls undo()
    │
    ▼
history.undo_stack empty? ──► return Disallowed(NothingToUndo)
    │
    ▼
entry = history.undo_stack.pop()
committed_state = entry.prior_state   ← snapshot restore
history.redo_stack.push(entry.frame.clone())
return Undone(entry.frame)
```

---

## Anti-Patterns

### Anti-Pattern 1: Eager State Clone on Every Dispatch

**What people do:** Clone the entire committed state at the start of every dispatch call.

**Why it's wrong:** Violates the CoW contract; unnecessary performance cost for large states; the prior implementation (v0.4.0) had this bug with `dispatch_preview()`.

**Do this instead:** Use `WorkingState::Borrowed` until the first diff application. Clone only on first write.

### Anti-Pattern 2: Behavior State Outside the Main State Tree

**What people do:** Store behavior counters, cooldowns, or flags as `mut` fields on the `Behavior` struct itself.

**Why it's wrong:** State mutated inside a behavior struct bypasses the committed state, breaks undo/redo, and is not serializable. This was the exact bug in v0.4.0.

**Do this instead:** All behavior-local state lives inside `State` (e.g., `state.behaviors.my_behavior_counter`). Behaviors read it via `&S` and change it only by emitting diffs.

### Anti-Pattern 3: Behaviors That Mutate Working State Directly

**What people do:** Pass `&mut S` to `evaluate()` and mutate inline.

**Why it's wrong:** Bypasses the diff system, loses trace, makes replay impossible, violates atomicity.

**Do this instead:** `evaluate()` receives `&S` only. All mutations go through `BehaviorResult::Continue(diffs)`.

### Anti-Pattern 4: Public Undo/Redo Stacks

**What people do:** Expose history stacks as public fields on `Engine`.

**Why it's wrong:** Breaks encapsulation; allows external code to corrupt history; was identified as a v0.4.0 bug.

**Do this instead:** History is a private field. Only `undo()` and `redo()` methods can modify it.

### Anti-Pattern 5: Memory-Address-Based Behavior Ordering

**What people do:** Use `std::ptr::addr_of` or hash of behavior reference for ordering tiebreaking.

**Why it's wrong:** Not deterministic across runs, compilations, or allocations. Was the v0.4.0 tiebreaker bug.

**Do this instead:** `(order_key, behavior.name())` — both stable and deterministic.

---

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Engine ↔ Behavior | `evaluate(&input, &state) -> BehaviorResult<D, O>` | Read-only state access; pure function from engine's perspective |
| Engine ↔ Diff | `diff.apply(&mut working_state)`, `diff.trace_entries()` | Two trait methods on user's diff type |
| Engine ↔ History | Internal only — `HistoryEntry` push/pop | Never exposed in public API |
| User ↔ Engine | `dispatch()`, `undo()`, `redo()` only | Minimal surface area |

---

## Scaling Considerations

This is a library, not a service. "Scaling" means state size and behavior count.

| Scale | Consideration |
|-------|---------------|
| < 20 behaviors, small state | Current design: no concerns. Vec + single clone if needed. |
| 50-200 behaviors, large state (AI lookahead) | CoW avoids clone on read-only dispatches. For AI lookahead (many dispatches), ensure state Clone is efficient. The snapshot-based undo adds one extra clone per commit — acceptable. |
| Substate granularity | If state is very large and only a few substates change per dispatch, the user can implement internal CoW in their own State type. The engine contract does not prevent this. |

---

## Sources

- ARCHITECTURE.md in repository root (primary source, HIGH confidence)
- PROJECT.md in `.planning/PROJECT.md` (requirements context, HIGH confidence)
- [Rust std::borrow::Cow documentation](https://doc.rust-lang.org/std/borrow/enum.Cow.html) — confirms `ToOwned` requirement making hand-rolled approach cleaner for this use case (HIGH confidence)
- [Enum vs Trait Object — Possible Rust](https://www.possiblerust.com/guide/enum-or-trait-object) — static vs dynamic dispatch tradeoffs (MEDIUM confidence)
- [enum_dispatch crate](https://crates.io/crates/enum_dispatch) — alternative to `Box<dyn Trait>` if zero-cost dispatch needed post-MVP (MEDIUM confidence)

---
*Architecture research for: HerdingCats — Rust deterministic turn-based game engine library*
*Researched: 2026-03-13*
