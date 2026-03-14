# Phase 2: Dispatch - Research

**Researched:** 2026-03-13
**Domain:** Rust `std::borrow::Cow`, dispatch loop design, atomic commit semantics
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Apply<E> trait**: `pub trait Apply<E: EngineSpec> { fn apply(&self, state: &mut E::State) -> Vec<E::Trace>; }` — single call, no forgetting trace. Lives in `src/apply.rs`. Bound added to `EngineSpec::Diff`: `type Diff: Clone + std::fmt::Debug + Apply<Self>;`
- **Engine construction**: `Engine<E>::new(state: E::State, behaviors: Vec<Box<dyn Behavior<E>>>) -> Self`. Behaviors sorted by `(order_key, name)` at construction, never again. `pub fn state(&self) -> &E::State` read-only getter. Lives in `src/engine.rs`.
- **Reversibility enum**: `pub enum Reversibility { Reversible, Irreversible }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Lives in `src/reversibility.rs`.
- **CoW working state**: Use `std::borrow::Cow<'a, E::State>` directly inside `dispatch()` — NO custom WorkingState type. `Cow::Borrowed(&self.state)` at dispatch start; `working.to_mut()` on first diff application clones once. Cow is private to `dispatch()`.
- **CoW granularity**: Whole-state only for Phase 2 — substate-granular CoW is a v2 optimization.
- **CoW verification**: Pointer equality — dispatch a no-op input, check `engine.state() as *const _` is same address before and after.
- **Module additions**: `src/apply.rs`, `src/reversibility.rs`, `src/engine.rs`. `src/lib.rs` updated with three new `pub use` re-exports and `mod` declarations.
- **Phase 3 forward-compatibility**: Leave room in `Engine<E>` struct for `undo_stack` and `redo_stack` fields (even as empty `Vec<_>` placeholders).

### Claude's Discretion

- Whether `Engine<E>` derives `Debug` (E::State: Debug already bound — yes is reasonable)
- Exact trace accumulation: `Vec<E::Trace>` collected across all diffs, stored in `Frame.trace`
- Exact error conditions for `EngineError::InvalidState` in dispatch (engine invariant violations only)

### Deferred Ideas (OUT OF SCOPE)

- Substate-granular CoW — fine-grained CoW for AI look-ahead is a v2 performance optimization
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DISP-01 | `WorkingState<S>` provides CoW semantics — reads from committed state until first write, clones only on first write | `std::borrow::Cow<'a, E::State>` with `to_mut()` is the exact mechanism; locked as implementation approach |
| DISP-02 | `dispatch(input, reversibility)` evaluates behaviors in deterministic `(order_key, name)` order, applies diffs immediately, appends trace at moment of diff application, commits `Frame` atomically if non-empty | Dispatch loop pattern documented; Apply<E> trait enforces trace coupling |
| DISP-03 | `Frame<I, D, T>` stores `input`, `diff` collection, and `trace` as canonical committed record | Frame<E> already exists in outcome.rs with `input: E::Input`, `diff: E::Diff`, `trace: E::Trace`; note: diff field is `E::Diff` (single), not Vec — planner must decide collection type |
| DISP-04 | `dispatch()` takes an explicit `Reversibility` parameter — callers cannot omit the declaration | Achieved by making `reversibility: Reversibility` a required positional parameter in the function signature |
</phase_requirements>

---

## Summary

Phase 2 implements the core dispatch pipeline of the HerdingCats engine. The key technical pieces are: `Apply<E>` trait (diff application + trace generation in one call), `Reversibility` enum (explicit parameter encoding), `Engine<E>` struct with sorted behaviors, and a `dispatch()` method using `std::borrow::Cow<'a, E::State>` for zero-cost CoW semantics.

All decisions are locked by CONTEXT.md. The implementation is a pure Rust standard-library exercise — no external dependencies, no novel algorithms. The entire complexity lives in the lifetime handling of `Cow<'a, E::State>` inside `dispatch()` and correctly threading the mutable borrow through the behavior loop.

