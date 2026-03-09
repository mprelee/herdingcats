---
phase: 02-engine-property-tests
plan: 01
subsystem: testing
tags: [proptest, property-based-testing, rust, engine, undo, replay_hash]

# Dependency graph
requires:
  - phase: 01-module-split-and-foundation
    provides: Engine, Operation, Rule, Transaction, RuleLifetime public API with full module split
provides:
  - "#[cfg(test)] mod props block in engine.rs with four proptest property tests (PROP-01 through PROP-04)"
  - "Machine-verifiable correctness proofs for undo roundtrip, preview isolation, rule lifetime boundaries, and cancelled tx isolation"
affects: [03-backgammon-board]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Sibling test modules: mod tests for unit tests, mod props for proptest property tests — both inline in source file"
    - "Rc<Cell<u32>> for observable side effects inside rules without accessing private engine fields"
    - "Indirect lifetime/enabled isolation check: compare post-preview dispatch results against reference engine rather than inspecting private fields"
    - "op_sequence_strategy() shared factory function for Vec<CounterOp> generation (0..=20 ops)"

key-files:
  created: []
  modified:
    - src/engine.rs

key-decisions:
  - "CounterOp in mod props re-declares Inc/Dec only (no Reset) — Reset requires knowing current state at generation time, excluded to keep strategies stateless"
  - "PROP-02 indirect isolation check: dispatch same ops post-preview against a reference engine to confirm lifetimes/enabled were not mutated by preview"
  - "PROP-03 uses Rc<Cell<u32>> trigger_count in CountingRule to observe before() call count without accessing private engine.enabled or engine.lifetimes"
  - "PROP-04 uses NoRule/Permanent to avoid Turns unconditional decrement complication — asserts state+hash only"

patterns-established:
  - "Property test fixtures (CounterOp, NoRule, CountingRule) are self-contained in mod props — no cross-module imports from mod tests"
  - "Both engine.read() and engine.replay_hash() asserted in undo roundtrip tests — state-only assertion is insufficient"

requirements-completed: [PROP-01, PROP-02, PROP-03, PROP-04]

# Metrics
duration: 8min
completed: 2026-03-08
---

# Phase 2 Plan 01: Engine Property Tests Summary

**Four proptest property tests covering undo roundtrip, preview isolation, rule lifetime off-by-one, and cancelled tx isolation — all using Engine<i32, CounterOp, (), u8> with shared fixtures in mod props**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-08T~04:00:00Z
- **Completed:** 2026-03-08T~04:08:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `#[cfg(test)] mod props` block to `engine.rs` with four proptest property tests
- PROP-01: arbitrary apply+undo sequences restore both `engine.read()` and `engine.replay_hash()`
- PROP-02: `dispatch_preview` leaves state and replay_hash unchanged; lifetime/enabled isolation confirmed indirectly
- PROP-03 (Turns + Triggers): rule disabled at exactly n dispatches/triggers — no off-by-one
- PROP-04: cancelled transaction leaves state and replay_hash bitwise identical to pre-dispatch snapshot

## Task Commits

Each task was committed atomically:

1. **Task 1: mod props skeleton, shared fixtures, and PROP-01** - `631d219` (feat)
2. **Task 2: PROP-02, PROP-03, PROP-04** - `11c9bdb` (feat)

## Files Created/Modified

- `src/engine.rs` - Added 305 lines: `mod props` block with fixtures, strategy, and five proptest functions

## Decisions Made

- Re-declared `CounterOp` with Inc/Dec only in `mod props` (excluded Reset to keep strategies stateless)
- Used `Rc<Cell<u32>>` in `CountingRule` to observe before() call count without accessing private fields
- PROP-02 uses an indirect verification approach: after dispatch_preview, dispatches same ops against a fresh reference engine — divergence would indicate lifetime/enabled mutation
- PROP-04 uses `NoRule` with `Permanent` lifetime to avoid Turns unconditional-decrement edge case, asserting state+hash only

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All four PROP requirements complete; engine correctness is now machine-verifiable
- Phase 3 (backgammon board) can proceed — validate `[i8; 26]` bearing-off representation early before building proptest strategies around it (existing blocker from STATE.md)

---
*Phase: 02-engine-property-tests*
*Completed: 2026-03-08*
