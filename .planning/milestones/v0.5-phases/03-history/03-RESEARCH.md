# Phase 3: History - Research

**Researched:** 2026-03-13
**Domain:** Rust snapshot-based undo/redo for a zero-dependency game engine
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- `HistoryDisallowed` enum lives in `src/outcome.rs` alongside `Outcome` and `EngineError`
- Variants are exactly `NothingToUndo` and `NothingToRedo`
- `undo()` returns `Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError>`
- `redo()` returns `Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError>`
- Separate variants per reason — callers can pattern-match on specific reasons
- A successful `Committed` dispatch clears the redo stack
- `NoChange` dispatches do NOT clear the redo stack
- `Disallowed` / `Aborted` / `InvalidInput` do NOT clear the redo stack
- `undo_depth() -> usize` and `redo_depth() -> usize` ship in Phase 3
- `HistoryDisallowed` is re-exported at crate root via `pub use crate::outcome::HistoryDisallowed`
- `HistoryDisallowed` is NOT `#[non_exhaustive]` — two variants are a complete stable contract

### Claude's Discretion

- Internal undo stack layout: `Vec<(E::State, Frame<E>)>` tuples vs. parallel `Vec<E::State>` and `Vec<Frame<E>>` vectors
- Derives on `HistoryDisallowed` (reasonable: `Debug, Clone, Copy, PartialEq, Eq`)
- Exact rustdoc wording for `undo()`, `redo()`, `undo_depth()`, `redo_depth()`

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HIST-01 | `undo()` returns `Result<Outcome, EngineError>` with `Undone(frame)` or `Disallowed(NothingToUndo)` | Stack layout decision determines what to pop and return; `Outcome::Undone` variant already exists |
| HIST-02 | `redo()` returns `Result<Outcome, EngineError>` with `Redone(frame)` or `Disallowed(NothingToRedo)` | Mirror of HIST-01; `Outcome::Redone` variant already exists |
| HIST-03 | Committing an irreversible transition erases both undo and redo stacks | `Frame.reversibility` field already stored; check on `Committed` path in `dispatch()` |
| HIST-04 | Undo stores a full state snapshot per frame — no reverse-diff trait requirement on user types | Snapshot approach chosen in design; `E::State: Clone` bound already present via `Cow` usage |
</phase_requirements>

---

## Summary

Phase 3 wires snapshot-based undo/redo into the already-built `Engine<E>`. The implementation is deliberately simple: store a full `E::State` clone before each `Committed` dispatch, push it to `undo_stack`, and pop it back out on `undo()`. The redo stack mirrors this in reverse.

The codebase from Phase 2 is already well-positioned for this work. `undo_stack` and `redo_stack` placeholder fields (`Vec<E::State>`) exist on `Engine<E>` with `#[allow(dead_code)]`. `Outcome::Undone(F)` and `Outcome::Redone(F)` are already defined in `outcome.rs`. The `Frame` struct already stores `reversibility`. `E::State: Clone` is already implied because `Cow<'_, E::State>` requires `Clone`.

The only new type is `HistoryDisallowed`, and it adds two variants. The only structural decision at Claude's discretion is whether to store state snapshots as a single `Vec<(E::State, Frame<E>)>` tuple vec or as parallel `Vec<E::State>` / `Vec<Frame<E>>` vecs. This research recommends the tuple approach for locality and to avoid index-synchronization bugs.

**Primary recommendation:** Use `Vec<(E::State, Frame<E>)>` for both undo and redo stacks. Each entry bundles the pre-transition snapshot with the committed frame. `undo()` pops from undo_stack, restores state, pushes `(current_state, frame)` onto redo_stack. `redo()` reverses this exactly.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std | — | `Vec`, `Clone`, ownership, `Cow` | Zero-dep library; only std allowed |

No external libraries. The entire history mechanism uses Rust's ownership model.

### Supporting

