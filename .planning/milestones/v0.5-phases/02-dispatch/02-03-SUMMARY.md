---
phase: 02-dispatch
plan: 03
subsystem: engine
tags: [rust, cow, dispatch, determinism, behaviors, tdd]

# Dependency graph
requires:
  - phase: 02-dispatch/02-01
    provides: Apply<E> trait and Reversibility enum
  - phase: 02-dispatch/02-02
    provides: EngineSpec trait, BehaviorResult, Frame, Outcome, EngineError
provides:
  - Engine<E> struct with new(), state(), dispatch()
  - CoW dispatch pipeline with lazy state cloning
  - Deterministic behavior ordering by (order_key, name)
  - herdingcats::Engine in flat public namespace
affects: [03-history, 04-examples-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "CoW dispatch: Cow::Borrowed(&self.state) until first diff; Cow::to_mut() on first apply"
    - "Atomic commit: self.state replaced exactly once after full loop, never mid-loop"
    - "Field-level borrow splitting: &self.behaviors and Cow borrow of self.state coexist safely"
    - "Behavior sort: Vec::sort_by with (order_key, name) — deterministic, no address-based tiebreaking"
    - "#[allow(dead_code)] on Phase 3 placeholder fields (undo_stack, redo_stack)"

key-files:
  created:
    - src/engine.rs
  modified:
    - src/lib.rs

key-decisions:
  - "CoW heap-pointer comparison: test uses Vec::as_ptr() (inner buffer address) not &Vec (struct address in engine field) — field address never changes between dispatches"
  - "Apply trait must be in scope in engine.rs (use crate::apply::Apply) for diff.apply() method to resolve even though Apply<E> is a bound on EngineSpec::Diff"

patterns-established:
  - "dispatch() passes &*working (Deref on Cow) to behaviors — ensures later behaviors see earlier diffs, not self.state"
  - "BehaviorResult::Stop returns Outcome::Aborted immediately — no frame committed, state unchanged"
  - "Outcome::NoChange returned when diffs.is_empty() — no into_owned(), no allocation"

requirements-completed: [DISP-01, DISP-02, DISP-03, DISP-04]

# Metrics
duration: 2min
completed: 2026-03-14
---

# Phase 2 Plan 3: Engine Dispatch Summary

**`Engine<E>` with CoW dispatch pipeline: behaviors sorted once by `(order_key, name)`, state cloned lazily on first diff, frames committed atomically — all 31 unit tests and 8 doctests green.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-14T00:42:20Z
- **Completed:** 2026-03-14T00:44:30Z
- **Tasks:** 2 (TDD: RED-GREEN-REFACTOR combined in single commits)
- **Files modified:** 2

## Accomplishments
- `Engine<E>` struct with `new()` (sorts behaviors by `(order_key, name)`), `state()`, and `dispatch(input, reversibility)`
- CoW dispatch: borrows `self.state` via `Cow::Borrowed`, clones exactly once on first `diff.apply()` call via `Cow::to_mut()`
- `BehaviorResult::Stop` halts dispatch immediately returning `Outcome::Aborted`, leaving state unchanged
- No-op dispatch returns `Outcome::NoChange` without cloning (heap pointer equality confirmed by test)
- `Frame<E>` populated with input, diffs, traces, and reversibility at atomic commit time
- `herdingcats::Engine` exported in flat namespace via `pub use crate::engine::Engine`
- Phase 3 placeholder `undo_stack`/`redo_stack` fields present on struct (suppressed with `#[allow(dead_code)]`)

## Task Commits

Each task was committed atomically:

1. **Task 1: Engine struct + new() + state() + dispatch() + all tests** - `3f47490` (feat)
2. **Task 2: Wire Engine into lib.rs public API** - `bee7c6f` (feat)

_Note: dispatch() was implemented alongside Task 1 since all 12 tests (3 Task-1 + 9 Task-2) were written and verified together in one RED-GREEN cycle._

## Files Created/Modified
- `src/engine.rs` - Engine<E> struct, new(), state(), dispatch(), 12 unit tests covering all DISP-01 through DISP-04 requirements
- `src/lib.rs` - Added `mod engine` declaration and `pub use crate::engine::Engine` re-export

## Decisions Made
- **CoW pointer test technique:** Compared `Vec::as_ptr()` (inner heap buffer address) rather than `&engine.state as *const Vec<u8>` (engine field address). The field address is stable regardless of CoW — only the heap buffer allocation changes when Cow clones.
- **Apply import:** `use crate::apply::Apply` is required in `engine.rs` even though `Apply<E>` is a bound on `EngineSpec::Diff` — Rust requires the trait to be in scope for method call resolution.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed CoW pointer comparison technique**
- **Found during:** Task 1 (cow_clones_on_first_diff test)
- **Issue:** Plan suggested comparing `engine.state() as *const _ as *const u8` but the `Engine` struct field address never changes (it's always `&self.state`). Only the inner heap buffer allocation changes after a clone.
- **Fix:** Changed test to compare `engine.state().as_ptr()` (the `Vec<u8>` heap buffer pointer) which correctly distinguishes cloned vs. uncloned state.
- **Files modified:** src/engine.rs
- **Verification:** `cow_clones_on_first_diff` and `cow_no_clone_on_no_op_dispatch` both pass
- **Committed in:** 3f47490 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug in test technique)
**Impact on plan:** Necessary correction — the original pointer comparison would have made the no-clone test pass vacuously and the clone test always fail. No scope creep.

## Issues Encountered
- `diff.apply()` method not found: `use crate::apply::Apply` was missing from engine.rs imports. Added immediately (Rule 3 - Blocking). Resolved in first compilation attempt.

## Next Phase Readiness
- Phase 3 (History) can begin: `Engine` struct has `undo_stack` and `redo_stack` placeholder fields ready to be populated with snapshot-based undo/redo logic
- All 4 DISP requirements verified green
- `herdingcats::Engine` is the complete dispatch API surface downstream code will use

---
*Phase: 02-dispatch*
*Completed: 2026-03-14*
