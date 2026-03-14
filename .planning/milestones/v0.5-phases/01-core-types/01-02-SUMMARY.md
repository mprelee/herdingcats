---
phase: 01-core-types
plan: 02
subsystem: core
tags: [rust, trait, behavior, outcome, engine-error, frame, type-system]

# Dependency graph
requires:
  - phase: 01-01
    provides: EngineSpec trait with 6 associated types and correct bounds
provides:
  - Behavior<E: EngineSpec> trait with 3 dyn-safe methods
  - BehaviorResult<D, O> enum with Continue/Stop variants
  - Outcome<F, N> enum with all 7 variants (#[must_use])
  - Frame<E: EngineSpec> struct with input/diff/trace public fields
  - EngineError enum with 3 variants (#[non_exhaustive])
  - Flat crate re-exports: use herdingcats::{EngineSpec, Behavior, BehaviorResult, Outcome, Frame, EngineError}
affects:
  - 02-dispatch (WorkingState<E> dispatch uses Behavior<E>, returns Outcome<Frame<E>, E::NonCommittedInfo>)
  - 03-history (Frame<E> is the history element; EngineError used in undo/redo results)
  - 04-examples (tictactoe/backgammon implement Behavior<E> and match on Outcome variants)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Structural mutation prevention: evaluate(&self, input: &E::Input, state: &E::State) — immutable borrow enforced by type system, no run-time checks"
    - "Dyn-safe Behavior<E>: no method-level generics, Box<dyn Behavior<MySpec>> is valid"
    - "Flat crate API: private mod declarations + pub use re-exports — users write use herdingcats::X not use herdingcats::spec::X"
    - "#[non_exhaustive] on EngineError with wildcard-required doc example; #[must_use] on Outcome"
    - "compile_fail doc-test pattern: demonstrates *state = Default::default() fails to compile"

key-files:
  created:
    - src/behavior.rs
    - src/outcome.rs
  modified:
    - src/lib.rs
    - examples/tictactoe.rs
    - examples/backgammon.rs

key-decisions:
  - "Outcome is NOT #[non_exhaustive] — 7 variants are a stable public contract; downstream can match exhaustively"
  - "EngineError IS #[non_exhaustive] — engine internals may surface new error kinds in future versions"
  - "lib.rs uses private mod (not pub mod) for all 3 modules — rustdoc inlines docs, users see flat herdingcats:: namespace"
  - "compile_fail doc-test on Behavior::evaluate documents structural mutation prevention for CORE-02"

patterns-established:
  - "Behavior pattern: implement name()/order_key()/evaluate() for each game rule; engine sorts by (order_key, name)"
  - "Outcome matching: exhaustive match on all 7 variants — no wildcard needed (not #[non_exhaustive])"

requirements-completed: [CORE-02, CORE-03, CORE-04, CORE-05]

# Metrics
duration: 3min
completed: 2026-03-14
---

# Phase 1 Plan 02: Behavior, Outcome, Frame, and EngineError Summary

**`Behavior<E>` trait (3 dyn-safe methods, structural mutation prevention via immutable state borrow), `BehaviorResult<D,O>` enum, `Outcome<F,N>` with 7 variants (#[must_use]), `Frame<E>` struct, `EngineError` (#[non_exhaustive]) — all wired via flat `herdingcats::*` API in private-module lib.rs**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-14T00:00:44Z
- **Completed:** 2026-03-14T00:03:29Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Defined `Behavior<E: EngineSpec>` trait with 3 methods; `evaluate()` takes `&E::State` (structural compile-time mutation prevention documented via `compile_fail` doc-test)
- Defined `Outcome<F, N>` with all 7 variants (`Committed`, `Undone`, `Redone`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted`) with `#[must_use]` — stable public contract, not `#[non_exhaustive]`
- Wired flat `herdingcats::*` API: private `mod` declarations + `pub use` re-exports — downstream code uses `use herdingcats::{EngineSpec, Behavior, BehaviorResult, Outcome, Frame, EngineError}` with no module path
- 12 unit tests + 4 doc-tests (including 1 `compile_fail`) all passing; clippy and doc both clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Write Behavior trait and BehaviorResult enum** - `ffe808e` (feat)
2. **Task 2: Write Outcome, Frame, and EngineError** - `3e712cd` (feat)
3. **Task 3: Wire lib.rs re-exports and verify full API surface** - `7959c72` (feat)

**Plan metadata:** (pending final docs commit)

_Note: TDD tasks — tests and implementation written together (tests require trait/types to compile)_

## Files Created/Modified
- `src/behavior.rs` - Behavior<E> trait and BehaviorResult<D, O> enum with full rustdoc and 5 unit tests
- `src/outcome.rs` - Outcome<F, N>, Frame<E>, EngineError with full rustdoc and 6 unit tests
- `src/lib.rs` - Replaced `pub mod spec` with private mod declarations + 6 flat pub use re-exports
- `examples/tictactoe.rs` - Added `fn main() {}` stub (Phase 4 placeholder)
- `examples/backgammon.rs` - Added `fn main() {}` stub (Phase 4 placeholder)

## Decisions Made
- `Outcome` is **not** `#[non_exhaustive]` — the 7 variants are a stable public contract; downstream code can match exhaustively without a wildcard arm
- `EngineError` **is** `#[non_exhaustive]` — engine internals may surface new error kinds in future versions; doc-example shows required wildcard arm
- `lib.rs` uses private `mod` (not `pub mod`) for all 3 modules — matches ARCHITECTURE.md spec and prevents `herdingcats::spec::EngineSpec` sub-path exposure

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] TestSpec struct in outcome.rs tests missing derives**
- **Found during:** Task 2 (TDD verify step)
- **Issue:** `TestSpec` in `#[cfg(test)]` had no `#[derive(Debug, Clone, PartialEq)]`; `Frame<TestSpec>` clone/PartialEq propagate from the spec's field types, requiring the spec itself to satisfy those bounds
- **Fix:** Added `#[derive(Debug, Clone, PartialEq)]` to test-local `TestSpec`; also removed unreachable `_ => "unknown"` wildcard from internal `EngineError` match (within-crate `#[non_exhaustive]` doesn't require wildcard)
- **Files modified:** `src/outcome.rs`
- **Verification:** `cargo test --lib -- outcome` passed with 6 tests
- **Committed in:** `3e712cd` (Task 2 commit)

**2. [Rule 3 - Blocking] Empty example files prevented cargo test**
- **Found during:** Task 3 (full cargo test)
- **Issue:** `examples/tictactoe.rs` and `examples/backgammon.rs` were empty placeholder files; Rust requires a `main` function in binary examples, causing `E0601` compile errors
- **Fix:** Added minimal `fn main() {}` stub to each file with a comment noting Phase 4 intent
- **Files modified:** `examples/tictactoe.rs`, `examples/backgammon.rs`
- **Verification:** `cargo test` passed with all 16 tests
- **Committed in:** `7959c72` (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation and correctness. No scope creep.

## Issues Encountered
- `#[non_exhaustive]` on `EngineError`: within the same crate the wildcard arm is unreachable (compiler enforces this correctly); the wildcard is required only in downstream crates. Documented via doc-test instead of unit test.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Complete Phase 1 type surface stable: `EngineSpec`, `Behavior`, `BehaviorResult`, `Outcome`, `Frame`, `EngineError` all accessible at `herdingcats::*`
- Phase 2 (Dispatch) can immediately write `use herdingcats::{EngineSpec, Behavior, Outcome, Frame}` and begin `WorkingState<E>` and `Engine` implementation
- Zero warnings, clippy clean, doc clean — no technical debt to carry forward
- Phase 2 flag from STATE.md still applies: `Apply<S>` and `Traced<T>` trait bounds need validation against backgammon use case

---
*Phase: 01-core-types*
*Completed: 2026-03-14*