One subtle design tension: `Frame<E>` in `outcome.rs` has `diff: E::Diff` (a single diff value), but dispatch accumulates `Vec<E::Diff>`. The caller's `E::Diff` type must itself be a collection (e.g. `Vec<StateDiff>`) or the `Apply<E>` call needs to aggregate. The `Apply<E>` trait operates on `&self` where `self: E::Diff` — so the user's Diff type IS the collection, applied in one shot.

**Primary recommendation:** Implement in strict dependency order: `apply.rs` → `reversibility.rs` → `engine.rs` (with `spec.rs` updated last to add the Apply bound). Keep `dispatch()` internal Cow lifetime scoped to the function — do not expose it in any public signature.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::borrow::Cow` | stdlib | Clone-on-write smart pointer | Zero-dependency; exact semantics needed; blanket ToOwned impl for Clone types |
| `std::ptr::eq` | stdlib | Pointer equality for CoW test | Stable, idiomatic way to compare addresses without unsafe code |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None | — | — | Zero-dependency library; no additions |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `std::borrow::Cow` | Custom WorkingState enum | Custom type is more explicit but is extra code with identical semantics; locked against this |
| Whole-state CoW | Substate-granular CoW | Fine-grained is faster for AI look-ahead but adds design complexity; deferred to v2 |

**Installation:**
```bash
# No additional dependencies — zero-dependency crate
```

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── spec.rs          # EngineSpec trait (MODIFIED: Add Apply<Self> bound to Diff)
├── behavior.rs      # Behavior<E> trait (UNCHANGED)
├── outcome.rs       # Outcome<F,N>, Frame<E>, EngineError (UNCHANGED)
├── apply.rs         # Apply<E> trait (NEW)
├── reversibility.rs # Reversibility enum (NEW)
├── engine.rs        # Engine<E> struct + dispatch() (NEW)
└── lib.rs           # pub use re-exports (MODIFIED: 3 new entries + 3 new mods)
```

### Pattern 1: Apply<E> Trait — Combined Apply + Trace

**What:** A single trait method that mutates the state AND returns traces in one call. This structurally prevents the "forgot to emit trace" bug.

**When to use:** Implementing this trait on the user's `E::Diff` type.

```rust
// src/apply.rs
use crate::spec::EngineSpec;

/// Applies this diff to the working state and returns trace entries generated
/// by this mutation.
///
/// Application and trace generation are combined in one call: this structurally
/// prevents a diff from being applied without emitting its corresponding trace.
pub trait Apply<E: EngineSpec> {
    fn apply(&self, state: &mut E::State) -> Vec<E::Trace>;
}
```

**Key: The `EngineSpec::Diff` bound** — `src/spec.rs` line 36 must become:
```rust
type Diff: Clone + std::fmt::Debug + Apply<Self>;
```
This requires importing `crate::apply::Apply` in `spec.rs`.

### Pattern 2: Cow Inside dispatch() — No Exposed Lifetime

**What:** `std::borrow::Cow<'a, E::State>` used as a local variable inside `dispatch()`. The lifetime `'a` is bound to `&self` — it never appears in the return type.

**When to use:** Standard dispatch, every call.

```rust
// src/engine.rs — dispatch() sketch
use std::borrow::Cow;

pub fn dispatch(
    &mut self,
    input: E::Input,
    reversibility: Reversibility,
) -> Result<Outcome<Frame<E>, E::NonCommittedInfo>, EngineError> {
    // Cow::Borrowed — zero cost, no clone yet
    let mut working: Cow<'_, E::State> = Cow::Borrowed(&self.state);
    let mut diffs: Vec<E::Diff> = Vec::new();
    let mut traces: Vec<E::Trace> = Vec::new();

    for behavior in &self.behaviors {
        match behavior.evaluate(&input, &*working) {
            BehaviorResult::Stop(info) => {
                return Ok(Outcome::Aborted(info));
            }
            BehaviorResult::Continue(new_diffs) => {
                for diff in new_diffs {
                    // to_mut() clones state on FIRST call only
                    let new_traces = diff.apply(working.to_mut());
                    traces.extend(new_traces);
                    diffs.push(diff);
                }
            }
        }
    }

    if diffs.is_empty() {
        return Ok(Outcome::NoChange);
    }

    // Commit atomically — consume Cow, update self.state
    self.state = working.into_owned();
    let frame = Frame { input, diff: /* aggregate */, trace: /* aggregate */ };
    Ok(Outcome::Committed(frame))
}
```

