---
phase: 05-architecture-alignment
plan: 03
subsystem: api
tags: [rust, enginesspec, traits, documentation, readme]

# Dependency graph
requires:
  - phase: 05-architecture-alignment/05-02
    provides: Frame<E> pure data record, Apply trait trace contract

provides:
  - EngineSpec::State requires Clone + Debug only (no Default bound)
  - README.md covering all 8 core terms with dispatch walkthrough and Quick Start

affects: [future phases, library users]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Engine::new() receives initial State — no Default constraint imposed on user types"
    - "README mirrors ARCHITECTURE.md terminology exactly — no new terms introduced"

key-files:
  created: [README.md]
  modified: [src/spec.rs]

key-decisions:
  - "EngineSpec::State has no Default bound — callers supply initial state to Engine::new(), engine never calls Default internally"
  - "Test updated to use explicit vec![] construction instead of ::default() — proves the bound is truly absent"

patterns-established:
  - "Trait bounds on EngineSpec associated types reflect only what the engine actually calls — no convenience bounds"

requirements-completed: [SC-3, SC-7]

# Metrics
duration: 5min
completed: 2026-03-14
---

# Phase 5 Plan 03: EngineSpec Default Bound Removal and README Summary

**EngineSpec::State narrowed to Clone + Debug only, README documents all 8 architecture terms with dispatch walkthrough and Quick Start**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-14T03:02:00Z
- **Completed:** 2026-03-14T03:07:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Removed spurious `Default` bound from `EngineSpec::State` — engine never calls `Default`, callers provide initial state to `Engine::new()`
- Updated spec.rs test to use explicit `vec![]` construction, proving the bound is truly gone
- Wrote README.md covering all 8 core terms (Input, State, Behavior, Diff, Trace, Frame, Outcome, Engine)
- README includes numbered dispatch steps matching ARCHITECTURE.md, undo/redo snapshot model, and working Quick Start example

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove Default bound from EngineSpec::State** - `4eb13b8` (fix)
2. **Task 2: Write README.md architecture description** - `8b3aa65` (docs)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/spec.rs` - Removed `+ Default` from State associated type bound; updated doc comment and test
- `README.md` - New file: 161-line architecture description covering all 8 core terms

## Decisions Made

- EngineSpec::State has no Default bound — callers supply initial state to Engine::new(), engine never calls Default internally
- Test updated to use explicit `vec![]` construction instead of `::default()` — proves the bound is truly absent

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

Pre-existing `cargo doc` warnings in `src/outcome.rs` (unresolved links to `Engine::undo`, `Engine::redo`, `Reversibility`) were present before this plan. Not introduced by this plan's changes, left as-is per scope boundary rules.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 5 is complete. All 3 plans executed:
- 05-01: NonCommittedOutcome refactoring and BehaviorResult<D,N> update
- 05-02: Frame pure data record and Apply trace contract
- 05-03: EngineSpec Default bound removal and README

The codebase now fully matches ARCHITECTURE.md. Ready for v0.5.0 release.

---
*Phase: 05-architecture-alignment*
*Completed: 2026-03-14*