None. This is internal engine wiring.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Full state snapshot (clone) | Reverse-diff / undo-patch trait | Simpler for users — no `Reversible` trait burden; uses more memory for large states (flagged acceptable for MVP) |
| `Vec<(E::State, Frame<E>)>` tuples | Parallel `Vec<E::State>` + `Vec<Frame<E>>` | Tuple approach: single push/pop, no index synchronization risk, slightly better cache locality. Parallel vecs: marginally smaller push overhead. Tuple approach is lower risk. |

---

## Architecture Patterns

### Recommended Stack Layout

Store both undo and redo stacks as paired `(state_snapshot, frame)` tuples:

```rust
undo_stack: Vec<(E::State, Frame<E>)>,
redo_stack: Vec<(E::State, Frame<E>)>,
```

**Undo stack semantics:** each entry is `(state_before_that_dispatch, frame_of_that_dispatch)`. The top entry is the most recently committed frame.

**Redo stack semantics:** each entry is `(state_before_that_redo_or_undo, frame)`. Mirrors undo stack after an undo.

### Pattern 1: dispatch() — push to undo_stack on Committed, clear redo_stack

**What:** Before atomically committing, capture `self.state.clone()` as the pre-transition snapshot. After successful commit, push `(snapshot, frame.clone())` to `undo_stack`. If `Irreversible`, additionally truncate both stacks.

**When to use:** Every time `dispatch()` reaches the `Committed` path (non-empty diffs).

**Example:**
```rust
// Inside dispatch(), at the Committed path:
let prior_state = self.state.clone(); // snapshot before commit
self.state = working.into_owned();    // atomic commit

let frame = Frame { input, diffs, traces, reversibility };

// Phase 3 additions:
self.redo_stack.clear();              // new branch erases future
self.undo_stack.push((prior_state, frame.clone()));

if reversibility == Reversibility::Irreversible {
    self.undo_stack.clear();
    self.redo_stack.clear();
}

Ok(Outcome::Committed(frame))
```

Note: clear redo before push, then conditional undo clear. For Irreversible, the newly-pushed entry is also erased — the commit goes through (state changes), but history is wiped clean.

### Pattern 2: undo()

**What:** Pop from `undo_stack`, restore state, push current state + frame to `redo_stack`, return `Undone(frame)`.

**When to use:** Called directly by the engine user.

**Example:**
```rust
pub fn undo(
    &mut self,
) -> Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError> {
    match self.undo_stack.pop() {
        None => Ok(Outcome::Disallowed(HistoryDisallowed::NothingToUndo)),
        Some((prior_state, frame)) => {
            let current_state = std::mem::replace(&mut self.state, prior_state);
            self.redo_stack.push((current_state, frame.clone()));
            Ok(Outcome::Undone(frame))
        }
    }
}
```

### Pattern 3: redo()

**What:** Pop from `redo_stack`, restore state, push current state + frame to `undo_stack`, return `Redone(frame)`.

**Example:**
```rust
pub fn redo(
    &mut self,
) -> Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError> {
    match self.redo_stack.pop() {
        None => Ok(Outcome::Disallowed(HistoryDisallowed::NothingToRedo)),
        Some((prior_state, frame)) => {
            let current_state = std::mem::replace(&mut self.state, prior_state);
            self.undo_stack.push((current_state, frame.clone()));
            Ok(Outcome::Redone(frame))
        }
    }
}
```

### Pattern 4: HistoryDisallowed definition

**What:** New enum in `src/outcome.rs`. Two stable variants.

**Example:**
```rust
/// The reason an [`Engine::undo`] or [`Engine::redo`] call was disallowed.
///
/// Returned as `Disallowed(reason)` from [`Engine::undo`] and [`Engine::redo`].
/// This enum is not `#[non_exhaustive]` — its two variants are a complete
/// stable public contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryDisallowed {
    /// `undo()` was called but the undo stack is empty.
    NothingToUndo,
    /// `redo()` was called but the redo stack is empty.
    NothingToRedo,
}
```

### Pattern 5: undo_depth / redo_depth

**What:** Simple delegating methods. Near-zero cost.

**Example:**
```rust
pub fn undo_depth(&self) -> usize {
    self.undo_stack.len()
}

