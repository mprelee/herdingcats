# Phase 5: Reversibility and Behavior Lifecycle - Research

**Researched:** 2026-03-10
**Domain:** Rust trait API design, borrow-checker patterns, undo/redo state machines
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Inactive behaviors (`is_active()` returns false) skip `before()` and `after()` hooks
- Inactive behaviors still receive `on_dispatch()` and `on_undo()` calls unconditionally
- `on_undo()` fires on ALL behaviors unconditionally when `engine.undo()` is called — no enabled_snapshot needed
- `before()`/`after()` hooks remain `&self` (immutable self) — no change
- `dispatch_preview` checks `is_active()` before firing `before`/`after` — pure dry run, never calls `on_dispatch`/`on_undo`
- Empty actions (`tx.mutations.is_empty()` after all `before()` hooks) are silently no-op'd — no frame pushed, no `on_dispatch` called
- Cancelled actions are silently no-op'd — same path as empty
- `dispatch()` return type stays `()`
- `CommitFrame` loses `lifetime_snapshot` and `enabled_snapshot` fields
- `Engine` loses `lifetimes: HashMap` and `enabled: HashSet` fields
- No safeguard for all-inactive engine state — developer responsibility

### Claude's Discretion

- Borrow conflict resolution in `on_dispatch`/`on_undo` pass — behaviors need `&mut self` while engine iterates. Implementation approach (index-based iteration, `iter_mut()` in a separate pass, etc.) is Claude's choice.
- `DispatchError` vs outcome type — shape is Claude's discretion if added in future phases
- Whether `dispatch_preview` rolls back `is_active()` state — not needed since `is_active(&self)` is immutable

### Deferred Ideas (OUT OF SCOPE)

- `DispatchOutcome` return type from `dispatch()` — decided against for now
- Safeguard for all-inactive engine state — explicitly deferred, developer responsibility
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REV-01 | `Mutation` trait gains `fn is_reversible(&self) -> bool { true }` as a default method | Trait default methods in Rust — zero breaking change, backward-compatible |
| REV-02 | `Action<M>` derives reversibility from mutations at commit time — reversible iff all mutations return `is_reversible() = true` | `.iter().all(|m| m.is_reversible())` at commit; no struct field needed |
| REV-03 | Engine clears undo stack when committing irreversible `Action` — undo barrier | `self.undo_stack.clear()` before pushing nothing (irreversible = no frame push + clear existing stack) |
| REV-04 | Reversible `Action` commits push `CommitFrame` as before; undo/redo semantics unchanged | Existing push logic retained behind reversibility gate |
| LIFE-01 | `Behavior` trait gains `fn is_active(&self) -> bool { true }` default method | `&self` immutable — no borrow conflict with iteration |
| LIFE-02 | `Behavior` trait gains `fn on_dispatch(&mut self) {}` default method — called after each committed action (incl. redo) | Requires `&mut self` — separate `iter_mut()` pass needed |
| LIFE-03 | `Behavior` trait gains `fn on_undo(&mut self) {}` default method — called on undo | Same separate `iter_mut()` pass pattern |
| LIFE-04 | `engine.add_behavior(behavior)` replaces `add_behavior` signature — no lifetime parameter | Already matches current signature — just remove `enabled.insert` and `lifetimes.insert` calls |
| LIFE-05 | Engine removes `lifetimes: HashMap` + `enabled: HashSet`; uses per-dispatch `behavior.is_active()` | Remove fields from `Engine`, remove `use std::collections::{HashMap, HashSet}` if unused |
| LIFE-06 | Engine calls `on_dispatch()`/`on_undo()` on ALL behaviors in separate pass after state mutations, avoiding borrow conflicts | Separate `iter_mut()` loop after state-mutation loop — key architectural point |
</phase_requirements>

## Summary

Phase 5 is a focused API extension and engine cleanup phase. The work divides into two parallel tracks: (1) the reversibility track, which adds `is_reversible()` to the `Mutation` trait and derives `Action` reversibility at commit time to enforce an undo barrier; and (2) the behavior lifecycle track, which adds three new default methods to `Behavior` (`is_active`, `on_dispatch`, `on_undo`) and removes the `RuleLifetime`/`enabled` HashMap mechanism that previously managed behavior activation externally.

