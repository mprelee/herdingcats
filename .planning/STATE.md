---
gsd_state_version: 1.0
milestone: v0.5
milestone_name: milestone
status: planning
stopped_at: Completed 06-fill-gaps-03-PLAN.md
last_updated: "2026-03-14T03:49:19.394Z"
last_activity: 2026-03-13 — Roadmap created, ready to begin planning Phase 1
progress:
  total_phases: 6
  completed_phases: 6
  total_plans: 16
  completed_plans: 16
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-13)

**Core value:** An ordered set of statically known behaviors resolves every input deterministically, so complex rule interactions are never ambiguous.
**Current focus:** Phase 1 - Core Types

## Current Position

Phase: 1 of 4 (Core Types)
Plan: 0 of ? in current phase
Status: Ready to plan
Last activity: 2026-03-14 - Completed quick task 2: BehaviorEval type alias, README polish, Apply contract docs

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
| Phase 01-core-types P01 | 1 | 1 tasks | 2 files |
| Phase 01-core-types P02 | 3min | 3 tasks | 5 files |
| Phase 02-dispatch P01 | 2min | 2 tasks | 3 files |
| Phase 02-dispatch P02 | 15min | 2 tasks | 5 files |
| Phase 02-dispatch P03 | 2min | 2 tasks | 2 files |
| Phase 03-history P01 | 5 | 1 tasks | 2 files |
| Phase 03-history P02 | 5min | 2 tasks | 2 files |
| Phase 04-examples-and-tests P02 | 1min | 1 tasks | 1 files |
| Phase 04-examples-and-tests P01 | 2min | 1 tasks | 1 files |
| Phase 04-examples-and-tests P03 | 3min | 2 tasks | 1 files |
| Phase 05-architecture-alignment P01 | 4min | 2 tasks | 6 files |
| Phase 05-architecture-alignment P02 | 3min | 1 tasks | 3 files |
| Phase 05-architecture-alignment P03 | 5min | 2 tasks | 2 files |
| Phase 06-fill-gaps P01 | 15min | 2 tasks | 7 files |
| Phase 06-fill-gaps P02 | 3min | 1 tasks | 0 files |
| Phase 06-fill-gaps P03 | 2min | 2 tasks | 2 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Build from scratch on maddie-edits: Prior v0.4.0 had fundamental design mismatches (behavior state outside main tree, address-based ordering, eager clone, public stacks)
- Static behavior set only: Preserves tight typing and determinism
- CoW working state: Avoids performance penalty for large-state AI look-ahead
- (order_key, behavior_name) ordering: Deterministic tiebreaker without relying on memory address
- [Phase 01-core-types]: Used pub mod spec (not mod spec) in lib.rs to satisfy clippy dead_code without adding pub use — preserves Plan 02 flat re-export job
- [Phase 01-core-types]: Outcome is not #[non_exhaustive] — 7 variants are stable public contract; EngineError IS #[non_exhaustive] — engine may surface new errors in future versions
- [Phase 01-core-types]: lib.rs uses private mod declarations + pub use re-exports — flat herdingcats::* namespace, no sub-path exposure
- [Phase 02-dispatch]: Private mod apply/reversibility in lib.rs for test compilation; pub use deferred to Plan 02
- [Phase 02-dispatch]: Reversibility not #[non_exhaustive] — two variants are complete stable contract
- [Phase 02-dispatch]: EngineSpec: Sized added to resolve Apply<E: EngineSpec> implicit Sized requirement — all EngineSpec impls are unit structs, always Sized
- [Phase 02-dispatch]: Each test module adds Apply<TestSpec> for u8 in cfg(test) to comply with new Diff bound — avoids orphan issues
- [Phase 02-dispatch]: apply::tests::TestSpec::Diff changed to AppendByte — existing concrete Apply implementor in that module
- [Phase 02-dispatch]: CoW pointer test uses Vec::as_ptr() (heap buffer address) not &Vec (field address) — field address never changes between dispatches
- [Phase 02-dispatch]: Apply trait must be explicitly imported in engine.rs (use crate::apply::Apply) for diff.apply() method call resolution even with Apply<E> bound on EngineSpec::Diff
- [Phase 03-history]: HistoryDisallowed not #[non_exhaustive] — NothingToUndo and NothingToRedo are the complete stable public API
- [Phase 03-history]: Frame<E> Clone/PartialEq use manual impls with associated type bounds — derive macro adds E: Clone/PartialEq which unit struct specs cannot satisfy
- [Phase 03-history]: undo()/redo() return Outcome<Frame<E>, HistoryDisallowed> not Outcome<Frame<E>, E::NonCommittedInfo> — intentional asymmetry, HistoryDisallowed is the specific reason type
- [Phase 03-history]: Irreversible dispatch order: push to undo_stack then clear both stacks — state change commits, all history erased
- [Phase 04-examples-and-tests]: All behaviors return Continue([]) for inputs they don't handle — no abort/abort-early short circuit needed in simple demos
- [Phase 04-examples-and-tests]: BackgammonState.black_pos uses #[allow(dead_code)] — kept for domain clarity even though not exercised in the demo
- [Phase 04-examples-and-tests]: CheckWin reads _input to simulate post-placement board state since behaviors evaluate pre-apply
- [Phase 04-examples-and-tests]: NoChange demonstrated via zero-behavior engine — cleanest approach for 4-behavior game loop
- [Phase 04-examples-and-tests]: Doc comments before proptest! macro converted to regular comments — rustdoc warns on /// before macro invocations
- [Phase 04-examples-and-tests]: Op enum annotated with #[allow(dead_code)] — enum variants used only inside proptest! macro body, not detected by lint
- [Phase 05-01]: NonCommittedOutcome lives in outcome.rs (not behavior.rs) to avoid circular deps — behavior.rs imports from outcome.rs
- [Phase 05-01]: From<NonCommittedOutcome<N>> for Outcome<F, N> enables dispatch Stop arm to use outcome.into() with no hardcoded Aborted coercion
- [Phase 05-01]: BehaviorResult type param renamed O→N for consistency with Outcome<F, N> naming; Stop variant now wraps NonCommittedOutcome<N>
- [Phase 05-02]: Frame<E> is a pure data record with no dispatch-protocol fields — Reversibility belongs to the history stack tuple, not the frame itself
- [Phase 05-02]: Apply trait doc now enforces trace contract: each state-mutating call MUST return at least one trace entry (was previously 'empty Vec is valid')
- [Phase 05-architecture-alignment]: EngineSpec::State has no Default bound — callers supply initial state to Engine::new(), engine never calls Default internally
- [Phase 05-architecture-alignment]: Test updated to use explicit vec![] construction instead of ::default() — proves Default bound is truly absent from EngineSpec
- [Phase 06-fill-gaps]: BehaviorDef<E> is a plain struct with fn pointer fields — eliminates trait objects and Box<dyn Behavior<E>> from the behavior system
- [Phase 06-fill-gaps]: Engine stores Vec<BehaviorDef<E>>; dispatch calls (behavior.evaluate)(...) with parentheses required for fn pointer call syntax
- [Phase 06-fill-gaps]: Example migration folded into Plan 06-01 BehaviorDef refactor commit — examples and engine changes interdependent; Plan 06-02 verified completion

