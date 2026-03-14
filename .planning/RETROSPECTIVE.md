# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v0.5 — MVP Clean Reimplementation

**Shipped:** 2026-03-14
**Phases:** 6 | **Plans:** 16 | **Commits:** 185

### What Was Built
- Complete deterministic turn-based engine with EngineSpec trait, BehaviorDef fn pointers, CoW dispatch, and snapshot undo/redo
- 3,380 LOC Rust, zero dependencies, 72 unit tests + 7 doctests + 2 proptest suites
- Two working game examples (tic-tac-toe, backgammon) validating the full API
- Architecture document (ARCHITECTURE.md) fully matched by implementation

### What Worked
- Building from scratch on a clean branch avoided incremental hack accumulation from v0.4.0
- Type dependency ordering (Core Types -> Dispatch -> History -> Examples) prevented rework
- BehaviorDef struct with fn pointers was simpler than Behavior trait with trait objects — added late (Phase 6) but cleaned up the entire API
- proptest suites caught edge cases that hand-written tests missed

### What Was Inefficient
- Phase 5 (Architecture Alignment) and Phase 6 (Fill Gaps) were reactive — fixing mismatches after the fact. Could have been caught earlier by comparing against ARCHITECTURE.md during Phase 1-3
- ROADMAP.md progress table and Phase 3 checkbox fell out of sync with actual completion status
- Nyquist validation flags never updated post-execution (all `nyquist_compliant: false` despite passing verification)

### Patterns Established
- `BehaviorDef<E>` struct with fn pointer fields as the canonical behavior representation
- `NonCommittedOutcome<N>` wrapper for semantic Stop reasons (InvalidInput/Disallowed/Aborted)
- `Apply` trace contract as trusted invariant, enforced by `debug_assert!` in dispatch
- `(order_key, name)` deterministic ordering at construction time

### Key Lessons
1. Compare implementation against architecture doc continuously, not just at the end — Phases 5-6 were entirely catch-up work
2. fn pointers constrain signatures exactly — `&Vec<T>` cannot be swapped to `&[T]` without changing `EngineSpec::State`, which is a breaking API change
3. Snapshot-based undo is simple but has known memory implications for long sessions — acceptable for MVP, needs revisiting

### Cost Observations
- Model mix: balanced profile (sonnet executors, inherit planners)
- Sessions: ~4 planning+execution sessions over 6 days
- Notable: Phases 1-4 executed quickly; Phases 5-6 were alignment work that could have been avoided

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Commits | Phases | Key Change |
|-----------|---------|--------|------------|
| v0.5 | 185 | 6 | Clean reimplementation from scratch; added 2 reactive alignment phases |

### Cumulative Quality

| Milestone | Tests | Doctests | Zero-Dep |
|-----------|-------|----------|----------|
| v0.5 | 72 | 7 | Yes |

### Top Lessons (Verified Across Milestones)

1. Build from architecture doc first, validate continuously — reactive alignment is expensive
2. fn pointer APIs constrain signatures exactly — plan for this when choosing State types
