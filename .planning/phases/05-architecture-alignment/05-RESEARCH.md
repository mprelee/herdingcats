# Phase 5: Architecture Alignment - Research

**Researched:** 2026-03-13
**Domain:** Rust refactoring — cross-cutting API changes to a zero-dependency library
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Stop outcome dispatch — NonCommittedOutcome wrapper**
- Add `pub enum NonCommittedOutcome<N> { InvalidInput(N), Disallowed(N), Aborted(N) }` in `src/outcome.rs`, re-exported at crate root
- `BehaviorResult<D, N>` (renamed from `<D, O>`) becomes `Continue(Vec<D>)` | `Stop(NonCommittedOutcome<N>)`
- Dispatch preserves the outcome directly: `Stop(outcome) => return Ok(outcome.into())` via `From<NonCommittedOutcome<N>> for Outcome<F, N>`
- Behaviors now explicitly choose `InvalidInput`, `Disallowed`, or `Aborted` — no more coercion to `Aborted`

**Trace enforcement — doc contract, no structural change**
- `Apply::apply()` trait signature stays as-is: `fn apply(&self, state: &mut E::State) -> Vec<E::Trace>`
- Remove the doc line "Returning an empty Vec is valid for diffs that produce no trace" from `apply.rs`
- Replace with: "Each call MUST return at least one trace entry describing the mutation. The engine does not accept empty trace from a state-mutating diff."
- This is a docs-only change

**Frame shape — remove reversibility**
- `Frame<E>` becomes `{ input: E::Input, diffs: Vec<E::Diff>, traces: Vec<E::Trace> }` — no `reversibility` field
- Reversibility moves to history metadata: `undo_stack: Vec<(E::State, Frame<E>, Reversibility)>`
- Remove `reversibility` from Frame's manual `Clone` and `PartialEq` impls
- Update `outcome.rs` to remove `use crate::reversibility::Reversibility` from Frame

**Static behavior composition — keep trait objects, construction-only**
- Keep `Vec<Box<dyn Behavior<E>>>` in `Engine<E>`
- No `.register_behavior()` API (none exists currently — confirmed)
- Behaviors passed at `Engine::new()` only, sorted once at construction

**EngineSpec bounds — remove State: Default**
- Change `type State: Clone + std::fmt::Debug + Default` to `type State: Clone + std::fmt::Debug`
- `Engine::new()` takes initial state as parameter — `Default` is never called by engine code
- Update all test `TestSpec` impls that relied on `Default`

**Outcome semantics tightening**
- All semantics already correct — no behavioral changes, docs/comment updates only

**Execution order (locked)**
- Wave 1: Fix BehaviorResult/NonCommittedOutcome/Outcome contract (foundation)
- Wave 2: Trace docs + Frame shape (depends on stable outcome contract)
- Wave 3: EngineSpec bounds + docs/README + test updates

### Claude's Discretion
- Exact `From<NonCommittedOutcome<N>> for Outcome<F, N>` impl details
- Whether `NonCommittedOutcome` gets `#[derive(Debug, Clone, PartialEq)]` (reasonable yes)
- README structure and content (must describe: input-driven dispatch, static behavior set, CoW, behaviors emit diffs, engine applies immediately, traces during application, Frame {input, diff, trace}, undo/redo)
- How to handle the `Reversibility` import in outcome.rs after Frame no longer uses it

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

## Summary

Phase 5 is a cross-cutting refactoring phase with no new features. The implementation has drifted from ARCHITECTURE.md in six specific ways, each requiring surgical changes. The most foundational change is `NonCommittedOutcome<N>` — it renames a type param, wraps `BehaviorResult::Stop`, and adds a `From` impl — so everything else ripples from it. Frame shape removal of `reversibility` is the second most disruptive change, touching engine struct fields, stack tuple types, Frame construction, Frame Clone/PartialEq impls, and all test assertions on `frame.reversibility`.

The discipline here is: do not hand-roll anything, do not introduce new types beyond what is listed in CONTEXT.md, and verify each change compiles before moving to the next. The codebase is small enough that `cargo test --all` is the full verification gate. Tests that currently pass against `frame.reversibility` will fail after the Frame shape change — that is expected and those tests must be updated in the same wave.

