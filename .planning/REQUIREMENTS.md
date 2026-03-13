# Requirements: HerdingCats

**Defined:** 2026-03-13
**Core Value:** An ordered set of statically known behaviors resolves every input deterministically, so complex rule interactions are never ambiguous.

## v1 Requirements

### Core Types

- [x] **CORE-01**: Library user can define engine type params via a single `EngineSpec` trait (bundles S, I, D, T, O, K)
- [ ] **CORE-02**: Library user can implement `Behavior` trait with `name() -> &'static str`, `order_key() -> K`, `evaluate(&I, &S) -> BehaviorResult<D, O>`
- [ ] **CORE-03**: `BehaviorResult<D, O>` provides `Continue(Vec<D>)` and `Stop(O)` variants
- [ ] **CORE-04**: `Outcome<F, N>` enum provides `Committed(F)`, `Undone(F)`, `Redone(F)`, `NoChange`, `InvalidInput(N)`, `Disallowed(N)`, `Aborted(N)`
- [ ] **CORE-05**: `EngineError` is distinct from `Outcome` — reserved for engine-internal failures only

### Dispatch

- [ ] **DISP-01**: `WorkingState<S>` provides CoW semantics — reads from committed state until first write, clones substate only on first write
- [ ] **DISP-02**: `dispatch(input, reversibility)` evaluates behaviors in deterministic `(order_key, name)` order, applies diffs immediately, appends trace at moment of diff application, commits `Frame` atomically if non-empty
- [ ] **DISP-03**: `Frame<I, D, T>` stores `input`, `diff` collection, and `trace` as canonical committed record
- [ ] **DISP-04**: `dispatch()` takes an explicit `Reversibility` parameter — callers cannot omit the declaration

### History

- [ ] **HIST-01**: `undo()` returns `Result<Outcome, EngineError>` with `Undone(frame)` or `Disallowed(NothingToUndo)`
- [ ] **HIST-02**: `redo()` returns `Result<Outcome, EngineError>` with `Redone(frame)` or `Disallowed(NothingToRedo)`
- [ ] **HIST-03**: Committing an irreversible transition erases both undo and redo stacks
- [ ] **HIST-04**: Undo stores a full state snapshot per frame (no reverse-diff trait requirement on user types)

### Examples

- [ ] **EXAM-01**: Tic-tac-toe example compiles and runs, demonstrating dispatch, all outcome variants, and undo/redo
- [ ] **EXAM-02**: Backgammon example compiles and runs, demonstrating dice-roll irreversibility clearing undo history

### Testing

- [ ] **TEST-01**: Unit tests cover dispatch outcomes, undo/redo behavior, all 15 core invariants, and edge cases
- [ ] **TEST-02**: Property tests (proptest) verify determinism, atomicity, and undo/redo correctness across arbitrary operation sequences

## v2 Requirements

### Ergonomics

- **ERGO-01**: Private history stacks with `undo_depth()` / `redo_depth()` query API (no direct stack access)
- **ERGO-02**: Derive macros for common `Diff` trait implementations

### Advanced Features

- **ADV-01**: `NeedsChoice` outcome for interactive dispatch branching
- **ADV-02**: History replay / frame iterator API for time-travel debugging
- **ADV-03**: History pruning policy (max-depth) for AI-heavy sessions

## Out of Scope

| Feature | Reason |
|---------|--------|
| Runtime behavior registration | Static behavior set is a core architectural invariant; runtime registration breaks determinism guarantees |
| Dynamic substates / dynamic dispatch as design center | Contradicts compile-time structure goals and future DSL path |
| Separate validation pass | Circumvents ordering semantics; validation happens during behavior evaluation |
| `NeedsChoice` | Requires suspending/resuming dispatch state; needs concrete use case before design |
| Real-time / autonomous advancement | Not a real-time engine by design |
| DSL / card-text compilation | Long-term direction; requires mature behavior model first |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Complete |
| CORE-02 | Phase 1 | Pending |
| CORE-03 | Phase 1 | Pending |
| CORE-04 | Phase 1 | Pending |
| CORE-05 | Phase 1 | Pending |
| DISP-01 | Phase 2 | Pending |
| DISP-02 | Phase 2 | Pending |
| DISP-03 | Phase 2 | Pending |
| DISP-04 | Phase 2 | Pending |
| HIST-01 | Phase 3 | Pending |
| HIST-02 | Phase 3 | Pending |
| HIST-03 | Phase 3 | Pending |
| HIST-04 | Phase 3 | Pending |
| EXAM-01 | Phase 4 | Pending |
| EXAM-02 | Phase 4 | Pending |
| TEST-01 | Phase 4 | Pending |
| TEST-02 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 17 total
- Mapped to phases: 17
- Unmapped: 0

---
*Requirements defined: 2026-03-13*
*Last updated: 2026-03-13 after roadmap creation*
