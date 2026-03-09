---
phase: 03-backgammon-example-and-integration-properties
plan: "02"
subsystem: examples
tags: [rust, backgammon, proptest, rules, engine, undo-redo, display]

# Dependency graph
requires:
  - phase: 03-01
    provides: BgState, BackgammonOp, Operation<BgState> impl, checker_count, 10 unit tests
  - phase: 01-module-split-and-foundation
    provides: Engine, Rule, Transaction, RuleLifetime — wired here
  - phase: 02-engine-property-tests
    provides: prop_assert_eq!(engine.read() + engine.replay_hash()) pattern — applied here
provides:
  - BackgammonEvent enum (RollDice, Move)
  - RollDiceRule: pushes RollDiceOp and sets tx.irreversible=false (non-undoable)
  - MoveRule: dispatches to MoveOp/ReEnterOp/BearOffOp based on state inspection
  - Display for BgState: compact single-line board format
  - main() narrative: labeled 4-section demo (roll, move 1, move 2, undo)
  - prop_board_conservation (BACK-05): checker_count stays 30 over apply+undo sequences
  - prop_per_die_undo (BACK-06): engine.read() and engine.replay_hash() match pre-dispatch after undo
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "RollDiceRule sets tx.irreversible=false in before() — canonical rule-controlled non-undoable pattern"
    - "MoveRule inspects state.board[to] to determine captured flag for MoveOp"
    - "prop_per_die_undo: prop_assume!(state_before.board[from] > 0) guards against empty-source moves"
    - "prop_board_conservation: apply+undo pairs test conservation independently of game validity"
    - "BackgammonOp derives Debug — required by proptest Strategy::prop_map bound"

key-files:
  created: []
  modified:
    - examples/backgammon.rs

key-decisions:
  - "RollDiceRule.before() sets tx.irreversible=false (not main()) — rules own behavioral semantics"
  - "MoveRule assumes player_sign=+1 (White) for the demo — keeps rule simple without Player enum in this example"
  - "Added #[derive(Debug)] to BackgammonOp — proptest prop_map requires Debug on strategy output types"
  - "checker_count annotated #[allow(dead_code)] — only used in cfg(test), Rust warns when running without test mode"
  - "prop_board_conservation uses apply+undo pairs rather than engine dispatch — tests op-level conservation invariant orthogonally to game state validity"

patterns-established:
  - "Pattern: Set tx.irreversible=false in rule.before() for non-undoable events — not at call site"
  - "Pattern: Use prop_assume! to guard against generated strategies that produce invalid board positions"

requirements-completed: [BACK-01, BACK-03, BACK-05, BACK-06]

# Metrics
duration: 3min
completed: "2026-03-09"
---

# Phase 3 Plan 02: Engine Wiring, Display, main() Narrative, and Property Tests Summary

**BackgammonEvent + RollDiceRule (tx.irreversible=false) + MoveRule + labeled main() demo + prop_board_conservation and prop_per_die_undo — completing all six BACK requirements**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-09T07:35:57Z
- **Completed:** 2026-03-09T07:38:17Z
- **Tasks:** 2 (TDD pattern applied: RED by inspection, GREEN first-pass with one auto-fix)
- **Files modified:** 1

## Accomplishments
- Replaced stub Display with compact single-line board format showing all 24 board points, bar counts, home counters, and dice state with used flags
- Added BackgammonEvent enum (RollDice, Move) and two rules:
  - RollDiceRule: pushes RollDiceOp and sets tx.irreversible=false (non-undoable pattern — architectural centrepiece of BACK-03)
  - MoveRule: state-inspecting rule that dispatches to MoveOp/ReEnterOp/BearOffOp based on from/to values
- Replaced stub main() with 4-labeled-section demo: roll dice, move 1 with board print, move 2 with board print, undo with board print
- Added `#[cfg(test)] mod props` with two proptest properties:
  - prop_board_conservation (BACK-05): apply+undo pairs preserve checker_count == 30 over 0..=20 ops
  - prop_per_die_undo (BACK-06): engine.read() AND engine.replay_hash() both match pre-dispatch after undo
- All 12 tests green (10 unit from Plan 01 + 2 new property tests)
- Full cargo test suite clean: 19 lib tests + 15 doctests, no regressions

## Task Commits

Each task committed atomically:

1. **Task 1: Display, Events, Rules, and main() narrative** - `2b95726` (feat)
2. **Task 2: BACK-05 and BACK-06 proptest property tests** - `4be43b7` (feat)

## Files Created/Modified
- `examples/backgammon.rs` - 865-line complete backgammon example (was 574 lines); added Display, BackgammonEvent, RollDiceRule, MoveRule, main(), #[cfg(test)] mod props

## Decisions Made
- RollDiceRule.before() sets tx.irreversible=false, not main(). Rules own their behavioral contract — the call site should not need to know which events are undoable.
- MoveRule hardcodes player_sign=+1 (White) — the demo only shows White moving; a production implementation would carry player identity in the event or state.
- Added #[derive(Debug)] to BackgammonOp — proptest's Strategy::prop_map bound requires Debug on the output type; this was discovered at compile time during Task 2.
- prop_board_conservation uses apply+undo pairs at the op level, not engine dispatches. This tests the Operation trait's conservation property independently of game validity, consistent with CONTEXT.md decision: "op-level generation: bypasses event/rule layer, focused on conservation invariant".
- prop_per_die_undo uses prop_assume!(state_before.board[from] > 0) rather than constraining the strategy — keeps strategy simple, discards cases where the generated source point is empty after dice setup.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] BackgammonOp missing #[derive(Debug)] for proptest**
- **Found during:** Task 2 (cargo test --example backgammon)
- **Issue:** proptest's `Strategy::prop_map` requires `Debug` on the output type; 13 compiler errors appeared pointing to BackgammonOp
- **Fix:** Added `Debug` to the `#[derive(Clone)]` on BackgammonOp
- **Files modified:** examples/backgammon.rs
- **Verification:** cargo test --example backgammon: 12 tests green
- **Committed in:** 4be43b7 (Task 2 commit)

**2. [Rule 2 - Missing Critical] Added #[allow(dead_code)] to checker_count**
- **Found during:** Task 2 (cargo run --example backgammon)
- **Issue:** checker_count only appears in #[cfg(test)] blocks; Rust warns about it when not compiling for test
- **Fix:** Added #[allow(dead_code)] attribute to checker_count fn
- **Files modified:** examples/backgammon.rs
- **Verification:** cargo run --example backgammon: no warnings
- **Committed in:** 4be43b7 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing derive, 1 dead_code lint)
**Impact on plan:** Both fixes necessary for clean compilation. No scope creep.

## Issues Encountered
None beyond the two auto-fixed deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All six BACK requirements completed across Plans 01 and 02: BACK-01 (state), BACK-02 (ops), BACK-03 (irreversible), BACK-04 (unit tests), BACK-05 (conservation), BACK-06 (undo roundtrip)
- Phase 3 is now complete
- No blockers

---
*Phase: 03-backgammon-example-and-integration-properties*
*Completed: 2026-03-09*