**Primary recommendation:** Work wave-by-wave in the locked order. Each wave should leave `cargo test --all` green before the next wave begins.

---

## Current State Inventory

### What Exists Today (pre-Phase 5)

| File | Relevant Current State | Change Required |
|------|----------------------|-----------------|
| `src/outcome.rs` | `Frame<E>` has `reversibility: Reversibility` field; no `NonCommittedOutcome` type | Add `NonCommittedOutcome`, remove `reversibility` from Frame |
| `src/behavior.rs` | `BehaviorResult<D, O>`, `Stop(O)` where O is raw `E::NonCommittedInfo` | Rename O→N, change `Stop(O)` to `Stop(NonCommittedOutcome<N>)` |
| `src/engine.rs` | `undo_stack: Vec<(E::State, Frame<E>)>` (no Reversibility in tuple); dispatch `BehaviorResult::Stop(info) => Ok(Outcome::Aborted(info))`; `Frame { ..., reversibility }` construction | Move Reversibility to stack tuple, update Stop dispatch arm, remove reversibility from Frame construction |
| `src/spec.rs` | `type State: Clone + std::fmt::Debug + Default` | Remove `Default` bound |
| `src/apply.rs` | Doc says "Returning an empty Vec is valid" | Replace doc line |
| `src/lib.rs` | No `NonCommittedOutcome` re-export | Add `pub use crate::outcome::NonCommittedOutcome` |
| `examples/tictactoe.rs` | `BehaviorResult::Stop("string".to_string())` — raw string, always becomes `Aborted` | Wrap in `NonCommittedOutcome::Disallowed(...)` or appropriate variant |
| `examples/backgammon.rs` | No `BehaviorResult::Stop` calls (behaviors only use `Continue`) | No change to behavior impls; verify compiles with new API |
| `README.md` | Empty (one blank line) | Write full architecture description |

### Confirmed: undo_stack Tuple is 2-tuple Today

Current engine.rs line 67-68:
```rust
undo_stack: Vec<(E::State, Frame<E>)>,
redo_stack: Vec<(E::State, Frame<E>)>,
```

After Phase 5 this becomes a 3-tuple: `Vec<(E::State, Frame<E>, Reversibility)>`.

The `Reversibility` value for each entry is the value that was passed to `dispatch()` for that frame.

### Confirmed: dispatch Stop Arm Currently Hardcodes Aborted

Current engine.rs dispatch match:
```rust
BehaviorResult::Stop(info) => {
    return Ok(Outcome::Aborted(info));
}
```

After Phase 5, `info` is `NonCommittedOutcome<E::NonCommittedInfo>` and the arm becomes:
```rust
BehaviorResult::Stop(outcome) => {
    return Ok(outcome.into());
}
```

Where `From<NonCommittedOutcome<N>> for Outcome<F, N>` maps each variant.

### Confirmed: tictactoe.rs Uses Stop with Raw Strings

All four behaviors in tictactoe.rs that call `BehaviorResult::Stop` pass raw `String` values today, meaning they all produce `Outcome::Aborted`. After the change, each must explicitly choose `NonCommittedOutcome::Disallowed(...)` or `NonCommittedOutcome::InvalidInput(...)` as semantically appropriate. The example comments currently say "→ produces Aborted in dispatch" — these must also be updated.

### Confirmed: backgammon.rs Has No Stop Calls

Both backgammon behaviors (`RollDiceBehavior`, `MovePieceBehavior`) only return `BehaviorResult::Continue`. The example will not need behavior-level changes for the NonCommittedOutcome migration. It will need to compile cleanly with the new API types.

### Confirmed: spec.rs Tests Use Default-derived State

`src/spec.rs` test `engine_spec_associated_types_satisfy_bounds` calls `<TestSpec as EngineSpec>::State::default()`. After removing the `Default` bound from `EngineSpec`, this test will fail to compile — it must be updated to use explicit construction instead.

### Confirmed: outcome.rs Tests Reference frame.reversibility

Tests in `src/outcome.rs`: `frame_stores_reversibility`, `frame_is_constructable_cloneable_and_eq`, `frame_stores_vec_diffs_and_vec_traces` all construct `Frame { ..., reversibility: ... }` and assert on `frame.reversibility`. These will break when the field is removed and must be updated.

