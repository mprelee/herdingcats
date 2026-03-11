---
phase: 04-core-rename
plan: 03
subsystem: api
tags: [rust, examples, rename, mutation, behavior, action, version]

# Dependency graph
requires:
  - "04-core-rename-01: mutation.rs, behavior.rs, action.rs, lib.rs — new public API contracts"
  - "04-core-rename-02: engine.rs rewritten with add_behavior — examples can now build"
provides:
  - "examples/tictactoe.rs: updated to Mutation/Behavior/Action API names, compiles and runs"
  - "examples/backgammon.rs: updated to Mutation/Behavior/Action API names, compiles and runs"
  - "Cargo.toml: version bumped to 0.3.0"
affects:
  - 05-reversibility

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "impl Mutation<S> replaces impl Operation<S> in example types"
    - "impl Behavior<S,M,I,P> replaces impl Rule<S,O,E,P> in example behaviors"
    - "tx.mutations.push() replaces tx.ops.push() in before/after hooks"
    - "engine.add_behavior(b) replaces engine.add_rule(b, RuleLifetime::Permanent)"
    - "Action::new() replaces Transaction::new()"
    - "tx.irreversible = false removed — field no longer exists on Action"

key-files:
  created: []
  modified:
    - examples/tictactoe.rs
    - examples/backgammon.rs
    - Cargo.toml

key-decisions:
  - "dice rolls are now undoable in Phase 4 (no irreversible field on Action); Phase 5 will restore non-undoable semantics via is_reversible() on RollDiceOp"
  - "backgammon prop test updated to remove tx.irreversible = false and assert dice rolls are undoable"

patterns-established:
  - "Example files serve as integration smoke tests: cargo run --example confirms end-to-end API rename is correct"

requirements-completed:
  - REN-04

# Metrics
duration: 10min
completed: 2026-03-10
---

# Phase 4 Plan 03: Examples Update Summary

**Both examples updated to Mutation/Behavior/Action API, Cargo.toml bumped to 0.3.0 — Phase 4 Core Rename complete across all files**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-11T00:42:xx Z
- **Completed:** 2026-03-11T00:52:xx Z
- **Tasks:** 3 (2 auto + 1 checkpoint)
- **Files modified:** 3

## Accomplishments

- Updated examples/tictactoe.rs: `impl Mutation<Game>`, `impl Behavior<...>` for PlayRule and WinRule, `tx.mutations.push()`, `engine.add_behavior()`, `Action::new()`
- Updated examples/backgammon.rs: same renames; removed `tx.irreversible = false` lines (field removed); updated tests and prop tests to reflect dice rolls now being undoable; added Phase 5 note for restoring non-undoable semantics
- Bumped Cargo.toml from 0.2.1 to 0.3.0
- All 6 verification checks passed: cargo build clean, cargo test passes, both examples run correctly, no old names in src/ or examples/, Cargo.toml at 0.3.0

## Task Commits

Each task was committed atomically:

1. **Task 1: Update examples/tictactoe.rs** - `618c0de` (feat)
2. **Task 2: Update examples/backgammon.rs and bump Cargo.toml version** - `78bf77b` (feat)
3. **Task 3: Checkpoint — Verify complete Phase 4 rename** - approved by user (all 6 checks passed)

## Files Created/Modified

- `examples/tictactoe.rs` - Updated: Operation->Mutation, Rule->Behavior, Transaction->Action, tx.ops->tx.mutations, add_rule->add_behavior
- `examples/backgammon.rs` - Updated: same renames, removed tx.irreversible field usage, updated tests for undoable dice rolls
- `Cargo.toml` - Version bumped from 0.2.1 to 0.3.0

## Decisions Made

- Dice rolls are now undoable in Phase 4 (Action has no `irreversible` field); Phase 5 will restore non-undoable dice roll semantics via `is_reversible()` on `RollDiceOp`
- Backgammon prop test updated to remove `tx.irreversible = false` and reflect that dice rolls are now undoable through the undo stack

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None — both examples compiled and ran correctly on first attempt. The checkpoint verification found one noted caveat: `RuleLifetime` remains as a private internal enum in engine.rs (intentional, documented in Plan 02 decisions, removed in Phase 5).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 4 Core Rename is fully complete: all src/ files and both examples use new API names exclusively
- `cargo build`, `cargo test`, and both `cargo run --example` commands pass cleanly
- Cargo.toml at 0.3.0 signals the rename milestone publicly
- Phase 5 (reversibility) can begin: `is_reversible()` on Mutation, non-undoable dice rolls, undo barrier semantics

---
*Phase: 04-core-rename*
*Completed: 2026-03-10*
