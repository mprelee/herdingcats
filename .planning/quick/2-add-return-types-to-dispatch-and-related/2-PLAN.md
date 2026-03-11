---
phase: quick-2
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/engine.rs
  - src/lib.rs
  - examples/tictactoe.rs
  - examples/backgammon.rs
autonomous: true
requirements: [QUICK-2]
must_haves:
  truths:
    - "engine.dispatch() returns Some(action) when mutations applied, None when cancelled or no mutations"
    - "engine.dispatch_preview() returns the Action as-mutated-by-behaviors (for caller inspection)"
    - "All existing tests pass"
    - "Examples compile with updated call sites"
  artifacts:
    - path: src/engine.rs
      provides: "Updated dispatch() -> Option<Action<M>> and dispatch_preview() -> Action<M>"
      contains: "Option<Action<M>>"
  key_links:
    - from: src/engine.rs
      to: examples/tictactoe.rs
      via: "dispatch return value — call sites must handle Option or use let _ ="
      pattern: "engine\\.dispatch\\("
---

<objective>
Add meaningful return types to `Engine::dispatch` and `Engine::dispatch_preview`.

`dispatch` returns `Option<Action<M>>`: `Some(action)` when the action was not cancelled and had mutations (state was affected), `None` when cancelled or mutations list was empty. The `Action<M>` type already derives `Clone`, so we clone `tx` before moving it into the `CommitFrame`.

`dispatch_preview` returns `Action<M>` (the action after behaviors have run their `before`/`after` hooks). This gives callers visibility into what mutations behaviors injected during the dry run — useful for AI look-ahead.

Purpose: Callers currently have no way to know if dispatch did anything without inspecting state before and after. The return value makes "did this event have any effect?" a first-class question.
Output: Updated engine.rs with new signatures, updated doc examples, updated call sites in examples/*.
</objective>

<execution_context>
@/Users/mprelee/.claude/get-shit-done/workflows/execute-plan.md
@/Users/mprelee/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/Users/mprelee/herdingcats/.planning/STATE.md

<interfaces>
<!-- Current signatures to change -->

From src/engine.rs:

```rust
// Current — returns nothing
pub fn dispatch(&mut self, mut event: I, mut tx: Action<M>) { ... }
pub fn dispatch_preview(&mut self, mut event: I, mut tx: Action<M>) { ... }
```

From src/action.rs:
```rust
#[derive(Clone)]
pub struct Action<M> {
    pub mutations: Vec<M>,
    pub deterministic: bool,
    pub cancelled: bool,
}
```

<!-- dispatch() logic summary (relevant for understanding return point):
  - Runs before() on all active behaviors
  - If !tx.cancelled: applies mutations, runs after()
  - If !tx.cancelled && !tx.mutations.is_empty(): hashes, pushes CommitFrame (moves tx into frame), calls on_dispatch()
  - If tx.cancelled OR tx.mutations.is_empty(): nothing committed
  Currently tx is moved into CommitFrame unconditionally when committed.
-->
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Update dispatch() and dispatch_preview() signatures and return values</name>
  <files>src/engine.rs</files>
  <behavior>
    - dispatch(cancelled action) -> None
    - dispatch(empty mutations) -> None
    - dispatch(valid mutations, reversible) -> Some(action) with action.mutations populated
    - dispatch(valid mutations, irreversible) -> Some(action)
    - dispatch_preview() -> Action<M> reflecting any mutations behaviors injected during before()/after()
  </behavior>
  <action>
Change `dispatch` signature to `pub fn dispatch(&mut self, mut event: I, mut tx: Action<M>) -> Option<Action<M>>`.

Implementation: clone `tx` *before* the commit block so we can return it. The clone must happen after behaviors have run (so it captures any behavior-injected mutations). Specifically:

1. After the `after()` behavior loop (line ~319), before the `if !tx.cancelled && !tx.mutations.is_empty()` block, clone: `let committed_tx = tx.clone();`
2. Inside the commit block, move `tx` into the CommitFrame as before.
3. At the end of the function: return `Some(committed_tx)` if we entered the commit block, `None` otherwise.

The cleanest approach: use a boolean flag `let mut committed = false;` set to `true` inside the `if !tx.cancelled && !tx.mutations.is_empty()` block, then return at the end:
```rust
if committed { Some(committed_tx) } else { None }
```
where `committed_tx` is the clone taken after behaviors run.

Change `dispatch_preview` signature to `pub fn dispatch_preview(&mut self, mut event: I, mut tx: Action<M>) -> Action<M>`.

Implementation: after rolling back state/hash (lines ~251-252), return `tx`. No clone needed — tx is owned and rollback only touches `self.state` and `self.replay_hash`, not `tx`.

Update all doc examples in the doc comments for both functions to use `let _action = engine.dispatch(...)` or `let _preview = engine.dispatch_preview(...)` so they compile. Update the lib.rs quick-start example similarly (line 73: `engine.dispatch((), tx);` → `let _ = engine.dispatch((), tx);`).

Update internal test call sites in src/engine.rs: most tests call `engine.dispatch((), tx);` — prepend `let _ =` to each such call. The tests themselves test state/undo behavior, not the return value, so this is mechanical. For tests specifically about the new return value (the tdd behavior above), write new test functions.
  </action>
  <verify>
    <automated>cd /Users/mprelee/herdingcats && cargo test --lib 2>&1 | tail -20</automated>
  </verify>
  <done>
    `cargo test --lib` passes. `dispatch` returns `Option&lt;Action&lt;M&gt;&gt;`, `dispatch_preview` returns `Action&lt;M&gt;`. Tests covering None (cancelled), None (empty), Some (valid) all green.
  </done>
</task>

<task type="auto">
  <name>Task 2: Update examples and run full test suite</name>
  <files>examples/tictactoe.rs, examples/backgammon.rs, src/lib.rs</files>
  <action>
Update every call site in examples that ignores the return value. Pattern: `engine.dispatch(...)` with no binding → `let _ = engine.dispatch(...)`. Same for any `engine.dispatch_preview(...)` call sites.

Files to touch:
- examples/tictactoe.rs line 255: `engine.dispatch(GameEvent::Play { idx: m }, tx);` → `let _ = engine.dispatch(...);`
- examples/backgammon.rs lines 474, 481, 498, 844, 857, 882: same pattern
- src/lib.rs line 73 (doc example): `engine.dispatch((), tx);` → `let _ = engine.dispatch((), tx);`

Then run `cargo test --all` to confirm examples compile and all tests pass.
  </action>
  <verify>
    <automated>cd /Users/mprelee/herdingcats && cargo test --all 2>&1 | tail -30</automated>
  </verify>
  <done>
    `cargo test --all` and `cargo build --examples` both succeed with zero errors. No warnings about unused Results.
  </done>
</task>

</tasks>

<verification>
```bash
cd /Users/mprelee/herdingcats && cargo test --all && cargo build --examples && cargo clippy -- -D warnings
```
All green = done.
</verification>

<success_criteria>
- `engine.dispatch()` signature is `-> Option<Action<M>>`
- Returns `Some(action)` when `!cancelled && !mutations.is_empty()`
- Returns `None` when cancelled or mutations empty
- `engine.dispatch_preview()` signature is `-> Action<M>`
- All existing tests pass
- Examples compile
- No new clippy warnings
</success_criteria>

<output>
After completion, create `.planning/quick/2-add-return-types-to-dispatch-and-related/2-SUMMARY.md`
</output>