**Note on Frame.diff:** `Frame<E>` has `diff: E::Diff` (single value, not Vec). The user's `E::Diff` type must itself represent a collection (e.g. `Vec<StateDiff>`) if multiple diffs are emitted. The dispatch loop collects `Vec<E::Diff>` but can only store one `E::Diff` in Frame. Two options exist:

1. **Collect single aggregate diff**: Require that the user's `E::Diff` wraps a collection internally (the ARCHITECTURE.md describes `DiffCollection` as "a flat `Vec<StateDiff>`" or "a nested patch object" — the user's Diff IS the collection)
2. **Vec<E::Diff> in Frame**: Change `Frame.diff` to `Vec<E::Diff>`

Looking at `outcome.rs` line 26: `pub diff: E::Diff` — it's a single. The architecture description says "the exact collection type is user-defined" — meaning the user's `E::Diff` type already IS a collection. The dispatch loop's internal `Vec<E::Diff>` must be converted to a single `E::Diff` at commit time, OR the planner chooses `Frame.diff` to be `Vec<E::Diff>`.

**Recommendation (Claude's discretion):** Change `Frame.diff` to `Vec<E::Diff>` in outcome.rs as part of Phase 2. This is simpler and more flexible — no "wrap a Vec in your Diff" burden on users. The Phase 1 Frame used `E::Diff` as a single value but Phase 2 is the first time dispatch actually runs and produces multiple diffs. This is a natural correction.

### Pattern 3: Behavior Sort at Construction

**What:** Sort `Vec<Box<dyn Behavior<E>>>` once at construction time by `(order_key, name)`.

**When to use:** `Engine::new()` — after sort, behaviors are never reordered.

```rust
// src/engine.rs — Engine::new()
pub fn new(state: E::State, mut behaviors: Vec<Box<dyn Behavior<E>>>) -> Self {
    behaviors.sort_by(|a, b| {
        a.order_key().cmp(&b.order_key())
            .then_with(|| a.name().cmp(b.name()))
    });
    Engine {
        state,
        behaviors,
        // Phase 3 placeholders:
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
    }
}
```

**Key:** `sort_by` is stable in Rust's stdlib. For deterministic ordering, stability doesn't matter here because `(order_key, name)` is already a total order (assuming unique names — which is a user contract, not enforced by the engine).

### Pattern 4: Reversibility — Explicit Positional Parameter

**What:** `Reversibility` is a required positional parameter. Callers literally cannot omit it — there's no default, no `Option`, no overloading.

```rust
// src/reversibility.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    /// This transition can be undone.
    Reversible,
    /// This transition is irreversible; committing it clears undo/redo history.
    Irreversible,
}
```

**Usage in dispatch:** `dispatch()` receives `reversibility: Reversibility`. Phase 2 records it but doesn't act on it (that's Phase 3's job). The value should be stored in `Frame<E>` or passed to history recording. Actually — Phase 3 needs it to decide whether to clear stacks. Phase 2 can store it alongside the frame or simply accept it to satisfy DISP-04 and leave Phase 3 to use it from the `Frame`.

**Recommendation:** Add `reversibility: Reversibility` to `Frame<E>` in Phase 2 so Phase 3 can inspect it from history. This means modifying `outcome.rs` Frame struct.

### Pattern 5: CoW Pointer Equality Test

**What:** Verify that no clone happens during a no-op dispatch by comparing raw pointer addresses.

```rust
#[test]
fn cow_no_clone_on_no_op_dispatch() {
    // Setup: engine with a behavior that emits no diffs for input 0
    let engine = /* ... */;
    let ptr_before = engine.state() as *const _;
    let _outcome = engine.dispatch(/* no-op input */, Reversibility::Reversible);
    let ptr_after = engine.state() as *const _;
    assert_eq!(ptr_before, ptr_after, "state must not be cloned on no-op dispatch");
}
```

**How it works:** `engine.state()` returns `&E::State`. Casting to `*const _` gives the raw address. If the state was cloned (new allocation), the pointer would differ. If no clone happened (Cow stayed Borrowed), the pointer is the same because `self.state` was never replaced.

**Important caveat:** This test works only because the no-op path keeps `Cow::Borrowed(&self.state)` and then discards it (no `into_owned()` call). The `self.state` field itself was never touched. After a successful dispatch, `self.state` is replaced with `working.into_owned()` — so the pointer will differ even for a single-diff dispatch. Test only no-op inputs.

### Anti-Patterns to Avoid

- **Cloning state at dispatch start**: `let working = self.state.clone()` is the v0.4.0 bug. Never do this.
- **Exposing Cow in public API**: `Cow<'a, E::State>` must not appear in any public function signature. It's an implementation detail of `dispatch()`.
- **Behaviors mutating via Cow**: Behaviors receive `&*working` (a shared borrow) — not `&mut`. The structural type system prevents mutation. Enforce this at the call site.
- **Sorting behaviors after construction**: Sort only once. Mutable access to behaviors list post-construction should not exist.
- **Applying diffs after the loop**: Diffs must be applied immediately during the loop, not collected and applied afterward. Later behaviors must see earlier changes.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Clone-on-write semantics | Custom enum with `Borrowed`/`Owned` variants | `std::borrow::Cow` | Already exists, battle-tested, stdlib zero-cost |
| Stable sort of behaviors | Custom stable-sort implementation | `Vec::sort_by` | stdlib sort is stable, O(n log n) |
| Pointer equality test | Custom clone counter / instrumentation | `as *const _` cast + `assert_eq!` | One-liner, no test infrastructure needed |

**Key insight:** `std::borrow::Cow` was designed precisely for this use case. The blanket `impl<T: Clone> ToOwned for T` means any `E::State: Clone` works automatically.

---

## Common Pitfalls

### Pitfall 1: Cow Lifetime Ties to &self Borrow

**What goes wrong:** `Cow<'a, E::State>` with `'a` tied to `&self` means `self` is mutably reborrowed (for `working.to_mut()`) while the lifetime `'a` of the Cow still references `&self.state`. Rust's borrow checker will reject this if both borrows are live simultaneously.

**Why it happens:** `Cow::Borrowed(&self.state)` borrows `self.state`. `working.to_mut()` needs `&mut self.behaviors[i]` (indirectly) — but Rust sees `self` as immutably borrowed through the Cow.

**How to avoid:** The Cow references `self.state`. During the behavior loop, we iterate `&self.behaviors` (immutable). We never mutably access `self.state` or `self.behaviors` during the loop — only `working`. This is fine because the Cow borrows `self.state` but the loop iterates `self.behaviors`. The key is that `self.state` and `self.behaviors` are separate fields. Rust field-level borrow splitting allows this.

**Warning signs:** Compiler error "cannot borrow `self` as mutable because it is also borrowed as immutable". Resolution: ensure the Cow borrow and the behaviors iteration reference separate fields.

### Pitfall 2: Modifying self.state Before dispatch Fully Completes

**What goes wrong:** Partially applying changes to `self.state` then hitting an early-return path (Stop, error), leaving committed state in a partially mutated form.

**Why it happens:** Attempting to "optimize" by writing to `self.state` mid-loop.

**How to avoid:** Never touch `self.state` until the very end — `self.state = working.into_owned()` is the ONLY assignment. All mutation goes through `working.to_mut()`. The atomic commit guarantee comes from this discipline.

### Pitfall 3: Frame.diff Type Mismatch

**What goes wrong:** `Frame<E>` has `diff: E::Diff` (single value), but dispatch accumulates `Vec<E::Diff>`. Trying to stuff a `Vec<E::Diff>` into `E::Diff` without updating the Frame definition.

**Why it happens:** Phase 1 defined Frame with a single `diff: E::Diff` field. Phase 2 is the first time multiple diffs are actually accumulated.

**How to avoid:** Update `Frame<E>` in `outcome.rs` to use `Vec<E::Diff>` (and `Vec<E::Trace>` for consistency if trace accumulation mirrors diffs). This is a breaking change from the Phase 1 type definition but Phase 2 is the correct place to make it since dispatch is being implemented here.

### Pitfall 4: Apply Bound Creates Circular Import

**What goes wrong:** `spec.rs` imports `Apply<Self>` from `apply.rs`, but `apply.rs` imports `EngineSpec` from `spec.rs`. Circular module dependency.

**Why it happens:** `type Diff: Clone + Debug + Apply<Self>` requires `Apply` in scope inside `spec.rs`.

**How to avoid:** Rust handles this fine with `use crate::apply::Apply;` in `spec.rs`. Module circular imports (not crate) are allowed in Rust — both `spec.rs` and `apply.rs` can reference each other as long as neither defines a type that structurally contains the other. The bound is just a trait reference, not a structural containment. This compiles correctly.

### Pitfall 5: Behaviors Evaluated Against Stale State

**What goes wrong:** Behavior N evaluates against the state before behavior N-1's diffs were applied.

**Why it happens:** Passing `&self.state` (committed) instead of `&*working` to each behavior's `evaluate()` call.

**How to avoid:** All `evaluate()` calls use `&*working` (Deref on Cow gives `&E::State` pointing to either the borrowed or owned working copy). Never pass `&self.state` inside the loop.

---

## Code Examples

Verified patterns from official sources:

### std::borrow::Cow with T: Clone
```rust
// Source: https://doc.rust-lang.org/std/borrow/enum.Cow.html
// Blanket impl: impl<T: Clone> ToOwned for T { type Owned = T; }
// This means Cow<'a, MyState> works for any MyState: Clone
use std::borrow::Cow;

let state = MyState::default();
let mut working: Cow<'_, MyState> = Cow::Borrowed(&state);

// First write — clones state exactly once
working.to_mut().apply_something();

// Subsequent writes — no additional clones
working.to_mut().apply_something_else();

// Consume and commit
let new_state: MyState = working.into_owned();
```

### sort_by with tuple key
```rust
// Source: std::slice sort documentation
// Stable sort on (order_key, name) — deterministic total order
behaviors.sort_by(|a, b| {
    a.order_key().cmp(&b.order_key())
        .then_with(|| a.name().cmp(b.name()))
});
```

### Pointer equality for CoW verification
```rust
// Source: https://doc.rust-lang.org/std/ptr/fn.eq.html
// Cast &T to *const T to compare addresses, not values
let ptr_before: *const E::State = engine.state() as *const _;
engine.dispatch(no_op_input, Reversibility::Reversible).unwrap();
let ptr_after: *const E::State = engine.state() as *const _;
assert!(std::ptr::eq(ptr_before, ptr_after));
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Eager `state.clone()` in dispatch (v0.4.0) | `Cow::Borrowed` + lazy `to_mut()` | v0.5.0 | No clone for read-heavy AI look-ahead |
| Memory address tiebreaker for behavior order (v0.4.0) | `(order_key, name)` sort | v0.5.0 | Fully deterministic across runs |

---

## Open Questions

1. **Frame.diff: Vec<E::Diff> vs single E::Diff**
   - What we know: `outcome.rs` currently has `diff: E::Diff` (single). Dispatch accumulates `Vec<E::Diff>`. The ARCHITECTURE.md says the diff collection type is "user-defined."
   - What's unclear: Should `Frame.diff` become `Vec<E::Diff>`, or should the `Apply<E>` trait consume a full Vec (making E::Diff itself the collection)?
   - Recommendation: Change `Frame.diff` to `Vec<E::Diff>` in Phase 2. This is the natural fit for how dispatch works and makes the user's Diff type a simple atomic mutation rather than a collection wrapper.

2. **Frame.trace: Vec<E::Trace> vs single E::Trace**
   - What we know: `outcome.rs` currently has `trace: E::Trace` (single). Apply<E> returns `Vec<E::Trace>`.
   - Recommendation: Change `Frame.trace` to `Vec<E::Trace>` in Phase 2 for the same reasons.

3. **Frame.reversibility field**
   - What we know: Phase 3 needs to know whether a committed frame was reversible to decide whether to clear history.
   - Recommendation: Add `reversibility: Reversibility` to `Frame<E>` in Phase 2. This is cleaner than threading it through a separate structure.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — `#[cfg(test)]` modules in each source file, consistent with Phase 1 |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --doc` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DISP-01 | CoW: no clone until first diff applied | unit | `cargo test cow_no_clone_on_no_op_dispatch` | ❌ Wave 0 (in `src/engine.rs` tests) |
| DISP-01 | CoW: clone happens on first diff, not before | unit | `cargo test cow_clones_on_first_diff` | ❌ Wave 0 |
| DISP-02 | Behaviors evaluated in deterministic (order_key, name) order | unit | `cargo test dispatch_evaluates_in_deterministic_order` | ❌ Wave 0 |
| DISP-02 | Later behaviors see earlier diffs applied | unit | `cargo test later_behavior_sees_earlier_diffs` | ❌ Wave 0 |
| DISP-02 | Diffs + trace accumulated at moment of application | unit | `cargo test trace_appended_at_diff_application` | ❌ Wave 0 |
| DISP-02 | Stop variant halts evaluation immediately | unit | `cargo test stop_halts_dispatch` | ❌ Wave 0 |
| DISP-03 | Frame committed only when diffs non-empty | unit | `cargo test no_frame_on_no_diffs` | ❌ Wave 0 |
| DISP-03 | Frame contains input, diffs, trace | unit | `cargo test frame_contains_input_diffs_trace` | ❌ Wave 0 |
| DISP-04 | dispatch() signature requires Reversibility param | compile-check | `cargo test` (compile error if missing) | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo test --doc`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/apply.rs` — test: `apply_trait_compiles_and_returns_traces`
- [ ] `src/reversibility.rs` — test: `reversibility_is_copy_and_eq`
- [ ] `src/engine.rs` — tests: all DISP-01 through DISP-04 tests listed above
- [ ] Update `src/spec.rs` — verify Apply<Self> bound compiles with existing TestSpec (requires TestSpec's Diff to implement Apply)
- [ ] Update `src/outcome.rs` — if Frame.diff/trace changed to Vec, update existing tests

*(No new test files needed — all tests live in `#[cfg(test)]` modules per established project pattern)*

---

## Sources

### Primary (HIGH confidence)
- `https://doc.rust-lang.org/std/borrow/enum.Cow.html` — Cow type, to_mut() behavior, Borrowed/Owned variants
- `https://doc.rust-lang.org/std/borrow/trait.ToOwned.html` — Blanket impl<T: Clone> ToOwned for T confirmed
- `https://doc.rust-lang.org/std/ptr/fn.eq.html` — std::ptr::eq for pointer address comparison
- Existing `src/spec.rs`, `src/behavior.rs`, `src/outcome.rs`, `src/lib.rs` — direct read of Phase 1 output
- `ARCHITECTURE.md` — authoritative design document, read in full

### Secondary (MEDIUM confidence)
- `CONTEXT.md` — all locked decisions verified against ARCHITECTURE.md

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — stdlib only, verified against official docs
- Architecture: HIGH — locked decisions from CONTEXT.md + verified Cow semantics
- Pitfalls: HIGH — borrow checker behavior verified by understanding Rust ownership rules; Cow pattern well-established

**Research date:** 2026-03-13
**Valid until:** Stable for the Rust edition (2024) in use — no expiry concern; stdlib Cow API is stable and unchanged since Rust 1.0.