pub fn redo_depth(&self) -> usize {
    self.redo_stack.len()
}
```

### Recommended Project Structure (unchanged)

```
src/
├── spec.rs          # EngineSpec trait
├── behavior.rs      # Behavior trait + BehaviorResult
├── apply.rs         # Apply trait (Diff → state mutation)
├── reversibility.rs # Reversibility enum
├── outcome.rs       # Outcome, Frame, EngineError, HistoryDisallowed (new)
├── engine.rs        # Engine<E> — dispatch, undo, redo, depth queries (modified)
└── lib.rs           # pub use re-exports (add HistoryDisallowed)
```

No new modules. `HistoryDisallowed` joins `outcome.rs`.

### Anti-Patterns to Avoid

- **Storing frames separately from snapshots:** Parallel `Vec<E::State>` and `Vec<Frame<E>>` creates index-synchronization risk. Any bug that pushes to one but not the other corrupts history silently. Tuple vec eliminates this class of bug.
- **Cloning state after commit instead of before:** The snapshot must be of the state _before_ the dispatch, not after. Post-commit clone would store the wrong state (the committed state, not the prior state).
- **Clearing redo before checking for Irreversible:** When dispatch is Irreversible, both stacks must be cleared. The newly-pushed entry on undo_stack must also be cleared. Order matters: commit first, then clear both stacks including what was just pushed.
- **Pushing to undo_stack on non-Committed outcomes:** `NoChange`, `Disallowed`, `Aborted`, `InvalidInput` must never modify the history stacks.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Stack pop + state restore | Custom state-diff reverse trait | Full clone snapshot | Reverse-diff is complex, requires user types to impl `Reversible`, and has edge cases with non-invertible diffs (e.g. dice outcomes). Snapshot is simpler and correct. |
| Tracking dirty history | Complex dirty flags | `Vec::clear()` | Single-timeline undo: clear-on-new-commit is sufficient. No need for epoch counters or generation IDs. |

**Key insight:** Snapshot undo is the correct approach for a library where user diff types have no reversal semantics. It trades memory for simplicity and correctness. This is the right MVP tradeoff.

---

## Common Pitfalls

### Pitfall 1: Wrong snapshot timing

**What goes wrong:** State is cloned after `self.state = working.into_owned()` instead of before. The "prior state" stored is actually the new committed state — undo restores to where you already are.

**Why it happens:** It feels natural to clone `self.state` after updating it. But the snapshot needs to be the _pre-commit_ value.

**How to avoid:** Capture `let prior_state = self.state.clone()` before the `self.state = working.into_owned()` line.

**Warning signs:** `undo()` returns `Undone(frame)` but state is unchanged after the call.

### Pitfall 2: Irreversible clear order

**What goes wrong:** Clearing undo_stack before pushing the new entry, then forgetting to clear the push — or clearing only redo but not undo.

**Why it happens:** The requirement is subtle: the irreversible commit _does_ change state (commit goes through), but then both history stacks are nuked — including the entry just pushed.

**How to avoid:** Push to undo_stack unconditionally on Committed, then `if reversibility == Irreversible { self.undo_stack.clear(); self.redo_stack.clear(); }`.

**Warning signs:** After an irreversible commit, `undo_depth() == 1` instead of `0`.

### Pitfall 3: redo_stack not cleared on Committed

**What goes wrong:** New dispatch after undo does not clear redo_stack. Caller can redo to a future that is no longer reachable from the new branch.

**Why it happens:** It's easy to forget the redo-clear in the happy-path dispatch flow.

**How to avoid:** Always `self.redo_stack.clear()` when a `Committed` outcome is produced, before or after pushing to undo_stack (order between these two doesn't matter for correctness).

**Warning signs:** After undo + new dispatch, `redo_depth() > 0`.

### Pitfall 4: Clone bound on E::State

**What goes wrong:** Compiler error — `E::State` doesn't have a `Clone` bound visible at the undo/redo call sites in `engine.rs`.

**Why it happens:** The existing `Cow` usage in `dispatch()` requires `Clone` implicitly through the `ToOwned` bound on `Cow`, but this is not an explicit named bound on `EngineSpec::State`. Adding the snapshot clone makes this bound explicit and required.

**How to avoid:** Add `E::State: Clone` to the `EngineSpec` trait definition (in `spec.rs`) as a supertrait bound or as an associated type constraint. Confirm that the Phase 1 `EngineSpec` definition either already has it or that it flows through `ToOwned`. Given `Cow<'_, E::State>` requires `E::State: ToOwned + Clone` already, the bound likely compiles — but verify with `cargo check` immediately after adding snapshot logic.

