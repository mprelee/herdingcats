# Roadmap: HerdingCats v0.5.0

## Overview

v0.5.0 is a clean reimplementation of HerdingCats on the `maddie-edits` branch, building from scratch to match ARCHITECTURE.md exactly. The work follows the type dependency graph: core type contracts must be locked before dispatch logic is written, dispatch must be correct before undo/redo is layered on, and examples validate the full API against real game logic at the end. Every phase delivers an independently testable capability; later phases cannot begin until earlier phases compile cleanly and pass their criteria.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Core Types** - Define all public type contracts, traits, and enums that the entire library depends on (completed 2026-03-14)
- [x] **Phase 2: Dispatch** - Implement CoW working state and the ordered, atomic dispatch algorithm (completed 2026-03-14)
- [ ] **Phase 3: History** - Implement undo/redo with snapshot strategy and irreversibility boundary
- [x] **Phase 4: Examples and Tests** - Implement tic-tac-toe and backgammon examples; write unit and property tests (completed 2026-03-14)
- [x] **Phase 5: Architecture Alignment** - Align codebase with ARCHITECTURE.md: NonCommittedOutcome, Frame shape, EngineSpec bounds, trace contract, docs (completed 2026-03-14)
- [ ] **Phase 6: Fill Gaps** - Replace Behavior trait with BehaviorDef struct, add trace contract tests, update docs

## Phase Details

### Phase 1: Core Types
**Goal**: All public type contracts are defined and compile, so every downstream phase builds on a stable, correct API surface
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05
**Success Criteria** (what must be TRUE):
  1. A library user can define `EngineSpec` with associated types S, I, D, T, O, K and the crate compiles with no warnings
  2. A library user can implement `Behavior` with `name()`, `order_key()`, and `evaluate(&I, &S)` — the signature structurally prevents behaviors from mutating state
  3. `BehaviorResult<D, O>` variants `Continue(Vec<D>)` and `Stop(O)` are constructable and pattern-matchable
  4. `Outcome<F, N>` variants `Committed`, `Undone`, `Redone`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted` are all present and the type compiles
  5. `EngineError` is a distinct type from `Outcome` and is marked `#[non_exhaustive]`
**Plans**: 2 plans

Plans:
- [ ] 01-01-PLAN.md — Define EngineSpec trait (CORE-01)
- [ ] 01-02-PLAN.md — Define Behavior, BehaviorResult, Outcome, Frame, EngineError; wire lib.rs (CORE-02, CORE-03, CORE-04, CORE-05)

### Phase 2: Dispatch
**Goal**: Callers can submit inputs and receive deterministic, atomically committed `Outcome` results with correct CoW semantics
**Depends on**: Phase 1
**Requirements**: DISP-01, DISP-02, DISP-03, DISP-04
**Success Criteria** (what must be TRUE):
  1. `Engine::dispatch(input, reversibility)` evaluates behaviors in deterministic `(order_key, name)` order and returns `Result<Outcome, EngineError>`
  2. Diffs are applied immediately during dispatch (later behaviors see state changes from earlier behaviors in the same dispatch)
  3. `WorkingState<S>` does not clone committed state until the first diff is applied — confirmed by inserting a counter or inspection in a test
  4. `Frame<I, D, T>` is committed atomically: if dispatch produces no diffs, no frame is written and `NoChange` is returned
  5. Callers must pass an explicit `Reversibility` argument to `dispatch()` — the compiler rejects calls that omit it
**Plans**: 3 plans

Plans:
- [ ] 02-01-PLAN.md — Apply trait + Reversibility enum (DISP-01, DISP-04)
- [ ] 02-02-PLAN.md — Update Frame<E> Vec fields + Apply bound on EngineSpec::Diff (DISP-01, DISP-03)
- [ ] 02-03-PLAN.md — Engine struct, new(), state(), dispatch() (DISP-01, DISP-02, DISP-03, DISP-04)

### Phase 3: History
**Goal**: Callers can undo and redo transitions, and committing an irreversible input permanently clears history
**Depends on**: Phase 2
**Requirements**: HIST-01, HIST-02, HIST-03, HIST-04
**Success Criteria** (what must be TRUE):
  1. `Engine::undo()` returns `Undone(frame)` when history is non-empty, and `Disallowed(NothingToUndo)` when the stack is empty
  2. `Engine::redo()` returns `Redone(frame)` when redo stack is non-empty, and `Disallowed(NothingToRedo)` when it is empty
  3. Committing a `Reversibility::Irreversible` dispatch erases both undo and redo stacks — verified by calling `undo()` afterward and observing `Disallowed`
  4. Undoing restores state to the exact snapshot captured before the undone transition — no `Reversible` trait required on user diff types
**Plans**: 2 plans

