---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: pest feature (PEG parser)
status: Executing Phase 6
stopped_at: Completed 06-01-PLAN.md
last_updated: "2026-03-09T00:00:00.000Z"
last_activity: 2026-03-09 — Completed 06-01-PLAN.md
progress:
  total_phases: 3
  completed_phases: 2
  total_plans: 9
  completed_plans: 6
  percent: 89
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-09 after starting v1.1 milestone)

**Core value:** The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest make this machine-verifiable, not just manually checked.
**Current focus:** v1.1 Pest Feature (PEG Parser) — executing Phase 6

## Current Position

Phase: 6
Plan: 06-02
Status: Executing Phase 6
Last activity: 2026-03-09 — Completed 06-01-PLAN.md
Progress: [█████████░] 89%

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 4. DSL Scope and Semantic Contract | 2 | 2026-03-09 | 1.0 |
| 5. Build-Time Compiler and Example Integration | 3 | 2026-03-09 | 1.0 |
| 6. Validation and Release Hardening | 1 | 2026-03-09 | 1.0 |

**Recent Trend:**
- Last 5 plans: 04-02, 05-01, 05-02, 05-03, 06-01
- Trend: steady

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
- [Phase 05-build-time-compiler-and-example-integration]: Runtime codegen emits a nested `generated_rules` module so included output has a stable namespace inside consumer crates
- [Phase 05-build-time-compiler-and-example-integration]: Generated runtime code references consumer types from parent scope and uses fully qualified `herdingcats` paths to avoid `include!` import conflicts
- [Phase 05-build-time-compiler-and-example-integration]: Integration proof uses a real nested Cargo consumer executed from tests rather than a mocked registration path
- [Phase 06-validation-and-release-hardening]: Generated-op and generated-rule invariants are validated through the real nested consumer crate, not a separate mock harness
- [Phase 06-validation-and-release-hardening]: `examples/dsl_consumer/build.rs` accepts `HERDINGCATS_DSL_RULES_PATH` so compile-fail diagnostics can exercise the real build pipeline with temporary authored DSL files
- [Phase 06-validation-and-release-hardening]: The example consumer now exposes a reusable library surface plus thin demo binary so generated behavior can be tested directly without duplicating fixture types

### Roadmap Evolution

- Phase 1 added: Add checks for clippy and cargo fmt, etc

### Pending Todos

None yet.

### Blockers/Concerns

None yet.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 1 | Add clippy + fmt CI checks | 2026-03-09 | 73d8910 | [1-add-clippy-fmt-ci-checks](./quick/1-add-clippy-fmt-ci-checks/) |

## Session Continuity

Last session: 2026-03-09T08:37:25.187Z
Stopped at: Completed 05-02-PLAN.md
Resume file: None
