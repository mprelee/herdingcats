---
phase: 02-engine-property-tests
verified: 2026-03-08T09:00:00Z
status: passed
score: 4/4 must-haves verified
re_verification: false
---

# Phase 2: Engine Property Tests Verification Report

**Phase Goal:** The engine's determinism and undo/redo correctness are machine-verifiable — proptest runs confirm all core invariants hold for arbitrary inputs
**Verified:** 2026-03-08
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                                  | Status     | Evidence                                                                                 |
|----|--------------------------------------------------------------------------------------------------------|------------|------------------------------------------------------------------------------------------|
| 1  | `cargo test` passes with proptest suites confirming arbitrary apply+undo sequences restore both state AND replay_hash | ✓ VERIFIED | `prop_01_undo_roundtrip` passes; asserts both `engine.read()` and `engine.replay_hash()` at lines 835-836 |
| 2  | `dispatch_preview` is confirmed by property test to leave state and replay_hash identical to before the call | ✓ VERIFIED | `prop_02_preview_isolation` passes; asserts state (line 868) and hash (line 873) unchanged; also verifies lifetime/enabled isolation indirectly via reference engine comparison (line 896) |
| 3  | `Turns(n)` and `Triggers(n)` rules are confirmed by property test to disable at exactly n dispatches / n `before()` calls — no off-by-one | ✓ VERIFIED | `prop_03_turns_lifetime` and `prop_03_triggers_lifetime` both pass; each asserts `trigger_count_clone.get() == n` after n dispatches, then asserts still `== n` after n+1 dispatch |
| 4  | A cancelled transaction is confirmed by property test to leave state and replay_hash bitwise identical to a snapshot taken before dispatch | ✓ VERIFIED | `prop_04_cancelled_tx_isolation` passes; sets `tx.cancelled = true`, then asserts `engine.read() == state_before` and `engine.replay_hash() == hash_before` at lines 1016-1017 |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact        | Expected                                              | Status     | Details                                                                  |
|-----------------|-------------------------------------------------------|------------|--------------------------------------------------------------------------|
| `src/engine.rs` | `#[cfg(test)] mod props` block with four proptest property tests | ✓ VERIFIED | `mod props` at line 722; 298 lines (lines 722–1020), exceeds 80-line minimum; contains all required fixtures and all five proptest functions |

**Artifact level checks:**

- Level 1 (Exists): `src/engine.rs` exists, 1020 lines total.
- Level 2 (Substantive): `mod props` block starts at line 722, runs to line 1020 (298 lines). Contains `CounterOp` fixture, `NoRule` fixture, `CountingRule` fixture with `Rc<Cell<u32>>`, `op_sequence_strategy()`, and five `proptest!` blocks covering PROP-01 through PROP-04.
- Level 3 (Wired): `mod props` is `#[cfg(test)]` and inside `src/engine.rs` — it is compiled as part of the test binary. All five property test functions are confirmed executed by `cargo test -- props` output showing 5 passed.

### Key Link Verification

| From       | To                                    | Via                                                                   | Status     | Details                                                                                         |
|------------|---------------------------------------|-----------------------------------------------------------------------|------------|-------------------------------------------------------------------------------------------------|
| `mod props` | `Engine::dispatch` + `Engine::undo`  | PROP-01: snapshot → dispatch loop → undo loop → `prop_assert_eq!`    | ✓ WIRED    | `prop_assert_eq!` at lines 835-836 asserts both `engine.read()` and `engine.replay_hash()` after full undo sequence |
| `mod props` | `Engine::dispatch_preview`           | PROP-02: snapshot state+hash → `dispatch_preview` → `prop_assert_eq!` | ✓ WIRED   | `engine.dispatch_preview((), preview_tx)` at line 865; assertions at lines 868 and 873; reference engine divergence check at line 896 |
| `mod props` | `RuleLifetime::Turns` / `RuleLifetime::Triggers` | PROP-03: `CountingRule` with `Rc<Cell<u32>> trigger_count` observable | ✓ WIRED | `trigger_count` Rc shared between `CountingRule` and test body; `trigger_count_clone.get()` asserted equal to `n` after n dispatches |
| `mod props` | `tx.cancelled`                       | PROP-04: set `tx.cancelled = true`, snapshot before, compare after dispatch | ✓ WIRED | `tx.cancelled = true` at line 1012; `engine.dispatch((), tx)` at line 1014; assertions at lines 1016-1017 |

