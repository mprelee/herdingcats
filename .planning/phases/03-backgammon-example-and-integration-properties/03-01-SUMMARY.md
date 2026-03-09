---
phase: 03-backgammon-example-and-integration-properties
plan: "01"
subsystem: testing
tags: [rust, backgammon, property-testing, operation, undo-redo]

# Dependency graph
requires:
  - phase: 01-module-split-and-foundation
    provides: Operation<S> trait, Engine, RuleLifetime — the core contract this example implements
  - phase: 02-engine-property-tests
    provides: established property-test patterns; CounterOp roundtrip conventions followed here
provides:
  - BgState struct with [i8; 26] board encoding (white/black bar at indices 24/25), white_home, black_home, dice, dice_used
  - BackgammonOp enum: RollDiceOp, MoveOp (place-empty + hit-blot), ReEnterOp, BearOffOp
  - Operation<BgState> impl for BackgammonOp with all apply/undo correctness rules
  - checker_count() conservation helper (board abs-sum + homes)
  - 10 unit tests covering all variant roundtrips, dice_used restoration, and checker conservation
affects: [03-02-integration-properties]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "BgState board encoding: [i8; 26] with positive=White, negative=Black, bar at [24]/[25]"
    - "player_sign: i8 (+1 White, -1 Black) passed into op variants for uniform arithmetic"
    - "MoveOp.undo() sets dice_used[die_index]=false unconditionally (pitfall 3 guard)"
    - "BearOffOp writes white_home/black_home counters — never board[26] (pitfall 5 guard)"
    - "Test pattern: use minimal BgState { board: [0;26], ... } for isolated op tests"

key-files:
  created:
    - examples/backgammon.rs
  modified: []

key-decisions:
  - "checker_count tests on isolated states (not BgState::new()) to avoid 30-checker invariant collision in unit tests"
  - "BackgammonOp variants carry player_sign as i8 field rather than encoding player as enum — enables uniform arithmetic across all op variants"
  - "checker_count is fn (not pub fn) since BgState is private to the example file; pub would cause private_interfaces warning"
  - "dead_code allows on BackgammonPriority and BackgammonOp — both used in Plan 02 (Rules/main wiring)"

patterns-established:
  - "Pattern: Use minimal zero-initialized BgState for op unit tests, not BgState::new(), to avoid interference with the 30-checker starting invariant"
  - "Pattern: All BackgammonOp apply/undo implementations guard dice_used[die_index] unconditionally"

requirements-completed: [BACK-01, BACK-02, BACK-04]

# Metrics
duration: 3min
completed: "2026-03-09"
---

# Phase 3 Plan 01: BgState + BackgammonOp Data Model with Roundtrip Unit Tests Summary

**[i8;26] backgammon board with four op variants (MoveOp/ReEnterOp/BearOffOp/RollDiceOp), player_sign arithmetic, and 10 unit tests proving all apply/undo roundtrips and checker conservation**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-09T06:30:12Z
- **Completed:** 2026-03-09T06:33:23Z
- **Tasks:** 1 (TDD: RED + GREEN in one pass — implementation was written until tests passed)
- **Files modified:** 1

## Accomplishments
- BgState struct with [i8; 26] board encoding, standard starting position in BgState::new(), and all required fields
- BackgammonOp enum with all four variants implementing Operation<BgState> trait
- All correctness pitfalls guarded: dice_used restored unconditionally in undo(), BearOffOp writes home counters not board[26], bar sign convention followed uniformly
- checker_count() helper verified at 30 for standard position and conserved across all op roundtrips
- 10 unit tests all green, cargo build --example backgammon clean (no warnings)

## Task Commits

Each task was committed atomically:

1. **Task 1: BgState + BackgammonOp implementation with unit tests** - `8f3c777` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified
- `examples/backgammon.rs` - 574-line backgammon data model: BackgammonPriority, BgState, BackgammonOp, Operation<BgState> impl, checker_count, 10 unit tests

## Decisions Made
- Used minimal zero-initialized BgState for most unit tests rather than BgState::new() — the standard starting position has exactly 30 checkers, so adding even one extra checker for test setup would break checker_count == 30 assertions. Only the checker_count_standard_start test uses BgState::new().
- BackgammonOp variants carry player_sign: i8 (+1 White, -1 Black) as a field rather than a Player enum — this lets all board arithmetic stay uniform (board[idx] += player_sign, board[idx] -= player_sign) without match arms per variant.
- checker_count changed from pub fn to fn — BgState is private to the example file; pub fn with a private type triggers Rust's private_interfaces lint.
- Added #[allow(dead_code)] to BackgammonPriority and BackgammonOp — both will be wired into Rules/main in Plan 02; the allow silences the lint without compromising the structure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unit tests using BgState::new() for isolated op tests**
- **Found during:** Task 1 (first test run)
- **Issue:** Tests for MoveOp (hit blot), ReEnterOp, BearOffOp, and checker_count preservation were modifying the standard 30-checker BgState::new() by adding extra checkers, causing checker counts of 31-32 instead of the expected preserved count
- **Fix:** Changed isolated op tests to construct minimal BgState { board: [0;26], ... } with only the checkers needed for that test; changed assert_eq!(checker_count, 30) to capture count_before and verify preservation
- **Files modified:** examples/backgammon.rs
- **Verification:** All 10 tests green after fix
- **Committed in:** 8f3c777 (Task 1 commit)

**2. [Rule 2 - Missing Critical] Fixed compiler warnings before final verification**
- **Found during:** Task 1 (cargo build --example backgammon)
- **Issue:** Three warnings: private_interfaces (pub fn checker_count with private BgState), dead_code on BackgammonPriority and BackgammonOp
- **Fix:** Changed checker_count to fn; added #[allow(dead_code)] to the two types used in Plan 02
- **Files modified:** examples/backgammon.rs
- **Verification:** cargo build --example backgammon: no warnings
- **Committed in:** 8f3c777 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 test correctness bug, 1 compiler warning cleanup)
**Impact on plan:** Both fixes necessary for test correctness and clean build. No scope creep.

## Issues Encountered
None beyond the auto-fixed test setup issue documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Foundation complete: BgState, BackgammonOp, and checker_count are ready for Plan 02 (Rules, Events, Display, main wiring) and Plan 03 (proptest strategy integration)
- All four op variants proven correct via apply/undo roundtrip tests — Phase 3 MEDIUM-confidence risk (BACK-02 bearing-off edge cases) is now resolved
- No blockers for Plan 02

---
*Phase: 03-backgammon-example-and-integration-properties*
*Completed: 2026-03-09*