### Confirmed: engine.rs Tests Reference frame.reversibility

`frame_contains_input_diffs_trace` in `src/engine.rs` asserts `frame.reversibility == Reversibility::Irreversible`. This will fail after the Frame shape change and must be updated to remove that assertion (the frame no longer carries reversibility).

---

## Standard Stack

This is a zero-dependency Rust library. No external crates are introduced in this phase.

### Core
| Tool | Version | Purpose |
|------|---------|---------|
| `cargo test --all` | Rust stable | Full test suite including unit tests and proptests |
| `cargo clippy --all-targets -- -D warnings` | Rust stable | Lint gate (no new warnings allowed) |
| `cargo doc --no-deps` | Rust stable | Verify rustdoc examples compile |

### Dev Dependencies (unchanged)
| Crate | Version | Purpose |
|-------|---------|---------|
| `proptest` | 1.10 | Property-based tests already written; must pass after changes |

**No new dependencies are added in this phase.**

---

## Architecture Patterns

### Wave Sequencing Pattern

This phase uses locked wave ordering. Each wave must leave `cargo test --all` green.

```
Wave 1 — Outcome contract foundation (outcome.rs, behavior.rs, engine.rs dispatch arm, lib.rs)
  ├─ Add NonCommittedOutcome<N> enum
  ├─ Rename BehaviorResult type param O→N, change Stop(O)→Stop(NonCommittedOutcome<N>)
  ├─ Add From<NonCommittedOutcome<N>> for Outcome<F, N>
  ├─ Update engine.rs Stop dispatch arm
  ├─ Update all test StopBehavior impls to return Stop(NonCommittedOutcome::Aborted(...))
  ├─ Update examples Stop calls
  └─ Add lib.rs re-export

Wave 2 — Trace docs + Frame shape (outcome.rs, engine.rs)
  ├─ Remove doc line from apply.rs
  ├─ Remove reversibility field from Frame<E>
  ├─ Update Frame Clone/PartialEq impls
  ├─ Move Reversibility to undo_stack/redo_stack tuples
  ├─ Update Frame construction in dispatch()
  └─ Fix all tests referencing frame.reversibility

Wave 3 — EngineSpec bounds + docs/README (spec.rs, all files)
  ├─ Remove Default bound from EngineSpec::State
  ├─ Fix spec.rs test that calls ::default()
  ├─ Write README.md
  └─ Final cargo doc --no-deps verification
```

### From Impl Pattern for NonCommittedOutcome

The `From` conversion is the clean Rust idiom for the Stop→Outcome mapping:

```rust
// Source: CONTEXT.md decision
impl<F, N> From<NonCommittedOutcome<N>> for Outcome<F, N> {
    fn from(nco: NonCommittedOutcome<N>) -> Self {
        match nco {
            NonCommittedOutcome::InvalidInput(n) => Outcome::InvalidInput(n),
            NonCommittedOutcome::Disallowed(n) => Outcome::Disallowed(n),
            NonCommittedOutcome::Aborted(n) => Outcome::Aborted(n),
        }
    }
}
```

This is placed in `outcome.rs` alongside both types. The dispatch arm becomes:
```rust
BehaviorResult::Stop(outcome) => return Ok(outcome.into()),
```

### Frame Construction Pattern (Post Wave 2)

Frame construction in dispatch() currently includes `reversibility`. After Wave 2:

```rust
// Before (current)
let frame = Frame {
    input,
    diffs,
    traces,
    reversibility,
};
self.undo_stack.push((prior_state, frame.clone()));

// After (Phase 5)
let frame = Frame {
    input,
    diffs,
    traces,
};
self.undo_stack.push((prior_state, frame.clone(), reversibility));
```

The `Reversibility` value is checked from the tuple when needed in the undo/redo/irreversibility logic. However, undo() and redo() currently do not need to consult reversibility after commit — the clearing happens at commit time. So the Reversibility value in the tuple is stored for completeness but only used in the commit path to determine whether to clear stacks.

