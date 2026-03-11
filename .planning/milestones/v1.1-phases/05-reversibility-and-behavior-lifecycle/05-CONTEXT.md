# Phase 5: Reversibility and Behavior Lifecycle - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Add `is_reversible()` to the `Mutation` trait, derive `Action` reversibility from its mutations at commit time, enforce an undo barrier when committing an irreversible action, and replace the `RuleLifetime`/`enabled` HashMap mechanism with per-behavior self-managed lifecycle methods (`is_active`, `on_dispatch`, `on_undo`). No new examples or tests — that is Phase 6.

</domain>

<decisions>
## Implementation Decisions

### Inactive behavior hooks

- Inactive behaviors (`is_active()` returns false) skip `before()` and `after()` hooks
- Inactive behaviors **still receive** `on_dispatch()` and `on_undo()` calls — they are "sleeping," not removed. A behavior can track dispatches while inactive and self-reactivate.
- `on_undo()` fires on **all behaviors unconditionally** when `engine.undo()` is called — no need to snapshot which behaviors were active at dispatch time. Simplifies undo and removes the need for `enabled_snapshot` in `CommitFrame`.

### `before()`/`after()` hook mutability

- Both hooks remain `&self` (immutable self) — no change from Phase 4.
- Clean separation of concerns: `before`/`after` = observe events and inject/cancel mutations; `on_dispatch`/`on_undo` = update internal behavior state.
- A behavior that needs to modify its own state in response to a dispatch uses `on_dispatch`, not `before`.
- Reversibility is a **commit-time concern only** — `before`/`after` hooks do not need to know or care about `is_reversible()`.

### `dispatch_preview` semantics

- `dispatch_preview` checks `is_active()` before firing `before`/`after` hooks — same as a real dispatch — so the preview accurately reflects what `dispatch()` would do.
- `dispatch_preview` **never calls** `on_dispatch()` or `on_undo()` — it is a pure dry run. Behavior state is never mutated in preview.
- Return type unchanged: `dispatch_preview` stays `()`.

### Empty and cancelled actions

- `Action::default()` (no mutations) is the canonical no-op value — already implemented from Phase 4.
- **Empty actions are silently no-op'd**: if `tx.mutations.is_empty()` after all `before()` hooks run, the engine skips the CommitFrame push and does not call `on_dispatch()`. No error signaled.
- **Cancelled actions are silently no-op'd**: same path — no frame, no `on_dispatch()`.
- `dispatch()` return type stays `()` — "no change" is implicit.

### All-inactive engine state

- No safeguard. An engine where all behaviors are inactive will silently no-op all dispatches.
- Expected usage: developers always register at least one behavior. All-inactive is either a valid end state (game over) or a developer mistake — not the engine's concern.

### `CommitFrame` cleanup

- Remove `lifetime_snapshot: HashMap<&'static str, RuleLifetime>` — no longer needed.
- Remove `enabled_snapshot: HashSet<&'static str>` — replaced by unconditional `on_undo()` calls on all behaviors.
- `RuleLifetime` enum and `lifetimes`/`enabled` fields on `Engine` are also removed (LIFE-05).

### Claude's Discretion

- Borrow conflict resolution in the `on_dispatch`/`on_undo` pass — behaviors need `&mut self` while the engine iterates. Implementation approach (index-based iteration, `iter_mut()` in a separate pass, etc.) is Claude's choice.
- `DispatchError` vs outcome type — if any signaling is added in future phases, shape is Claude's discretion.
- Whether `dispatch_preview` rolls back `is_active()` state (it can't mutate behavior state anyway since `is_active(&self)` is immutable — no rollback needed).

</decisions>

<specifics>
## Specific Ideas

- The `Action::default()` as canonical no-op was explicitly confirmed — it's already the right primitive. Don't add a separate "NoChange" type or change dispatch's return type.
- "All behaviors inactive = dead engine" is an expected developer responsibility, like registering behaviors at all. The engine does not guard against this.
- The sleeping behavior model (inactive but still receives on_dispatch/on_undo) enables turn-limited behaviors like "fires 3 times then stops" without storing lifecycle state outside the behavior struct itself.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets

- `src/mutation.rs` — `pub trait Mutation<S>: Clone` — add `fn is_reversible(&self) -> bool { true }` as a default method here
- `src/behavior.rs` — `pub trait Behavior<S, M, I, P>` — add `fn is_active(&self) -> bool { true }`, `fn on_dispatch(&mut self) {}`, `fn on_undo(&mut self) {}` as default methods
- `src/action.rs` — `pub struct Action<M>` with `Default` already implemented — no struct changes needed; reversibility derived from mutations at commit time in engine
- `src/engine.rs` — `CommitFrame<S, M>` — remove `lifetime_snapshot` and `enabled_snapshot` fields; `Engine` — remove `lifetimes: HashMap` and `enabled: HashSet` fields

### Established Patterns

- Section banners (`// ============================================================`) in every file — required
- `where` clause on separate lines from `impl`/`fn` signatures
- `#[derive(Clone)]` on snapshotted types — `CommitFrame` keeps `Clone`
- `mod tests` (unit) + `mod props` (proptest) sibling pattern in `engine.rs`
- All undo property tests assert both `engine.state` (read) AND `engine.replay_hash()` — locked from Phase 2
- `before`/`after` hooks already use `&self` — keep as-is

### Integration Points

- `engine.dispatch()` — add empty/cancelled no-op check, add separate `iter_mut()` pass for `on_dispatch()` after state mutation pass
- `engine.undo()` — replace `self.lifetimes = frame.lifetime_snapshot.clone()` + `self.enabled = frame.enabled_snapshot.clone()` with `on_undo()` call on all behaviors
- `engine.redo()` — same pattern as undo: replace snapshot restore with `on_dispatch()` call on all behaviors
- `engine.add_behavior()` — remove `self.enabled.insert(id)` and `self.lifetimes.insert(id, RuleLifetime::Permanent)`
- `engine.dispatch_preview()` — add `is_active()` check in behavior loop; do NOT add `on_dispatch()` call

</code_context>

<deferred>
## Deferred Ideas

- `DispatchOutcome` return type from `dispatch()` — discussed, decided against for now. `Action::default()` is the no-op primitive; return type stays `()`.
- Safeguard for all-inactive engine state — explicitly deferred. Developer responsibility.

</deferred>

---

*Phase: 05-reversibility-and-behavior-lifecycle*
*Context gathered: 2026-03-11*