**Warning signs:** `the trait bound E::State: Clone is not satisfied` on `self.state.clone()` in `engine.rs`.

### Pitfall 5: HistoryDisallowed asymmetry with dispatch Disallowed

**What goes wrong:** Caller confusion — `dispatch()` returns `Disallowed(E::NonCommittedInfo)` but `undo()` / `redo()` return `Disallowed(HistoryDisallowed)`. The generic parameter differs between the two uses.

**Why it happens:** `Outcome<F, N>` is parameterized — the `N` for `undo`/`redo` calls is `HistoryDisallowed`, not `E::NonCommittedInfo`. This is intentional but surprising at first glance.

**How to avoid:** Document this asymmetry explicitly in rustdoc for `undo()` and `redo()`. The return type spells it out: `Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError>`.

---

## Code Examples

Verified patterns from codebase inspection:

### Existing Field Declarations (from engine.rs, Phase 2 state)

```rust
// Phase 3 removes #[allow(dead_code)] and changes Vec<E::State> to Vec<(E::State, Frame<E>)>
#[allow(dead_code)]
undo_stack: Vec<E::State>,
#[allow(dead_code)]
redo_stack: Vec<E::State>,
```

Phase 3 replaces these with:
```rust
undo_stack: Vec<(E::State, Frame<E>)>,
redo_stack: Vec<(E::State, Frame<E>)>,
```

And updates `Engine::new()` accordingly:
```rust
Engine {
    state,
    behaviors,
    undo_stack: Vec::new(),
    redo_stack: Vec::new(),
}
```

(`Vec::new()` works for both the old `Vec<E::State>` and the new `Vec<(E::State, Frame<E>)>`.)

### Existing Outcome variants already defined (from outcome.rs)

```rust
// These already exist — Phase 3 just uses them:
Undone(F),
Redone(F),
Disallowed(N),  // N will be HistoryDisallowed for undo/redo calls
```

### std::mem::replace usage for efficient state swap

```rust
// Avoids double-clone: replace returns old value, sets new value atomically.
// Source: std::mem::replace docs
let current_state = std::mem::replace(&mut self.state, prior_state);
```

This pattern is idiomatic for "swap and capture previous" in Rust without needing a temporary clone.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Behavior state stored engine-internally (v0.4.0) | Behavior state in main state tree | v0.5.0 design | Snapshot undo is correct — captures behavior state automatically |
| Address-based ordering tiebreaker (v0.4.0) | `(order_key, name)` ordering | v0.5.0 design | Deterministic frame content — snapshots replay consistently |
| Public undo/redo stack fields (v0.4.0) | Private stacks with `undo_depth()` / `redo_depth()` query API | v0.5.0 + Phase 3 ERGO-01 promotion | Callers can query depth without triggering operations |

**Deprecated/outdated:**
- `Vec<E::State>` for stacks: replaced by `Vec<(E::State, Frame<E>)>` tuples in Phase 3
- `#[allow(dead_code)]` on stack fields: removed when Phase 3 activates them

---

## Open Questions

1. **`E::State: Clone` bound location**
   - What we know: `Cow<'_, E::State>` in `dispatch()` already requires `E::State: ToOwned`, which implies `Clone` for most types. The `Clone` impl is present for all practical `E::State` types.
   - What's unclear: Whether `spec.rs`'s `EngineSpec` trait already has an explicit `Clone` bound on `State`, or whether it flows implicitly through Cow's bounds.
   - Recommendation: In Wave 1 Task 1, run `cargo check` immediately after adding `self.state.clone()` to confirm this compiles. If it fails, add `type State: Clone + ...` to `EngineSpec` in `spec.rs`.