Actually: looking at the current engine.rs dispatch logic, the irreversibility check happens with the local `reversibility` variable, not from the Frame. So after Frame shape change, the commit path stays:

```rust
if reversibility == Reversibility::Irreversible {
    self.undo_stack.clear();
    self.redo_stack.clear();
}
```

The tuple type changes but this code path remains the same. The `Reversibility` value in the tuple is stored but the existing logic doesn't read it back from the stack — that is fine for MVP.

### Manual Clone/PartialEq Pattern for Frame<E>

Frame uses manual impls to avoid adding `E: Clone` / `E: PartialEq` bounds (since E is a unit struct spec). After removing `reversibility`:

```rust
impl<E: EngineSpec> Clone for Frame<E>
where
    E::Input: Clone,
    E::Diff: Clone,
    E::Trace: Clone,
{
    fn clone(&self) -> Self {
        Frame {
            input: self.input.clone(),
            diffs: self.diffs.clone(),
            traces: self.traces.clone(),
            // reversibility REMOVED
        }
    }
}

impl<E: EngineSpec> PartialEq for Frame<E>
where
    E::Input: PartialEq,
    E::Diff: PartialEq,
    E::Trace: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
            && self.diffs == other.diffs
            && self.traces == other.traces
            // reversibility comparison REMOVED
    }
}
```

The `use crate::reversibility::Reversibility;` import at the top of `outcome.rs` can be removed entirely after Frame no longer references it.

### NonCommittedOutcome Derives

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NonCommittedOutcome<N> {
    /// The input does not make sense in the current context.
    InvalidInput(N),
    /// The input is meaningful, but a behavior or rule forbids it.
    Disallowed(N),
    /// Dispatch began but a behavior halted it in a controlled abort.
    Aborted(N),
}
```

This is not `#[non_exhaustive]` — same reasoning as `Outcome`: these three variants are the stable public contract.

### BehaviorResult Rename Pattern

The type param rename from `O` to `N` is cosmetic but meaningful:

```rust
// Before
pub enum BehaviorResult<D, O> {
    Continue(Vec<D>),
    Stop(O),
}

// After
pub enum BehaviorResult<D, N> {
    Continue(Vec<D>),
    Stop(NonCommittedOutcome<N>),
}
```

All callers that construct `BehaviorResult::Stop(...)` must change. In tests:
```rust
// Before
BehaviorResult::Stop("halted".to_string())

// After
BehaviorResult::Stop(NonCommittedOutcome::Aborted("halted".to_string()))
```

The docstring example in `behavior.rs` also needs updating.

### Example Behavior Semantic Update Pattern

In tictactoe.rs, all `BehaviorResult::Stop` returns are currently rule violations (illegal moves). These are semantically `Disallowed`, not `Aborted`. The distinction matters per ARCHITECTURE.md:
- `Disallowed` — a behavior or rule forbids the move (e.g., "cell already occupied")
- `Aborted` — a fatal precondition failed during resolution
- `InvalidInput` — the input doesn't make sense in context

Recommended mapping for tictactoe.rs:
- `ValidateTurn`: `game is over` → `NonCommittedOutcome::Disallowed("game is over".to_string())`
- `ValidateCell`: `out of bounds` → `NonCommittedOutcome::InvalidInput("out of bounds".to_string())`
- `ValidateCell`: `cell already occupied` → `NonCommittedOutcome::Disallowed("cell already occupied".to_string())`

The example's print_dispatch comment that says "(unreachable via dispatch in MVP engine)" must also be removed — after this change, `InvalidInput` IS reachable via dispatch.

### Anti-Patterns to Avoid

- **Don't remove `Reversibility` from public API**: It's still used by `Engine::dispatch()` callers and in `undo_stack` tuples. Only remove it from `Frame`.
- **Don't add `Default` bound back**: The point of Wave 3 is removing it. Tests must use explicit struct construction.
- **Don't merge waves**: Each wave changes semantics; merging makes rollback impossible and makes compilation errors harder to isolate.
- **Don't change Apply trait signature**: The trace doc change is doc-only. The `fn apply(&self, state: &mut E::State) -> Vec<E::Trace>` signature is unchanged.
- **Don't add `#[non_exhaustive]` to NonCommittedOutcome**: This enum is a stable public contract, same as `Outcome` and `Reversibility`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| NonCommittedOutcome → Outcome mapping | Custom match in dispatch | `impl From<NonCommittedOutcome<N>> for Outcome<F, N>` + `.into()` |
| Frame field access with no reversibility | Store Reversibility somewhere else in Frame | Move to stack tuple `(E::State, Frame<E>, Reversibility)` |
| Test behavior returning Stop variants | Complex wrapper types | `NonCommittedOutcome::Aborted(...)` directly in test behavior evaluate() |

