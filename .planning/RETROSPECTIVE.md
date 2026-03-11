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

## Milestone: v1.1 — Rename & Reversibility

**Shipped:** 2026-03-11
**Phases:** 4 (Phases 4-7) | **Plans:** 10 | **Timeline:** 2 days
**Commits:** 47 total (feat, docs, test, chore)

### What Was Built

- Full API rename: `Operation→Mutation`, `Rule→Behavior`, `Transaction→Action`, `add_rule→add_behavior`; `RuleLifetime` removed from public API; version bumped to 0.3.0
- Opt-out reversibility: `is_reversible()` default `true` on `Mutation` trait; engine enforces undo barrier (`undo_stack.clear()` + `redo_stack.clear()`) on irreversible commit
- Self-managing behavior lifecycle: `is_active`, `on_dispatch`, `on_undo` default methods on `Behavior` trait; engine calls them in separate `iter_mut()` passes after state mutations
- Property tests `prop_05`/`prop_06` in `mod props` machine-verifying irreversible barrier and reversible-after-barrier semantics; backgammon updated to `RollDiceOp.is_reversible()=false`
- Full rustdoc with Mealy/Moore state machine framing, ASCII dispatch pipeline diagram, and runnable doctests on `is_reversible`, `is_active`, `on_dispatch`, `on_undo`
- Extended edge-case unit tests: empty action, all-irrev, mixed mutations, on_undo fires, deactivation isolation (35 unit tests + 21 doctests total)

### What Worked

- **Interface-first rename**: Establishing new API contracts (Mutation/Behavior/Action) as trait/struct files before rewiring engine.rs prevented "rename mid-engine" confusion — Phase 4 plan 01 created new files, plan 02 rewired engine, clean sequencing
- **Separate lifecycle passes**: Deciding to call `on_dispatch`/`on_undo` in separate `iter_mut()` loops (after state mutation loop) immediately resolved the borrow conflict concern — no borrow checker fights during implementation
- **Rc<Cell<u32>> pattern**: Using `Rc<Cell<u32>>` to observe boxed behavior internal state from test scope (after `add_behavior` moves it) was the correct pattern — established cleanly in Phase 6 and reused in Phase 7 doctests
- **prop_06 structured strategy**: Using `(prefix, suffix)` proptest strategy for prop_06 (guarantees Irrev barrier present + known state arithmetic) was more reliable than generating arbitrary sequences — avoids degenerate cases where no irreversible commit occurs
- **Phase 7 last**: Writing doctests after Phase 6 finalized the API (especially `can_undo()`/`can_redo()`) meant doctests required zero rework

### What Was Inefficient

- **Phase 5 SUMMARY.md frontmatter**: `requirements_completed` field not populated in 05-01 and 05-02 SUMMARY files — caused "partial" status in 3-source audit cross-reference. Root cause: executor didn't include this field. Either populate during execution or accept as known gap.
- **Nyquist VALIDATION.md sign-off gap** (same as v1.0): Phase 4, 6, 7 have no VALIDATION.md; Phase 5 VALIDATION.md exists but not completed. Validation happened implicitly but sign-off wasn't recorded.
- **MILESTONES.md CLI gap** (same as v1.0): `gsd-tools milestone complete` returned empty accomplishments because `one_liner` frontmatter field is never populated by executor.
- **Phase 4 plan count**: ROADMAP listed 3 plans but 4 were executed (04-04 added for dead_code gap closure). The initial plan count estimate was slightly off; gap required re-verification pass.

### Patterns Established

- **`is_reversible()` opt-out pattern**: Default `true` on `Mutation` means simple mutations are zero-cost; override only for inputs that commit visible game state (dice rolls, drawing cards). This is the canonical Mealy machine "irreversible input" pattern.
- **Lifecycle pass ordering**: `on_dispatch()`/`on_undo()` calls ALWAYS happen in a separate `iter_mut()` pass after all state mutations are applied — never interleaved. This prevents borrow conflicts and ensures all state is visible to lifecycle hooks.
- **Unconditional lifecycle calls**: `on_dispatch`/`on_undo` fire on ALL behaviors regardless of `is_active()` status — sleeping behaviors still track history. Only `before`/`after` hooks are gated by `is_active()`.
- **`can_undo()`/`can_redo()` for external crates**: Private stack fields accessed directly in `mod props` (sibling module); public `can_undo()`/`can_redo()` methods for external callers (examples, downstream crates). The boundary is the crate API.
- **Action turbofish for empty dispatches**: `engine.dispatch(input, Action::<MyMutation>::new())` — Rust cannot infer `M` with zero mutations pushed; turbofish needed in doctests and empty-action tests.

### Key Lessons

1. **Separate lifecycle passes before touching engine**: The decision to use separate `iter_mut()` loops for lifecycle was made during CONTEXT.md planning (locked decision). Locking this early prevented the borrow checker fight from becoming a discovery during execution.
2. **Trait default methods are zero-cost extension points**: Adding `is_reversible()`, `is_active()`, `on_dispatch()`, `on_undo()` as default-no-op methods meant all existing implementors compiled without changes — API evolution without ecosystem breakage.
3. **Mealy/Moore framing in docs pays dividends**: Writing `lib.rs` crate-level docs with explicit Mealy machine terminology gave all subsequent phase work a consistent vocabulary — the ASCII dispatch pipeline diagram served as a reference throughout Phases 5-7.

### Cost Observations

- Model mix: 100% sonnet (all agents — planner, plan-checker, executor, verifier, integration-checker, auditor)
- Sessions: 4+ (plan-phase × 4, execute-phase × 4, audit-milestone, complete-milestone)
- Notable: Zero gap-closure phases needed — tech_debt status only; all 25/25 requirements satisfied on first pass across all 4 phases

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 3 | 6 | First milestone — baseline established |
| v1.1 | 4 | 10 | Breaking rename + reversibility model; interface-first sequencing; trait default methods |

### Cumulative Quality

| Milestone | Unit Tests | Doctests | Property Tests | Regressions |
|-----------|-----------|----------|----------------|-------------|
| v1.0 | 19 | 15 | 7 (5 engine + 2 backgammon) | 0 |
| v1.1 | 35 | 21 | 8 (6 engine + 2 backgammon) | 0 |

### Top Lessons (Verified Across Milestones)

1. Structure waves so MEDIUM-confidence architectural concerns are validated (TDD phase) before dependent phases build on top of them.
2. Always assert both `engine.read()` and `engine.replay_hash()` after undo — single-state assertion misses hash corruption.
3. Lock "separate lifecycle passes" and similar borrow-constraint decisions in CONTEXT.md before planning — prevents executor discovery of blocking issues mid-plan.
4. Trait default methods are the right API evolution pattern for zero-cost extension — all existing implementors compile unchanged, new behavior is opt-in.
