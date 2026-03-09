# Roadmap: herdingcats — Refactor & Test

## Overview

The library's single `src/lib.rs` is split into five concept-focused modules (Phase 1), then the engine's correctness guarantees are made machine-verifiable with proptest property tests against simple types (Phase 2), and finally the backgammon example is built as both a runnable demo and a proptest harness for the headline non-determinism and partial-move-undo properties (Phase 3).

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Module Split and Foundation** - Split lib.rs into five modules, add proptest dev-dep, inline unit tests in every module, and add rustdoc to all public items (completed 2026-03-09)
- [x] **Phase 2: Engine Property Tests** - Machine-verify the engine's core correctness guarantees — undo/redo roundtrips, hash determinism, preview isolation, rule lifetime lifecycle, and cancelled transaction invariant — using proptest against simple concrete types (completed 2026-03-09)
- [ ] **Phase 3: Backgammon Example and Integration Properties** - Build examples/backgammon.rs as a runnable demo and proptest harness, verifying board conservation and per-die undo correctness

## Phase Details

### Phase 1: Module Split and Foundation
**Goal**: The library compiles as a properly structured multi-module crate with identical public API, inline unit tests in every module, and rustdoc on all public items
**Depends on**: Nothing (first phase)
**Requirements**: MOD-01, MOD-02, MOD-03, TEST-01, TEST-02, TEST-03, TEST-04, DOC-01, DOC-02, DOC-03, DOC-04
**Success Criteria** (what must be TRUE):
  1. `cargo test --examples` passes with tictactoe compiling and running identically to before the split — no public API items missing
  2. Five module files exist (`hash.rs`, `operation.rs`, `transaction.rs`, `rule.rs`, `engine.rs`) and `lib.rs` contains only `mod` declarations and explicit `pub use` re-exports
  3. Every module file contains an inline `#[cfg(test)]` block; `Operation` apply+undo roundtrip and `hash_bytes()` determinism are both tested
  4. Every public type, trait, and method has a `///` rustdoc comment including a `/// # Examples` block; `cargo doc --no-deps` generates without warnings
**Plans**: 3 plans

Plans:
- [ ] 01-01-PLAN.md — Module split: create 5 module files, thin lib.rs facade, add proptest dep
- [ ] 01-02-PLAN.md — Unit tests: #[cfg(test)] blocks in all modules, counter fixture, apply+undo roundtrip
- [ ] 01-03-PLAN.md — Rustdoc: /// on all public items, // on internals, # Examples blocks, zero doc warnings

### Phase 2: Engine Property Tests
**Goal**: The engine's determinism and undo/redo correctness are machine-verifiable — proptest runs confirm all core invariants hold for arbitrary inputs
**Depends on**: Phase 1
**Requirements**: PROP-01, PROP-02, PROP-03, PROP-04
**Success Criteria** (what must be TRUE):
  1. `cargo test` runs proptest suites confirming arbitrary sequences of apply+undo return both the original state AND the original `replay_hash`
  2. `dispatch_preview` is confirmed by property test to leave `state`, `replay_hash`, `lifetimes`, and the enabled rule set identical to before the call
  3. `RuleLifetime::Turns(n)` and `RuleLifetime::Triggers(n)` rules are confirmed by property test to disable at exactly the right counts — no off-by-one
  4. A cancelled transaction (`tx.cancelled = true`) is confirmed by property test to leave `state` and `replay_hash` bitwise identical to a snapshot taken before dispatch
**Plans**: 1 plan

Plans:
- [ ] 02-01-PLAN.md — Four proptest property tests in mod props: PROP-01 undo roundtrip, PROP-02 preview isolation, PROP-03 lifetime correctness, PROP-04 cancelled tx isolation

### Phase 3: Backgammon Example and Integration Properties
**Goal**: A runnable backgammon example demonstrates the engine handling non-determinism and partial-move undo, and proptest integration properties verify board conservation and per-die undo correctness
**Depends on**: Phase 2
**Requirements**: BACK-01, BACK-02, BACK-03, BACK-04, BACK-05, BACK-06
**Success Criteria** (what must be TRUE):
  1. `cargo run --example backgammon` succeeds and prints a short game sequence showing dice roll, move, and undo
  2. The board representation (`[i8; 26]` + bear-off counters) correctly handles all four `Move` event variants: place on empty point, hit blot, re-enter from bar, and bear off
  3. `RollDice` and `Move` produce separate `CommitFrame` entries so `engine.undo()` can reverse a single die's move without reversing the dice roll
  4. Property test confirms total checker count across all points + bars + home counters is invariant across any sequence of valid moves
  5. Property test confirms `engine.undo()` after a single `Move` dispatch fully restores both `state` and `replay_hash` to pre-move values
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Module Split and Foundation | 3/3 | Complete   | 2026-03-09 |
| 2. Engine Property Tests | 1/1 | Complete   | 2026-03-09 |
| 3. Backgammon Example and Integration Properties | 0/TBD | Not started | - |