---

## Common Pitfalls

### Pitfall 1: Reversibility Import Orphan in outcome.rs

**What goes wrong:** After Frame no longer has a `reversibility` field, `use crate::reversibility::Reversibility` at the top of `outcome.rs` becomes unused. The compiler emits a dead_code/unused-import warning (or error with `-D warnings`).

**How to avoid:** Remove the import from `outcome.rs` during Wave 2. Reversibility is still used in `engine.rs` — no issue there.

**Warning signs:** `cargo clippy -- -D warnings` fails with unused import on `outcome.rs`.

### Pitfall 2: Stop Arm Type Mismatch

**What goes wrong:** After `BehaviorResult::Stop` wraps `NonCommittedOutcome<N>` instead of `N`, any test or behavior that passes a raw `N` value to `Stop(...)` fails to compile. The error message is type mismatch, not a missing import.

**How to avoid:** Update every `BehaviorResult::Stop(...)` call site in the same wave as the type definition change. There are four in tictactoe.rs (two in ValidateCell, one in ValidateTurn), several in engine.rs tests (StopBehavior), and docstring examples in behavior.rs.

**Warning signs:** `error[E0308]: mismatched types` on any `BehaviorResult::Stop(...)` call.

### Pitfall 3: frame.reversibility Assertion Failures

**What goes wrong:** Tests that assert `frame.reversibility == Reversibility::Reversible` fail with a field-not-found compile error after the Frame shape change. The test `frame_contains_input_diffs_trace` in engine.rs is the primary one. The tests in outcome.rs that construct Frame with `reversibility: ...` also fail.

**How to avoid:** In Wave 2, search for all occurrences of `frame.reversibility` and `reversibility:` in Frame struct literals and update them. After the change, the Frame-level reversibility assertion is simply removed (the test verifies other fields instead).

**Warning signs:** `error[E0609]: no field 'reversibility' on type 'Frame<TestSpec>'`.

### Pitfall 4: TestSpec Default Removal Cascade

**What goes wrong:** `src/spec.rs` tests call `<TestSpec as EngineSpec>::State::default()`. After removing `State: Default`, this fails. But more subtly, `examples/tictactoe.rs` calls `TicTacToeState::default()` and `Engine::<TicTacToeSpec>::new(TicTacToeState::default(), ...)`. The TicTacToeState type still derives `Default` — that derive is on the user's type, not constrained by EngineSpec. So `TicTacToeState::default()` continues to work fine. Only the `EngineSpec::State: Default` bound is removed; user types are free to implement Default on their own.

**How to avoid:** In Wave 3, update only the spec.rs test that calls `::State::default()` using the EngineSpec associated type path. Examples do not need changes for this specific issue.

**Warning signs:** `error[E0277]: the trait 'Default' is not implemented for ...` in spec.rs test code.

### Pitfall 5: Docstring Example Breakage

**What goes wrong:** `behavior.rs` has a rustdoc example showing `BehaviorResult::Stop("out of bounds".to_string())`. After the type change, this example will fail to compile during `cargo test --all` (rustdoc tests are run). The error appears as a test failure, not a compilation error in the main build.

**How to avoid:** Update the behavior.rs docstring example in Wave 1 alongside the type change. The new example should show `BehaviorResult::Stop(NonCommittedOutcome::Aborted("out of bounds".to_string()))` and also import `NonCommittedOutcome`.

**Warning signs:** `FAILED` in `cargo test --doc` output for behavior.rs examples.

### Pitfall 6: lib.rs Re-export Missing

**What goes wrong:** `NonCommittedOutcome` is added to `outcome.rs` but not re-exported via `lib.rs`. Users who try `use herdingcats::NonCommittedOutcome` get a compile error. The examples also need the import.

