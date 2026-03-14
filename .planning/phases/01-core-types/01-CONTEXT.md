# Phase 1: Core Types - Context

**Gathered:** 2026-03-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Define all public type contracts, traits, and enums that the entire library depends on. Deliverables: `EngineSpec` trait, `Behavior` trait, `BehaviorResult`, `Outcome`, `Frame`, and `EngineError`. Dispatch logic, history, and examples are separate phases. This phase compiles cleanly; later phases build on this stable API surface.

</domain>

<decisions>
## Implementation Decisions

### EngineSpec design
- Trait with associated types: `State`, `Input`, `Diff`, `Trace`, `NonCommittedInfo`, `OrderKey: Ord`
- Users define a unit struct and `impl EngineSpec for MySpec { type State = ...; ... }`
- Non-committed payload associated type is named `NonCommittedInfo`

### Behavior trait
- `Behavior<E: EngineSpec>` — one type param, tied to a spec
- Signature: `fn name(&self) -> &'static str`, `fn order_key(&self) -> E::OrderKey`, `fn evaluate(&self, input: &E::Input, state: &E::State) -> BehaviorResult<E::Diff, E::NonCommittedInfo>`
- User implements: `impl Behavior<MySpec> for MyBehavior { ... }`

### Frame
- Library-owned type: `pub struct Frame<E: EngineSpec> { pub input: E::Input, pub diff: E::Diff, pub trace: E::Trace }`
- Derives: `Debug, Clone, PartialEq`
- `Outcome<F, N>` in practice is `Outcome<Frame<E>, E::NonCommittedInfo>`

### Outcome
- Generic `Outcome<F, N>` with 7 variants: `Committed(F)`, `Undone(F)`, `Redone(F)`, `NoChange`, `InvalidInput(N)`, `Disallowed(N)`, `Aborted(N)`
- Derives: `Debug, Clone, PartialEq`

### EngineError
- Structured variants for MVP: `BehaviorPanic`, `InvalidState(String)`, `CorruptHistory`
- `#[non_exhaustive]`
- Derives: `Debug, Clone, PartialEq`
- No `std::error::Error` impl in this phase

### Module layout
- Flat crate root: all types re-exported at `herdingcats::*` — users write `use herdingcats::{EngineSpec, Behavior, ...}`
- Internal file grouping:
  - `src/spec.rs` — `EngineSpec` trait
  - `src/behavior.rs` — `Behavior` trait + `BehaviorResult`
  - `src/outcome.rs` — `Outcome`, `Frame`, `EngineError`
  - `src/lib.rs` — `pub use` re-exports only
- Full rustdoc comments on every public type, trait, and method

### Type bounds on EngineSpec associated types
- `State: Clone + Debug + Default` (Default needed for initial engine construction)
- `Input: Clone + Debug`
- `Diff: Clone + Debug`
- `Trace: Clone + Debug`
- `NonCommittedInfo: Clone + Debug`
- `OrderKey: Ord`

### Claude's Discretion
- Exact rustdoc wording and link structure
- Whether to add `#[must_use]` to `Outcome` and `Result<Outcome, EngineError>`
- Whether `BehaviorResult` gets `Debug + Clone + PartialEq` derives (reasonable yes)

</decisions>

<specifics>
## Specific Ideas

- State needs `Default` for engine construction — user noted this during the bounds discussion
- All core types should support `assert_eq!` in tests without wrestling with moves — drives `Clone + PartialEq` on `Outcome`, `Frame`, and `EngineError`

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — `src/lib.rs` is empty. Fresh start on `maddie-edits` branch.

### Established Patterns
- Zero runtime dependencies (constraint from PROJECT.md) — no external crates, only `std`
- Rust edition 2024

### Integration Points
- `src/lib.rs` becomes the re-export hub for Phase 1 types
- Phase 2 (`dispatch.rs`, `engine.rs`) will `use crate::spec::EngineSpec` and build `WorkingState<E>` on top of these types
- Phase 3 (`history.rs`) will use `Frame<E>` and `Outcome` directly

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-core-types*
*Context gathered: 2026-03-13*
