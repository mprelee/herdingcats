---
phase: quick-2
plan: 01
subsystem: behavior, docs
tags: [cleanup, type-alias, clippy, docs]
dependency_graph:
  requires: []
  provides: [BehaviorEval type alias, zero-warning build, polished docs]
  affects: [src/behavior.rs, src/lib.rs, src/apply.rs, README.md]
tech_stack:
  added: []
  patterns: [type alias for complex fn pointer, trusted invariant doc pattern]
key_files:
  created: []
  modified:
    - src/behavior.rs
    - src/lib.rs
    - src/apply.rs
    - README.md
decisions:
  - README Quick Start keeps .unwrap() on the Result layer; match is on the Outcome layer — dispatch returns Result<Outcome,...> so both layers need handling; unwrap is acceptable in a simple demo example
metrics:
  duration: 2min
  completed: 2026-03-14
  tasks_completed: 2
  files_modified: 4
---

# Quick Task 2: Final Cleanup — BehaviorEval Type Alias and Doc Polish Summary

**One-liner:** Added `BehaviorEval<E>` type alias to eliminate clippy::type_complexity warning, polished README Quick Start with Outcome match, and expanded Apply trace contract docs as a trusted invariant.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add BehaviorEval type alias and fix clippy warning | f1aa88a | src/behavior.rs, src/lib.rs |
| 2 | Polish README and clarify Apply contract docs | 152314b | README.md, src/apply.rs |

## What Was Built

### Task 1: BehaviorEval type alias

Added `pub type BehaviorEval<E>` above `BehaviorDef` in `src/behavior.rs`. The type alias names the evaluate fn pointer signature that was previously inlined in the `BehaviorDef.evaluate` field — this triggered clippy's `type_complexity` lint. Updated `BehaviorDef.evaluate` to use the alias. Added `BehaviorEval` to the `pub use` re-export in `src/lib.rs`.

### Task 2: README and Apply doc polish

- Updated "Architecture Status" to "Architecture Status (MVP)" in README
- Updated Quick Start: added `Outcome` import, changed `let engine` to `let mut engine`, replaced bare dispatch result with a `match outcome { Outcome::Committed(frame) => ..., other => ... }` block
- Added trace contract trusted invariant doc block (11 lines) above `pub trait Apply<E: EngineSpec>` in `src/apply.rs`, explaining why the contract cannot be enforced at the type level and how the engine uses `debug_assert!` instead

## Verification Results

- `cargo clippy` — zero warnings (type_complexity resolved)
- `cargo test` — all tests pass
- `cargo test --doc` — all doctests pass
- `cargo doc` — zero doc warnings
- `grep "BehaviorEval" src/lib.rs` — confirms re-export present

## Deviations from Plan

### Minor adjustment

**Task 2 — README dispatch call:** The plan showed `let outcome = engine.dispatch(...)` followed by `match outcome { Outcome::Committed(frame) => ... }`. However, `dispatch` returns `Result<Outcome<...>, EngineError>`, not bare `Outcome`. The plan's intent was to replace the old `.unwrap()` with a match on `Outcome` variants. Resolution: kept `.unwrap()` on the `Result` (acceptable in a simple example), then matched on the resulting `Outcome`. This accurately reflects the API while satisfying the plan's goal of showing `Outcome::Committed` handling.

## Self-Check: PASSED

- src/behavior.rs: FOUND
- src/lib.rs: FOUND
- src/apply.rs: FOUND
- README.md: FOUND
- Commit f1aa88a: FOUND
- Commit 152314b: FOUND
