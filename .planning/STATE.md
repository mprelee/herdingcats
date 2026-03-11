---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Rename & Reversibility
status: planning
stopped_at: Completed 04-core-rename-01-PLAN.md
last_updated: "2026-03-11T00:38:20.297Z"
last_activity: 2026-03-10 — Phase 7 added; all 25/25 v1.1 requirements mapped across Phases 4-7
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 3
  completed_plans: 1
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

### Pending Todos

None yet.

### Blockers/Concerns

None — v1.0 complete. v1.1 roadmap ready (Phases 4-7). Phase 4 unblocked.

## Session Continuity

Last session: 2026-03-11T00:38:20.295Z
Stopped at: Completed 04-core-rename-01-PLAN.md
Resume file: None
