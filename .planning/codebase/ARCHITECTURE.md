# Architecture

**Analysis Date:** 2026-03-13

## Pattern Overview

**Overall:** Deterministic input-driven state transition engine

**Key Characteristics:**
- Input-driven: Engine never advances without explicit input or undo/redo call
- Deterministic: Behaviors execute in stable `(order_key, name)` order
- Atomic: Dispatch either commits fully or no state changes become visible
- Ordered behavior resolution: Statically known set of behaviors checked sequentially
- Immediate diff application: Each applied diff mutates working state immediately
- Centralized mutation: Engine owns all state mutations, behaviors emit diffs only

## Layers

**Library API:**
- Purpose: Provides engine dispatch, undo/redo, and outcome construction
- Location: `src/lib.rs` (skeleton - to be implemented)
- Contains: Public traits and types for behavior, state, input, diff, trace definitions
- Depends on: Core engine implementation
- Used by: Game implementations (examples, external crates)

**Engine Core:**
- Purpose: Orchestrates behavior ordering, diff application, working state management, history tracking
- Location: To be implemented in `src/engine.rs` or module
- Contains: Dispatch algorithm, working state construction, history management, undo/redo logic
- Depends on: Behavior trait, input/output types
- Used by: Public API

**Behavior Evaluation:**
- Purpose: Executes individual behaviors and produces diffs
- Location: Statically composed at compile-time in user code
- Contains: User-defined behavior implementations satisfying the Behavior trait
- Depends on: Input, State, Diff types
- Used by: Engine during dispatch

**User State Composition:**
- Purpose: Defines domain state, behavior state, diffs, and traces for the specific game
- Location: User code (e.g., `examples/` implementations)
- Contains: State struct, DomainState, BehaviorState, StateDiff, and Trace types
- Depends on: Nothing from the engine
- Used by: Engine via generic parameters

## Data Flow

**Forward Dispatch:**

1. Caller submits `Input` to `engine.dispatch(input, state)`
2. Engine creates speculative **working state** from current committed state
3. Engine orders all behaviors by `(order_key, behavior_name)`
4. For each behavior in order:
   1. Call `behavior.evaluate(input, &working_state)` → `BehaviorResult<D, O>`
   2. If `Stop(outcome)`, halt immediately and discard working state, return non-committed outcome
   3. If `Continue(diffs)`, apply each diff sequentially:
      - Mutate working state through diff's `apply` method
      - Append trace entries at the same moment diff is applied
5. After all behaviors processed:
   - If no diffs were applied, return `NoChange` (non-committed)
   - Otherwise accumulate all applied diffs into a collection
6. Construct `Frame { input, diffs, trace }`
7. Commit atomically: Update internal state pointer and append frame to history
8. Return `Committed(frame)`

**State Management:**

- **Committed state**: The last successfully committed full state
- **Working state**: Copy-on-write speculative state created fresh for each dispatch
  - Reads from committed state until first write to a substate
  - Clone substates only when first written
  - Granularity is user-defined (coarse or fine-grained substates)
  - Later behaviors see changes from earlier applied diffs
  - If dispatch fails, discarded entirely

**Undo/Redo:**

- Undo: Move undo pointer backward one frame
  - Revert committed state to previous frame's pre-transition state
  - Pop current frame from redo stack
  - Return `Undone(frame)` with the frame being unwound
  - Irreversible boundaries erase undo/redo history across them

- Redo: Move redo pointer forward one frame
  - Replay the frame's diffs onto current state
  - Return `Redone(frame)`
  - Only valid if a frame was previously undone

## Key Abstractions

**Behavior:**
- Purpose: Statically known, compile-time-defined participant in resolution
- Examples: Movement validation, attack resolution, end-turn cleanup
- Pattern: Trait with `name()`, `order_key()`, `evaluate(input, state)` methods
- Evaluation: Takes `&Input` and `&State`, returns `BehaviorResult<Diff, Outcome>`
- Order: Behaviors within same order_key sort alphabetically by name
- State: Behavior-local state lives inside main `State` (not hidden in engine)

**Diff (State Mutation):**
- Purpose: User-defined description of a state mutation
- Pattern: Enum that mirrors state structure (e.g., `enum StateDiff { Domain(...), Behaviors(...) }`)
- Application: Each diff type implements an `apply(&mut state, &mut trace)` method
- Timing: Applied immediately during dispatch, never reconstructed later
- Coupling: Any diff that mutates state must append at least one trace entry

**Trace (Execution Record):**
- Purpose: Record of why state changed, for UI and replay
- Pattern: Accumulator built during dispatch, appended to as each diff applies
- Content: User-defined events/entries describing state changes
- Timing: Generated in execution order at the exact moment diffs apply
- Scope: Only produced on successful committed transitions