The primary Rust challenge in this phase is the borrow-checker conflict that arises when calling `&mut self` methods (`on_dispatch`, `on_undo`) on boxed trait objects stored in `self.behaviors` while the engine itself holds `&mut self`. The solution is a dedicated second iteration pass using `iter_mut()` that runs completely after the state-mutation pass, keeping the two concerns cleanly separated. The `is_active()` check uses `&self` and does not conflict with the existing immutable iteration pattern.

The `CommitFrame` cleanup (removing `lifetime_snapshot` and `enabled_snapshot`) is a straightforward field removal. After Phase 4, `RuleLifetime` was already reduced to a single `Permanent` variant with no behavioral weight — removing it and the two HashMap/HashSet fields simplifies the engine and eliminates snapshot overhead on every commit.

**Primary recommendation:** Implement in file order — `mutation.rs` first (add `is_reversible`), then `behavior.rs` (add three lifecycle methods), then `engine.rs` (remove old fields, add lifecycle pass, add reversibility gate). Each file compiles independently so incremental verification is possible.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust stdlib traits | edition 2024 | Default method implementations on traits | Zero-cost, backward-compatible, established pattern |
| proptest | 1.10 (dev) | Property-based tests for reversibility invariants | Already the project standard for engine correctness |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::marker::PhantomData` | stdlib | Keep `CommitFrame<S, M>` type-safe without `lifetime_snapshot` | Already present in CommitFrame — no change needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate `iter_mut()` pass for `on_dispatch` | `Arc<Mutex<>>` on each behavior | Separate pass is zero-overhead and simpler; Mutex adds runtime cost and complexity |
| `iter_mut()` pass | Index-based `for i in 0..self.behaviors.len()` | Either works; `iter_mut()` is more idiomatic Rust, index-based avoids potential range issues |
| Derive reversibility at commit time | Store `reversible: bool` field on `Action` | Commit-time derivation is the locked decision; field-on-struct was the old `tx.irreversible` pattern being removed |

**Installation:** No new dependencies. Phase 5 uses only existing Rust stdlib and the already-present `proptest` dev-dep.

## Architecture Patterns

### Recommended Change Order
```
src/mutation.rs     # Add is_reversible() default method — compile check
src/behavior.rs     # Add is_active(), on_dispatch(), on_undo() default methods — compile check
src/engine.rs       # Remove RuleLifetime, CommitFrame fields, Engine fields;
                    # Add lifecycle pass; Add reversibility gate — compile + test check
```

### Pattern 1: Trait Default Methods (Backward-Compatible Extension)

**What:** Add new methods to a trait with a provided default body so all existing implementors continue to compile without changes.

**When to use:** Every new method in this phase (is_reversible, is_active, on_dispatch, on_undo) uses this pattern.

**Example:**
```rust
// In mutation.rs
pub trait Mutation<S>: Clone {
    // ... existing methods ...

    /// Whether this mutation can be undone.
    ///
    /// Returns `true` by default. Override to return `false` for mutations
    /// that represent irreversible state changes (e.g., dice rolls, card draws).
    fn is_reversible(&self) -> bool {
        true
    }
}
```

```rust
// In behavior.rs
pub trait Behavior<S, M, I, P>
where
    S: Clone,
    M: Mutation<S>,
    P: Copy + Ord,
{
    // ... existing methods ...

    /// Whether this behavior participates in the current dispatch.
    ///
    /// Returns `true` by default. When `false`, the engine skips `before()`
    /// and `after()` for this behavior, but still calls `on_dispatch()`
    /// and `on_undo()` so the behavior can track history while dormant.
    fn is_active(&self) -> bool {
        true
    }

    /// Called after each committed action (including redo) on ALL behaviors.
    ///
    /// Use this to update behavior-internal state in response to a dispatch
    /// (e.g., decrement a charge counter, toggle an activation flag).
    fn on_dispatch(&mut self) {}

    /// Called after each undo on ALL behaviors.
    ///
    /// Use this to reverse behavior-internal state changes made in `on_dispatch`.
    fn on_undo(&mut self) {}
}
```

### Pattern 2: Separate Lifecycle Pass (Borrow Conflict Resolution)

**What:** After the state-mutation loop (which uses `&self` on behaviors for `before`/`after`), run a second, completely separate `iter_mut()` loop to call `on_dispatch`/`on_undo`.

**When to use:** Any time `&mut self` methods need to be called on items in a `Vec<Box<dyn Trait>>` owned by `self`.

**Why it works:** The two loops are sequentially non-overlapping. The first loop borrows `self.behaviors` immutably; that borrow ends before the second loop takes `&mut self.behaviors`.

**Example:**
```rust
// In engine.rs dispatch(), after all state mutations and after() hooks,
// but only when the action is not cancelled and not empty:

