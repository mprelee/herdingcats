---
phase: 02-dispatch
plan: 01
subsystem: engine
tags: [rust, traits, enums, dispatch, apply, reversibility]

# Dependency graph
requires:
  - phase: 01-core-types
    provides: EngineSpec trait with all associated types (State, Input, Diff, Trace, NonCommittedInfo, OrderKey)
provides:
  - Apply<E> trait in src/apply.rs — combined diff application + trace emission
  - Reversibility enum in src/reversibility.rs — Copy+Eq+Debug, compile-time enforced reversibility declaration
affects:
  - 02-02 (wires Apply and Reversibility into spec.rs and lib.rs, builds Engine struct and dispatch)
  - 02-03 (Engine::undo/redo uses Reversibility to manage history clearing)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Apply<E> trait: combined mutation + trace in one call prevents forgetting trace emission"
    - "Reversibility enum as explicit dispatch parameter: compiler enforces no omission"
    - "Private mod declarations in lib.rs for test compilation; pub use deferred to Plan 02"

key-files:
  created:
    - src/apply.rs
    - src/reversibility.rs
  modified:
    - src/lib.rs

key-decisions:
  - "Added private mod apply and mod reversibility to lib.rs to enable cargo test --lib; pub use re-exports deferred to Plan 02 as planned"
  - "Reversibility is not #[non_exhaustive] — two variants are stable public contract (same reasoning as Outcome)"

patterns-established:
  - "Apply<E>: apply(&self, state: &mut E::State) -> Vec<E::Trace> — application and trace generation are one atomic call"
  - "Reversibility::Reversible / Irreversible — exhaustive match required, no default variant"

requirements-completed: [DISP-01, DISP-04]

# Metrics
duration: 2min
completed: 2026-03-14
---

# Phase 2 Plan 01: Apply and Reversibility Summary

**`Apply<E>` trait and `Reversibility` enum established as the two foundational dispatch support types, with 5 passing tests verifying Copy, Eq, Debug, mutation behavior, and empty-trace no-op behavior**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-14T00:32:59Z
- **Completed:** 2026-03-14T00:34:36Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `Apply<E>` trait created: structurally enforces combined diff application + trace emission in one call
- `Reversibility` enum created: `Copy+Eq+Debug`, exhaustive match required, no default — compiler enforces explicit reversibility declaration at every dispatch site
- 17 total lib tests passing (12 Phase 1 + 2 Apply + 3 Reversibility)

## Task Commits

Each task was committed atomically:

1. **Task 1: Apply trait (RED-GREEN-REFACTOR)** - `fd2d627` (test — Apply<E> trait + tests)
2. **Task 2: Reversibility enum (RED-GREEN-REFACTOR)** - `a623a55` (feat — Reversibility enum + tests)

**Plan metadata:** (docs commit follows)

_Note: TDD tasks had combined RED+GREEN in one commit since plan provided both test behavior spec and implementation in the action block_

## Files Created/Modified

- `src/apply.rs` — `Apply<E>` trait with `fn apply(&self, &mut E::State) -> Vec<E::Trace>`, two behavior tests, full rustdoc
- `src/reversibility.rs` — `Reversibility` enum `(Reversible, Irreversible)` with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, three behavior tests, full rustdoc
- `src/lib.rs` — added private `mod apply;` and `mod reversibility;` (no pub use — Plan 02's responsibility)

## Decisions Made

- Added private `mod apply;` and `mod reversibility;` to `lib.rs` so `cargo test --lib` can compile the test modules. The plan's verify command (`cargo test --lib`) requires modules to be compiled. No `pub use` re-exports added — those remain Plan 02's scope.
- `Reversibility` is not `#[non_exhaustive]` — both variants are the complete stable contract; adding a third would be a breaking change by design (callers must handle both explicitly).

## Deviations from Plan

None — plan executed exactly as written. The private `mod` declarations in lib.rs are a necessary prerequisite for `cargo test --lib` to discover test modules and are consistent with "Plan 02 handles all lib.rs wiring (pub use)."

## Issues Encountered

Doctests in `src/apply.rs` and `src/reversibility.rs` fail under `cargo test --doc` because `Apply` and `Reversibility` are not yet re-exported via `pub use` in `lib.rs`. This is expected and documented in the plan: "Dead-code warning acceptable." `cargo test --lib` (the plan's specified verify command) passes with all 17 tests green.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `Apply<E>` and `Reversibility` are ready for Plan 02 to wire into `spec.rs` (adding `Apply<Self>` bound to `type Diff`) and `lib.rs` (adding `pub use apply::Apply; pub use reversibility::Reversibility;`)
- Plan 02 will build the `Engine` struct and `dispatch(input, reversibility)` method using both these types
- No blockers

---
*Phase: 02-dispatch*
*Completed: 2026-03-14*