### Requirements Coverage

| Requirement | Source Plan | Description                                                                                                   | Status      | Evidence                                                                               |
|-------------|-------------|---------------------------------------------------------------------------------------------------------------|-------------|----------------------------------------------------------------------------------------|
| PROP-01     | 02-01-PLAN  | Property test: arbitrary apply+undo sequences return original state AND original `replay_hash`               | ✓ SATISFIED | `prop_01_undo_roundtrip` asserts both `engine.read() == state_before` and `engine.replay_hash() == hash_before` (lines 835-836) |
| PROP-02     | 02-01-PLAN  | Property test: `dispatch_preview` leaves all four engine mutable fields identical after return               | ✓ SATISFIED | `prop_02_preview_isolation` verifies state and hash directly (lines 868, 873); verifies lifetime/enabled isolation indirectly via subsequent-dispatch reference comparison (line 896) |
| PROP-03     | 02-01-PLAN  | Property test: `Turns(n)` disabled after exactly n dispatches; `Triggers(n)` disabled after exactly n `before()` calls | ✓ SATISFIED | `prop_03_turns_lifetime` and `prop_03_triggers_lifetime` each assert exact boundary: fires n times, not n+1 |
| PROP-04     | 02-01-PLAN  | Property test: cancelled transaction (`tx.cancelled = true`) leaves `state` and `replay_hash` completely unchanged | ✓ SATISFIED | `prop_04_cancelled_tx_isolation` uses `NoRule`/`Permanent`, asserts state+hash only (lines 1016-1017) |

No orphaned requirements — all four PROP-01 through PROP-04 IDs declared in the PLAN frontmatter match the four requirements mapped to Phase 2 in REQUIREMENTS.md (traceability table, lines 82-85). No additional Phase 2 IDs appear in REQUIREMENTS.md that are missing from the plan.

### Anti-Patterns Found

No anti-patterns detected in `src/engine.rs` `mod props` block:

- No TODO/FIXME/placeholder comments.
- No stub implementations (`return null`, `return {}`, etc.).
- No console-log-only handlers.
- No empty `proptest!` blocks.
- Production code (`src/engine.rs` above the `#[cfg(test)]` boundary at line 582) is unchanged — the `mod props` addition is entirely within `#[cfg(test)]` scope.

### Human Verification Required

None. All four property tests execute deterministically in `cargo test`. There are no visual, real-time, or external service components in this phase.

### Test Execution Evidence

**Full property test run:**

```
running 5 tests
test engine::props::prop_03_turns_lifetime ... ok
test engine::props::prop_03_triggers_lifetime ... ok
test engine::props::prop_04_cancelled_tx_isolation ... ok
test engine::props::prop_01_undo_roundtrip ... ok
test engine::props::prop_02_preview_isolation ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out
```

**Full suite (no regressions):**

```
running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured
```

**Commits verified:**

- `631d219` — `feat(02-01): add mod props skeleton, fixtures, and PROP-01 undo roundtrip` (+124 lines to `src/engine.rs`)
- `11c9bdb` — `feat(02-01): add PROP-02, PROP-03, PROP-04 property tests` (+181 lines to `src/engine.rs`)

Both commits exist in the repository and reference the single modified file `src/engine.rs`.

### Summary

Phase 2 goal is fully achieved. All four must-have truths are verified by passing proptest runs against the actual codebase, not summary claims. The `mod props` block in `src/engine.rs` is substantive (298 lines), self-contained (fixtures re-declared, no cross-module imports from `mod tests`), and wired correctly (all five property test functions execute and pass under `cargo test`). Each requirement ID (PROP-01 through PROP-04) maps to a concrete, passing test with assertions covering the exact invariants specified. No production code was modified. No regressions in the 19 pre-existing unit tests.

---

_Verified: 2026-03-08_
_Verifier: Claude (gsd-verifier)_
