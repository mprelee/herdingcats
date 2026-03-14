---
phase: 03-history
plan: "02"
subsystem: engine
tags: [rust, undo, redo, history, snapshot, cow, engine]

# Dependency graph
requires:
  - phase: 03-history/03-01
    provides: HistoryDisallowed enum (NothingToUndo, NothingToRedo) used as undo/redo Disallowed payload
  - phase: 02-dispatch
    provides: Engine<E> struct, dispatch() method, Frame<E>, Outcome<F,N>, Reversibility
provides:
  - Engine::undo() — snapshot-based undo returning Outcome<Frame<E>, HistoryDisallowed>
  - Engine::redo() — snapshot-based redo returning Outcome<Frame<E>, HistoryDisallowed>
  - Engine::undo_depth() — count of available undo steps
  - Engine::redo_depth() — count of available redo steps
  - Upgraded undo_stack/redo_stack: Vec<(E::State, Frame<E>)> with snapshot capture in dispatch()
affects:
  - 04-examples (backgammon.rs exercises Irreversible dispatch clearing all history)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Snapshot-based undo: prior_state cloned BEFORE commit, stored as (E::State, Frame<E>) tuple — no Reversible trait on user Diff types"
    - "std::mem::replace for zero-overhead state swap in undo/redo (avoids double-clone)"
    - "Single-timeline invariant: redo_stack.clear() on every Committed, both stacks cleared on Irreversible"
    - "Manual Clone/PartialEq impls for Frame<E> with per-field bounds (E::Input/Diff/Trace: Clone/PartialEq) — derive macro adds E: Clone/PartialEq which E does not satisfy"

key-files:
  created: []
  modified:
    - src/engine.rs
    - src/outcome.rs

key-decisions:
  - "Frame<E> Clone and PartialEq are manually implemented with bounds on associated types (not E itself) — derive macro generated incorrect E: Clone/PartialEq bounds that could never be satisfied since E is typically a unit struct with no such impls"
  - "undo/redo return Outcome<Frame<E>, HistoryDisallowed> (not Outcome<Frame<E>, E::NonCommittedInfo>) — the N type parameter is HistoryDisallowed specifically, asymmetric from dispatch() by intentional design"
  - "Irreversible Committed dispatch: push to undo_stack then clear both stacks (state change goes through, ALL history erased)"

patterns-established:
  - "History methods (undo/redo/undo_depth/redo_depth) live in the same impl block as dispatch() in engine.rs"
  - "Test calls that discard Outcome results use let _ = binding to satisfy #[must_use] — never .unwrap() alone on a line"

requirements-completed:
  - HIST-01
  - HIST-02
  - HIST-03
  - HIST-04

# Metrics
duration: 5min
completed: 2026-03-14
---

# Phase 3 Plan 02: Undo/Redo History Summary

**Snapshot-based undo/redo on Engine<E> using Vec<(E::State, Frame<E>)> stacks — no Reversible trait, no diffs-in-reverse, just full state clones stored per commit**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-14T01:20:42Z
- **Completed:** 2026-03-14T01:25:57Z
- **Tasks:** 2 (both TDD with test + feat commits)
- **Files modified:** 2

## Accomplishments
- Upgraded `undo_stack`/`redo_stack` from placeholder `Vec<E::State>` to `Vec<(E::State, Frame<E>)>` — full history tuple
- `dispatch()` now captures `prior_state.clone()` before commit, clears redo on any Committed, clears both stacks on Irreversible
- `Engine::undo()` restores state from snapshot, pushes to redo, returns `Outcome::Undone(frame)`
- `Engine::redo()` restores state from snapshot, pushes to undo, returns `Outcome::Redone(frame)`
- `undo_depth()` and `redo_depth()` expose stack sizes for UI control enable/disable
- Fixed `Frame<E>` `Clone` and `PartialEq` derivation bug (derives added `E: Clone`/`E: PartialEq` bounds; manual impls use correct per-field bounds)
- 14 new tests (5 Task 1 + 9 Task 2) + all 34 prior tests pass — 48 total, zero warnings

## Task Commits

Each task was committed atomically (TDD pattern):

1. **Task 1: Upgrade stack fields, depth methods, dispatch snapshot** - `590a898` (feat)
2. **Task 2: undo() and redo() implementation** - `54d60ab` (feat)
3. **Task 2: Warning fix (must_use in tests)** - `6987e5b` (fix)

## Files Created/Modified
- `src/engine.rs` — Upgraded stack field types, updated `dispatch()` Committed path, added `undo_depth()`, `redo_depth()`, `undo()`, `redo()`, added 14 new tests, added `#[derive(Debug)]` to `TestSpec`
- `src/outcome.rs` — Replaced `#[derive(Clone, PartialEq)]` on `Frame<E>` with manual impls using correct associated type bounds

