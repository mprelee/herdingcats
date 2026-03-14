---
phase: 04-examples-and-tests
plan: "02"
subsystem: examples
tags: [rust, backgammon, irreversibility, undo, demo]

# Dependency graph
requires:
  - phase: 03-history
    provides: undo/redo engine with Reversibility::Irreversible clearing both stacks
provides:
  - Focused backgammon irreversibility demo showing RollDice (Irreversible) + MovePiece (Reversible) with undo history cleared
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Scripted demo pattern: fixed inputs, deterministic output — teaching tool not game"
    - "Print helper functions (print_dispatch, print_undo) encapsulate match-on-Outcome pattern for readable demos"

key-files:
  created:
    - examples/backgammon.rs
  modified: []

key-decisions:
  - "All behaviors return Continue([]) for inputs they don't handle — no abort/abort-early short circuit needed in simple demos"
  - "BackgammonState.black_pos uses #[allow(dead_code)] — kept for domain clarity even though not exercised in the demo"

patterns-established:
  - "Demo pattern: print helpers take &Result<Outcome<...>, ...> by reference — callers keep ownership for depth reads after"
  - "Irreversibility demo pattern: show undo_depth=0 redo_depth=0 right after Irreversible dispatch to make the effect visible"

requirements-completed: [EXAM-02]

# Metrics
duration: 1min
completed: 2026-03-14
---

# Phase 4 Plan 02: Backgammon Irreversibility Demo Summary

**Self-contained backgammon demo with RollDice (Irreversible) clearing both undo/redo stacks, making Disallowed(NothingToUndo) the inevitable and instructive final outcome**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-14T02:02:20Z
- **Completed:** 2026-03-14T02:03:17Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Replaced placeholder `fn main() {}` with a 258-line focused irreversibility teaching demo
- Five-step scripted sequence exactly matches CONTEXT.md locked sequence: Committed -> Committed -> Undone -> Committed[IRREVERSIBLE] -> Disallowed(NothingToUndo)
- undo_depth and redo_depth printed after every operation; both read 0 after the irreversible RollDice dispatch
- `cargo test` passes (2 doc tests, 0 failures, 0 warnings affecting new code)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement backgammon.rs focused irreversibility demo** - `5180166` (feat)

**Plan metadata:** (docs commit — see final commit below)

## Files Created/Modified

- `/Users/mprelee/herdingcats/examples/backgammon.rs` - Full backgammon irreversibility demo: BackgammonSpec, BackgammonState, BackgammonInput, BackgammonDiff, RollDiceBehavior, MovePieceBehavior, print helpers, scripted main()

## Decisions Made

- All behaviors return `Continue([])` for inputs they don't own — simple and explicit, no short-circuit needed in this demo
- `BackgammonState.black_pos` kept with `#[allow(dead_code)]` for domain realism even though only white pieces are moved in the demo

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Both Phase 4 examples (tictactoe.rs placeholder, backgammon.rs complete) are in place
- backgammon.rs fully demonstrates the irreversibility use case end-to-end
- Phase 4 plan 02 (this plan) is the final implementation plan; only documentation/summary work remains

## Self-Check: PASSED

- examples/backgammon.rs: FOUND
- 04-02-SUMMARY.md: FOUND
- Commit 5180166: FOUND

---
*Phase: 04-examples-and-tests*
*Completed: 2026-03-14*
