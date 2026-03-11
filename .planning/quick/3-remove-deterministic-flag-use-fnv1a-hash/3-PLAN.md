---
phase: quick-3
plan: 3
type: execute
wave: 1
depends_on: []
files_modified:
  - src/action.rs
  - src/engine.rs
  - src/lib.rs
  - examples/tictactoe.rs
  - examples/backgammon.rs
autonomous: true
requirements: [QUICK-3]

must_haves:
  truths:
    - "Action<M> has no deterministic field; all mutations always contribute to replay_hash"
    - "replay_hash accumulation uses pure fnv1a applied across the full mutation byte sequence per action"
    - "engine.dispatch(event) works without passing Action::new(); engine.dispatch_with(event, tx) accepts a custom action"
  artifacts:
    - path: "src/action.rs"
      provides: "Action<M> struct with mutations and cancelled only"
    - path: "src/engine.rs"
      provides: "dispatch(event), dispatch_with(event, tx), corrected replay_hash logic"
  key_links:
    - from: "src/engine.rs dispatch()"
      to: "src/engine.rs dispatch_with()"
      via: "calls dispatch_with(event, Action::new())"
    - from: "src/engine.rs dispatch_with()"
      to: "src/hash.rs fnv1a_hash()"
      via: "concatenated byte accumulation"
---

<objective>
Three cleanup changes to the herdingcats engine API and hash algorithm.

Purpose: Remove an unused field (deterministic), fix the replay hash to be a genuine fnv1a hash of all mutation bytes in an action, and eliminate boilerplate at call sites by making `dispatch(event)` the simple path.
Output: Cleaner Action struct, correct hash semantics, ergonomic dispatch API.
</objective>

<execution_context>
@/Users/mprelee/.claude/get-shit-done/workflows/execute-plan.md
@/Users/mprelee/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/Users/mprelee/herdingcats/.planning/STATE.md
@/Users/mprelee/herdingcats/src/action.rs
@/Users/mprelee/herdingcats/src/engine.rs
@/Users/mprelee/herdingcats/src/hash.rs
@/Users/mprelee/herdingcats/src/lib.rs
@/Users/mprelee/herdingcats/examples/tictactoe.rs
@/Users/mprelee/herdingcats/examples/backgammon.rs

<interfaces>
<!-- Key types and contracts the executor needs. Extracted from codebase. -->

From src/hash.rs:
```rust
pub(crate) const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub(crate) const FNV_PRIME: u64 = 0x100000001b3;
pub(crate) fn fnv1a_hash(bytes: &[u8]) -> u64;
```

Current Action<M> struct (src/action.rs):
```rust
pub struct Action<M> {
    pub mutations: Vec<M>,
    pub deterministic: bool,   // <-- to be removed
    pub cancelled: bool,
}
```

Current dispatch signature (src/engine.rs line 302):
```rust
pub fn dispatch(&mut self, mut event: I, mut tx: Action<M>) -> Option<Action<M>>
```

Current replay_hash accumulation (src/engine.rs lines 326-332):
```rust
if tx.deterministic {
    for m in &tx.mutations {
        let h = fnv1a_hash(&m.hash_bytes());
        self.replay_hash ^= h;
        self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
    }
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Remove deterministic field and fix replay_hash algorithm</name>
  <files>src/action.rs, src/engine.rs</files>
  <behavior>
    - Test 1: Action::new() creates action with only mutations (empty) and cancelled (false) — no deterministic field
    - Test 2: Two engines dispatching the same single mutation arrive at the same replay_hash
    - Test 3: Two engines dispatching same two mutations in same order arrive at same replay_hash
    - Test 4: replay_hash differs when mutations differ (sensitivity)
    - Test 5: replay_hash after undo matches replay_hash before the undone dispatch (existing behavior preserved)
  </behavior>
  <action>
**src/action.rs:**

Remove the `deterministic` field entirely from `Action<M>`:
- Remove `pub deterministic: bool,` from the struct definition
- Remove the field doc comment for `deterministic`
- Update the struct-level doc comment: remove the `- deterministic` bullet point, keep `- mutations` and `- cancelled`
- In `Action::new()`: remove `deterministic: true,` from the constructor
- Update `Action::new()` doc comment: remove mention of `deterministic` from the defaults list, remove the `assert!(tx.deterministic);` assertion from the doc example
- Update `Default::default()` impl: no change needed (calls `Self::new()`)
- In `#[cfg(test)] mod tests`:
  - `action_new_defaults`: remove `assert!(tx.deterministic);`
  - Add new test `action_has_mutations_and_cancelled_only` that verifies `tx.mutations.is_empty()` and `!tx.cancelled`

