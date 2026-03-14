---
phase: 03-history
plan: "01"
subsystem: engine
tags: [rust, enum, outcome, history, undo, redo]

# Dependency graph
requires:
  - phase: 02-dispatch
    provides: Outcome<F,N> enum with Disallowed(N) variant that HistoryDisallowed slots into
provides:
  - HistoryDisallowed enum with NothingToUndo and NothingToRedo variants
  - herdingcats::HistoryDisallowed re-exported at crate root
affects:
  - 03-history/03-02 (Engine::undo and Engine::redo return Outcome<Frame<E>, HistoryDisallowed>)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Non-exhaustive contract: HistoryDisallowed is NOT #[non_exhaustive] — two variants are a complete stable public contract, same as Reversibility"

key-files:
  created: []
  modified:
    - src/outcome.rs
    - src/lib.rs

key-decisions:
  - "HistoryDisallowed not #[non_exhaustive] — NothingToUndo and NothingToRedo are the complete stable public API for undo/redo disallowed reasons"

patterns-established:
  - "Disallowed reason types live in outcome.rs alongside Outcome<F,N> — co-location with the enum that wraps them"

requirements-completed:
  - HIST-01
  - HIST-02

# Metrics
duration: 5min
completed: 2026-03-14
---

# Phase 3 Plan 01: History Disallowed Summary

**HistoryDisallowed enum (NothingToUndo, NothingToRedo) defined in outcome.rs and re-exported at herdingcats crate root**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-14T01:14:00Z
- **Completed:** 2026-03-14T01:19:08Z
- **Tasks:** 1 (TDD: test commit + feat commit)
- **Files modified:** 2

## Accomplishments
- Added `HistoryDisallowed` enum to `src/outcome.rs` with `NothingToUndo` and `NothingToRedo` variants
- Derives `Debug, Clone, Copy, PartialEq, Eq`; not `#[non_exhaustive]` (stable two-variant contract)
- Re-exported at crate root: `pub use crate::outcome::HistoryDisallowed`
- All 33 unit tests and 8 doc-tests pass with no warnings

## Task Commits

Each task was committed atomically (TDD pattern — two commits for one task):

1. **RED: Failing tests for HistoryDisallowed** - `ee1d568` (test)
2. **GREEN: HistoryDisallowed implementation + re-export** - `bc7010b` (feat)

## Files Created/Modified
- `src/outcome.rs` — Added `HistoryDisallowed` enum after `EngineError`; added two tests in `#[cfg(test)]` block
- `src/lib.rs` — Extended outcome re-export line to include `HistoryDisallowed`

## Decisions Made
- HistoryDisallowed is not `#[non_exhaustive]` — same reasoning as `Reversibility`: two variants are a complete, stable public contract. Callers can match exhaustively without a wildcard arm.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- `HistoryDisallowed` is fully available at `herdingcats::HistoryDisallowed`
- Plan 02 can now implement `Engine::undo` and `Engine::redo` returning `Outcome<Frame<E>, HistoryDisallowed>`

---
*Phase: 03-history*
*Completed: 2026-03-14*
