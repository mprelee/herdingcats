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
- [ ] **Phase 4: Examples and Tests** - Implement tic-tac-toe and backgammon examples; write unit and property tests

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
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Types | 2/2 | Complete    | 2026-03-14 |
| 2. Dispatch | 3/3 | Complete   | 2026-03-14 |
| 3. History | 0/2 | Not started | - |
| 4. Examples and Tests | 0/? | Not started | - |
