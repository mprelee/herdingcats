---
phase: quick-final-cleanup
plan: "01"
subsystem: engine
tags: [debug_assert, trace-contract, docs, irreversible-history, tests]
dependency_graph:
  requires: []
  provides: [apply-trace-contract-enforcement, irreversible-history-semantics-tests]
  affects: [src/engine.rs, src/apply.rs, src/behavior.rs, README.md]
tech_stack:
  added: []
  patterns: [debug_assert-contract-enforcement, tdd-focused-semantics-tests]
key_files:
  created: []
  modified:
    - src/engine.rs
    - src/apply.rs
    - src/behavior.rs
    - README.md
decisions:
  - debug_assert placed inside the diff loop unconditionally — any emitted diff returning zero traces is a contract violation
  - IrrevSpec defined in irreversible_history_tests submod to avoid polluting outer test namespace
  - Pre-existing clippy::type_complexity warning on BehaviorDef fn pointer is out-of-scope (pre-existing), deferred
metrics:
  duration: ~8min
  completed: 2026-03-14T05:44:02Z
  tasks_completed: 2
  files_modified: 4
---

# Phase quick-01: Final Cleanup Pass Summary

**One-liner:** Locked Apply trace contract with debug_assert in dispatch, clarified static behavior docs with "no runtime registration", and added 5 focused irreversible history tests.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add debug_assert for Apply trace contract and clarify static behavior docs | 5f8e779 | src/engine.rs, src/apply.rs, src/behavior.rs, README.md |
| 2 | Add explicit irreversible history semantics tests | 373e7e7 | src/engine.rs |

## What Was Built

### Task 1: debug_assert + doc clarifications

**engine.rs:** Added `debug_assert!(!new_traces.is_empty(), ...)` immediately after `diff.apply(working.to_mut())` in the dispatch diff loop. The assert fires in debug/test builds; it is a no-op in release builds. It catches Apply implementations that mutate state but return an empty trace slice — a contract violation that was previously silent.

**apply.rs:** Added one sentence to `Apply::apply()` doc: "The engine enforces this contract with a `debug_assert!` in dispatch — violations panic in debug/test builds."

**behavior.rs:** Added "no runtime registration" to the module-level Design section. The phrase "no trait objects, no `dyn` dispatch" was already present; the addition completes the three-part claim: no trait objects, no dyn dispatch, no runtime registration.

**README.md:** Added `## Architecture Status` section before `## Zero Dependencies` listing all five v0.5.0 architecture properties.

### Task 2: Irreversible history semantics tests

Added `irreversible_history_tests` submodule in `src/engine.rs` with 5 focused tests:

1. `irreversible_history_reversible_commit_is_undoable` — dispatch Reversible, assert undo() returns Undone(frame)
2. `irreversible_history_irreversible_commit_clears_all_history` — 2x Reversible then Irreversible, assert both depths == 0
3. `irreversible_history_after_irreversible_commit_undo_returns_nothing_to_undo` — dispatch Irreversible, call undo(), assert Disallowed(NothingToUndo)
4. `irreversible_history_after_irreversible_commit_redo_returns_nothing_to_redo` — dispatch Irreversible, call redo(), assert Disallowed(NothingToRedo)
5. `irreversible_history_irreversible_commit_preceded_by_undo_clears_redo_too` — dispatch Reversible, undo, dispatch Irreversible, assert redo_depth==0

Each test is named with `irreversible_history_` prefix and has a doc comment explaining the exact semantic being locked down.

## Verification

- `cargo test` — 72 unit tests pass (67 pre-existing + 5 new)
- `cargo test --doc` — 7 doctests + 2 compile_fail tests pass
- `cargo clippy` — no new warnings (pre-existing type_complexity on BehaviorDef fn pointer is out-of-scope)

## Deviations from Plan

None — plan executed exactly as written.

## Deferred Items

**Pre-existing `clippy::type_complexity` on `BehaviorDef.evaluate` fn pointer type** (behavior.rs line 162): not caused by this plan's changes, out of scope per deviation rules.

## Self-Check: PASSED

Files verified:
- src/engine.rs — FOUND (modified: debug_assert in dispatch + irreversible_history_tests mod)
- src/apply.rs — FOUND (modified: doc update on apply())
- src/behavior.rs — FOUND (modified: "no runtime registration" added)
- README.md — FOUND (modified: Architecture Status section added)

Commits verified:
- 5f8e779 — FOUND (Task 1)
- 373e7e7 — FOUND (Task 2)
