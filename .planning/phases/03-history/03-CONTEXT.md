# Phase 3: History - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Add undo/redo to `Engine<E>`: callers can reverse and re-apply committed transitions, and committing an irreversible input permanently clears all history. Dispatch, core types, and examples are adjacent phases — this phase wires snapshot-based history into the already-built `Engine<E>` struct.

</domain>

<decisions>
## Implementation Decisions

### HistoryDisallowed — Disallowed payload for undo/redo
- Define `pub enum HistoryDisallowed { NothingToUndo, NothingToRedo }` in `src/outcome.rs`
- Lives alongside `Outcome` and `EngineError` — all outcome-adjacent types in one file
- Re-exported at crate root: `pub use crate::outcome::HistoryDisallowed`
- `undo()` returns `Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError>`
- `redo()` returns `Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError>`
- Separate variants for `NothingToUndo` and `NothingToRedo` — callers can pattern-match on specific reasons
- Asymmetric from `dispatch()` (which uses `E::NonCommittedInfo`) — this is intentional and correct

### New dispatch clears redo stack
- Single-timeline history: a successful `Committed` dispatch clears the redo stack
- `NoChange` dispatches do NOT clear redo — the working state was discarded, nothing committed
- Only successful commits (`Committed`) trigger redo stack erasure
- `Disallowed` / `Aborted` / `InvalidInput` outcomes also do NOT clear redo

### Stack depth query API — ships in Phase 3
- `pub fn undo_depth(&self) -> usize` — returns `self.undo_stack.len()`
- `pub fn redo_depth(&self) -> usize` — returns `self.redo_stack.len()`
- Callers need these to enable/disable UI controls without attempting and catching `Disallowed`
- Promotes ERGO-01 from v2 to Phase 3 MVP — cost is near-zero to implement

### Claude's Discretion
- Internal undo stack layout: `Vec<(E::State, Frame<E>)>` tuples (state snapshot before the transition + the frame) vs. parallel `Vec<E::State>` and `Vec<Frame<E>>` vectors — implementation detail
- Derives on `HistoryDisallowed` (reasonable: `Debug, Clone, Copy, PartialEq, Eq`)
- Whether `HistoryDisallowed` gets `#[non_exhaustive]` (probably no — two variants are a complete stable contract, same reasoning as `Reversibility`)
- Exact doc wording for `undo()` and `redo()`

</decisions>

<specifics>
## Specific Ideas

- ARCHITECTURE.md uses the exact names `NothingToUndo` and `NothingToRedo` as variant names — use those verbatim
- Pattern from Phase 2: `Reversibility` is not `#[non_exhaustive]` because its variants are a complete stable contract; same applies here to `HistoryDisallowed`
- `undo_depth()` / `redo_depth()` were ERGO-01 in v2 but the implementation cost in Phase 3 is trivial — shipping them now avoids a breaking-change-free but annoying future addition

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engine.rs`: `Engine<E>` already has `undo_stack: Vec<E::State>` and `redo_stack: Vec<E::State>` placeholder fields with `#[allow(dead_code)]` — Phase 3 removes the `#[allow(dead_code)]` and fills them with real logic
- `src/outcome.rs`: `Outcome<F, N>`, `Frame<E>`, `EngineError` — `HistoryDisallowed` added here; `Undone(F)` and `Redone(F)` variants already defined in `Outcome`
- `src/engine.rs` `dispatch()`: Currently commits without touching history stacks — Phase 3 adds undo stack push on `Committed`, redo stack clear on `Committed`

### Established Patterns
- Flat crate root re-exports: `src/lib.rs` adds `pub use crate::outcome::HistoryDisallowed`
- Private `mod` declarations in `lib.rs` — no new modules needed; `HistoryDisallowed` goes in existing `outcome.rs`
- Full rustdoc on all public types (set in Phase 1) — `HistoryDisallowed`, `undo()`, `redo()`, `undo_depth()`, `redo_depth()` all need docs
- Zero runtime dependencies

### Integration Points
- `src/engine.rs` `dispatch()`: Add undo stack push (prior state snapshot before commit) and redo stack clear (on `Committed`)
- `src/engine.rs`: Add `undo()`, `redo()`, `undo_depth()`, `redo_depth()` methods
- `src/outcome.rs`: Add `HistoryDisallowed` enum
- `src/lib.rs`: Add `pub use crate::outcome::HistoryDisallowed`
- Phase 3 flag from STATE.md: "Snapshot undo memory implications for long AI-heavy sessions — acceptable for MVP, flag for v0.5.x"

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-history*
*Context gathered: 2026-03-14*