**Frame (Transition Record):**
- Purpose: Canonical record of successful atomic transition
- Structure: `Frame { input: Input, diffs: DiffCollection, trace: Trace }`
- Persistence: Stored in history for undo/redo and replay
- Atomicity: Entire frame commits or nothing commits
- Immutability: Never modified after creation

**Outcome (Operation Result):**
- Purpose: Distinguish successful committed transitions from rejected/aborted attempts
- Variants:
  - `Committed(frame)`: Successful non-empty transition committed
  - `Undone(frame)`: Frame was undone (frame is the one being unwound)
  - `Redone(frame)`: Frame was redone (frame is the one being replayed)
  - `NoChange`: Dispatch completed with no state mutation
  - `InvalidInput(...)`: Input doesn't make sense in current context
  - `Disallowed(...)`: Input is meaningful but a behavior forbids it
  - `Aborted(...)`: Dispatch began but diff application or resolution couldn't safely proceed

**EngineError (Non-Domain Failure):**
- Purpose: Distinguish genuine engine/library failures from normal domain outcomes
- Usage: For impossible internal state, malformed configuration, or panic-like conditions
- Scope: Not part of normal dispatch semantics; indicates implementation bug

## Entry Points

**dispatch(input):**
- Location: Engine public API (to be `src/lib.rs` or public module)
- Triggers: Called by game/application layer with a user-defined `Input`
- Responsibilities:
  1. Create working state from committed state
  2. Order and evaluate all behaviors sequentially
  3. Apply diffs and trace immediately
  4. Commit atomically if non-empty
  5. Return `Result<Outcome, EngineError>`

**undo():**
- Location: Engine public API
- Triggers: Called explicitly by caller to undo last committed transition
- Responsibilities:
  1. Check if undo is possible (undo history exists, not blocked by irreversible boundary)
  2. Revert state to frame being undone
  3. Move history pointers
  4. Return `Result<Outcome, EngineError>` with `Undone(frame)`

**redo():**
- Location: Engine public API
- Triggers: Called explicitly by caller to redo a frame that was undone
- Responsibilities:
  1. Check if redo is possible (redo stack non-empty)
  2. Replay frame's diffs onto current state
  3. Move history pointers
  4. Return `Result<Outcome, EngineError>` with `Redone(frame)`

## Error Handling

**Strategy:** Two-level error model

**Level 1: Outcome (Domain Semantics)**
- Success: `Committed`, `Undone`, `Redone`
- Non-committed: `NoChange`, `InvalidInput`, `Disallowed`, `Aborted`
- These are normal operation outcomes, never errors

**Level 2: EngineError (Implementation Failures)**
- Returned as `Result<Outcome, EngineError>`
- Reserved for impossible internal engine state, bugs, or panic-like conditions
- Library user should distinguish:
  - "dispatch was rejected or safely aborted" (Outcome)
  - "engine malfunctioned" (EngineError)

**Patterns:**

- Validation: Happens during ordered behavior evaluation, not in separate pre-pass
  - Each behavior inspects working state and returns no diffs if conditions unmet
  - Later behaviors see updated state and may reach different conclusions
  - This preserves path-dependent rule application

- Early termination: Behavior returns `Stop(outcome)` to halt dispatch
  - Remaining behaviors are not evaluated
  - Working state is discarded
  - Non-committed outcome is returned

- Controlled failure: Behavior returns `Aborted(context)` to indicate safe failure
  - Used when diff application cannot safely proceed
  - Example: Missing entity during controlled state update
  - Not for engine bugs, only for domain-level safe aborts

## Cross-Cutting Concerns

**Logging:** Not part of MVP; library user provides logging through trace inspection

**Validation:** Happens during behavior evaluation against working state
- Path-dependent: Later behaviors see earlier state changes
- Not pre-validated: No separate validation pass before behavior resolution
- Inclusive: Invalid inputs return `InvalidInput` outcome; disallowed by rules return `Disallowed`

**Authentication:** Not applicable; this is a game engine library

**State Visibility:**
- Only committed state is visible to caller
- Working state is internal and discarded on failure
- Behavior state lives in main state tree for undo/redo/serialization correctness

**Determinism:**
- Behavior ordering: `(order_key, behavior_name)` deterministic sort
- Diff ordering: Within a behavior, diffs applied in returned order
- Trace ordering: Built in execution order, never reconstructed
- No random elements in core engine; RNG managed by user through state and diffs

**Atomicity:**
- Either entire dispatch succeeds and commits, or no visible state changes
- No partial commits or rollback during failure
- Working state isolated from committed state until success

---

*Architecture analysis: 2026-03-13*
