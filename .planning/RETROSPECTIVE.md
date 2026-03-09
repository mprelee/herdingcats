# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

---

## Milestone: v1.0 — Refactor and Test

**Shipped:** 2026-03-09
**Phases:** 3 | **Plans:** 6 | **Timeline:** 1 day
**Commits:** 48 total (7 feat, 27 docs, remainder chore/fix)

### What Was Built

- 5-module crate split from monolithic `src/lib.rs` — `hash`, `operation`, `transaction`, `rule`, `engine` with thin re-export facade and `#![warn(missing_docs)]` compile-time gate
- Full rustdoc on all public items with paradigm-teaching prose on `Operation` and `Rule` traits; 15 runnable doctests, 0 `cargo doc` warnings
- 4 proptest property tests in `mod props` block: undo roundtrip, preview isolation, rule lifetime exactness, cancelled-tx isolation
- `examples/backgammon.rs` (865 lines): `BgState`, 4 `BackgammonOp` variants, `RollDiceRule` (non-undoable via `tx.irreversible=false`), `MoveRule`, labeled `main()` demo, 2 integration property tests
- Total test suite: 19 unit tests + 15 doctests + 5 + 2 proptest property tests — all green, zero regressions

### What Worked

- **TDD split for Phase 3**: Plan 03-01 (data model + unit tests) before Plan 03-02 (engine wiring + proptests) correctly surfaced the BACK-02 MEDIUM-confidence bearing-off risk before building property tests on top of it
- **op-level proptest generation**: Generating `Vec<BackgammonOp>` directly (bypassing event/rule layer) for BACK-05 kept the strategy simple and decoupled from game-legality
- **CONTEXT.md design decisions**: Having locked decisions (BgState shape, `tx.irreversible` usage, op-level generation) before planning made plans specific and research-confirmable rather than speculative
- **Sibling `mod tests` + `mod props`**: Clean separation between deterministic unit tests and property-based tests — established in Phase 2, naturally reused in Phase 3
- **Phase 2 → Phase 3 pattern inheritance**: `prop_assert_eq!(engine.read(), ...) + prop_assert_eq!(engine.replay_hash(), ...)` established in PROP-01, directly reused in BACK-06 with no rework

### What Was Inefficient

- **Nyquist VALIDATION.md sign-off gap**: All 3 phases have VALIDATION.md files with accurate validation strategies, but `nyquist_compliant` was never set to `true` during execution. The validation happened implicitly (tests ran, passed), but the sign-off wasn't automated. Consider automating the sign-off check after wave completion.
- **MILESTONES.md CLI gap**: `gsd-tools milestone complete` produced an empty accomplishments list because the `one_liner` field in SUMMARY.md frontmatter wasn't populated by gsd-executor. Either populate `one_liner` during plan execution, or accept manual entry at milestone close.
- **Doctest `transaction.rs` placeholder comment**: The `// A Transaction<i32> where the op type is i32 (placeholder type here)` comment is technically accurate but reads as a stub to automated scanners. A clearer comment would avoid the confusion.

### Patterns Established

- **`tx.irreversible = false` in `rule.before()`**: The canonical pattern for non-undoable events (dice rolls, random draws) — push ops to state but skip CommitFrame. Document this in public API docs as a first-class pattern.
- **Separate `ModuleOp` variant for special cases**: `BearOffOp` as distinct variant (not sentinel in `MoveOp.to`) — whenever a move type has fundamentally different undo semantics, use a dedicated variant.
- **Two-assertion undo test contract**: Always assert both `engine.read()` and `engine.replay_hash()` after undo — state-only assertion is insufficient to detect hash corruption. This is now locked in PROP-01 and BACK-06.
- **Test fixture isolation**: Use minimal zero-initialized state structs for unit tests of individual op variants, not `State::new()` which may contain pre-existing values that interfere.

### Key Lessons

1. **MEDIUM-confidence architectural items need explicit TDD phase ordering**: The `[i8; 26]` bearing-off concern (STATE.md) was resolved correctly by making Phase 3 Wave 1 a TDD plan that validated the data model before Wave 2 built property tests on top. Future milestones: identify confidence gaps early and structure waves to validate them first.
2. **The `tx.irreversible` naming inversion is a user-facing hazard**: New implementors will misread `tx.irreversible = false` as "this is irreversible" rather than "don't record this for undo." Consider renaming to `tx.record_undo` (default `true`) in v2 to match behavioral intent.
3. **Op-level proptest strategies are preferable to event-level for invariant tests**: Testing conservation properties at the op level decouples the property test from game-rule correctness. Reserve event-level strategies for integration tests that specifically require rule system participation.

### Cost Observations

- Model mix: 100% sonnet (all agents — researcher, planner, checker, executor, verifier)
- Sessions: 4 (plan-phase, execute-phase, audit-milestone, complete-milestone)
- Notable: 3-phase milestone completed in 1 day with zero gap-closure iterations — clean first-pass execution due to strong CONTEXT.md pre-planning

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 3 | 6 | First milestone — baseline established |

### Cumulative Quality

| Milestone | Unit Tests | Doctests | Property Tests | Regressions |
|-----------|-----------|----------|----------------|-------------|
| v1.0 | 19 | 15 | 7 (5 engine + 2 backgammon) | 0 |

### Top Lessons (Verified Across Milestones)

1. Structure waves so MEDIUM-confidence architectural concerns are validated (TDD phase) before dependent phases build on top of them.
2. Always assert both `engine.read()` and `engine.replay_hash()` after undo — single-state assertion misses hash corruption.