// Pass 2: behavior lifecycle — separate iter_mut() to satisfy borrow checker
for behavior in self.behaviors.iter_mut() {
    behavior.on_dispatch();
}
```

```rust
// In engine.rs undo(), after reversing mutations:
for behavior in self.behaviors.iter_mut() {
    behavior.on_undo();
}
```

### Pattern 3: Commit-Time Reversibility Gate

**What:** At the point of committing (after `before`/`after` hooks have run, after mutations are applied), check whether the action is reversible and branch:
- Reversible: push `CommitFrame` onto undo stack as before (clear redo stack)
- Irreversible: clear the undo stack (undo barrier), do NOT push a frame

**When to use:** Every `dispatch()` call — the gate runs unconditionally when `!tx.cancelled && !tx.mutations.is_empty()`.

**Example:**
```rust
// After applying mutations and running after() hooks:
if !tx.cancelled && !tx.mutations.is_empty() {
    let is_reversible = tx.mutations.iter().all(|m| m.is_reversible());

    if tx.deterministic {
        for m in &tx.mutations {
            let h = fnv1a_hash(&m.hash_bytes());
            self.replay_hash ^= h;
            self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
        }
    }

    if is_reversible {
        let hash_after = self.replay_hash;
        self.undo_stack.push(CommitFrame {
            tx,
            state_hash_before: hash_before,
            state_hash_after: hash_after,
            _marker: std::marker::PhantomData,
        });
        self.redo_stack.clear();
    } else {
        // Undo barrier: irreversible commit clears all prior undo history
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    // Lifecycle pass — runs for both reversible and irreversible commits
    for behavior in self.behaviors.iter_mut() {
        behavior.on_dispatch();
    }
}
```

### Pattern 4: is_active() Check in Behavior Loops

**What:** Replace `if self.enabled.contains(behavior.id())` with `if behavior.is_active()` everywhere behaviors are conditionally executed.

**When to use:** All four behavior loops — `before` pass in `dispatch`, `after` pass in `dispatch`, `before` pass in `dispatch_preview`, `after` pass in `dispatch_preview`.

**Example:**
```rust
// Before (old):
for behavior in &self.behaviors {
    if self.enabled.contains(behavior.id()) {
        behavior.before(&self.state, &mut event, &mut tx);
    }
}

// After (new):
for behavior in &self.behaviors {
    if behavior.is_active() {
        behavior.before(&self.state, &mut event, &mut tx);
    }
}
```

### Pattern 5: CommitFrame Field Removal

**What:** Remove `lifetime_snapshot: HashMap<&'static str, RuleLifetime>` and `enabled_snapshot: HashSet<&'static str>` from `CommitFrame`. The `RuleLifetime` enum itself is also removed.

**Why:** Undo no longer needs to restore behavior enabled-state because `on_undo()` is called unconditionally on all behaviors — the behaviors self-manage their rollback.

**Example:**
```rust
// After (simplified CommitFrame):
#[derive(Clone)]
struct CommitFrame<S, M> {
    tx: Action<M>,
    state_hash_before: u64,
    state_hash_after: u64,
    _marker: std::marker::PhantomData<S>,
}
```

### Pattern 6: redo() Lifecycle Parity

**What:** `redo()` must also call `on_dispatch()` on all behaviors (not `on_undo()`), because redo re-applies an action forward. This keeps behavior state consistent across the full undo/redo cycle.

**Example:**
```rust
pub fn redo(&mut self) {
    if let Some(frame) = self.redo_stack.pop() {
        for m in &frame.tx.mutations {
            m.apply(&mut self.state);
        }
        self.replay_hash = frame.state_hash_after;
        self.undo_stack.push(frame);

        // Redo = forward dispatch — call on_dispatch, not on_undo
        for behavior in self.behaviors.iter_mut() {
            behavior.on_dispatch();
        }
    }
}
```

### Pattern 7: dispatch_preview — is_active() with No Lifecycle Calls

**What:** `dispatch_preview` gains `is_active()` checks but explicitly does NOT call `on_dispatch()` or `on_undo()`. Also removes restoration of `lifetime_snapshot`/`enabled_snapshot` (those fields no longer exist to restore).

**Example:**
```rust
pub fn dispatch_preview(&mut self, mut event: I, mut tx: Action<M>) {
    let state_snapshot = self.state.clone();
    let hash_snapshot = self.replay_hash;

    for behavior in &self.behaviors {
        if behavior.is_active() {
            behavior.before(&self.state, &mut event, &mut tx);
        }
    }

    if !tx.cancelled {
        for m in &tx.mutations {
            m.apply(&mut self.state);
        }
    }

    for behavior in self.behaviors.iter().rev() {
        if behavior.is_active() {
            behavior.after(&self.state, &event, &mut tx);
        }
    }

    // Restore — only state and hash; no lifetime/enabled fields remain
    self.state = state_snapshot;
    self.replay_hash = hash_snapshot;
}
```

### Anti-Patterns to Avoid

- **Restoring behavior state in dispatch_preview:** `is_active()` is `&self` — it can't mutate, so there's nothing to restore. Adding rollback code here is unnecessary.
- **Calling `on_dispatch()` inside the `before`/`after` loop:** This mixes state-observation hooks with lifecycle-mutation hooks. They must be separate passes.
- **Checking `is_active()` before calling `on_dispatch()`/`on_undo()`:** The locked decision is that these fire unconditionally on ALL behaviors, even inactive ones.
- **Conditionally clearing undo stack only when undo stack is non-empty:** The clear should always happen on irreversible commit, even if the stack is already empty. This avoids conditional logic and is correct.
- **Forgetting to clear redo_stack on irreversible commit:** An irreversible commit invalidates both undo and redo history.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Borrow conflict for `&mut self` methods on vec items | Custom unsafe pointer tricks, Rc/RefCell wrapping | Separate sequential `iter_mut()` pass | Safe, idiomatic Rust; borrow ends before second loop starts |
| Reversibility tracking | `reversible: bool` field on `Action`, or separate `IrreversibleAction` type | `tx.mutations.iter().all(|m| m.is_reversible())` at commit | Simpler, no struct change, correct per requirements |
| Behavior enabled-state | External HashMap in engine | `behavior.is_active()` method on the behavior itself | Behaviors self-manage arbitrary state (counters, toggles) without engine coupling |

**Key insight:** The borrow checker enforces what good architecture already demands: state-observation passes (immutable iteration) must be complete before lifecycle-mutation passes (mutable iteration). The separate-pass pattern is not a workaround — it's the correct separation of concerns.

## Common Pitfalls

### Pitfall 1: Empty Action No-Op Check Position

**What goes wrong:** Checking `tx.mutations.is_empty()` before running `before()` hooks. A behavior might add mutations in its `before()` hook — the check must happen AFTER all hooks have run.

**Why it happens:** Intuitive to check early as an optimization.

**How to avoid:** The emptiness check runs at the commit gate, after `before()`/`after()` loops complete.

**Warning signs:** Tests where a behavior adds mutations via `before()` show the action as a no-op when it should commit.

### Pitfall 2: on_dispatch / on_undo Firing Order on redo()

**What goes wrong:** Calling `on_undo()` instead of `on_dispatch()` in `redo()`, or calling neither.

**Why it happens:** `redo()` re-applies mutations forward; it is semantically a dispatch. Behaviors must advance their state just as they would for a real dispatch.

**How to avoid:** `redo()` calls `on_dispatch()`. Only `undo()` calls `on_undo()`.

**Warning signs:** Stateful behavior (e.g., turn counter) diverges between an engine that undo/redo cycles and one that never does.

### Pitfall 3: Forgetting to Remove HashMap/HashSet Import

**What goes wrong:** After removing `lifetimes: HashMap` and `enabled: HashSet` from `Engine`, the `use std::collections::{HashMap, HashSet}` at the top of `engine.rs` becomes a dead import. Rust warns on `dead_code` / `unused_import`.

**Why it happens:** Easy to forget import cleanup after struct field removal.

**How to avoid:** After removing fields, let the compiler guide — `unused import` warning flags it immediately. Also check `dispatch_preview` which previously restored `lifetime_snapshot` and `enabled_snapshot`.

**Warning signs:** Compiler warning `unused import: std::collections::HashMap`.

### Pitfall 4: is_active() in Behavior Loop vs on_dispatch() Pass

**What goes wrong:** Adding an `if behavior.is_active()` guard before `on_dispatch()` calls, which would prevent sleeping behaviors from tracking dispatches.

**Why it happens:** Symmetry instinct — `is_active()` guards `before()`/`after()`, so it seems natural to guard lifecycle hooks too.

**How to avoid:** The locked decision: `on_dispatch()`/`on_undo()` fire unconditionally. No `is_active()` check in the lifecycle pass.

**Warning signs:** A test for "behavior deactivates after N dispatches using on_dispatch counter" fails to track dispatches while inactive.

### Pitfall 5: Undo of Irreversible Commit

**What goes wrong:** After clearing the undo stack on an irreversible commit, `engine.undo()` is called — it must be a no-op (undo stack is empty).

**Why it happens:** Existing `undo()` implementation already handles empty stack correctly (early return `if let Some(frame)`). No extra guard needed.

**How to avoid:** No change to `undo()` needed for this case — just confirm existing behavior is preserved.

**Warning signs:** `undo()` panics or returns an error on empty stack (it should silently no-op).

### Pitfall 6: CommitFrame Clone Derivation After Field Removal

**What goes wrong:** Removing `HashMap`/`HashSet` fields from `CommitFrame` while `#[derive(Clone)]` remains — this is fine since `Action<M>` is already `Clone`. But verify the removed fields' types (`HashMap`, `HashSet`) no longer appear anywhere that would prevent compilation.

**Why it happens:** Mechanical field removal sometimes misses usage sites.

**How to avoid:** Grep for `lifetime_snapshot` and `enabled_snapshot` after removal; also for `RuleLifetime` which is fully removed.

**Warning signs:** Compilation error referencing removed field names.

### Pitfall 7: dispatch_preview Snapshot Restoration

**What goes wrong:** `dispatch_preview` previously restored `self.lifetimes = lifetime_snapshot` and `self.enabled = enabled_snapshot`. After removing those fields, these lines must be deleted — they no longer exist.

**Why it happens:** Mechanical change may miss these restoration lines.

**How to avoid:** The snapshot/restore block in `dispatch_preview` shrinks to only two lines: restore `self.state` and `self.replay_hash`.

**Warning signs:** Compilation error on deleted field access.

## Code Examples

Verified patterns from code analysis of current codebase:

### Current dispatch() structure (before changes)
```rust
pub fn dispatch(&mut self, mut event: I, mut tx: Action<M>) {
    let hash_before = self.replay_hash;
    let lifetime_snapshot = self.lifetimes.clone();     // REMOVE
    let enabled_snapshot = self.enabled.clone();         // REMOVE

    for behavior in &self.behaviors {
        if self.enabled.contains(behavior.id()) {        // CHANGE to is_active()
            behavior.before(&self.state, &mut event, &mut tx);
        }
    }

    if !tx.cancelled {
        for m in &tx.mutations {
            m.apply(&mut self.state);
        }
    }

    for behavior in self.behaviors.iter().rev() {
        if self.enabled.contains(behavior.id()) {        // CHANGE to is_active()
            behavior.after(&self.state, &event, &mut tx);
        }
    }

    if !tx.cancelled {                                   // ADD: && !tx.mutations.is_empty()
        if tx.deterministic {
            for m in &tx.mutations {
                let h = fnv1a_hash(&m.hash_bytes());
                self.replay_hash ^= h;
                self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
            }
        }

        let hash_after = self.replay_hash;

        self.undo_stack.push(CommitFrame {               // GATE behind is_reversible check
            tx,
            state_hash_before: hash_before,
            state_hash_after: hash_after,
            lifetime_snapshot,                           // REMOVE field
            enabled_snapshot,                            // REMOVE field
            _marker: std::marker::PhantomData,
        });

        self.redo_stack.clear();
    }
    // ADD: on_dispatch() pass (unconditional, all behaviors, after commit gate)
}
```

### Target dispatch() structure (after changes)
```rust
pub fn dispatch(&mut self, mut event: I, mut tx: Action<M>) {
    let hash_before = self.replay_hash;

    for behavior in &self.behaviors {
        if behavior.is_active() {
            behavior.before(&self.state, &mut event, &mut tx);
        }
    }

    if !tx.cancelled {
        for m in &tx.mutations {
            m.apply(&mut self.state);
        }
    }

    for behavior in self.behaviors.iter().rev() {
        if behavior.is_active() {
            behavior.after(&self.state, &event, &mut tx);
        }
    }

    if !tx.cancelled && !tx.mutations.is_empty() {
        let is_reversible = tx.mutations.iter().all(|m| m.is_reversible());

        if tx.deterministic {
            for m in &tx.mutations {
                let h = fnv1a_hash(&m.hash_bytes());
                self.replay_hash ^= h;
                self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
            }
        }

        if is_reversible {
            let hash_after = self.replay_hash;
            self.undo_stack.push(CommitFrame {
                tx,
                state_hash_before: hash_before,
                state_hash_after: hash_after,
                _marker: std::marker::PhantomData,
            });
            self.redo_stack.clear();
        } else {
            self.undo_stack.clear();
            self.redo_stack.clear();
        }

        for behavior in self.behaviors.iter_mut() {
            behavior.on_dispatch();
        }
    }
}
```

### Target undo() structure (after changes)
```rust
pub fn undo(&mut self) {
    if let Some(frame) = self.undo_stack.pop() {
        for m in frame.tx.mutations.iter().rev() {
            m.undo(&mut self.state);
        }

        self.replay_hash = frame.state_hash_before;
        // REMOVED: self.lifetimes = frame.lifetime_snapshot.clone();
        // REMOVED: self.enabled = frame.enabled_snapshot.clone();

        self.redo_stack.push(frame);

        // Lifecycle: unconditional, all behaviors
        for behavior in self.behaviors.iter_mut() {
            behavior.on_undo();
        }
    }
}
```

### Stateful Behavior Pattern (sleeping behavior model)
```rust
// A behavior that fires for exactly N dispatches, then deactivates.
// Tracks dispatches even while inactive via on_dispatch().
struct ChargeBehavior {
    charges: u32,
}

impl Behavior<MyState, MyMutation, MyEvent, u8> for ChargeBehavior {
    fn id(&self) -> &'static str { "charge_behavior" }
    fn priority(&self) -> u8 { 10 }

    fn is_active(&self) -> bool {
        self.charges > 0
    }

    fn on_dispatch(&mut self) {
        if self.charges > 0 {
            self.charges -= 1;
        }
    }

    fn on_undo(&mut self) {
        self.charges += 1;
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lifetimes: HashMap<&'static str, RuleLifetime>` + `enabled: HashSet` in Engine | `behavior.is_active()` per dispatch | Phase 5 | Behaviors self-manage arbitrary lifecycle state; engine is simpler |
| `tx.irreversible: bool` field on `Action` (removed in Phase 4) | `tx.mutations.iter().all(|m| m.is_reversible())` at commit | Phase 5 | Per-mutation reversibility — finer granularity, no struct field |
| `CommitFrame` snapshots behavior enabled-state for undo restore | `on_undo()` called unconditionally on all behaviors | Phase 5 | No snapshot overhead, behaviors own their undo rollback logic |
| `RuleLifetime::Permanent` (only remaining variant after Phase 4 cleanup) | Removed entirely | Phase 5 | Dead code fully eliminated |

**Deprecated/outdated:**
- `RuleLifetime` enum: removed entirely in Phase 5 (Phase 4 already removed Turns/Triggers variants)
- `lifetime_snapshot` field on `CommitFrame`: removed in Phase 5
- `enabled_snapshot` field on `CommitFrame`: removed in Phase 5
- `self.lifetimes = frame.lifetime_snapshot.clone()` in `undo()`: removed in Phase 5
- `self.enabled = frame.enabled_snapshot.clone()` in `undo()`: removed in Phase 5
- `self.enabled.insert(id)` and `self.lifetimes.insert(id, ...)` in `add_behavior()`: removed in Phase 5

## Open Questions

1. **Hash update ordering for irreversible actions**
   - What we know: REV-03 clears the undo stack on irreversible commit. The replay hash still updates for deterministic mutations (hash is about replay verification, not undoability).
   - What's unclear: Should `replay_hash` update before or after the reversibility check? Answer: before — hash update and undo stack management are independent concerns. The hash records what happened; the undo stack records what can be undone. Code example above shows hash update before reversibility branch.
   - Recommendation: Update hash before the reversibility branch. This is correct behavior — the replay hash is a forward-only fingerprint.

2. **on_dispatch vs on_undo call site when action has no mutations (empty/cancelled)**
   - What we know: Empty and cancelled actions are silently no-op'd — no `on_dispatch` call.
   - What's confirmed: The `on_dispatch()` pass is inside the `if !tx.cancelled && !tx.mutations.is_empty()` gate. This is the locked decision.
   - Recommendation: No ambiguity — implement exactly as specified.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | proptest 1.10 |
| Config file | none — inline `#[cfg(test)]` in each module file |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo run --example backgammon && cargo run --example tictactoe` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REV-01 | `is_reversible()` default returns `true` | unit | `cargo test mutation` | ❌ Wave 0 |
| REV-02 | Action reversible iff all mutations reversible | proptest | `cargo test prop` | ❌ Wave 0 |
| REV-03 | Irreversible commit clears undo stack | proptest | `cargo test prop` | ❌ Wave 0 |
| REV-04 | Reversible commits push CommitFrame; undo/redo work | proptest | `cargo test prop_01` | ✅ (PROP-01 covers roundtrip) |
| LIFE-01 | `is_active()` default returns `true` | unit | `cargo test behavior` | ❌ Wave 0 |
| LIFE-02 | `on_dispatch()` called after committed action | unit | `cargo test engine` | ❌ Wave 0 |
| LIFE-03 | `on_undo()` called on all behaviors on undo | unit | `cargo test engine` | ❌ Wave 0 |
| LIFE-04 | `add_behavior` takes no lifetime parameter | compile (smoke test) | `cargo test` | ✅ (already matches) |
| LIFE-05 | No `lifetimes`/`enabled` fields on Engine | compile | `cargo test` | ✅ (after field removal) |
| LIFE-06 | on_dispatch/on_undo called in separate pass; no borrow conflict | compile + unit | `cargo test` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo run --example backgammon && cargo run --example tictactoe`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/mutation.rs` — unit test for `is_reversible()` default returns `true`
- [ ] `src/behavior.rs` — unit test for `is_active()` default, `on_dispatch()`/`on_undo()` default no-ops
- [ ] `src/engine.rs` — unit test for stateful behavior using `on_dispatch` counter (deactivates after N)
- [ ] `src/engine.rs` — unit test for `on_undo()` called on all behaviors including inactive
- [ ] `src/engine.rs` — proptest for irreversible commit clears undo stack
- [ ] `src/engine.rs` — proptest for reversible commits after irreversible are individually undoable

Note: REV-04 is covered by existing PROP-01 (undo roundtrip) — no new test needed for the base case. New tests target the reversibility gate specifically (TEST-02, TEST-03 in REQUIREMENTS.md, deferred to Phase 6).

## Sources

### Primary (HIGH confidence)

- Rust reference — trait default methods: stable since Rust 1.0, backward-compatible API extension pattern
- Code analysis of `src/engine.rs` — direct inspection of current `dispatch`, `undo`, `redo`, `dispatch_preview`, `add_behavior` implementations
- Code analysis of `src/mutation.rs`, `src/behavior.rs`, `src/action.rs` — direct inspection of current trait signatures
- `05-CONTEXT.md` — locked user decisions forming the implementation specification
- `REQUIREMENTS.md` — REV-01 through LIFE-06 requirement definitions

### Secondary (MEDIUM confidence)

- Rust borrow checker behavior for sequential `iter()` / `iter_mut()` loops on same field — well-established pattern; separate loops do not create overlapping borrows

### Tertiary (LOW confidence)

- None — all findings are grounded in direct codebase inspection and Rust language specification

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; Rust default methods are stable language features
- Architecture: HIGH — patterns derived from direct codebase inspection and locked CONTEXT.md decisions
- Pitfalls: HIGH — identified from mechanical analysis of exact code paths being changed
- Code examples: HIGH — derived from reading actual current code; target examples follow directly from locked decisions

**Research date:** 2026-03-10
**Valid until:** N/A — research is codebase-specific, not ecosystem-dependent; valid until Phase 5 files change
