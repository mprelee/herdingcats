---
phase: 04-core-rename
plan: 01
subsystem: api
tags: [rust, traits, rename, mutation, behavior, action]

# Dependency graph
requires: []
provides:
  - "src/mutation.rs: Mutation<S> trait (apply/undo/hash_bytes)"
  - "src/behavior.rs: Behavior<S,M,I,P> trait (before/after/id/priority) using Action<M>"
  - "src/action.rs: Action<M> struct (mutations/deterministic/cancelled)"
  - "src/lib.rs: public API re-exports for Mutation, Behavior, Action, Engine"
affects:
  - 04-core-rename-02
  - 04-core-rename-03

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Trait naming: Mutation replaces Operation, Behavior replaces Rule, Action replaces Transaction"
    - "Action<M> has exactly three fields: mutations, deterministic, cancelled (no irreversible)"
    - "Behavior<S,M,I,P>: type param E renamed to I (Input), O renamed to M (Mutation)"

key-files:
  created:
    - src/mutation.rs
    - src/behavior.rs
    - src/action.rs
  modified:
    - src/lib.rs

key-decisions:
  - "irreversible field removed from Action<M) — undo barrier semantics handled separately in Phase 5"
  - "RuleLifetime enum removed entirely — behaviors self-manage via is_active/on_dispatch/on_undo (Phase 5)"
  - "Old files (operation.rs, rule.rs, transaction.rs) left on disk — engine.rs still references them until Plan 02"

patterns-established:
  - "Interface-first rename: new API contracts established before engine wiring updated"
  - "Compilation gate: lib.rs declares new modules, engine.rs compile errors expected until Plan 02"

requirements-completed:
  - REN-01
  - REN-02
  - REN-03

# Metrics
duration: 2min
completed: 2026-03-11
---

# Phase 4 Plan 01: Core Rename Summary

**New public API contracts: Mutation<S>, Behavior<S,M,I,P>, and Action<M> traits/struct created with zero occurrences of old names (Operation/Rule/Transaction/RuleLifetime)**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-11T00:35:21Z
- **Completed:** 2026-03-11T00:37:30Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Created src/mutation.rs: Mutation<S> trait with apply/undo/hash_bytes methods identical to Operation<S>
- Created src/action.rs: Action<M> struct with mutations/deterministic/cancelled (irreversible and RuleLifetime removed)
- Created src/behavior.rs: Behavior<S,M,I,P> trait with before/after/id/priority using Action<M> in signatures
- Updated src/lib.rs: new module declarations and re-exports, old modules removed

## Task Commits

Each task was committed atomically:

1. **Task 1: Create src/mutation.rs (Operation -> Mutation)** - `4721add` (feat)
2. **Task 2: Create src/behavior.rs and src/action.rs** - `c5f7f6e` (feat)
3. **Task 3: Update src/lib.rs** - `002e4b9` (feat)

## Files Created/Modified
- `src/mutation.rs` - New file: Mutation<S> trait replacing Operation<S>
- `src/action.rs` - New file: Action<M> struct replacing Transaction<O>, irreversible/RuleLifetime removed
- `src/behavior.rs` - New file: Behavior<S,M,I,P> trait replacing Rule<S,O,E,P>
- `src/lib.rs` - Updated: declares new modules, re-exports new public API names

## Decisions Made
- Removed `irreversible` field from Action<M): undo barrier semantics belong to Phase 5 reversibility work
- Removed `RuleLifetime` enum entirely: behaviors will self-manage lifetime via is_active/on_dispatch/on_undo in Phase 5
- Old files (operation.rs, rule.rs, transaction.rs) intentionally left on disk; engine.rs references them and will be updated in Plan 02

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - cargo check confirms all 3 errors are in engine.rs only (expected), new files compile cleanly.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- New API contracts (Mutation, Behavior, Action) ready for Plan 02 to update engine.rs
- lib.rs compilation blocked on engine.rs update (Plan 02) — expected and by design
- Old files still present on disk, will be deleted in Plan 02

---
*Phase: 04-core-rename*
*Completed: 2026-03-11*