**How to avoid:** Add `pub use crate::outcome::NonCommittedOutcome;` to `lib.rs` in Wave 1.

**Warning signs:** `error[E0432]: unresolved import` in any file that tries to use `herdingcats::NonCommittedOutcome`.

---

## Code Examples

### NonCommittedOutcome Definition

```rust
// Place in src/outcome.rs, before the Outcome enum
/// The explicit non-committed outcome returned by a behavior's Stop.
///
/// Behaviors must choose the semantically correct variant:
/// - `InvalidInput` — the input doesn't make sense in the current context
/// - `Disallowed` — the input is meaningful but a rule forbids it
/// - `Aborted` — dispatch began but must be halted in a controlled abort
///
/// This type is **not** `#[non_exhaustive]` — its three variants are a complete
/// stable public contract.
#[derive(Debug, Clone, PartialEq)]
pub enum NonCommittedOutcome<N> {
    InvalidInput(N),
    Disallowed(N),
    Aborted(N),
}

impl<F, N> From<NonCommittedOutcome<N>> for Outcome<F, N> {
    fn from(nco: NonCommittedOutcome<N>) -> Self {
        match nco {
            NonCommittedOutcome::InvalidInput(n) => Outcome::InvalidInput(n),
            NonCommittedOutcome::Disallowed(n) => Outcome::Disallowed(n),
            NonCommittedOutcome::Aborted(n) => Outcome::Aborted(n),
        }
    }
}
```

### BehaviorResult After Wave 1

```rust
// src/behavior.rs
pub enum BehaviorResult<D, N> {
    Continue(Vec<D>),
    Stop(NonCommittedOutcome<N>),
}

pub trait Behavior<E: EngineSpec> {
    fn evaluate(
        &self,
        input: &E::Input,
        state: &E::State,
    ) -> BehaviorResult<E::Diff, E::NonCommittedInfo>;
}
```

### Updated Test StopBehavior

```rust
// src/engine.rs tests — update StopBehavior after Wave 1
use crate::outcome::NonCommittedOutcome;

struct StopBehavior;

impl Behavior<TestSpec> for StopBehavior {
    fn name(&self) -> &'static str { "stopper" }
    fn order_key(&self) -> u32 { 0 }
    fn evaluate(&self, _input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Stop(NonCommittedOutcome::Aborted("halted".to_string()))
    }
}
```

### Updated stop_halts_dispatch Test Assertion

After Wave 1, the test `stop_halts_dispatch` currently asserts `Outcome::Aborted(...)`. This assertion remains correct — the `From` impl maps `NonCommittedOutcome::Aborted` to `Outcome::Aborted`. No change needed to this assertion itself.

### EngineSpec After Wave 3

```rust
// src/spec.rs
pub trait EngineSpec: Sized {
    /// The game state type. Must support cloning (for CoW snapshots) and
    /// debug formatting. Default construction is NOT required by the engine —
    /// `Engine::new()` takes the initial state as a parameter.
    type State: Clone + std::fmt::Debug;
    // ... rest unchanged
}
```

### Updated spec.rs Test After Wave 3

```rust
// Replace the Default call
// Before:
let state = <TestSpec as EngineSpec>::State::default();