**src/engine.rs:**

Replace the current replay_hash accumulation block. The current code:
```rust
if tx.deterministic {
    for m in &tx.mutations {
        let h = fnv1a_hash(&m.hash_bytes());
        self.replay_hash ^= h;
        self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
    }
}
```

Replace with a proper fnv1a hash of all mutation bytes concatenated for this action, then fold that single hash into the running `replay_hash`:
```rust
// Collect all mutation bytes for this action into one sequence
let mut action_bytes: Vec<u8> = Vec::new();
for m in &tx.mutations {
    action_bytes.extend_from_slice(&m.hash_bytes());
}
// Hash the full sequence with fnv1a, then fold into running hash
let action_hash = fnv1a_hash(&action_bytes);
self.replay_hash ^= action_hash;
self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
```

Remove the `if tx.deterministic {` conditional entirely — all mutations always contribute to the hash. The `FNV_PRIME` import is still needed for the fold.

Update `FNV_OFFSET` import: it is still used in `Engine::new()` to initialize `replay_hash`. Verify the import line `use crate::hash::{FNV_OFFSET, FNV_PRIME, fnv1a_hash};` remains correct.

Update all doc comments in `engine.rs` that reference `deterministic` or describe the hash accumulation:
- `replay_hash()` doc: remove "where the action is `deterministic`" — all dispatches now contribute
- Update the sentence "Two engine instances that have processed the same sequence of deterministic mutations will have identical replay hashes, regardless of any non-deterministic commits in between." to "Two engine instances that have processed the same sequence of mutations will have identical replay hashes."

In `#[cfg(test)] mod tests`, find any tests that check `tx.deterministic` or test hash behavior gated on the deterministic flag and update them. Specifically:
- Any test that sets `tx.deterministic = false` to suppress hashing no longer works that way — remove or update those tests to reflect that all mutations now hash
- Existing tests that verify `replay_hash` changes after dispatch should still pass as-is
  </action>
  <verify>
    <automated>cargo test --lib 2>&1 | tail -5</automated>
  </verify>
  <done>
    `cargo test --lib` passes. `Action<M>` has no `deterministic` field. replay_hash accumulation concatenates all mutation bytes per action then applies fnv1a, no conditional gating. Doc comments updated.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add dispatch(event) convenience; rename current to dispatch_with</name>
  <files>src/engine.rs, src/lib.rs, examples/tictactoe.rs, examples/backgammon.rs</files>
  <behavior>
    - Test 1: engine.dispatch(event) compiles and behaves identically to engine.dispatch_with(event, Action::new())
    - Test 2: engine.dispatch_with(event, tx) works for callers that need to pre-populate tx or pass a custom action
    - Test 3: dispatch_preview signature is unchanged (callers pass tx explicitly)
    - Test 4: return type of dispatch(event) is Option<Action<M>>, same as dispatch_with
  </behavior>
  <action>
**src/engine.rs:**

Rename the current `dispatch` method to `dispatch_with`. The signature becomes:
```rust
pub fn dispatch_with(&mut self, event: I, tx: Action<M>) -> Option<Action<M>>
```
The body is unchanged.

