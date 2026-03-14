---
phase: 01-core-types
plan: 01
subsystem: core
tags: [rust, trait, associated-types, engine-spec]

# Dependency graph
requires: []
provides:
  - EngineSpec trait with 6 associated types and correct bounds
  - src/spec.rs as foundational type contract for all downstream plans
affects:
  - 01-02 (behavior trait depends on EngineSpec)
  - 01-03 (outcome/frame types depend on EngineSpec)
  - 02-dispatch (WorkingState<E> depends on EngineSpec)
  - 03-history (Frame<E> and history depend on EngineSpec)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "EngineSpec as a type-parameter bundle: <E: EngineSpec> replaces <S, I, D, T, O, K>"
    - "Unit struct pattern: users define struct MySpec; impl EngineSpec for MySpec { ... }"
    - "Module declared as pub mod spec in lib.rs; flat pub use re-export deferred to Plan 02"

key-files:
  created:
    - src/spec.rs
  modified:
    - src/lib.rs

key-decisions:
  - "Used pub mod spec (not mod spec) in lib.rs to satisfy clippy dead_code lint without adding pub use — preserves Plan 02's job of flat re-exports"

patterns-established:
  - "EngineSpec pattern: single type-parameter bundle trait eliminates generic explosion throughout the library"

requirements-completed: [CORE-01]

# Metrics
duration: 1min
completed: 2026-03-13
---

# Phase 1 Plan 01: EngineSpec Trait Summary

**`pub trait EngineSpec` with 6 associated types (State/Input/Diff/Trace/NonCommittedInfo/OrderKey) and correct bounds, with full rustdoc and compile-proof test**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-13T23:57:31Z
- **Completed:** 2026-03-13T23:58:48Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Defined `EngineSpec` trait eliminating `<S, I, D, T, O, K>` generic explosion — all library code now uses `<E: EngineSpec>`
- Established all 6 associated type bounds exactly matching CONTEXT.md: `State: Clone+Debug+Default`, `Input/Diff/Trace/NonCommittedInfo: Clone+Debug`, `OrderKey: Ord`
- Full rustdoc on trait and every associated type, including usage example
- Compile-proof `#[cfg(test)]` module verifying all bounds are satisfiable

## Task Commits

Each task was committed atomically:

1. **Task 1: Write EngineSpec trait with test scaffold** - `f6ad54e` (feat)

**Plan metadata:** (pending)

_Note: TDD task — implemented trait and tests together since tests require trait to compile_

## Files Created/Modified
- `src/spec.rs` - EngineSpec trait with 6 associated types, rustdoc, and compile-proof test
- `src/lib.rs` - Added `pub mod spec;` to include the module in compilation

## Decisions Made
- Used `pub mod spec;` (not `mod spec;`) in lib.rs to prevent clippy `dead_code` warning on `EngineSpec`, while deferring the flat `pub use crate::spec::EngineSpec` re-export to Plan 02 as specified

## Deviations from Plan

None - plan executed exactly as written.

The one minor judgment call (using `pub mod` vs `mod` to satisfy clippy) is consistent with the plan's intent and does not conflict with Plan 02's scope.

## Issues Encountered
- Initial `mod spec;` (private) in lib.rs caused clippy `-D warnings` failure: `trait EngineSpec is never used`. Fixed by changing to `pub mod spec;` — this exposes the module path but does not add the flat re-export (`pub use crate::spec::EngineSpec`) that Plan 02 specifically owns.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `EngineSpec` trait stable and correct — Plan 02 can immediately add `pub use crate::spec::EngineSpec` to lib.rs
- Plans 01-02 (Behavior trait) and 01-03 (Outcome/Frame/EngineError) can both proceed as they only need `EngineSpec`
- Zero warnings, clippy clean — no technical debt to carry forward

---
*Phase: 01-core-types*
*Completed: 2026-03-13*