2. **Frame clone in undo/redo push**
   - What we know: `Frame<E>` has `#[derive(Clone)]`. Pushing to redo_stack from undo requires `frame.clone()` since `frame` is moved into `Outcome::Undone(frame)`.
   - What's unclear: Whether `E::Diff: Clone` and `E::Trace: Clone` are already bounded on `EngineSpec`. `Frame<E>` derives `Clone` only if `E::Input`, `E::Diff`, `E::Trace` all implement `Clone`.
   - Recommendation: Read `src/spec.rs` during implementation (Wave 1) and verify `EngineSpec` associated type bounds include `Clone` where needed. The existing `#[derive(Clone, PartialEq)]` on `Frame<E>` implies these bounds already exist.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `proptest 1.10` (dev-dependency) |
| Config file | None — Cargo handles test discovery |
| Quick run command | `cargo test -p herdingcats` |
| Full suite command | `cargo test -p herdingcats` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HIST-01 | `undo()` on non-empty stack returns `Undone(frame)` | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-01 | `undo()` on empty stack returns `Disallowed(NothingToUndo)` | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-02 | `redo()` on non-empty stack returns `Redone(frame)` | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-02 | `redo()` on empty stack returns `Disallowed(NothingToRedo)` | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-03 | Irreversible commit clears undo_stack: `undo_depth() == 0` | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-03 | Irreversible commit clears redo_stack: `redo_depth() == 0` | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-04 | Undo restores exact prior state snapshot | unit | `cargo test -p herdingcats engine::tests` | Wave 0: add tests to `engine.rs` |
| HIST-04 | No `Reversible` trait required on diff types | compile | `cargo build -p herdingcats` | Structural — passes if design correct |

All tests live in `#[cfg(test)]` modules inside `src/engine.rs` and `src/outcome.rs`, consistent with the pattern established in Phases 1 and 2.

### Sampling Rate

- **Per task commit:** `cargo test -p herdingcats`
- **Per wave merge:** `cargo test -p herdingcats`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/engine.rs` test module — add undo/redo/depth/irreversible tests
- [ ] `src/outcome.rs` test module — add `HistoryDisallowed` variant constructability tests

*(No new test files — all tests go in existing `#[cfg(test)]` modules per project convention.)*

---

## Sources

### Primary (HIGH confidence)

- Direct source read: `src/engine.rs` — current Engine struct, dispatch() implementation, placeholder undo/redo fields
- Direct source read: `src/outcome.rs` — Outcome variants (Undone, Redone, Disallowed already exist), EngineError, Frame definition
- Direct source read: `src/lib.rs` — re-export pattern for crate root
- Direct source read: `src/spec.rs` (indirectly via engine.rs imports) — EngineSpec trait
- Direct source read: `.planning/phases/03-history/03-CONTEXT.md` — all locked user decisions
- Direct source read: `ARCHITECTURE.md` — canonical design, undo/redo section, irreversibility policy
- Direct source read: `Cargo.toml` — zero prod-dependencies, proptest in dev-dependencies
- Rust std docs: `std::mem::replace`, `std::borrow::Cow`, `Vec::clear`, `Vec::pop`, `Vec::push`

### Secondary (MEDIUM confidence)

None required — this phase is entirely in-codebase wiring with no external libraries.

### Tertiary (LOW confidence)

None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero-dep library, all Rust std, confirmed by Cargo.toml
- Architecture: HIGH — derived directly from existing code + locked CONTEXT.md decisions
- Pitfalls: HIGH — derived from code inspection, v0.4.0 known issues in MEMORY.md, and Rust ownership patterns

**Research date:** 2026-03-13
**Valid until:** Stable — this is self-contained Rust implementation with no external dependencies
