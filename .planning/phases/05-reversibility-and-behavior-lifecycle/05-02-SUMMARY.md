---
phase: 05-reversibility-and-behavior-lifecycle
plan: "02"
subsystem: engine
tags: [engine, reversibility, lifecycle, undo-barrier, behaviors]
dependency_graph:
  requires: [05-01]
  provides: [05-03]
  affects: [src/engine.rs]
tech_stack:
  added: []
  patterns:
    - "is_active() delegation — behaviors self-report active state instead of engine-managed HashSet"
    - "Reversibility gate — tx.mutations.iter().all(|m| m.is_reversible()) at commit time"
    - "Undo barrier — irreversible commit clears both undo and redo stacks"
    - "Lifecycle passes — separate iter_mut() after commit/undo/redo to satisfy borrow checker"
key_files:
  created: []
  modified:
    - src/engine.rs
decisions:
  - "Lifecycle passes unconditionally call on_dispatch/on_undo on ALL behaviors regardless of is_active() — per locked decision from planning"
  - "Hash update happens before reversibility branch — replay_hash is a forward-only fingerprint independent of undoability"
  - "Empty action guard (!tx.mutations.is_empty()) added to dispatch commit gate — prevents spurious on_dispatch() calls and empty CommitFrames"
metrics:
  duration: "3 minutes"
  completed_date: "2026-03-11"
  tasks_completed: 2
  files_modified: 1
---

# Phase 5 Plan 02: Engine Reversibility Gate and Lifecycle Passes Summary

Engine.rs rewritten to remove all dead code (RuleLifetime, HashMap/HashSet, snapshot fields) and wire the trait contracts from Plan 01 into the runtime: is_active() delegation, reversibility gate enforcing the undo barrier, and on_dispatch/on_undo lifecycle passes.

## What Was Built

**Dead code removal:**
- Removed `RuleLifetime` enum entirely (was a single `Permanent` variant, never used polymorphically)
- Removed `lifetime_snapshot: HashMap<&'static str, RuleLifetime>` and `enabled_snapshot: HashSet<&'static str>` from `CommitFrame`
- Removed `enabled: HashSet<&'static str>` and `lifetimes: HashMap<&'static str, RuleLifetime>` from `Engine`
- Removed `use std::collections::{HashMap, HashSet}` (no longer needed)
- Updated `add_behavior()` to remove two dead insertions; removed now-unused `let id = behavior.id()` binding

**Active state delegation:**
- All `self.enabled.contains(behavior.id())` guards in `dispatch()`, `dispatch_preview()` replaced with `behavior.is_active()`
- Behaviors now self-manage active state via `is_active()` — no engine coupling required

**Reversibility gate in dispatch():**
- Commit now requires `!tx.cancelled && !tx.mutations.is_empty()` (added empty guard)
- `tx.mutations.iter().all(|m| m.is_reversible())` determines the commit branch
- Reversible path: push `CommitFrame`, clear redo stack (unchanged semantics)
- Irreversible path: clear both undo and redo stacks (undo barrier)
- Hash update happens before the branch — replay_hash is forward-only

**Lifecycle passes:**
- `dispatch()`: after successful commit, `behavior.on_dispatch()` called on all behaviors via separate `iter_mut()`
- `undo()`: after mutation reversal and redo push, `behavior.on_undo()` called on all behaviors
- `redo()`: after mutation re-application and undo push, `behavior.on_dispatch()` called (redo = forward)
- `dispatch_preview()`: no lifecycle calls — pure dry run unchanged

**New tests added:**
- `irreversible_commit_clears_undo_and_redo_stacks` — verifies undo barrier on irreversible commit
- `reversible_commit_after_irreversible_is_undoable` — verifies new reversible commits work past barrier
- `on_dispatch_called_on_all_behaviors` — verifies lifecycle pattern compiles and state advances
- `on_dispatch_not_called_for_cancelled_action` — verifies gate: cancelled = no lifecycle pass
- `on_dispatch_not_called_for_empty_mutations` — verifies gate: empty mutations = no lifecycle pass

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written with one minor consolidation noted below.

### Consolidation Note

The plan described Task 1 (dead code removal) and Task 2 (reversibility gate + lifecycle) as separate TDD passes. Since Task 1 explicitly stated "make both changes in this task together so the file compiles after this task completes" and Task 2 added new tests to the already-compiling file, both structural changes were written in a single coherent pass. The single commit captures Task 1's removals and Task 2's additions together since they were implemented simultaneously. All tests pass and zero warnings produced.

## Verification Results

- `cargo test --lib`: 28 tests pass (0 failed), including all 3 proptest properties
- `cargo build`: zero warnings, zero errors
- Dead code grep: 0 matches for RuleLifetime/lifetime_snapshot/enabled_snapshot/self.enabled/self.lifetimes
- Lifecycle method grep: is_reversible, is_active, on_dispatch, on_undo all present in engine.rs
- `cargo run --example tictactoe`: runs without errors
- `cargo run --example backgammon`: runs without errors

## Self-Check: PASSED

- File exists: `/Users/mprelee/herdingcats/src/engine.rs` - FOUND
- Commit exists: `2673b36` - FOUND
- All 28 lib tests passing - CONFIRMED
- Zero build warnings - CONFIRMED
