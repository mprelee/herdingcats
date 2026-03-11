---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Rename & Reversibility
status: Defining requirements
stopped_at: —
last_updated: "2026-03-10T00:00:00.000Z"
last_activity: 2026-03-10 — Milestone v1.1 started
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-10 after v1.1 milestone started)

**Core value:** The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest make this machine-verifiable, not just manually checked.
**Current focus:** v1.1 — Rename & Reversibility Refinement

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-03-10 — Milestone v1.1 started

Progress: [██████████] 100%

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
- Last 5 plans: n/a
- Trend: n/a

*Updated after each plan completion*
| Phase 01-module-split-and-foundation P01 | 8 | 2 tasks | 7 files |
| Phase 01-module-split-and-foundation P02 | 2 | 2 tasks | 5 files |
| Phase 01-module-split-and-foundation P03 | 12 | 2 tasks | 5 files |
| Phase 02-engine-property-tests P01 | 8 | 2 tasks | 1 files |
| Phase 03-backgammon-example-and-integration-properties P01 | 3 | 1 tasks | 1 files |
| Phase 03-backgammon-example-and-integration-properties P02 | 3 | 2 tasks | 1 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Roadmap: Three phases derived from natural requirement clusters (MOD+TEST+DOC, PROP, BACK)
- Roadmap: coarse granularity applied — DOC requirements folded into Phase 1 alongside module split
- Research: Module split must follow DAG order: hash → operation → transaction → rule → engine → lib.rs facade
- Research: Every undo/redo property test must assert both `engine.read()` and `engine.replay_hash()` — not just state
- Research: Phase 3 backgammon board representation is MEDIUM-confidence; validate `[i8; 26]` bearing-off edge cases before writing proptest strategies
- [Phase 01-module-split-and-foundation]: hash module is private (mod hash, not pub mod hash) — fnv1a_hash and FNV constants are pub(crate) only
- [Phase 01-module-split-and-foundation]: #![warn(missing_docs)] added to lib.rs in Plan 01 so Plan 03 doc work has compile-time guard from the start
- [Phase 01-module-split-and-foundation]: RuleLifetime enum placed in transaction.rs alongside Transaction struct — cohesion over granularity
- [Phase 01-module-split-and-foundation]: CounterOp uses Reset{prior:i32} to store prior value so undo can restore exactly — makes undo correctness self-documenting in tests
- [Phase 01-module-split-and-foundation]: Engine::new and Engine::read doctests must use a concrete Operation type — Infallible does not satisfy the Operation<S> bound
- [Phase 01-module-split-and-foundation]: Doctest examples define inline toy types rather than importing test fixtures, keeping examples self-contained and public-API-only
- [Phase 02-engine-property-tests]: CounterOp in mod props re-declares Inc/Dec only — Reset excluded to keep proptest strategies stateless
- [Phase 02-engine-property-tests]: PROP-02 indirect isolation check: compare post-preview dispatch results against reference engine instead of inspecting private fields
- [Phase 02-engine-property-tests]: PROP-03 uses Rc<Cell<u32>> trigger_count to observe CountingRule.before() calls without accessing private engine.enabled/lifetimes
- [Phase 02-engine-property-tests]: PROP-04 uses NoRule/Permanent — asserts state+hash only, avoids Turns unconditional-decrement edge case on cancelled dispatch
- [Phase 03-backgammon-example-and-integration-properties]: Backgammon op variants carry player_sign: i8 field for uniform arithmetic instead of Player enum
- [Phase 03-backgammon-example-and-integration-properties]: Isolated op unit tests use minimal BgState (not BgState::new()) to avoid 30-checker invariant collision
- [Phase 03-backgammon-example-and-integration-properties]: Phase 3 BACK-02 bearing-off risk resolved: [i8;26] encoding with BearOffOp writing home counters (not board[26]) verified correct via roundtrip tests
- [Phase 03-backgammon-example-and-integration-properties]: RollDiceRule.before() sets tx.irreversible=false (not main()) — rules own behavioral semantics
- [Phase 03-backgammon-example-and-integration-properties]: prop_board_conservation uses apply+undo pairs at op level — tests conservation independently of game state validity
- [Phase 03-backgammon-example-and-integration-properties]: BackgammonOp derives Debug — required by proptest Strategy::prop_map bound
- [Phase quick-1-add-clippy-fmt-ci-checks]: rustfmt.toml kept minimal (edition only) — no opinionated formatting rules beyond edition pin
- [Phase quick-1-add-clippy-fmt-ci-checks]: CI workflow already had correct fmt/clippy/test steps — no changes needed to rust.yml

### Roadmap Evolution

- Phase 1 added: Add checks for clippy and cargo fmt, etc

### Pending Todos

None yet.

### Blockers/Concerns

None — v1.0 complete. All phase risks resolved.
- ~~[Phase 3 risk]: Backgammon board representation (`[i8; 26]`) is MEDIUM-confidence.~~ **Resolved in Phase 3 Plan 01 — BearOffOp writes home counters, not board[26]; all roundtrip tests pass.**

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 1 | Add clippy + fmt CI checks | 2026-03-09 | 73d8910 | [1-add-clippy-fmt-ci-checks](./quick/1-add-clippy-fmt-ci-checks/) |

## Session Continuity

Last session: 2026-03-09T08:37:25.187Z
Stopped at: Completed quick task 1: Add clippy + fmt CI checks
Resume file: None
