---
phase: 05-architecture-alignment
plan: 02
subsystem: core
tags: [rust, frame, history, reversibility, cow, dispatch]

# Dependency graph
requires:
  - phase: 05-01
    provides: "NonCommittedOutcome, BehaviorResult::Stop wrapping, updated Outcome variants"
provides:
  - "Frame<E> with exactly 3 fields: input, diffs, traces (no reversibility)"
  - "undo_stack and redo_stack as Vec<(E::State, Frame<E>, Reversibility)>"
  - "Apply trait doc contract: MUST return at least one trace entry per mutation"
affects: [05-03, future phases using Frame or history stacks]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Reversibility is dispatch metadata stored in history tuples, not in Frame"
    - "Frame is a pure record: input, diffs, traces — no protocol/policy fields"

key-files:
  created: []
  modified:
    - src/apply.rs
    - src/outcome.rs
    - src/engine.rs

key-decisions:
  - "Frame<E> is a pure data record with no dispatch-protocol fields — Reversibility belongs to the history stack tuple, not the frame itself"
  - "Apply trait doc now enforces trace contract: each state-mutating call MUST return at least one trace entry (was previously 'empty Vec is valid')"

patterns-established:
  - "History stacks carry (prior_state, frame, reversibility) 3-tuples — reversibility preserved for potential future use (e.g., stack inspection) without polluting Frame"

requirements-completed: [SC-2, SC-4]

# Metrics
duration: 3min
completed: 2026-03-14
---

# Phase 05 Plan 02: Architecture Alignment — Frame/Reversibility Separation Summary

**Frame<E> reduced to pure 3-field record (input, diffs, traces); Reversibility moved to history stack tuples as (E::State, Frame<E>, Reversibility)**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-14T02:57:00Z
- **Completed:** 2026-03-14T02:59:54Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments
- Removed `reversibility` field from `Frame<E>` — frame is now a pure record of what happened, not how it was dispatched
- Updated undo_stack and redo_stack to `Vec<(E::State, Frame<E>, Reversibility)>` 3-tuples — reversibility stored alongside the snapshot
- Updated Apply trait doc comment: changed from "Returning an empty Vec is valid" to "MUST return at least one trace entry describing the mutation"
- Removed `frame_stores_reversibility` test (concept no longer applicable to Frame)
- Removed unused `Reversibility` import from `outcome.rs`
- All 64 tests pass; 6 doc tests pass; clippy clean with no warnings

## Task Commits

Each task was committed atomically:

1. **Task 1: Update Apply docs + remove reversibility from Frame + update stacks** - `0c1921d` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/apply.rs` - Updated `apply` method doc: MUST return at least one trace entry
- `src/outcome.rs` - Removed `reversibility` field from Frame<E>, removed Reversibility import, updated Clone/PartialEq impls, updated tests
- `src/engine.rs` - Changed stack types to 3-tuples, updated Frame construction (3 fields), updated undo/redo destructuring patterns, removed `frame.reversibility` assertion in test

## Decisions Made
- Frame<E> is a pure data record: no dispatch-protocol fields belong in it. Reversibility is "how this frame got here" metadata, not "what this frame contains" data — so it belongs in the history stack tuple.
- Apply trait doc tightened to enforce trace contract. The old "empty Vec is valid" wording was accurate for no-op diffs but contradicted ARCHITECTURE.md's intent that every mutation produces a trace. The new wording makes the contract explicit.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness
- Frame shape now matches ARCHITECTURE.md exactly
- History stacks carry full 3-tuple context for potential future stack inspection
- Ready for Phase 05-03 (if any remaining architecture alignment tasks exist)

---
*Phase: 05-architecture-alignment*
*Completed: 2026-03-14*