### Roadmap Evolution

- Phase 5 added: Architecture Alignment — align codebase with ARCHITECTURE.md (explicit non-committed outcomes, trace contract, static behavior composition, Frame/reversibility, EngineSpec bounds, outcome semantics, docs, tests)
- Phase 6 added: fill gaps

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 2 flag: `Apply<S>` and `Traced<T>` trait bounds need validation against backgammon use case before finalizing
- Phase 3 flag: Snapshot undo memory implications for long AI-heavy sessions — acceptable for MVP, flag for v0.5.x

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 1 | Final cleanup pass: fix malformed docs, tighten Apply contract, clarify static behavior wording, add irreversible history tests, optional polish | 2026-03-14 | 7a90157 | [1-final-cleanup-pass-fix-malformed-docs-ti](./quick/1-final-cleanup-pass-fix-malformed-docs-ti/) |
| 2 | Final cleanup: add BehaviorEval type alias to fix clippy warning, polish README Quick Start with Outcome match, expand Apply trace contract docs | 2026-03-14 | 152314b | [2-final-cleanup-behavioreval-type-alias-re](./quick/2-final-cleanup-behavioreval-type-alias-re/) |

## Session Continuity

Last session: 2026-03-14T05:52:22Z
Stopped at: Completed quick task 2: BehaviorEval type alias, README polish, Apply contract docs
Resume file: None
