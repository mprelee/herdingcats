---
phase: 02-dispatch
plan: 02
subsystem: core-types
tags: [rust, trait-bounds, apply, reversibility, frame, engine-spec]

# Dependency graph
requires:
  - phase: 02-01
    provides: Apply<E> trait and Reversibility enum (created but not yet wired)
  - phase: 01-core-types
    provides: EngineSpec, Frame, Outcome, Behavior — base contracts being updated
provides:
  - EngineSpec::Diff carries Apply<Self> bound (compile-enforced)
  - Frame<E> with diffs: Vec<E::Diff>, traces: Vec<E::Trace>, reversibility: Reversibility
  - herdingcats::Apply and herdingcats::Reversibility re-exported at crate root
affects:
  - 02-03 (engine.rs dispatch loop consumes Apply directly and builds Frame with Vec fields)
  - 03-history (reads Frame.reversibility to decide undo eligibility)
  - 04-examples (uses flat herdingcats::Apply namespace)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "EngineSpec: Sized — all EngineSpec impls are concrete unit structs; Sized is always true and resolves Apply<E: EngineSpec> implicit Sized requirement"
    - "Apply<TestSpec> for u8 pattern — test modules use primitive types as Diff; each test module adds its own impl in cfg(test) to satisfy the new bound without orphan issues"
    - "Diff type in apply::tests::TestSpec changed from u8 to AppendByte — AppendByte was already defined in that module as the concrete Apply implementor"

key-files:
  created: []
  modified:
    - src/spec.rs
    - src/outcome.rs
    - src/lib.rs
    - src/apply.rs
    - src/behavior.rs

key-decisions:
  - "Added EngineSpec: Sized bound to resolve Apply<E: EngineSpec> implicit Sized requirement — all EngineSpec implementors are concrete unit structs so Sized is always satisfied"
  - "Each test module adds its own Apply<TestSpec> for u8 impl in cfg(test) to bring u8 into compliance with the new Diff bound — avoids orphan rule issues"
  - "apply::tests::TestSpec changes Diff from u8 to AppendByte — AppendByte is defined in that module and already has the Apply impl; keeps tests self-contained"

patterns-established:
  - "Frame accumulates Vec<Diff> and Vec<Trace> — dispatch is multi-behavior; a single frame records all diffs/traces from all behaviors that fired"
  - "Reversibility stored on Frame — Phase 3 reads Frame.reversibility to determine undo eligibility without threading a separate context"

requirements-completed: [DISP-01, DISP-03]

# Metrics
duration: 15min
completed: 2026-03-13
---

# Phase 2 Plan 02: Type Corrections Summary

**EngineSpec::Diff bound upgraded to Apply<Self>, Frame<E> fields corrected to Vec<Diff>/Vec<Trace>/Reversibility, and Apply+Reversibility re-exported at crate root — all 19 lib tests and 7 doc tests pass**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-13T00:45:00Z
- **Completed:** 2026-03-13T01:00:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- `EngineSpec::Diff` now carries `Apply<Self>` as a required trait bound — the compiler enforces that any Diff type must know how to apply itself to the game state
- `Frame<E>` fields corrected from single `diff`/`trace` to `diffs: Vec<E::Diff>` / `traces: Vec<E::Trace>` / `reversibility: Reversibility` — matches the dispatch accumulation semantics
- `herdingcats::Apply` and `herdingcats::Reversibility` are now in the flat public namespace, unblocking doc tests that referenced them

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Apply<Self> bound to EngineSpec::Diff** - `cf9fee7` (feat)
2. **Task 2: Update Frame<E> fields + wire lib.rs** - `da26954` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `src/spec.rs` — Added `use crate::apply::Apply`, `EngineSpec: Sized` bound, `Apply<Self>` on Diff associated type, updated doctest, Apply<TestSpec> for u8 in test module
- `src/outcome.rs` — Added Reversibility import, changed Frame fields to Vec<Diff>/Vec<Trace>/Reversibility, updated test helper and added two new frame tests
- `src/lib.rs` — Added `pub use crate::apply::Apply` and `pub use crate::reversibility::Reversibility` re-exports
- `src/apply.rs` — Changed TestSpec::Diff from u8 to AppendByte; derived Debug+Clone on AppendByte; reordered struct before TestSpec impl
- `src/behavior.rs` — Added Apply<TestSpec> for u8 impl in test module to satisfy the new Diff bound

## Decisions Made
- Added `EngineSpec: Sized` — Rust requires the type parameter of `Apply<E: EngineSpec>` to be `Sized` (implicit). Since all EngineSpec impls are concrete unit structs, this is always true and adding it is strictly correct.
- Each test module's `Apply<TestSpec> for u8` impl lives in `#[cfg(test)]` — avoids touching library code for test-only compatibility shims.
- `apply::tests::TestSpec::Diff` changed to `AppendByte` (not u8) — AppendByte was the existing concrete type in that module; using it as the Diff type makes the spec consistent without adding a separate primitive impl.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `EngineSpec: Sized` bound**
- **Found during:** Task 1 (adding Apply<Self> bound)
- **Issue:** `Apply<E: EngineSpec>` has an implicit Sized requirement on E. Without `EngineSpec: Sized`, the `type Diff: ... + Apply<Self>` line fails to compile.
- **Fix:** Added `: Sized` to the `EngineSpec` trait definition
- **Files modified:** src/spec.rs
- **Verification:** cargo test --lib passes with all 19 tests green
- **Committed in:** cf9fee7 (Task 1 commit)

**2. [Rule 1 - Bug] Updated apply::tests::TestSpec to use AppendByte as Diff**
- **Found during:** Task 1 (adding Apply<Self> bound)
- **Issue:** apply::tests::TestSpec had `type Diff = u8` but u8 doesn't impl Apply<TestSpec> in that module (AppendByte does). The plan said to add Apply for u8, but that would conflict with the existing AppendByte impl.
- **Fix:** Changed TestSpec::Diff to AppendByte, added `#[derive(Debug, Clone)]` to AppendByte
- **Files modified:** src/apply.rs
- **Verification:** apply module tests pass
- **Committed in:** cf9fee7 (Task 1 commit)

**3. [Rule 1 - Bug] Updated spec.rs doctest to derive Debug+Clone on MyDiff**
- **Found during:** Task 2 (running doc tests)
- **Issue:** The updated spec.rs doctest used `struct MyDiff(u8)` without Debug or Clone, causing doc test failure since EngineSpec::Diff requires both.
- **Fix:** Added `#[derive(Debug, Clone)]` to MyDiff in the doctest
- **Files modified:** src/spec.rs
- **Verification:** cargo test --doc passes
- **Committed in:** cf9fee7 (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 1 — bugs discovered during implementation)
**Impact on plan:** All fixes required for correct compilation. No scope creep; changes confined to the exact files the plan specified plus behavior.rs (test module only).

## Issues Encountered
- The `Apply<Self>` bound caused a cascade: every test module that used `type Diff = u8` without an Apply impl needed updating. The plan anticipated this for spec.rs, outcome.rs, and behavior.rs but not apply.rs (which had a different concrete type already).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `EngineSpec` is now fully contracted: Diff carries Apply<Self>, Frame carries Vec fields and Reversibility
- Plan 02-03 (engine.rs dispatch loop) can now be written against correct types from day one
- `herdingcats::Apply` and `herdingcats::Reversibility` are in the public API — doc examples across the codebase can use the flat namespace

---
*Phase: 02-dispatch*
*Completed: 2026-03-13*