Plans:
- [ ] 03-01-PLAN.md — HistoryDisallowed enum + crate root re-export (HIST-01, HIST-02)
- [ ] 03-02-PLAN.md — Engine stack upgrade, dispatch() snapshot/clear, undo(), redo(), depth queries, tests (HIST-01, HIST-02, HIST-03, HIST-04)

### Phase 4: Examples and Tests
**Goal**: Two real games validate the public API under real conditions, and automated tests enforce all 15 core invariants
**Depends on**: Phase 3
**Requirements**: EXAM-01, EXAM-02, TEST-01, TEST-02
**Success Criteria** (what must be TRUE):
  1. `cargo run --example tictactoe` compiles and runs, demonstrating dispatch, all `Outcome` variants, and undo/redo without panics
  2. `cargo run --example backgammon` compiles and runs, demonstrating that a dice-roll dispatch with `Reversibility::Irreversible` clears undo history
  3. `cargo test` passes all unit tests covering dispatch outcomes, undo/redo behavior, and the 15 core invariants
  4. `cargo test` passes property tests that verify determinism, atomicity, and undo/redo correctness across arbitrary operation sequences generated by proptest
**Plans**: 3 plans

Plans:
- [ ] 04-01-PLAN.md — Tic-tac-toe scripted demo (all 7 Outcome variant arms, undo/redo) (EXAM-01)
- [ ] 04-02-PLAN.md — Backgammon irreversibility demo (RollDice clears history) (EXAM-02)
- [ ] 04-03-PLAN.md — 15 invariant unit tests + 2 proptest suites in engine.rs (TEST-01, TEST-02)

### Phase 5: Architecture Alignment
**Goal**: Align the v0.5.0 codebase with ARCHITECTURE.md exactly — fix mismatches in non-committed outcome dispatch, trace contract, Frame shape, EngineSpec bounds, outcome semantics, and documentation
**Depends on**: Phase 4
**Requirements**: SC-1, SC-2, SC-3, SC-4, SC-5, SC-6, SC-7
**Success Criteria** (what must be TRUE):
  1. `BehaviorResult::Stop` wraps `NonCommittedOutcome<N>` with `InvalidInput`, `Disallowed`, `Aborted` variants — behaviors explicitly choose the non-committed reason
  2. `Frame<E>` contains only `input`, `diffs`, `traces` — no `reversibility` field; reversibility is stored in history stack tuples
  3. `EngineSpec::State` requires `Clone + Debug` only — no `Default` bound
  4. Apply trait docs state each call MUST return at least one trace entry
  5. `cargo test` passes all existing tests updated to the new API
  6. `cargo run --example tictactoe` and `cargo run --example backgammon` compile and run with updated API
  7. README.md describes the architecture model
**Plans**: 3 plans

Plans:
- [ ] 05-01-PLAN.md — NonCommittedOutcome contract + BehaviorResult update + example migrations (SC-1, SC-5, SC-6)
- [ ] 05-02-PLAN.md — Frame shape cleanup + Apply trace doc contract (SC-2, SC-4)
- [ ] 05-03-PLAN.md — EngineSpec Default removal + README (SC-3, SC-7)

### Phase 6: Fill Gaps
**Goal**: Replace `Behavior` trait with `BehaviorDef<E>` plain struct (fn pointers), add trace contract tests, and update all docs/examples to match
**Depends on**: Phase 5
**Requirements**: GAP-01, GAP-02, GAP-03, GAP-04, GAP-05, GAP-06
**Success Criteria** (what must be TRUE):
  1. `Behavior` trait is fully removed — `BehaviorDef<E>` struct with fn pointers is the sole behavior representation
  2. Engine stores `Vec<BehaviorDef<E>>` sorted by `(order_key, name)` at construction
  3. Trace contract tests verify: mutating diff returns >= 1 trace, no-op diff may return 0
  4. Both examples compile and run using `BehaviorDef` construction
  5. ARCHITECTURE.md and README.md describe `BehaviorDef`, not `Behavior` trait
  6. `cargo test` passes all tests
**Plans**: 3 plans

Plans:
- [ ] 06-01-PLAN.md — Replace Behavior trait with BehaviorDef struct, update engine, add trace contract tests (GAP-01, GAP-02, GAP-03)
- [ ] 06-02-PLAN.md — Migrate tictactoe and backgammon examples to BehaviorDef (GAP-04)
- [ ] 06-03-PLAN.md — Update ARCHITECTURE.md and README.md for BehaviorDef (GAP-05, GAP-06)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Types | 2/2 | Complete    | 2026-03-14 |
| 2. Dispatch | 3/3 | Complete   | 2026-03-14 |
| 3. History | 0/2 | Not started | - |
| 4. Examples and Tests | 3/3 | Complete    | 2026-03-14 |
| 5. Architecture Alignment | 3/3 | Complete    | 2026-03-14 |
| 6. Fill Gaps | 0/3 | Not started | - |
