# Roadmap: herdingcats — Rename & Reversibility

## Milestones

- ✅ **v1.0 Refactor and Test** — Phases 1-3 (shipped 2026-03-09)
- 🚧 **v1.1 Rename & Reversibility** — Phases 4-7 (in progress)

## Phases

<details>
<summary>✅ v1.0 Refactor and Test (Phases 1-3) — SHIPPED 2026-03-09</summary>

- [x] Phase 1: Module Split and Foundation (3/3 plans) — completed 2026-03-09
- [x] Phase 2: Engine Property Tests (1/1 plan) — completed 2026-03-09
- [x] Phase 3: Backgammon Example and Integration Properties (2/2 plans) — completed 2026-03-09

Full details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

### 🚧 v1.1 Rename & Reversibility (In Progress)

**Milestone Goal:** Refine the public API naming and reversibility model to better reflect Mealy/Moore state machine design intent for turn-based games.

- [x] **Phase 4: Core Rename** - Rename Operation→Mutation, Rule→Behavior, Transaction→Action; remove RuleLifetime; all names consistent across codebase (completed 2026-03-11)
- [x] **Phase 5: Reversibility and Behavior Lifecycle** - Add is_reversible() to Mutation, derive Action reversibility, enforce undo barrier, add is_active/on_dispatch/on_undo to Behavior (completed 2026-03-11)
- [x] **Phase 6: Tests and Examples** - Proptest coverage for new reversibility model, stateful behavior unit test, update both examples to compile and run under new API (completed 2026-03-11)
- [x] **Phase 7: Documentation and Extended Tests** - Comprehensive rustdoc for all renamed types and new lifecycle methods; extended unit tests covering edge cases in reversibility and behavior lifecycle (completed 2026-03-11)

## Phase Details

### Phase 4: Core Rename
**Goal**: All public API names accurately reflect state machine semantics — Mutation, Behavior, Action replace Operation, Rule, Transaction throughout the codebase
**Depends on**: Phase 3 (v1.0 complete)
**Requirements**: REN-01, REN-02, REN-03, REN-04
**Success Criteria** (what must be TRUE):
  1. `cargo build` succeeds with no warnings — `Mutation<S>`, `Behavior<S,M,I,P>`, `Action<M>` are the public names in lib.rs re-exports
  2. `cargo test` passes — all unit tests and doctests reference new names with no compilation errors
  3. `cargo run --example tictactoe` and `cargo run --example backgammon` compile and run without behavioral changes
  4. `RuleLifetime` no longer appears anywhere in the public API or source (dead code eliminated)
**Plans**: 3 plans

Plans:
- [ ] 04-01-PLAN.md — Create mutation.rs, behavior.rs, action.rs; update lib.rs re-exports
- [ ] 04-02-PLAN.md — Rewrite engine.rs with new types, add_behavior; delete old source files
- [ ] 04-03-PLAN.md — Update examples (tictactoe, backgammon); bump Cargo.toml to 0.3.0

### Phase 5: Reversibility and Behavior Lifecycle
**Goal**: Mutations self-report reversibility, Actions derive their reversibility from mutations at commit time, irreversible commits clear the undo stack, and Behaviors self-manage their own lifecycle via is_active/on_dispatch/on_undo hooks
**Depends on**: Phase 4
**Requirements**: REV-01, REV-02, REV-03, REV-04, LIFE-01, LIFE-02, LIFE-03, LIFE-04, LIFE-05, LIFE-06
**Success Criteria** (what must be TRUE):
  1. A `Mutation` implementation can override `is_reversible()` to return `false`; an `Action` containing that mutation is treated as irreversible by the engine
  2. After committing an irreversible `Action`, `engine.undo()` returns an error or no-op — the undo stack is empty
  3. Reversible `Action`s committed after an irreversible one are individually undoable; the undo stack empties when the barrier is reached
  4. `engine.add_behavior(b)` replaces `engine.add_rule(b, lifetime)` — no lifetime parameter required; behaviors with `is_active() = false` are skipped per dispatch
  5. `on_dispatch()` and `on_undo()` are called on all behaviors after state mutations are applied — behaviors can update internal counters without borrow conflicts
