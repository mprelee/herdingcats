---
phase: quick-final-cleanup
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/engine.rs
  - src/behavior.rs
  - src/apply.rs
  - README.md
autonomous: true
requirements: []

must_haves:
  truths:
    - "debug_assert in dispatch catches Apply implementations that mutate state but return zero traces"
    - "All doc comments accurately describe BehaviorDef as fn-pointer structs, not trait objects"
    - "Irreversible history semantics are locked down with explicit named tests"
    - "All rustdoc examples compile (cargo test --doc passes)"
  artifacts:
    - path: "src/engine.rs"
      provides: "debug_assert for Apply trace contract, updated doc wording"
    - path: "src/behavior.rs"
      provides: "Clarified static behavior wording"
    - path: "README.md"
      provides: "Optional architecture status note"
  key_links:
    - from: "src/engine.rs dispatch()"
      to: "Apply::apply()"
      via: "debug_assert!(!new_traces.is_empty()) after diff.apply()"
      pattern: "debug_assert.*traces.*empty"
---

<objective>
Final cleanup pass: tighten the Apply trace contract with a debug_assert in dispatch, clarify "static behavior" wording to match the actual BehaviorDef fn-pointer implementation, and add explicit irreversible history tests.

Purpose: Lock down contracts and documentation accuracy before v0.5.0 release.
Output: Tightened engine dispatch, accurate docs, comprehensive irreversible history tests.
</objective>

<execution_context>
@/Users/mprelee/.claude/get-shit-done/workflows/execute-plan.md
@/Users/mprelee/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@README.md
@src/engine.rs
@src/behavior.rs
@src/apply.rs
@ARCHITECTURE.md

<interfaces>
<!-- Key types and contracts the executor needs. -->

From src/engine.rs dispatch() lines 138-143:
```rust
BehaviorResult::Continue(new_diffs) => {
    for diff in new_diffs {
        let new_traces = diff.apply(working.to_mut());
        traces.extend(new_traces);
        diffs.push(diff);
    }
}
```

From src/apply.rs:
```rust
pub trait Apply<E: EngineSpec> {
    /// Each call MUST return at least one trace entry describing the mutation.
    fn apply(&self, state: &mut E::State) -> Vec<E::Trace>;
}
```

From src/behavior.rs:
```rust
pub struct BehaviorDef<E: EngineSpec> {
    pub name: &'static str,
    pub order_key: E::OrderKey,
    pub evaluate: fn(&E::Input, &E::State) -> BehaviorResult<E::Diff, E::NonCommittedInfo>,
}
```

From src/engine.rs lines 169-173 (irreversible handling):
```rust
if reversibility == Reversibility::Irreversible {
    self.undo_stack.clear();
    self.redo_stack.clear();
}
```

Existing irreversible tests (engine.rs lines 563-574, 747-767):
- `irreversible_committed_dispatch_clears_both_stacks` — checks undo/redo depth == 0
- `irreversible_commit_clears_both_stacks` — checks depth + Disallowed outcomes
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add debug_assert for Apply trace contract and clarify static behavior docs</name>
  <files>src/engine.rs, src/apply.rs, src/behavior.rs, README.md</files>
  <action>
1. In `src/engine.rs` dispatch(), after `let new_traces = diff.apply(working.to_mut());` (line 140), add a debug_assert that enforces the trace contract:
   ```rust
   debug_assert!(
       !new_traces.is_empty(),
       "Apply::apply() contract violation: diff mutated state but returned zero trace entries. \
        Every state-mutating diff MUST return at least one trace entry."
   );
   ```
   This fires only in debug/test builds. In release builds it is a no-op.
   Note: This asserts unconditionally in the diff loop (we know we are inside the `new_diffs` iterator, so a diff was emitted — if it produced zero traces, the contract is violated). The true no-op case is when a behavior returns `Continue(vec![])` with zero diffs, which never enters this loop.

