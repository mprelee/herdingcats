---
phase: 04-examples-and-tests
plan: "01"
subsystem: examples
tags: [example, tutorial, tictactoe, dispatch, undo, redo, outcome]
dependency_graph:
  requires: []
  provides: [tictactoe-example]
  affects: []
tech_stack:
  added: []
  patterns:
    - EngineSpec unit struct with all 6 associated types wired
    - Behavior zero-size structs with order_key deterministic ordering
    - Apply<E> on Diff enum with per-variant trace emission
    - Exhaustive match on all 7 Outcome variants with comment for unreachable arm
key_files:
  created:
    - examples/tictactoe.rs
  modified: []
decisions:
  - CheckWin behavior uses _input to simulate post-placement board state since behaviors evaluate against pre-apply state
  - NoChange demonstrated via a second zero-behavior engine (cleanest approach given 4-behavior main engine always emits diffs)
  - ValidateTurn guards game_over with Stop → Aborted (not NoChange) — avoids need for a separate GameOverGuard behavior
  - Disallowed demonstrated by draining undo stack to empty then calling undo() once more
metrics:
  duration: "2min"
  completed: "2026-03-14"
  tasks: 1
  files: 1
---

# Phase 4 Plan 01: Tic-Tac-Toe Scripted Demo Summary

Scripted 4-behavior tic-tac-toe demo (ValidateTurn, ValidateCell, PlaceMarker, CheckWin) exercising all 6 runtime Outcome variants plus exhaustive InvalidInput match arm, readable top-to-bottom as a HerdingCats API tutorial.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement tictactoe.rs scripted demo | 6457df8 | examples/tictactoe.rs |

## Outcome Coverage

All 7 Outcome variant arms are present in the match statements:

| Variant | Demonstrated At Runtime | How |
|---------|------------------------|-----|
| Committed | Steps 1, 2, 8 | Successful moves; X wins row 0 in step 8 |
| Aborted | Steps 3, 6, 9 | Occupied cell, out-of-bounds, post-game-over |
| Undone | Step 4 | undo() on non-empty stack |
| Redone | Step 5 | redo() after undo |
| NoChange | Step 7 | Dispatch into zero-behavior engine |
| Disallowed | Step 10 | undo() on exhausted stack → NothingToUndo |
| InvalidInput | Match arm only | Comment: unreachable via dispatch in MVP engine |

## Verification Results

- `cargo run --example tictactoe` exits 0, no panics
- All 6 runtime Outcome variants appear in terminal output
- `[dispatch]`, `[undo]`, `[redo]` labels present throughout
- 71 tests pass (63 unit + 6 doc + 2 compile-fail), 0 regressions
- File is 486 lines (minimum: 150) — readable as annotated tutorial

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### Design Clarifications (not deviations)

**1. CheckWin uses _input for hypothetical board check**
- **Context:** Behaviors evaluate against pre-apply state, so the board doesn't have the new marker yet when CheckWin runs.
- **Resolution:** CheckWin reads _input to get the target cell and builds a hypothetical board to test for 3-in-a-row. This is correct per the architecture — behaviors may read the input freely.

**2. NoChange via a second engine**
- **Context:** The main 4-behavior engine always emits at least one diff for valid moves or returns Stop for invalid ones. There is no natural NoChange path in the 4-behavior game loop.
- **Resolution:** Created a local `no_behavior_engine` with an empty behavior list. Dispatch with no behaviors and no diffs → NoChange. This is the cleanest way to demonstrate the variant without contriving game logic.

## Self-Check: PASSED

- examples/tictactoe.rs — FOUND
- commit 6457df8 — FOUND
- 04-01-SUMMARY.md — FOUND
