---
phase: 04-core-rename
plan: "04"
subsystem: engine
tags: [rust, enum, dead-code, cargo]

# Dependency graph
requires:
  - phase: 04-core-rename/04-03
    provides: Renamed engine internals with Behavior/Action/Mutation API; zero-warning build was blocked by dead RuleLifetime variants
provides:
  - "src/engine.rs with single-variant RuleLifetime enum (Permanent only)"
  - "cargo build with zero warnings — Phase 4 success criterion #1 satisfied"
affects: [04-core-rename, 05-reversibility]

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - src/engine.rs

key-decisions:
  - "Remove Turns/Triggers variants rather than suppressing dead_code warning — cleaner than allow(dead_code) since Phase 5 removes the entire lifetimes map"

patterns-established: []

requirements-completed: [REN-04]

# Metrics
duration: 2min
completed: 2026-03-11
---

# Phase 04 Plan 04: Remove Dead RuleLifetime Variants Summary

**Deleted unreachable `Turns(u32)` and `Triggers(u32)` variants from private `RuleLifetime` enum in `src/engine.rs`, eliminating the only dead_code warning and satisfying Phase 4's zero-warning build requirement.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-11T01:01:25Z
- **Completed:** 2026-03-11T01:02:56Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- `RuleLifetime` enum reduced to `Permanent`-only — no dead variants remain
- Removed unreachable `Triggers` decrement block from `dispatch`
- Removed unreachable `Turns` decrement loop from `dispatch`
- Updated `dispatch` docstring to remove stale references to `Turns`/`Triggers`
- `cargo build` emits zero warnings — Phase 4 success criterion #1 fully satisfied
- All 17 unit tests pass, all 15 doc tests pass, both examples exit 0

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove dead RuleLifetime variants and their pattern-match arms** - `2e42a38` (fix)
2. **Task 2: Confirm full test suite still passes** - no commit (verification only, no files changed)

**Plan metadata:** (docs commit — created below)

## Files Created/Modified
- `src/engine.rs` - Simplified `RuleLifetime` enum, removed dead dispatch branches, updated docstring

## Decisions Made
- Remove the dead variants rather than suppress the warning: `#[allow(dead_code)]` would mask a real issue. Since Phase 5 removes the entire `lifetimes` map anyway, cleaning up the dead variants now is strictly correct.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness
- Phase 4 is now complete: all renames applied, `cargo build` emits zero warnings, all 17 tests pass
- Phase 5 (Reversibility) is unblocked — can introduce `is_active()`/`is_reversible()` on `Behavior` and remove the `lifetimes` map entirely

---
*Phase: 04-core-rename*
*Completed: 2026-03-11*

## Self-Check: PASSED

- `src/engine.rs` — modified and committed in `2e42a38`
- Commit `2e42a38` exists: confirmed via `git log`
- `grep "Turns\|Triggers" src/engine.rs` returns no output: VERIFIED
- `cargo build` zero warnings: VERIFIED
- All 17 tests pass: VERIFIED
- Both examples exit 0: VERIFIED
