---
phase: 06-fill-gaps
plan: "01"
subsystem: behavior-dispatch
completed: "2026-03-14"
duration: "15min"
tags: [behavior, dispatch, refactor, fn-pointers, trace]
dependency_graph:
  requires: []
  provides: [BehaviorDef-struct, fn-pointer-dispatch, trace-contract-tests]
  affects: [src/behavior.rs, src/engine.rs, src/lib.rs, src/apply.rs, src/outcome.rs, examples/tictactoe.rs, examples/backgammon.rs]
tech_stack:
  added: []
  patterns: [fn-pointer-dispatch, plain-struct-behaviors]
key_files:
  created: []
  modified:
    - src/behavior.rs
    - src/engine.rs
    - src/lib.rs
    - src/apply.rs
    - src/outcome.rs
    - examples/tictactoe.rs
    - examples/backgammon.rs
decisions:
  - BehaviorDef is a plain struct with fn pointer fields — no trait objects, no dyn dispatch
  - Debug impl for BehaviorDef is manual — prints evaluate field as fn(...) placeholder
  - Engine tests use named standalone fn pointers (not closures) — fn pointers cannot close over values
  - Examples updated alongside library code — examples are part of the public API contract
  - outcome.rs doc link updated from Behavior trait to BehaviorDef struct
requirements: [GAP-01, GAP-02, GAP-03]
metrics:
  tasks_completed: 2
  files_modified: 7
---

# Phase 6 Plan 01: BehaviorDef Struct + Trace Contract Tests Summary

**One-liner:** Replaced `Behavior` trait + `Box<dyn Behavior<E>>` with `BehaviorDef<E>` plain struct using fn pointer `evaluate` field; added two trace-contract tests to apply.rs.

## What Was Built

Eliminated trait objects from the behavior system. The `Behavior<E>` trait and all `Box<dyn Behavior<E>>` patterns have been removed from the codebase. In their place, `BehaviorDef<E>` is a plain struct with three public fields: `name: &'static str`, `order_key: E::OrderKey`, and `evaluate: fn(&E::Input, &E::State) -> BehaviorResult<E::Diff, E::NonCommittedInfo>`.

The engine now stores `Vec<BehaviorDef<E>>`, sorts by `(order_key, name)` using struct field access, and calls `(behavior.evaluate)(&input, &*working)` — the parentheses required because Rust would otherwise parse it as a method call.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Replace Behavior trait with BehaviorDef struct + update engine + lib.rs | 35cc33b | src/behavior.rs, src/engine.rs, src/lib.rs, examples/tictactoe.rs, examples/backgammon.rs |
| 2 | Add trace contract tests to apply.rs | d1754d3 | src/apply.rs |

## Decisions Made

- **BehaviorDef is a plain struct:** No trait implementation needed. Users populate `name`, `order_key`, and `evaluate` fields directly. Matches user preference for fn pointers over dyn-safety patterns.
- **Manual Debug impl:** fn pointers do not derive Debug in a useful way. The impl prints `evaluate` as `"fn(...)"` — enough to identify the struct in debug output without noise.
- **Named standalone fns for tests:** Engine tests need behaviors with different diff values (10, 20, 30, etc.). Since fn pointers cannot close over values, each distinct diff-byte value requires its own named fn (`tracing_10_eval`, `tracing_20_eval`, etc.).
- **Examples updated alongside library:** Both `tictactoe.rs` and `backgammon.rs` used `Behavior` trait and `Box<dyn Behavior<...>>`. Updated to use `BehaviorDef` with named fn pointers. The `CheckWin` struct with its `has_winner` helper became a module-level `has_winner` fn + `check_win` fn.
- **outcome.rs doc link fixed:** The `NonCommittedOutcome` rustdoc referenced `crate::behavior::Behavior` (broken link). Updated to `crate::behavior::BehaviorDef` — `cargo doc --no-deps` now compiles cleanly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Broken intra-doc link in outcome.rs**
- **Found during:** Task 1 verification (cargo doc --no-deps)
- **Issue:** `NonCommittedOutcome` rustdoc linked to `crate::behavior::Behavior` which no longer exists
- **Fix:** Updated link to `crate::behavior::BehaviorDef`
- **Files modified:** src/outcome.rs
- **Commit:** 3bddd69

**2. [Rule 1 - Bug] Examples also used Behavior trait**
- **Found during:** Task 1 execution (cargo test compilation)
- **Issue:** examples/tictactoe.rs and examples/backgammon.rs imported `Behavior` and used `Box<dyn Behavior<...>>`
- **Fix:** Converted struct+impl patterns to standalone fn pointers + BehaviorDef construction
- **Files modified:** examples/tictactoe.rs, examples/backgammon.rs
- **Commit:** 35cc33b (included in main task commit)

## Verification Results

```
cargo test: 67 passed; 0 failed (65 unit + 2 new trace_contract)
cargo doc --no-deps: 0 warnings
grep trait Behavior src/: OK (removed)
grep dyn Behavior src/: OK (removed)
grep Box::new.*Behavior src/: OK (removed)
grep BehaviorDef src/lib.rs: pub use crate::behavior::{BehaviorDef, BehaviorResult}
cargo test trace_contract: 2 passed
```

## Self-Check: PASSED

All source files exist. All commits present in git log. 67 tests pass.
