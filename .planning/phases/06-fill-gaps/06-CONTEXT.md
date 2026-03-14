# Phase 6: Fill Gaps - Context

**Gathered:** 2026-03-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Final cleanup pass on the `maddie-edits` branch to fully align with ARCHITECTURE.md. Replace trait-object-centered behavior storage with a plain-struct function-pointer approach. Update docs, examples, tests, ARCHITECTURE.md, and README to match. Add trace contract tests. No new features.

</domain>

<decisions>
## Implementation Decisions

### Behavior storage — BehaviorDef struct with fn pointers
- Remove the `Behavior` trait entirely
- Replace with `BehaviorDef<E: EngineSpec>` — a plain struct containing:
  - `name: &'static str`
  - `order_key: E::OrderKey`
  - `evaluate: fn(&E::Input, &E::State) -> BehaviorResult<E::Diff, E::NonCommittedInfo>`
- Engine stores `Vec<BehaviorDef<E>>`, sorted once at construction by `(order_key, name)`
- No mutation after construction — no push/pop/insert API
- No trait objects (`dyn`), no macro-generated tuple impls, no proc macros
- Do not add runtime behavior registration

### Ordering — keep (order_key, name) sorting
- `BehaviorDef` entries sorted by `(order_key, name)` at `Engine::new()`
- Order is explicit and deterministic regardless of construction order
- Matches existing ARCHITECTURE.md contract

### Trace contract tests
- Add at least one test: a mutating diff returns at least one trace entry
- Add at least one test: a true no-op diff may return zero trace entries
- These reinforce the public Apply contract tightened in Phase 5

### Behavior docs rewrite
- Remove all emphasis on dyn-safety and `Box<dyn Behavior<E>>`
- Emphasize: static compile-time behavior set, deterministic ordering, behaviors return diffs only, engine applies diffs centrally
- Update module-level and type-level rustdoc

### ARCHITECTURE.md update
- Sync with BehaviorDef approach (currently describes trait-based behaviors)
- Keep the 8 core terms consistent: Input, State, Behavior, Diff, Trace, Frame, Outcome, Engine

### README update
- Update the Phase 5 README to reflect BehaviorDef instead of trait-based Behavior
- Keep architecture description structure intact

### Claude's Discretion
- Exact field visibility on BehaviorDef (pub fields vs constructor)
- Whether to keep `behavior.rs` as a module or merge into another
- How to handle the compile-fail doctest that currently tests Behavior trait bounds
- Internal Engine field naming changes

</decisions>

<specifics>
## Specific Ideas

- Do not reintroduce: before/after hooks, runtime behavior registration, "effect" terminology, frame-embedded reversibility metadata
- Preserve all Phase 5 fixes: NonCommittedOutcome propagation, Frame {input, diffs, traces}, immediate sequential diff application
- "Do not do manual trait impls for tuple sizes — that path gets annoying fast and tends to make the crate feel like type-level gymnastics"
- User preference order: plain struct of fn pointers > sealed Vec wrapper > tuple impls

</specifics>

<code_context>
## Existing Code Insights

### Files That Change
- `src/behavior.rs` — Remove `Behavior` trait, replace with `BehaviorDef<E>` struct
- `src/engine.rs` — Change `Vec<Box<dyn Behavior<E>>>` to `Vec<BehaviorDef<E>>`, update dispatch loop, update all tests
- `src/lib.rs` — Update re-exports (`BehaviorDef` instead of `Behavior`)
- `src/apply.rs` — Add trace contract tests
- `examples/tictactoe.rs` — Replace `impl Behavior` blocks with `BehaviorDef` construction
- `examples/backgammon.rs` — Same
- `ARCHITECTURE.md` — Update behavior section to describe BehaviorDef
- `README.md` — Update behavior description and examples

### Established Patterns
- Flat crate root re-exports via `pub use` in `lib.rs`
- Manual `Clone`/`PartialEq` impls on `Frame<E>` (derive bound workaround)
- Zero runtime dependencies
- Full rustdoc on all public types

### Integration Points
- `BehaviorDef.evaluate` fn pointer replaces `Behavior::evaluate()` trait method in dispatch loop
- `BehaviorDef.name` and `BehaviorDef.order_key` replace `Behavior::name()` and `Behavior::order_key()` trait methods
- Sorting in `Engine::new()` changes from `sort_by` on trait methods to `sort_by` on struct fields

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 06-fill-gaps*
*Context gathered: 2026-03-13*
