# Phase 7: Documentation and Extended Tests - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Write comprehensive rustdoc with Mealy/Moore state machine framing for all v1.1 public types and new trait methods; add runnable doctests for `is_reversible`, `is_active`, `on_dispatch`, `on_undo`; expand the crate-level `lib.rs` overview; add edge-case unit tests for reversibility (TEST-07) and behavior lifecycle (TEST-08). No new API features.

</domain>

<decisions>
## Implementation Decisions

### Mealy/Moore framing depth and placement
- Use **proper state machine vocabulary**: events are inputs, mutations are outputs, behaviors are the transition function, engine is the automaton. Not a textbook definition, but explicit enough that a reader familiar with automata theory recognizes the model.
- Framing lives in **`lib.rs` crate-level doc only** — the `//!` block. Type-level docs in mutation.rs, behavior.rs, engine.rs do not need to be rewritten with state machine vocabulary (they already have good prose from prior phases). Module-level prose stays focused on the type's role, not the theory.
- The crate-level doc gets: (1) model description with state machine framing, (2) four concepts (Engine, Behavior, Mutation, Action) with one-line roles, (3) Mealy machine data flow as an **ASCII diagram**, (4) a quick-start runnable code snippet.

### Crate-level lib.rs doc structure
- **Full overview**: 3-4 paragraphs, not just the current single line
- **ASCII diagram**: the dispatch pipeline — Event (I) → [Behavior.before] → [Mutation.apply] → [Behavior.after] → State' — with undo/redo shown as the reverse path
- **Quick-start snippet**: self-contained runnable doctest (20-30 lines) — define a State type, a Mutation impl, a Behavior impl, create an Engine, dispatch an event, assert state changed. Becomes a `cargo test --doc` test.
- The current tagline "herdingcats — a deterministic, undoable game-event engine" is preserved or refined, not discarded.

### Doctests for new v1.1 methods
- **Self-contained per method**: each doctest defines its own minimal types inline. Matches existing style (apply, undo, hash_bytes doctests are self-contained). Readers can copy-paste any single method's example.
- **Show override + engine effect**: not just the return value — show the behavior working in an Engine context:
  - `is_reversible`: define a DiceOp returning `false`, dispatch it, assert `engine.can_undo()` is `false`
  - `is_active`: define a behavior returning `false`, dispatch an event, assert behavior's before() hook had no effect (action went through)
  - `on_dispatch`: use `Rc<Cell<u32>>` shared counter, dispatch twice, assert counter == 2
  - `on_undo`: use `Rc<Cell<u32>>` counter incremented in on_dispatch, decremented in on_undo, dispatch then undo, assert counter == 0
- **`Rc<Cell<T>>` pattern** for observing behavior-internal state from test scope — established in Phase 6, consistent with TEST-04 pattern

### TEST-07: Reversibility edge cases
The following are MISSING and need new unit tests (already covered: all-irreversible clears undo stack; is_active skips hooks):
- **Mixed mutations → treated as irreversible**: Action with one `Rev` mutation + one `Irrev` mutation; assert (a) `can_undo()` is false after commit, (b) a reversible Action dispatched BEFORE the mixed one cannot be undone (undo stack cleared). Tests both the flag AND the consequence.
- **Empty Action is reversible** (implicitly no-op): empty Action does not push to undo stack — no commit means no frame. This is already tested via `on_dispatch_not_called_for_empty_mutations` but should be explicitly named with a `can_undo()` assertion.

### TEST-08: Behavior lifecycle edge cases
The following are MISSING and need new unit tests (already covered: is_active=false skips hooks; on_dispatch fires on inactive):
- **`on_undo()` fires when a reversible action is undone**: dispatch one reversible action, undo it, assert `on_undo` counter == 1 (using `Rc<Cell<u32>>`).
- **Behavior deactivating during dispatch does not corrupt already-queued hooks**: Two behaviors registered. BehaviorA deactivates in `on_dispatch()` (sets internal flag so `is_active()` returns false from next dispatch). BehaviorB has a `before()` hook. After BehaviorA deactivates, BehaviorB's `before()` still runs on subsequent dispatches — deactivating A doesn't affect B. Verifies engine doesn't re-check `is_active()` mid-dispatch for other behaviors.

### Claude's Discretion
- Exact ASCII diagram layout and line lengths in lib.rs
- Whether to use `# Examples` or `# Example` heading in doctests (match existing convention)
- Order of the four new doctests within their respective trait definitions
- Whether empty-Action explicit test is a new test function or the assertion is appended to existing `on_dispatch_not_called_for_empty_mutations`

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Rc<Cell<u32>>` pattern — established in `stateful_behavior_n_dispatches` (engine.rs line 929) — use for on_dispatch/on_undo doctests
- `CounterOp` fixture — defined in both `mod tests` and `mod props` — can be re-used inline in doctests (small, well-understood)
- `Engine::can_undo()` / `Engine::can_redo()` — added in Phase 6 (engine.rs) — use in is_reversible doctest; no need for `undo_stack.len()` assertions

### Established Patterns
- Doctest style: `# use herdingcats::Mutation;` hidden imports, concrete `assert!` or `assert_eq!` statements, `#[derive(Clone)]` on mutation types
- Inline `#[cfg(test)]` block structure: section banners + `mod tests` (unit) + `mod props` (proptest sibling)
- All undo assertions use both state value AND `replay_hash()` — maintain this where applicable

### Integration Points
- `src/lib.rs`: expand `//!` block with model prose + ASCII diagram + quick-start doctest
- `src/mutation.rs`: add doctest to `is_reversible` method (currently has prose, no runnable example)
- `src/behavior.rs`: add doctests to `is_active`, `on_dispatch`, `on_undo` (currently have prose, no runnable examples)
- `src/engine.rs` `mod tests`: add `mixed_mutations_treated_as_irreversible`, `on_undo_fires_on_undo`, `deactivation_mid_dispatch_does_not_corrupt_hooks` unit tests; optionally strengthen `on_dispatch_not_called_for_empty_mutations` with explicit `can_undo()` assertion

</code_context>

<specifics>
## Specific Ideas

- The ASCII diagram should show the full dispatch pipeline and the undo/redo reverse path. Compact but complete.
- The quick-start doctest in lib.rs should be the minimal "hello world" — define a counter state, a simple mutation, a passthrough behavior, dispatch, assert. Not a game demo, just the pipeline.
- The `on_undo` doctest showing dispatch → undo → counter == 0 is the most instructive: it proves the symmetry between on_dispatch and on_undo in one short example.
- Mixed mutations test: use the existing `MixedOp` (Rev/Irrev) fixture already in engine.rs `mod tests` — no new types needed.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-documentation-and-extended-tests*
*Context gathered: 2026-03-10*
