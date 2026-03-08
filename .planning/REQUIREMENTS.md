# Requirements: herdingcats — Refactor & Test

**Defined:** 2026-03-08
**Core Value:** The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest should make this machine-verifiable, not just manually checked.

## v1 Requirements

### Module Structure

- [ ] **MOD-01**: `src/lib.rs` is split into `src/hash.rs`, `src/operation.rs`, `src/transaction.rs`, `src/rule.rs`, `src/engine.rs` — one concept per file
- [ ] **MOD-02**: `src/lib.rs` becomes a thin re-export facade (`mod` declarations + `pub use` for all currently-public items); `hash` module stays `pub(crate)` only
- [ ] **MOD-03**: `examples/tictactoe.rs` compiles and runs identically after the split — no changes to example code, no changes to public API surface

### Unit Tests

- [ ] **TEST-01**: Every source file (`hash.rs`, `operation.rs`, `transaction.rs`, `rule.rs`, `engine.rs`) contains an inline `#[cfg(test)]` module with unit tests
- [ ] **TEST-02**: `proptest = "1.10"` added to `[dev-dependencies]` in `Cargo.toml` — zero impact on release build
- [ ] **TEST-03**: `Operation` apply+undo roundtrip verified: `op.apply(&mut s); op.undo(&mut s)` returns state identical to before apply, for every op variant
- [ ] **TEST-04**: `hash_bytes()` returns non-empty `Vec<u8>` for every `Operation` variant, and identical input produces identical output (determinism)

### Engine Property Tests

- [ ] **PROP-01**: Property test (proptest): arbitrary sequences of `apply` then `undo` return the original state AND the original `replay_hash` — not just state equality
- [ ] **PROP-02**: Property test (proptest): `dispatch_preview` leaves all four engine mutable fields identical after return — `state`, `replay_hash`, `lifetimes`, `enabled` set
- [ ] **PROP-03**: Property test (proptest): `RuleLifetime::Turns(n)` rule is disabled after exactly `n` dispatches; `RuleLifetime::Triggers(n)` rule is disabled after exactly `n` `before()` calls
- [ ] **PROP-04**: Property test (proptest): a cancelled transaction (`tx.cancelled = true`) leaves `state` and `replay_hash` completely unchanged — verified by snapshot before and after dispatch

### Backgammon Example

- [ ] **BACK-01**: `examples/backgammon.rs` is a runnable example (`cargo run --example backgammon` succeeds) demonstrating a short game sequence
- [ ] **BACK-02**: Board representation: `[i8; 26]` where indices 0–23 are points (positive = White, negative = Black, magnitude = count), index 24 = White bar, 25 = Black bar, plus two `u8` bear-off counters
- [ ] **BACK-03**: `RollDice` event produces its own `CommitFrame` (committed separately from moves), enabling per-die undo — this is architecturally distinct from `Move` events
- [ ] **BACK-04**: `Move` event covers: place checker on empty point, hit opponent blot (sends to bar), re-enter from bar, bear off to home
- [ ] **BACK-05**: Property test (proptest): board conservation — total checker count across all points + bars + home is invariant across any sequence of valid moves
- [ ] **BACK-06**: Property test (proptest): per-die undo — after dispatching a `Move` event for one die, `engine.undo()` fully restores both `state` and `replay_hash` to pre-move values

### Documentation

- [ ] **DOC-01**: Every public type, trait, and method has a `///` rustdoc comment explaining its role in the paradigm — not just what it does but *why it exists* and *how it fits the engine model*
- [ ] **DOC-02**: Key internal structs (`CommitFrame`) and the FNV hash function have `//` or `///` comments explaining their role even though they are not public
- [ ] **DOC-03**: Every public method has a `/// # Examples` block with a runnable code example demonstrating its use in context
- [ ] **DOC-04**: Trait definitions (`Operation`, `Rule`) include paradigm-teaching prose: what the abstraction represents, the contract it enforces, and how implementors should think about it

## v2 Requirements

### Deeper Property Coverage

- **PROP-V2-01**: State machine fuzzing via `proptest-state-machine` crate — arbitrary interleaved dispatch/undo/redo sequences with invariant checks after each step
- **PROP-V2-02**: `replay_hash` collision resistance property — two distinct operation sequences produce distinct hashes with high probability

### Backgammon Completeness

- **BACK-V2-01**: Doubles handling — roll doubles gives 4 moves instead of 2
- **BACK-V2-02**: Doubling cube — separate event type and CommitFrame

## Out of Scope

| Feature | Reason |
|---------|--------|
| High-level documentation (crate-level docs, README, docs.rs landing page) | Explicitly requested out of scope; source is the textbook |
| New Engine API features | This milestone is refactor + test only — no API changes |
| Doubling cube / Crawford rule / Jacoby rule | Not needed for engine stress testing; v2 if ever |
| Tournament/match scoring | Out of library's purpose |
| Async or networked gameplay | Not part of this library's purpose |
| proptest-derive proc-macro | Added complexity for marginal gain; write prop_compose! strategies manually |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MOD-01 | Phase 1 | Pending |
| MOD-02 | Phase 1 | Pending |
| MOD-03 | Phase 1 | Pending |
| TEST-01 | Phase 1 | Pending |
| TEST-02 | Phase 1 | Pending |
| TEST-03 | Phase 1 | Pending |
| TEST-04 | Phase 1 | Pending |
| DOC-01 | Phase 1 | Pending |
| DOC-02 | Phase 1 | Pending |
| DOC-03 | Phase 1 | Pending |
| DOC-04 | Phase 1 | Pending |
| PROP-01 | Phase 2 | Pending |
| PROP-02 | Phase 2 | Pending |
| PROP-03 | Phase 2 | Pending |
| PROP-04 | Phase 2 | Pending |
| BACK-01 | Phase 3 | Pending |
| BACK-02 | Phase 3 | Pending |
| BACK-03 | Phase 3 | Pending |
| BACK-04 | Phase 3 | Pending |
| BACK-05 | Phase 3 | Pending |
| BACK-06 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-08*
*Last updated: 2026-03-08 after initial definition*
