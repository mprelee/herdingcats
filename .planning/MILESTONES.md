# Milestones

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