**Plans**: 2 plans

Plans:
- [ ] 05-01-PLAN.md — Add is_reversible() to Mutation; add is_active/on_dispatch/on_undo to Behavior
- [ ] 05-02-PLAN.md — Rewrite engine.rs: remove dead code, add reversibility gate, add lifecycle passes

### Phase 6: Tests and Examples
**Goal**: The new reversibility model and behavior lifecycle are verified by property-based and unit tests; both examples compile and run correctly under the final v1.1 API
**Depends on**: Phase 5
**Requirements**: TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06
**Success Criteria** (what must be TRUE):
  1. `cargo test` passes — all existing tests updated to new names, no failures
  2. New proptest verifies: any `Action` with an irreversible mutation results in an empty undo stack after commit
  3. New proptest verifies: reversible `Action`s after an irreversible commit are individually undoable; undo halts at the barrier
  4. New unit test verifies: a stateful `Behavior` using `on_dispatch` counter deactivates after N dispatches (replaces RuleLifetime::Turns proptest)
  5. `cargo run --example backgammon` runs correctly with dice roll mutation returning `is_reversible() = false` and `RollDiceRule` using `on_dispatch`/`is_active`
**Plans**: 2 plans

Plans:
- [ ] 06-01-PLAN.md — engine.rs: name audit (TEST-01), prop_05/prop_06 reversibility proptests (TEST-02/03), stateful behavior unit test (TEST-04)
- [ ] 06-02-PLAN.md — examples: backgammon update with is_reversible/on_dispatch/demo/proptest (TEST-05), tictactoe confirm (TEST-06)

### Phase 7: Documentation and Extended Tests
**Goal**: Comprehensive rustdoc for all renamed types and new lifecycle methods; extended unit tests covering edge cases in reversibility and behavior lifecycle
**Depends on**: Phase 6
**Requirements**: DOC-01, DOC-02, DOC-03, TEST-07, TEST-08
**Success Criteria** (what must be TRUE):
  1. `cargo doc --no-deps` completes with zero warnings — all public types (`Mutation<S>`, `Behavior<S,M,I,P>`, `Action<M>`, `Engine<S,M,I,P>`) and all new trait methods (`is_reversible`, `is_active`, `on_dispatch`, `on_undo`) have rustdoc with Mealy/Moore framing in module-level prose
  2. `cargo test --doc` passes — runnable doctests on all new trait methods execute correctly and demonstrate expected usage
  3. `cargo test` passes with new unit tests for reversibility edge cases: empty `Action` (no mutations) is reversible; `Action` where all mutations are irreversible clears the undo stack; `Action` with mixed mutations (some reversible, some not) is treated as irreversible
  4. `cargo test` passes with new unit tests for behavior lifecycle edge cases: `is_active() = false` skips `before`/`after` hooks; `on_undo()` fires when a reversible action is undone; a behavior that deactivates during a dispatch sequence does not corrupt already-queued hooks
**Plans**: 2 plans

Plans:
- [ ] 07-01-PLAN.md — lib.rs crate doc + doctests for is_reversible, is_active, on_dispatch, on_undo
- [ ] 07-02-PLAN.md — engine.rs edge-case tests: mixed mutations, empty action, on_undo fires, deactivation-mid-dispatch

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Module Split and Foundation | v1.0 | 3/3 | Complete | 2026-03-09 |
| 2. Engine Property Tests | v1.0 | 1/1 | Complete | 2026-03-09 |
| 3. Backgammon Example and Integration Properties | v1.0 | 2/2 | Complete | 2026-03-09 |
| 4. Core Rename | 4/4 | Complete   | 2026-03-11 | - |
| 5. Reversibility and Behavior Lifecycle | 2/2 | Complete   | 2026-03-11 | - |
| 6. Tests and Examples | 2/2 | Complete    | 2026-03-11 | - |
| 7. Documentation and Extended Tests | 2/2 | Complete   | 2026-03-11 | - |