// After (explicit construction):
let state: <TestSpec as EngineSpec>::State = vec![];
```

### README.md Required Content

The README must describe (per CONTEXT.md locked decisions):
1. Input-driven dispatch
2. Static behavior set
3. CoW working state
4. Behaviors emit diffs, engine applies immediately
5. Traces generated during diff application
6. Frame `{input, diff, trace}`
7. Undo/redo

The 8 core terms from ARCHITECTURE.md: Input, State, Behavior, Diff, Trace, Frame, Outcome, Engine. Do not introduce new terminology.

---

## State of the Art

This phase aligns the implementation with an already-documented spec. No SOTA research required — the authority is ARCHITECTURE.md.

| Old Approach | Current Approach | Change |
|--------------|-----------------|--------|
| `BehaviorResult::Stop(N)` coerces to `Outcome::Aborted` | `BehaviorResult::Stop(NonCommittedOutcome<N>)` maps explicitly | Behaviors now signal intent |
| `Frame` carries `reversibility` field | `Frame` = `{input, diffs, traces}` per spec | Reversibility is dispatch metadata, not frame data |
| `EngineSpec::State: Default` | `EngineSpec::State: Clone + Debug` only | Engine never calls Default; callers provide initial state |
| Apply doc allows empty trace | Apply doc requires at least one trace for state-mutating diffs | Invariant 8 is now documented |

---

## Open Questions

1. **Redo stack tuple type after Frame shape change**
   - What we know: undo_stack changes from `Vec<(E::State, Frame<E>)>` to `Vec<(E::State, Frame<E>, Reversibility)>`
   - What's unclear: Does redo_stack also need Reversibility in the tuple? Redo doesn't consult Reversibility at redo-time (the irreversibility check happens at dispatch-time only).
   - Recommendation: Add Reversibility to both stack tuples for structural symmetry and future-proofing. CONTEXT.md says "move to history metadata" generically. This is a Claude's Discretion area.

2. **tictactoe.rs InvalidInput vs Disallowed for out-of-bounds**
   - What we know: ARCHITECTURE.md defines InvalidInput as "doesn't make sense in context", Disallowed as "meaningful but forbidden"
   - What's unclear: Is "row=3, col=3" an InvalidInput (bad data) or Disallowed (valid game, illegal move)?
   - Recommendation: Use `InvalidInput` — an out-of-bounds coordinate is structurally malformed input, not a rule violation. `Disallowed` is for valid-but-rejected moves.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + proptest 1.10 |
| Config file | none (Cargo.toml dev-dependencies) |
| Quick run command | `cargo test --all 2>&1 \| tail -20` |
| Full suite command | `cargo test --all` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| (Phase 5 has no new requirements) | All changes are corrections to existing behavior | — | `cargo test --all` | All test files exist |

All 17 v1 requirements are already marked complete. Phase 5 is a correctness alignment — tests from prior phases must continue passing after each wave.

**Key invariant tests that must remain green:**
- `stop_halts_dispatch` — will need Stop variant update but same assertion
- `frame_contains_input_diffs_trace` — must remove `frame.reversibility` assertion
- `frame_is_constructable_cloneable_and_eq` — must remove reversibility from Frame construction
- `engine_spec_associated_types_satisfy_bounds` — must replace `::default()` call
- All 15 invariant tests — must pass throughout

### Sampling Rate

- **Per wave commit:** `cargo test --all`
- **Per wave merge:** `cargo test --all && cargo clippy --all-targets -- -D warnings && cargo doc --no-deps`
- **Phase gate:** Full suite green + clippy clean + doc examples compile before verification

### Wave 0 Gaps

None — existing test infrastructure covers all phase requirements. No new test files are needed for this phase. Some existing tests will require updates as part of the wave work itself (documented in Common Pitfalls above).

---

## Sources

### Primary (HIGH confidence)

- Direct code reading of `src/outcome.rs`, `src/behavior.rs`, `src/engine.rs`, `src/spec.rs`, `src/apply.rs`, `src/lib.rs` — current implementation state
- `ARCHITECTURE.md` — authoritative spec document
- `.planning/phases/05-architecture-alignment/05-CONTEXT.md` — locked implementation decisions
- `examples/tictactoe.rs`, `examples/backgammon.rs` — example files to be updated

### Secondary (MEDIUM confidence)

- Rust reference on `From` trait — standard idiom for type conversion; HIGH confidence this is the right pattern
- Rust reference on `#[non_exhaustive]` — confirmed rationale for why NonCommittedOutcome should NOT use it

### Tertiary (LOW confidence)

None — all claims in this document are verified against the actual codebase or authoritative project documents.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero-dependency Rust library, no external choices to make
- Architecture: HIGH — all decisions locked in CONTEXT.md, implementation verified against actual source files
- Pitfalls: HIGH — all pitfalls identified from direct source code reading, not speculation

**Research date:** 2026-03-13
**Valid until:** This research is tied to the exact current codebase state. It is valid until any wave changes source files (at which point the "Current State Inventory" section is partially stale, but the patterns and pitfalls remain accurate).
