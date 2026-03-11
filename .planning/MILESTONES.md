# Milestones

## v1.1 Rename & Reversibility (Shipped: 2026-03-11)

**Phases completed:** 4 phases (4-7), 10 plans
**Code:** 3,476 lines Rust; 43 files changed, +6,736 / -877 lines net
**Timeline:** 2 days (2026-03-10 → 2026-03-11)
**Requirements:** 25/25 v1.1 requirements satisfied

**Key accomplishments:**
- Renamed all public API types: `Operation→Mutation`, `Rule→Behavior`, `Transaction→Action`, `add_rule→add_behavior`; `RuleLifetime` removed; version bumped to 0.3.0
- Added opt-out reversibility (`is_reversible()` default `true`) to `Mutation` trait; engine enforces undo barrier on irreversible commits, clearing undo+redo stack
- Self-managing behavior lifecycle (`is_active`, `on_dispatch`, `on_undo`) on `Behavior` trait replaces external `RuleLifetime` enum; behaviors hold arbitrary internal state
- Property tests `prop_05`/`prop_06` machine-verify undo barrier and reversible-after-irreversible semantics; backgammon updated with irreversible dice rolls and `on_dispatch` counter
- Full rustdoc with Mealy/Moore state machine framing, ASCII dispatch pipeline diagram, and runnable doctests on all new trait methods; `cargo doc` zero warnings
- Edge-case unit tests for reversibility (empty action, all-irrev, mixed mutations) and lifecycle (on_undo fires, deactivation isolation)

**Archive:** `.planning/milestones/v1.1-ROADMAP.md`, `.planning/milestones/v1.1-REQUIREMENTS.md`

---

## v1.0 Refactor and Test (Shipped: 2026-03-09)

**Phases completed:** 3 phases, 6 plans
**Code:** 2,647 lines Rust across 7 files (+2,367 net insertions)
**Timeline:** 1 day (2026-03-08 → 2026-03-09)
**Requirements:** 21/21 v1 requirements satisfied

**Key accomplishments:**
- Split monolithic `src/lib.rs` into 5 concept-focused modules (`hash`, `operation`, `transaction`, `rule`, `engine`) with thin re-export facade and `#![warn(missing_docs)]` compile-time gate
- Added full rustdoc with paradigm-teaching prose on `Operation` and `Rule` traits; 15 runnable doctests, 0 `cargo doc` warnings
- Four proptest property tests machine-verifying undo roundtrip, `dispatch_preview` isolation, rule lifetime exactness, and cancelled-tx isolation
- `examples/backgammon.rs` (865 lines) demonstrating non-undoable dice rolls (`tx.irreversible=false`) and per-die undo correctness
- Two integration property tests: board conservation invariant (30 checkers) and per-die undo fully restores state + `replay_hash`

**Archive:** `.planning/milestones/v1.0-ROADMAP.md`, `.planning/milestones/v1.0-REQUIREMENTS.md`

---

