---
phase: 04-core-rename
plan: 02
subsystem: engine
tags: [rust, engine, rename, mutation, behavior, action, undo, redo]

# Dependency graph
requires:
  - "04-core-rename-01: mutation.rs, behavior.rs, action.rs, lib.rs"
provides:
  - "src/engine.rs: Engine<S,M,I,P> runtime using Mutation/Behavior/Action"
affects:
  - 04-core-rename-03

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Engine field renamed: rules -> behaviors"
    - "Public method renamed: add_rule(b, lifetime) -> add_behavior(b) — no lifetime param"
    - "CommitFrame<S,O> -> CommitFrame<S,M> using Action<M> with mutations field"
    - "dispatch guard: if tx.irreversible && !tx.cancelled -> if !tx.cancelled (every commit is undoable)"
    - "Hashing gated by tx.deterministic (not tx.irreversible) — fixes latent bug where deterministic flag was ignored"
    - "Internal RuleLifetime kept as private enum for Phase 4 compatibility; Phase 5 will remove"

key-files:
  created: []
  modified:
    - src/engine.rs
  deleted:
    - src/operation.rs
    - src/rule.rs
    - src/transaction.rs

key-decisions:
  - "dispatch hashing now gated by tx.deterministic (not tx.irreversible): old code hashed all ops in irreversible tx regardless of deterministic flag — this was a bug; new code correctly separates commit path from hash path"
  - "PROP-03 turns/triggers tests deleted: they tested public RuleLifetime::Turns/Triggers behavior which is removed from the API; proptest infrastructure retained"
  - "Internal RuleLifetime enum remains private in engine.rs for Phase 4; Phase 5 will replace with per-dispatch is_active() checks on Behavior"

requirements-completed:
  - REN-01
  - REN-02
  - REN-03

# Metrics
duration: 3min
completed: 2026-03-11
---

# Phase 4 Plan 02: Engine Rename Summary

**Engine<S,M,I,P> fully wired to Mutation/Behavior/Action — add_behavior(b) replaces add_rule(b, lifetime), old source files deleted, all 17 lib tests pass**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-11T00:39:18Z
- **Completed:** 2026-03-11T00:42:xx Z
- **Tasks:** 2
- **Files modified:** 1 (engine.rs); 3 deleted (operation.rs, rule.rs, transaction.rs)

## Accomplishments

- Rewrote src/engine.rs: all imports updated to Mutation/Behavior/Action; type params O->M, E->I; internal field rules->behaviors
- `add_behavior(b)` replaces `add_rule(b, lifetime)` — no lifetime parameter exposed; always inserts internal `RuleLifetime::Permanent`
- `CommitFrame<S,M>` stores `Action<M>` with `mutations` field instead of `Transaction<O>` with `ops`
- `dispatch`: guard changed from `if tx.irreversible && !tx.cancelled` to `if !tx.cancelled` — every committed action is undoable
- Hashing in dispatch correctly gated by `tx.deterministic` (was incorrectly inside `irreversible` block before)
- Updated all doc examples, internal test module, and property test module to use new API names
- Removed PROP-03 turns/triggers property tests (tested public RuleLifetime behavior, now removed from API)
- Deleted src/operation.rs, src/rule.rs, src/transaction.rs — no longer referenced by lib.rs

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite src/engine.rs with new types and add_behavior** - `5e4817c` (feat)
2. **Task 2: Delete old source files** - `8b813b1` (feat)

## Files Created/Modified

- `src/engine.rs` - Rewritten: new types, add_behavior, CommitFrame updated, all tests/doctests updated
- `src/operation.rs` - Deleted
- `src/rule.rs` - Deleted
- `src/transaction.rs` - Deleted

## Decisions Made

- Separated hashing gate from commit gate in `dispatch`: hashing now uses `tx.deterministic` (semantically correct), commit uses `!tx.cancelled` (every action is undoable). The old code incorrectly tied hashing to `tx.irreversible`.
- PROP-03 (turns/triggers lifetime) tests deleted — these tested behavior that is no longer part of the public API; the internal `RuleLifetime` enum that drives the decrement logic remains but is private.
- Internal `RuleLifetime` kept as a private enum in engine.rs for Phase 4 compatibility. Phase 5 will remove it when behaviors get `is_active()`/`on_dispatch()`/`on_undo()` hooks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Separated deterministic-hash gate from commit gate in dispatch**
- **Found during:** Task 1
- **Issue:** The plan explicitly noted: "The old code hashed all ops in an `irreversible` tx regardless of `tx.deterministic` — this was a bug that we fix here." The new code uses `if tx.deterministic { ... }` inside `if !tx.cancelled { ... }` as two separate checks.
- **Fix:** Implemented exactly as specified in the plan — the plan had pre-diagnosed the bug and prescribed the fix.
- **Files modified:** src/engine.rs
- **Commit:** 5e4817c

## Issues Encountered

None — cargo build and cargo test --lib both passed on the first attempt.

## Next Phase Readiness

- src/ now contains only: action.rs, behavior.rs, engine.rs, hash.rs, lib.rs, mutation.rs
- `cargo build` succeeds with zero errors from lib crate (examples still fail — expected, fixed in Plan 03)
- `cargo test --lib` passes all 17 tests
- Plan 03 can now update examples/ to use new API names

---
*Phase: 04-core-rename*
*Completed: 2026-03-11*
