# Phase 2: Dispatch - Context

**Gathered:** 2026-03-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement `Engine<E>` struct with `dispatch(input, reversibility)` — callers can submit inputs and receive deterministic, atomically committed `Outcome` results with correct CoW semantics. This includes the `Apply<E>` trait, `Reversibility` enum, `WorkingState` (using `std::borrow::Cow`), and the sorted behavior evaluation loop. History (undo/redo stacks) is Phase 3.

</domain>

<decisions>
## Implementation Decisions

### Apply<E> trait — diff application + trace generation
- Single combined trait: `pub trait Apply<E: EngineSpec> { fn apply(&self, state: &mut E::State) -> Vec<E::Trace>; }`
- Application and trace generation happen in one call — structurally prevents forgetting to emit trace
- Lives in `src/apply.rs`, re-exported at crate root: `pub use crate::apply::Apply`
- Added as a bound on `EngineSpec::Diff`: `type Diff: Clone + std::fmt::Debug + Apply<Self>;`
- This requires updating `src/spec.rs` to add the `Apply<Self>` bound to the `Diff` associated type

### Engine construction
- `Engine<E>::new(state: E::State, behaviors: Vec<Box<dyn Behavior<E>>>) -> Self`
- Behaviors are sorted by `(order_key, name)` once at construction time, never again
- `pub fn state(&self) -> &E::State` — read-only getter for current committed state (needed for rendering)
- `Engine<E>` lives in `src/engine.rs` alongside dispatch logic
- `Engine<E>` re-exported at crate root: `pub use crate::engine::Engine`

### Reversibility enum
- `pub enum Reversibility { Reversible, Irreversible }`
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — `Copy` is appropriate since it has no payload
- Lives in `src/reversibility.rs`, re-exported at crate root: `pub use crate::reversibility::Reversibility`

### CoW working state — use std::borrow::Cow
- Use `std::borrow::Cow<'a, E::State>` directly inside `dispatch()` — no custom `WorkingState` type
- `E::State: Clone` (already required by `EngineSpec`) → automatically `ToOwned<Owned = E::State>` → `Cow` works
- `Cow::Borrowed(&self.state)` at dispatch start; `working.to_mut()` on first diff application clones once
- `Cow` is a private implementation detail inside `dispatch()` — not a public type
- CoW granularity for Phase 2: whole-state (substate-granular CoW is a v2 optimization)

### CoW verification approach
- Verify "no clone until first diff" via pointer equality: dispatch a no-op input (no diffs produced), check `engine.state() as *const _` is the same address before and after
- No clone-counter instrumentation needed — pointer comparison is sufficient and clean

### Module additions for Phase 2
New files added to the established structure:
- `src/apply.rs` — `Apply<E>` trait
- `src/reversibility.rs` — `Reversibility` enum
- `src/engine.rs` — `Engine<E>` struct, `new()`, `state()`, `dispatch()`
- `src/lib.rs` updated to add the three new `pub use` re-exports and `mod` declarations

### Claude's Discretion
- Whether `Engine<E>` derives `Debug` (may require `E::State: Debug`, which is already bound — reasonable yes)
- Exact trace accumulation: `Vec<E::Trace>` collected across all diffs during dispatch, stored in `Frame.trace` (user-defined `E::Trace` is already a single type; trace collection is `Vec<E::Trace>`)
- Exact error conditions for `EngineError::InvalidState` in dispatch (engine invariant violations only)

</decisions>

<specifics>
## Specific Ideas

- "Can we use the built-in Cow type instead of implementing our own?" — yes, `std::borrow::Cow<'a, E::State>` is the right approach. Standard library doing the work.
- `Cow` stays internal to `dispatch()` — not a public API type

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/spec.rs`: `EngineSpec` trait — Phase 2 will add the `Apply<Self>` bound to `type Diff`
- `src/behavior.rs`: `Behavior<E>`, `BehaviorResult<D, O>` — `evaluate()` return type feeds directly into dispatch loop
- `src/outcome.rs`: `Outcome<F, N>`, `Frame<E>`, `EngineError` — dispatch returns `Result<Outcome<Frame<E>, E::NonCommittedInfo>, EngineError>`

### Established Patterns
- Flat crate root re-exports: `src/lib.rs` re-exports everything via `pub use crate::module::Type`
- Private `mod` declarations in `lib.rs` — Phase 2 adds `mod apply`, `mod reversibility`, `mod engine`
- Full rustdoc on all public types (set in Phase 1)
- Zero runtime dependencies

### Integration Points
- `src/spec.rs` is the one Phase 1 file that changes: `type Diff` gets the `Apply<Self>` bound added
- `src/engine.rs` depends on all of: `spec.rs`, `behavior.rs`, `outcome.rs`, `apply.rs`, `reversibility.rs`
- Phase 3 (`history.rs`) will depend on `Engine<E>` having a history store — Phase 2 should leave room in the `Engine` struct for `undo_stack` and `redo_stack` fields (even if they're `Vec<_>` placeholders that Phase 3 fills in)

</code_context>

<deferred>
## Deferred Ideas

- Substate-granular CoW — Phase 2 MVP uses whole-state `std::borrow::Cow`. Fine-grained CoW for AI look-ahead is a v2 performance optimization.

</deferred>

---

*Phase: 02-dispatch*
*Context gathered: 2026-03-13*
