---
phase: 06-tests-and-examples
plan: "02"
subsystem: testing
tags: [rust, proptest, backgammon, tictactoe, reversibility, is_reversible, on_dispatch]

# Dependency graph
requires:
  - phase: 05-reversibility-and-behavior-lifecycle
    provides: "is_reversible() on Mutation trait, on_dispatch() on Behavior trait, undo barrier in Engine::dispatch"
provides:
  - "backgammon example updated with RollDiceOp.is_reversible()=false, RollDiceRule.rolls_dispatched counter"
  - "prop_dice_roll_sets_undo_barrier proptest (BACK-07) — integration coverage of undo barrier in game context"
  - "Engine::can_undo() and Engine::can_redo() public accessor methods"
  - "tictactoe example confirmed to use all v1.1 API names"
affects:
  - "07-docs-and-extended-tests"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Irreversible ops (is_reversible=false) in game examples use can_undo()/can_redo() for barrier assertions, not private field access"
    - "Stateful behaviors hold counters (rolls_dispatched) incremented in on_dispatch() lifecycle hook"

key-files:
  created: []
  modified:
    - examples/backgammon.rs
    - src/engine.rs

key-decisions:
  - "Used Engine::can_undo() and Engine::can_redo() public methods instead of accessing private undo_stack field — private fields not accessible from external crate (examples)"
  - "tictactoe.rs needed no changes — TEST-06 satisfied by confirmation only"

patterns-established:
  - "Pattern: integration proptests for undo barriers use can_undo()/can_redo() public API, not private stack access"

requirements-completed: [TEST-05, TEST-06]

# Metrics
duration: 4min
completed: 2026-03-10
---

# Phase 6 Plan 02: Examples Update (v1.1 Reversibility API) Summary

**Backgammon example updated with RollDiceOp.is_reversible()=false, RollDiceRule.rolls_dispatched on_dispatch counter, new BACK-07 proptest, and Engine::can_undo()/can_redo() accessors; tictactoe confirmed unchanged.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-03-11T02:29:49Z
- **Completed:** 2026-03-11T02:37:37Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `is_reversible() = false` to `BackgammonOp::RollDiceOp` making dice rolls set an undo barrier
- Added `rolls_dispatched: u32` field to `RollDiceRule` with `on_dispatch()` increment lifecycle hook
- Replaced main() demo sequence to show Roll (barrier) → Move → Undo → Move again flow
- Removed all Phase 4/5 TODO comments from backgammon.rs (zero matches confirmed)
- Added `prop_dice_roll_sets_undo_barrier` proptest (BACK-07) using `engine.can_undo()`
- Added `Engine::can_undo()` and `Engine::can_redo()` public methods to lib for external crate access
- Confirmed tictactoe.rs already uses all v1.1 names — no changes needed (TEST-06)

## Task Commits

Each task was committed atomically:

1. **Task 1: Update backgammon example (TEST-05)** - `e0c0184` (feat)
2. **Task 2: Confirm tictactoe example (TEST-06)** - no commit needed (tictactoe unchanged)

**Plan metadata:** (to be added in final commit)

## Files Created/Modified
- `examples/backgammon.rs` - RollDiceOp.is_reversible()=false, RollDiceRule.rolls_dispatched counter with on_dispatch(), updated main(), new BACK-07 proptest, Phase 4/5 comments removed
- `src/engine.rs` - Added Engine::can_undo() and Engine::can_redo() public methods

## Decisions Made
- Added `Engine::can_undo()` and `Engine::can_redo()` public methods: the plan specified using `engine.undo_stack.is_empty()` directly in the proptest, but `undo_stack` is a private field not accessible from external crates (examples compile as external to the library). Added public accessor methods as the correct API-safe approach.
- tictactoe.rs confirmed correct without changes: grep audit found zero old API names (Operation/Transaction/RuleLifetime/Rule/add_rule); `cargo run --example tictactoe` exits cleanly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added Engine::can_undo() and Engine::can_redo() public methods**
- **Found during:** Task 1 (Add prop_dice_roll_sets_undo_barrier proptest)
- **Issue:** Plan specified `engine.undo_stack.is_empty()` in the proptest, but `undo_stack` is a private field on `Engine`. External crates (examples) cannot access private fields even with `use super::*` — `super` in an example refers to the example module, not `engine.rs`.
- **Fix:** Added `pub fn can_undo(&self) -> bool` and `pub fn can_redo(&self) -> bool` to `Engine` in `src/engine.rs`. Updated proptest to use `engine.can_undo()` and `engine.can_redo()`.
- **Files modified:** src/engine.rs
- **Verification:** `cargo test --example backgammon` passes all 13 tests including new proptest
- **Committed in:** e0c0184 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Auto-fix necessary for correctness — private field access would fail compilation. Public accessor methods are the correct API design.

## Issues Encountered
- None beyond the private field access issue documented above.

## Next Phase Readiness
- TEST-05 and TEST-06 requirements satisfied
- Phase 6 complete — backgammon and tictactoe examples serve as integration smoke tests for the v1.1 API
- Ready for Phase 7 (docs and extended tests)
- `Engine::can_undo()` and `Engine::can_redo()` are useful public methods for Phase 7 doctest coverage

## Self-Check: PASSED

---
*Phase: 06-tests-and-examples*
*Completed: 2026-03-10*