## Decisions Made
- `Frame<E>` `Clone` and `PartialEq`: manual implementations required. The `derive` macro generates `impl<E: Clone>` bounds, but `E` (a unit struct like `struct TestSpec;`) never implements `Clone` — the bounds should be on `E::Input`, `E::Diff`, `E::Trace`. Manual impls with `where E::Input: Clone, E::Diff: Clone, E::Trace: Clone` are the correct approach.
- `undo()`/`redo()` return `Outcome<Frame<E>, HistoryDisallowed>` not `Outcome<Frame<E>, E::NonCommittedInfo>` — intentional asymmetry. The disallowed reason for history operations is always `HistoryDisallowed`, not game-specific non-committed info.
- Irreversible order: `redo_stack.clear()` → `undo_stack.push((prior_state, frame.clone()))` → if Irreversible: `undo_stack.clear(); redo_stack.clear()`. The push-then-clear ensures the newly committed state is current but leaves no history.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Frame<E> Clone derive adding incorrect E: Clone bound**
- **Found during:** Task 1 (stack upgrade)
- **Issue:** `#[derive(Clone)]` on `Frame<E>` generates `impl<E: EngineSpec + Clone>` — but `E` is a unit struct with no Clone impl (e.g. `struct TestSpec;`). Compilation error: "the trait bound `Frame<_>: Clone` is not satisfied"
- **Fix:** Replaced `#[derive(Clone)]` with manual `impl<E: EngineSpec> Clone for Frame<E> where E::Input: Clone, E::Diff: Clone, E::Trace: Clone`
- **Files modified:** `src/outcome.rs`
- **Verification:** `cargo test` passes; existing `frame_is_constructable_cloneable_and_eq` test covers clone
- **Committed in:** `590a898` (Task 1 commit)

**2. [Rule 1 - Bug] Fixed Frame<E> PartialEq derive adding incorrect E: PartialEq bound**
- **Found during:** Task 2 (undo test comparing frames)
- **Issue:** Same derive pattern — `#[derive(PartialEq)]` adds `E: PartialEq` bound. Error: "binary operation `==` cannot be applied to type `outcome::Frame<engine::tests::TestSpec>`"
- **Fix:** Replaced `#[derive(PartialEq)]` with manual `impl<E: EngineSpec> PartialEq for Frame<E> where E::Input: PartialEq, E::Diff: PartialEq, E::Trace: PartialEq`
- **Files modified:** `src/outcome.rs`
- **Verification:** `undo_restores_prior_state_and_returns_undone_frame` test (which asserts `frame == committed_frame`) passes
- **Committed in:** `54d60ab` (Task 2 commit)

**3. [Rule 2 - Missing Critical] Added #[derive(Debug)] to TestSpec in engine tests**
- **Found during:** Task 2 (compilation of undo tests)
- **Issue:** `Frame<E>` derives `Debug` which generates `E: Debug` bound — `TestSpec` had no `Debug` impl. Error: "`engine::tests::TestSpec` doesn't implement `Debug`"
- **Fix:** Added `#[derive(Debug)]` to `struct TestSpec` in `engine.rs` test module
- **Files modified:** `src/engine.rs`
- **Verification:** All 48 tests compile and pass
- **Committed in:** `54d60ab` (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 Rule 1 bugs, 1 Rule 2 missing critical)
**Impact on plan:** All fixes required for compilation. Root cause: Rust `derive` macros add bounds on the generic parameter `E` itself, not its associated types — standard Rust footgun for generic structs. Manual impls are the idiomatic solution. No scope creep.

## Issues Encountered

The `Frame<E>` derive-vs-manual-impl issue is the canonical Rust "derive adds wrong bounds" problem. The correct pattern (manual impls with associated type bounds) is now established in `src/outcome.rs` and documented in `patterns-established`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- `Engine::undo()`, `Engine::redo()`, `undo_depth()`, `redo_depth()` fully implemented and tested
- All HIST-01 through HIST-04 requirements completed
- Phase 3 is complete — Phase 4 (examples and integration tests) can proceed
- `backgammon.rs` example should exercise `Reversibility::Irreversible` dispatch and verify both stacks clear

## Self-Check: PASSED

- FOUND: src/engine.rs
- FOUND: src/outcome.rs
- FOUND: .planning/phases/03-history/03-02-SUMMARY.md
- FOUND commit 590a898 (Task 1)
- FOUND commit 54d60ab (Task 2)
- FOUND commit 6987e5b (warning fix)
- FOUND commit 3daa2b1 (metadata)
- 48 unit tests pass, 0 warnings

---
*Phase: 03-history*
*Completed: 2026-03-14*
