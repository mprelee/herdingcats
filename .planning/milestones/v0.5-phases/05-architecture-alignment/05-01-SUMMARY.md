---
phase: 05-architecture-alignment
plan: 01
subsystem: api
tags: [rust, outcome, behavior, dispatch, non-committed]

# Dependency graph
requires:
  - phase: 04-examples-and-tests
    provides: Working tictactoe and backgammon examples that exercise the dispatch pipeline
provides:
  - NonCommittedOutcome<N> enum with InvalidInput, Disallowed, Aborted variants
  - From<NonCommittedOutcome<N>> for Outcome<F, N> impl — no hardcoded Aborted coercion
  - BehaviorResult::Stop now wraps NonCommittedOutcome<N>, behaviors choose outcome explicitly
  - Dispatch Stop arm uses outcome.into() via From impl
  - Both examples updated to use semantically correct outcome variants
affects: [05-02, 05-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Behaviors explicitly declare outcome type (InvalidInput/Disallowed/Aborted) via NonCommittedOutcome wrapper
    - From<NonCommittedOutcome<N>> for Outcome<F, N> enables zero-coercion dispatch

key-files:
  created: []
  modified:
    - src/outcome.rs
    - src/behavior.rs
    - src/engine.rs
    - src/lib.rs
    - src/spec.rs
    - examples/tictactoe.rs

key-decisions:
  - "NonCommittedOutcome lives in outcome.rs (not behavior.rs) to avoid circular deps — behavior.rs imports from outcome.rs"
  - "From<NonCommittedOutcome<N>> for Outcome<F, N> impl enables dispatch to map Stop(outcome) => outcome.into() with no hardcoded variant"
  - "NonCommittedOutcome not #[non_exhaustive] — three variants are the complete stable contract matching Outcome non-committed variants"
  - "BehaviorResult type param renamed O→N for consistency with Outcome<F, N> naming convention"
  - "Pre-existing clippy warnings fixed as part of this plan (clone_on_copy, doc_lazy_continuation, loop-index-only) — needed for verification pass"

patterns-established:
  - "Behaviors must wrap BehaviorResult::Stop payload in NonCommittedOutcome variant — no raw string/info payloads"
  - "Dispatch Stop arm: BehaviorResult::Stop(outcome) => return Ok(outcome.into())"

requirements-completed: [SC-1, SC-5, SC-6]

# Metrics
duration: 4min
completed: 2026-03-13
---

# Phase 5 Plan 01: BehaviorResult/NonCommittedOutcome/Outcome Contract Summary

**NonCommittedOutcome<N> wrapper added so behaviors explicitly choose InvalidInput/Disallowed/Aborted; dispatch maps via From impl instead of hardcoding Aborted**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-13T02:51:00Z
- **Completed:** 2026-03-13T02:55:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `NonCommittedOutcome<N>` enum to `src/outcome.rs` with three semantically distinct variants
- Added `From<NonCommittedOutcome<N>> for Outcome<F, N>` impl — dispatch `Stop` arm now uses `outcome.into()` instead of hardcoding `Aborted`
- Updated `BehaviorResult<D, N>::Stop` to wrap `NonCommittedOutcome<N>`, renamed type param `O` to `N`
- Updated `examples/tictactoe.rs` to use `InvalidInput` for out-of-bounds and `Disallowed` for occupied/game-over
- Re-exported `NonCommittedOutcome` at crate root — importable as `herdingcats::NonCommittedOutcome`

## Task Commits

1. **Task 1: Add NonCommittedOutcome and update BehaviorResult contract** - `d2df7a0` (feat)
2. **Task 2: Update examples and fix pre-existing clippy warnings** - `04594c1` (feat)

## Files Created/Modified

- `src/outcome.rs` — Added `NonCommittedOutcome<N>` enum + `From` impl before `Outcome` definition
- `src/behavior.rs` — Renamed type param O→N, `Stop(O)` → `Stop(NonCommittedOutcome<N>)`, updated doc example and test
- `src/engine.rs` — Updated dispatch Stop arm to `outcome.into()`, moved `NonCommittedOutcome` import to test module, fixed pre-existing doc comment clippy warning
- `src/lib.rs` — Added `pub use crate::outcome::NonCommittedOutcome` re-export
- `src/spec.rs` — Added `#[allow(clippy::clone_on_copy)]` on pre-existing test clones
- `examples/tictactoe.rs` — Updated three `BehaviorResult::Stop` calls with semantic variants; fixed pre-existing clippy warnings (doc comment style, loop variable)

## Decisions Made

- `NonCommittedOutcome` placed in `outcome.rs` to avoid circular deps (`behavior.rs` imports from `outcome.rs`, not vice versa)
- `From<NonCommittedOutcome<N>> for Outcome<F, N>` uses a simple match — zero overhead, each variant maps to the matching `Outcome` variant
- `NonCommittedOutcome` not `#[non_exhaustive]` — exactly three variants matching the three non-committed `Outcome` variants is the stable contract
- `BehaviorResult` type param `O` renamed to `N` to align with `Outcome<F, N>` naming convention throughout the codebase
- Pre-existing clippy warnings fixed inline (Rule 3 — blocking verification) — `clone_on_copy` in spec.rs/outcome.rs, `doc_lazy_continuation` in engine.rs, loop index in tictactoe.rs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed pre-existing clippy warnings to pass plan verification**
- **Found during:** Task 2 verification (cargo clippy --all-targets -- -D warnings)
- **Issue:** Pre-existing `clone_on_copy`, `doc_lazy_continuation`, and loop-index-only warnings were present in spec.rs, outcome.rs, engine.rs, and tictactoe.rs. These weren't visible before because tictactoe.rs couldn't compile without our Task 1 changes.
- **Fix:** Added `#[allow(clippy::clone_on_copy)]` in spec.rs and outcome.rs tests; restructured doc comment in engine.rs; replaced range-indexed loops with iterator pattern in tictactoe.rs `CheckWin`; converted module-level `///` doc comment to `//` in tictactoe.rs
- **Files modified:** src/spec.rs, src/outcome.rs, src/engine.rs, examples/tictactoe.rs
- **Verification:** `cargo clippy --all-targets -- -D warnings` passes cleanly
- **Committed in:** `04594c1` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (blocking)
**Impact on plan:** Fix was necessary to pass plan verification criterion. No new functionality added.

## Issues Encountered

- `NonCommittedOutcome` import in `engine.rs` top-level caused "unused import" warning — moved to `#[cfg(test)]` module where it is actually used. The `outcome.into()` dispatch arm resolves via the `From` impl without naming the type.

## Next Phase Readiness

- Outcome contract is now stable: behaviors choose `InvalidInput`, `Disallowed`, or `Aborted` explicitly
- Plan 02 can safely modify `Frame` shape (remove `reversibility` field) — outcome contract is independent
- All 65 library tests + 8 doc tests passing; clippy clean

## Self-Check: PASSED

- SUMMARY.md: FOUND
- src/outcome.rs: FOUND
- Commit d2df7a0: FOUND
- Commit 04594c1: FOUND

---
*Phase: 05-architecture-alignment*
*Completed: 2026-03-13*