2. In `src/apply.rs`, update the trait-level doc comment on `Apply` to mention the debug_assert enforcement:
   - After the existing line "Each call MUST return at least one trace entry...", add:
     "The engine enforces this contract with a `debug_assert!` in dispatch — violations panic in debug/test builds."

3. In `src/behavior.rs` module-level doc (lines 1-17), verify wording says:
   - "no trait objects, no `dyn` dispatch" (already present)
   - "plain struct with fn pointer fields" (already present)
   - "no runtime registration" — add if not present
   Ensure NO mention of "type-level tuple composition" exists.

4. In `README.md`, add a short "## Architecture Status" section before the "## Zero Dependencies" section:
   ```markdown
   ## Architecture Status

   v0.5.0 implements the full architecture described in `ARCHITECTURE.md`:

   - Static behavior set via `BehaviorDef` structs (fn pointers, no trait objects)
   - Copy-on-write working state (zero-clone until first diff)
   - Snapshot-based undo/redo (no `Reversible` trait burden)
   - Deterministic `(order_key, name)` behavior ordering
   - `Apply` trace contract enforced by `debug_assert!` in dispatch
   ```

5. Verify all existing doctests still compile: `cargo test --doc`
  </action>
  <verify>
    <automated>cd /Users/mprelee/herdingcats && cargo test --doc 2>&1 && cargo test 2>&1</automated>
  </verify>
  <done>debug_assert fires on trace contract violations in debug builds; static behavior wording is accurate; README has architecture status note; all tests pass</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add explicit irreversible history semantics tests</name>
  <files>src/engine.rs</files>
  <behavior>
    - Test: reversible_commit_is_undoable — dispatch with Reversible, assert undo() returns Undone(frame)
    - Test: irreversible_commit_clears_all_history — dispatch 2x Reversible, then 1x Irreversible, assert undo_depth==0 and redo_depth==0
    - Test: after_irreversible_commit_undo_returns_nothing_to_undo — dispatch Irreversible, call undo(), assert Disallowed(NothingToUndo)
    - Test: after_irreversible_commit_redo_returns_nothing_to_redo — dispatch Irreversible, call redo(), assert Disallowed(NothingToRedo)
    - Test: irreversible_commit_preceded_by_undo_clears_redo_too — dispatch Reversible, undo, dispatch Irreversible, assert redo_depth==0
  </behavior>
  <action>
Add a new `#[cfg(test)] mod irreversible_history_tests` block at the bottom of `src/engine.rs` (before the closing of the existing test infrastructure, or as a separate mod). Use the existing test helper pattern (TestSpec, make_engine, append_eval) already defined in the file's test modules.

Each test should be clearly named with `irreversible_` prefix and have a doc comment explaining the exact semantic being locked down. These tests complement the existing ones by being more explicit and focused — each tests exactly one irreversible semantic in isolation.

Use the existing helper `make_engine()` if available, or construct `Engine::<TestSpec>::new(vec![], vec![BehaviorDef { name: "append", order_key: 0u32, evaluate: append_eval }])` directly.
  </action>
  <verify>
    <automated>cd /Users/mprelee/herdingcats && cargo test irreversible_history 2>&1</automated>
  </verify>
  <done>5 new focused irreversible history tests pass, each locking down one semantic guarantee</done>
</task>

</tasks>

<verification>
All checks pass:
- `cargo test` — all unit tests pass (including new irreversible history tests)
- `cargo test --doc` — all doc examples compile and pass
- `cargo clippy` — no warnings
- debug_assert fires when running tests with a deliberately broken Apply impl (manual verification)
</verification>

<success_criteria>
1. `debug_assert!` in dispatch() catches Apply implementations returning empty traces for emitted diffs
2. Apply trait docs mention the debug_assert enforcement
3. Static behavior wording is accurate (fn pointers, no trait objects, no runtime registration, no tuple composition claims)
4. README has architecture status section
5. 5 new irreversible history tests pass
6. All existing tests and doctests continue to pass
</success_criteria>

<output>
After completion, create `.planning/quick/1-final-cleanup-pass-fix-malformed-docs-ti/1-SUMMARY.md`
</output>