Add a new `dispatch` method that is the simple path:
```rust
/// Dispatch `event` through all active behaviors using a fresh, empty action.
///
/// This is the ergonomic entry point for the common case where no pre-populated
/// action is needed. Behaviors inject mutations via their `before` hook.
/// For cases where you need to pass a pre-built action (e.g., with mutations
/// already pushed), use [`dispatch_with`](Engine::dispatch_with) instead.
///
/// Returns `Some(action)` if mutations were applied, `None` if cancelled or
/// no mutations produced.
///
/// # Examples
///
/// ```
/// use herdingcats::{Engine, Mutation, Behavior, Action};
///
/// #[derive(Clone)]
/// enum CounterOp { Inc }
/// impl Mutation<i32> for CounterOp {
///     fn apply(&self, s: &mut i32) { *s += 1; }
///     fn undo(&self, s: &mut i32)  { *s -= 1; }
///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
/// }
///
/// struct IncRule;
/// impl Behavior<i32, CounterOp, (), u8> for IncRule {
///     fn id(&self) -> &'static str { "inc" }
///     fn priority(&self) -> u8 { 0 }
///     fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
///         tx.mutations.push(CounterOp::Inc);
///     }
/// }
///
/// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
/// engine.add_behavior(IncRule);
/// let _ = engine.dispatch(());
/// assert_eq!(engine.state, 1);
/// ```
pub fn dispatch(&mut self, event: I) -> Option<Action<M>> {
    self.dispatch_with(event, Action::new())
}
```

Place `dispatch` immediately before `dispatch_with` in the impl block so the API reads: simple path first, then the with-tx variant.

Update `dispatch_with` doc comment:
- Title: "Dispatch `event` through all active behaviors with a custom pre-built action."
- Add note: "For the common case with no pre-built action, use [`dispatch`](Engine::dispatch)."
- Update all doc examples within `dispatch_with` to use `dispatch_with(event, tx)` not `dispatch(event, tx)`

Update all doc examples WITHIN `engine.rs` that currently call `engine.dispatch(event, tx)` or `let _ = engine.dispatch(event, tx)`:
- In method doc comments for `undo`, `redo`, `can_undo`, `can_redo`, `write`, `replay_hash`, `add_behavior`: where the example creates a `tx`, pushes to it, then calls `engine.dispatch(event, tx)` — update to call `engine.dispatch_with(event, tx)` since they use the pre-built tx
- Where the example calls `engine.dispatch((), Action::new())` — update to `engine.dispatch(())` (use simple path)

Update internal `#[cfg(test)]` test bodies in `engine.rs`:
- All `engine.dispatch((), tx)` calls where `tx` has mutations pushed → `engine.dispatch_with((), tx)`
- All `engine.dispatch((), Action::new())` calls (empty tx) → `engine.dispatch(())`
- All `engine.dispatch_preview((), tx)` calls → unchanged (dispatch_preview keeps its signature)

**src/lib.rs:**

Update the quick-start doctest. Current:
```rust
let mut tx = Action::new();
tx.mutations.push(CounterOp::Inc);
let _ = engine.dispatch((), tx);
```
Update to use `dispatch_with` since it pre-builds the tx:
```rust
let mut tx = Action::new();
tx.mutations.push(CounterOp::Inc);
let _ = engine.dispatch_with((), tx);
```
Also update the top-level module doc table row for `Action`: still accurate. Update the dispatch pipeline comment if it says `dispatch(event, tx)` anywhere.

**examples/tictactoe.rs:**

Current in `main()`:
```rust
let tx = Action::new();
let _ = engine.dispatch(GameEvent::Play { idx: m }, tx);
```
Behaviors inject all mutations — the `tx` is empty at call site. Change to:
```rust
let _ = engine.dispatch(GameEvent::Play { idx: m });
```
Remove the `let tx = Action::new();` line entirely.

**examples/backgammon.rs:**

All `engine.dispatch(event, Action::new())` call sites → `engine.dispatch(event)`.
All `engine.dispatch(event, tx)` call sites where `tx` has mutations → `engine.dispatch_with(event, tx)`.

Search the file for `engine.dispatch(` and update each:
- `engine.dispatch(BackgammonEvent::RollDice { d1: 3, d2: 5 }, Action::new())` → `engine.dispatch(BackgammonEvent::RollDice { d1: 3, d2: 5 })`
- Any dispatch call passing `Action::new()` → remove the arg, use `dispatch(event)`
- Any dispatch call passing a non-trivial `tx` (with mutations pushed) → change to `dispatch_with(event, tx)`
  </action>
  <verify>
    <automated>cargo test && cargo run --example tictactoe && cargo run --example backgammon 2>&1 | tail -10</automated>
  </verify>
  <done>
    `cargo test` passes (all unit + doc tests). Both examples compile and run. `engine.dispatch(event)` is the simple-path API. `engine.dispatch_with(event, tx)` exists for pre-built actions. No remaining `dispatch(event, Action::new())` call sites in examples or tests.
  </done>
</task>

</tasks>

<verification>
- `cargo clippy -- -D warnings` passes with no warnings
- `cargo doc --no-deps` generates docs without errors
- `grep -r "\.deterministic" src/ examples/` returns no matches
- `grep "dispatch(.*Action::new" src/ examples/` returns no matches (all converted)
</verification>

<success_criteria>
- `Action<M>` struct has exactly two fields: `mutations: Vec<M>` and `cancelled: bool`
- replay_hash accumulation concatenates all mutation bytes per action into one slice, hashes with fnv1a, then folds with `^= hash; wrapping_mul(FNV_PRIME)` — no per-mutation XOR/mul loop
- `engine.dispatch(event)` is the simple API; `engine.dispatch_with(event, tx)` accepts a pre-built action
- All call sites updated; `cargo test` and both examples pass
</success_criteria>

<output>
After completion, create `/Users/mprelee/herdingcats/.planning/quick/3-remove-deterministic-flag-use-fnv1a-hash/3-SUMMARY.md`
</output>
