# Milestones

## v0.5 MVP Clean Reimplementation (Shipped: 2026-03-14)

**Phases completed:** 6 phases, 16 plans, 0 tasks

**Key accomplishments:**
- Core type system with `EngineSpec` trait, `Outcome` enum, `Frame`, and `EngineError` — flat crate API with structural mutation prevention
- Copy-on-write dispatch pipeline with lazy state cloning and deterministic `(order_key, name)` behavior ordering
- Snapshot-based undo/redo with no `Reversible` trait burden; irreversible commits clear history
- `BehaviorDef` struct with fn pointers replacing trait objects — zero dynamic dispatch
- 72 tests including 15 named ARCHITECTURE.md invariant tests + 2 proptest suites
- Tic-tac-toe and backgammon examples validating the full API

**Known Tech Debt:**
- `Outcome::InvalidInput` variant structurally unreachable from `Engine::dispatch()` in MVP — present for exhaustive matching
- VALIDATION.md files have `nyquist_compliant: false` — not updated post-execution

---

