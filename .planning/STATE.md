---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Rename & Reversibility
status: planning
stopped_at: Completed 07-01-PLAN.md
last_updated: "2026-03-11T03:56:18.712Z"
last_activity: 2026-03-10 — Phase 7 added; all 25/25 v1.1 requirements mapped across Phases 4-7
progress:
  total_phases: 5
  completed_phases: 4
  total_plans: 10
  completed_plans: 10
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-10 after v1.1 milestone started)

**Core value:** The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest make this machine-verifiable, not just manually checked.
**Current focus:** v1.1 Phase 4 — Core Rename

## Current Position

Phase: 4 of 7 (Core Rename)
Plan: — of — in current phase
Status: Ready to plan
Last activity: 2026-03-10 — Phase 7 added; all 25/25 v1.1 requirements mapped across Phases 4-7

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0 (v1.1)
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: n/a
- Trend: n/a

*Updated after each plan completion*
| Phase 04-core-rename P01 | 2 | 3 tasks | 4 files |
| Phase 04-core-rename P02 | 3 | 2 tasks | 4 files |
| Phase 04-core-rename P03 | 10 | 3 tasks | 3 files |
| Phase 04-core-rename P04 | 2 | 2 tasks | 1 files |
| Phase 05-reversibility-and-behavior-lifecycle P01 | 5 | 2 tasks | 2 files |
| Phase 05-reversibility-and-behavior-lifecycle P02 | 3 | 2 tasks | 1 files |
| Phase 06-tests-and-examples P01 | 2 | 3 tasks | 1 files |
| Phase 06-tests-and-examples P02 | 4 | 2 tasks | 2 files |
| Phase 07-documentation-and-extended-tests P02 | 1 | 2 tasks | 1 files |
| Phase 07-documentation-and-extended-tests P01 | 3 | 2 tasks | 3 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Rename Operation→Mutation, Rule→Behavior, Transaction→Action — semantics overlap resolved; Rule conflicts with PEG parser terminology
- Remove RuleLifetime enum; behaviors self-manage via is_active/on_dispatch/on_undo — arbitrary state (charges, toggles, counters) without engine coupling
- Undo barrier: irreversible action clears undo stack — matches Mealy machine semantics for publicly visible information boundary
- Phase 4 must get all names right before Phase 5 touches reversibility — compilation gate enforces sequencing
- Phase 7 (docs + extended tests) added after Phase 6 — DOC-01/02/03 and TEST-07/08 require Phase 6 API to be final before writing doctests and edge-case unit tests
- [Phase 04-core-rename]: irreversible field removed from Action<M>; undo barrier semantics handled in Phase 5
- [Phase 04-core-rename]: RuleLifetime enum removed entirely; behaviors self-manage lifetime via is_active/on_dispatch/on_undo in Phase 5
- [Phase 04-core-rename]: Interface-first rename: new API contracts (Mutation/Behavior/Action) established before engine wiring updated
- [Phase 04-core-rename]: dispatch hashing gated by tx.deterministic (not tx.irreversible); commit gated by !tx.cancelled — separates two concerns that were incorrectly coupled
- [Phase 04-core-rename]: PROP-03 turns/triggers tests deleted — tested public RuleLifetime behavior removed from API; internal RuleLifetime remains private in engine.rs until Phase 5
- [Phase 04-core-rename]: dice rolls are now undoable in Phase 4 (no irreversible field on Action); Phase 5 will restore non-undoable semantics via is_reversible() on RollDiceOp
- [Phase 04-core-rename]: examples serve as integration smoke tests confirming end-to-end rename is correct across all files
- [Phase 04-core-rename]: Remove Turns/Triggers variants rather than suppressing dead_code — cleaner since Phase 5 removes the entire lifetimes map
- [Phase 05-reversibility-and-behavior-lifecycle]: Default implementations for is_reversible/is_active/on_dispatch/on_undo ensure all existing implementors compile without changes
- [Phase 05-02]: Lifecycle passes unconditionally call on_dispatch/on_undo on ALL behaviors regardless of is_active() — per locked decision
- [Phase 05-02]: Empty action guard added to dispatch commit gate — prevents spurious on_dispatch() calls on actions with no mutations
- [Phase 06-tests-and-examples]: MixedOp in mod props is a separate flat enum (Rev/Irrev) not wrapping CounterOp — mod tests MixedOp not visible from sibling mod props
- [Phase 06-tests-and-examples]: Rc<Cell<u32>> shared counter pattern for observing boxed behavior state from test scope after add_behavior moves it
- [Phase 06-tests-and-examples]: prop_06 uses structured (prefix, suffix) strategy to guarantee Irrev barrier always present and state arithmetic verifiable
- [Phase 06-02]: Used Engine::can_undo() and Engine::can_redo() public methods instead of accessing private undo_stack field for proptest barrier verification
- [Phase 06-02]: tictactoe.rs needs no changes for TEST-06 — all v1.1 API names already present from Phase 4
- [Phase 07-02]: Added empty_action_does_not_push_undo_stack as new test rather than modifying existing test — explicit can_undo() assertion kept separate for clarity
- [Phase 07-01]: engine.read() returns S (clone) not &S — doctests compare with == not &ref
- [Phase 07-01]: Action turbofish needed when dispatching empty action; Rust cannot infer M with no mutations pushed

### Pending Todos

None yet.

### Blockers/Concerns

None — v1.0 complete. v1.1 roadmap ready (Phases 4-7). Phase 4 unblocked.

## Session Continuity

Last session: 2026-03-11T03:56:18.710Z
Stopped at: Completed 07-01-PLAN.md
Resume file: None
