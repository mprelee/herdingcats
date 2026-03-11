---
phase: quick-3
plan: 3
subsystem: engine-api
tags: [api-cleanup, hash-algorithm, ergonomics]
dependency_graph:
  requires: []
  provides: [dispatch-simple-api, fnv1a-action-hash, clean-action-struct]
  affects: [engine.rs, action.rs, lib.rs, behavior.rs, mutation.rs, examples]
tech_stack:
  added: []
  patterns: [dispatch-simple-path, dispatch-with-custom-tx, fnv1a-concatenated-action-hash]
key_files:
  created: []
  modified:
    - src/action.rs
    - src/engine.rs
    - src/lib.rs
    - src/behavior.rs
    - src/mutation.rs
    - examples/tictactoe.rs
    - examples/backgammon.rs
decisions:
  - "dispatch(event) is the simple ergonomic path; dispatch_with(event, tx) accepts pre-built actions"
  - "replay_hash uses fnv1a of all concatenated action bytes per action (not per-mutation XOR loop)"
  - "All mutations always contribute to replay_hash — no deterministic flag gating"
metrics:
  duration_minutes: 9
  completed_date: "2026-03-11"
  tasks_completed: 2
  files_modified: 7
---

# Quick Task 3: Remove Deterministic Flag, Use FNV-1a Hash Summary

**One-liner:** Removed Action.deterministic field, replaced per-mutation XOR hash with fnv1a of concatenated action bytes, and split dispatch into `dispatch(event)` + `dispatch_with(event, tx)`.

## What Was Built

Three cleanup changes to the herdingcats engine API and hash algorithm:

1. **Removed `Action<M>.deterministic` field** — struct now has exactly two fields: `mutations: Vec<M>` and `cancelled: bool`. All mutations always contribute to replay_hash.

2. **Fixed replay_hash algorithm** — replaced the old conditional per-mutation XOR/mul loop (`if tx.deterministic { for m in &tx.mutations { xor/mul } }`) with a proper fnv1a hash of all mutation bytes concatenated per action, then folded into the running hash: `action_bytes.extend(m.hash_bytes()); replay_hash ^= fnv1a_hash(&action_bytes); replay_hash = replay_hash.wrapping_mul(FNV_PRIME)`.

3. **Ergonomic dispatch API** — added `dispatch(event) -> Option<Action<M>>` as the simple path (calls `dispatch_with(event, Action::new())`); renamed old `dispatch(event, tx)` to `dispatch_with(event, tx)` for pre-built actions.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Remove deterministic field and fix replay_hash algorithm | 38ce99f | src/action.rs, src/engine.rs, src/lib.rs |
| 2 | Add dispatch(event) convenience; rename current to dispatch_with | 31780cd | src/behavior.rs, src/mutation.rs, examples/tictactoe.rs, examples/backgammon.rs |

## Verification Results

- `cargo test --lib`: 45 tests pass
- `cargo test` (all including doctests): 67 tests pass
- `cargo clippy -- -D warnings`: no warnings
- `cargo doc --no-deps`: generates cleanly
- `grep -r "\.deterministic" src/ examples/ --include="*.rs"`: no matches
- `grep "dispatch(.*Action::new" src/ examples/ --include="*.rs"`: no matches
- `cargo run --example tictactoe`: runs correctly (X wins in 5 moves)
- `cargo run --example backgammon`: runs correctly (roll dice, move, undo, remove)

## Decisions Made

- **dispatch simple path:** `dispatch(event)` calls `dispatch_with(event, Action::new())`. This makes the common case (behaviors inject all mutations) ergonomic without breaking callers that need to pre-populate actions.
- **hash algorithm:** Concatenate all mutation `hash_bytes()` for the action into one `Vec<u8>`, apply `fnv1a_hash` once, then fold: `replay_hash ^= action_hash; replay_hash = replay_hash.wrapping_mul(FNV_PRIME)`. This is a genuine fnv1a of the full action byte sequence rather than independent per-mutation hashes.
- **No deterministic gating:** All mutations always contribute. The `deterministic` field was never set to `false` in practice; removing it simplifies the API and makes the invariant explicit.

## Deviations from Plan

**1. [Rule 2 - Missing coverage] Updated doctests in behavior.rs and mutation.rs**
- **Found during:** Task 2 verification (`cargo test` including doctests)
- **Issue:** `behavior.rs` doc examples for `is_active`, `on_dispatch`, `on_undo` and `mutation.rs` doc example for `is_reversible` still called the old `dispatch(event, tx)` signature, causing 4 doctest compilation failures
- **Fix:** Updated all 4 doctests to use `dispatch(())` (empty tx) or `dispatch_with(event, tx)` (pre-built tx) as appropriate; also updated `hash_bytes` doc in mutation.rs to describe the new concatenated algorithm
- **Files modified:** src/behavior.rs, src/mutation.rs
- **Commit:** 31780cd

## Self-Check: PASSED

- src/action.rs: FOUND
- src/engine.rs: FOUND
- 3-SUMMARY.md: FOUND
- commit 38ce99f: FOUND
- commit 31780cd: FOUND
