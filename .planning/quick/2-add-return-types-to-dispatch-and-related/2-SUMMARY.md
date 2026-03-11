---
phase: quick-2
plan: 01
subsystem: engine
tags: [rust, engine, dispatch, return-types, api]

# Dependency graph
requires: []
provides:
  - "engine.dispatch() returns Option<Action<M>>: Some when committed, None when cancelled or empty"
  - "engine.dispatch_preview() returns Action<M> after behaviors have run"
affects: [callers of dispatch/dispatch_preview in examples and downstream crates]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Clone tx after behaviors run, before moving into CommitFrame — captures behavior-injected mutations in return value"
    - "TDD: write failing tests first, verify RED, then implement GREEN"

key-files:
  created: []
  modified:
    - src/engine.rs
    - src/lib.rs
    - examples/tictactoe.rs
    - examples/backgammon.rs

key-decisions:
  - "Clone tx after all behaviors have run (post-after() hooks) so returned Action reflects behavior-injected mutations"
  - "Use boolean flag pattern avoided in favor of direct clone-before-move into CommitFrame"
  - "dispatch_preview returns tx directly (no clone needed — rollback only touches self.state and self.replay_hash)"

patterns-established:
  - "Callers that don't need the return value use let _ = engine.dispatch(...)"
  - "Callers that want to inspect effect use if let Some(action) = engine.dispatch(...)"

requirements-completed: [QUICK-2]

# Metrics
duration: 15min
completed: 2026-03-11
---

# Quick Task 2: Add Return Types to dispatch() and dispatch_preview() Summary

**`engine.dispatch()` now returns `Option<Action<M>>` (Some = committed, None = cancelled/empty) and `engine.dispatch_preview()` returns `Action<M>` for AI look-ahead inspection of behavior-injected mutations.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-11T04:35:00Z
- **Completed:** 2026-03-11T04:50:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- `dispatch()` signature updated to `-> Option<Action<M>>`: returns `Some(action)` when `!cancelled && !mutations.is_empty()`, `None` otherwise
- `dispatch_preview()` signature updated to `-> Action<M>`: returns the action after behaviors' `before`/`after` hooks have run
- TDD: 5 new tests covering None-cancelled, None-empty, Some-reversible, Some-irreversible, and dispatch_preview return value
- All 40 lib tests + 21 doc tests pass; examples compile; clippy clean

## Task Commits

1. **RED: Failing tests for return types** - `8e5da02` (test)
2. **GREEN: dispatch/dispatch_preview implementation** - `657f2c9` (feat)
3. **Task 2: Update examples and lib.rs call sites** - `7bb6aaf` (feat)

## Files Created/Modified

- `/Users/mprelee/herdingcats/src/engine.rs` - Updated dispatch/dispatch_preview signatures, internal test call sites, doc examples; added 5 new return-value tests
- `/Users/mprelee/herdingcats/src/lib.rs` - Updated quick-start doc example to use `let _ = engine.dispatch(...)`
- `/Users/mprelee/herdingcats/examples/tictactoe.rs` - Updated line 255 call site
- `/Users/mprelee/herdingcats/examples/backgammon.rs` - Updated 5 call sites (lines 474, 481, 498, 844, 857, 882)

## Decisions Made

- Cloned `tx` after behaviors have run but before moving into `CommitFrame`. This ensures the returned `Action<M>` reflects any mutations behaviors injected during `before()`/`after()` hooks.
- `dispatch_preview` returns `tx` directly (no clone) since state rollback only touches `self.state` and `self.replay_hash`, not `tx` itself.
- New tests placed in `props` module (where the `proptest!` infra and `MixedOp`/`MixedNoRule` fixtures live) rather than the `tests` module — fixed one test assertion that used the wrong `MixedOp::Irrev` state value.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong state assertion in dispatch_returns_some_for_irreversible_action test**
- **Found during:** Task 1 (GREEN phase)
- **Issue:** Test was placed in the `props` module where `MixedOp::Irrev` is a no-op state change (`*state += 0`), not `*state = 99` as in the `tests` module. The assertion `assert_eq!(engine.read(), 99)` was wrong for that context.
- **Fix:** Updated the assertion to `assert_eq!(engine.read(), 0)` with a clarifying comment explaining the behavior in the props module's MixedOp
- **Files modified:** src/engine.rs
- **Verification:** `cargo test --lib` passes with 40 tests
- **Committed in:** 657f2c9 (Task 1 feat commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in test assertion)
**Impact on plan:** Minor fix to ensure test correctness. No scope creep.

## Issues Encountered

None beyond the test assertion bug above.

## Next Phase Readiness

- `engine.dispatch()` return value is now a first-class API: callers can distinguish "did this event have any effect?" without inspecting state before and after
- `engine.dispatch_preview()` return enables AI look-ahead to inspect what mutations behaviors would inject
- No breaking changes to existing behavior — existing callers that ignore the return value compile unchanged with `let _ =`

## Self-Check: PASSED

- src/engine.rs - modified (dispatch return type)
- src/lib.rs - modified (doc example)
- examples/tictactoe.rs - modified (call site)
- examples/backgammon.rs - modified (call sites)
- Commits verified: 8e5da02, 657f2c9, 7bb6aaf present in git log

---
*Phase: quick-2*
*Completed: 2026-03-11*
