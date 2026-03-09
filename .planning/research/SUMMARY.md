# Project Research Summary

**Project:** herdingcats
**Domain:** Rust library crate — module refactoring with property-based testing for a generic stateful game rule engine
**Researched:** 2026-03-08
**Confidence:** HIGH

## Executive Summary

The herdingcats project is a Rust library crate implementing a generic game rule engine with undo/redo, deterministic replay hashing, and rule lifetime management. The milestone goal is twofold: refactor the current single-file `src/lib.rs` into a proper multi-module structure, and establish machine-verified correctness guarantees through property-based testing using proptest. Both tasks are well-understood engineering problems with established, official patterns in the Rust ecosystem. The recommended approach executes them in strict sequence — module split first, tests second — because the module structure determines where tests live, and the backgammon example (required as the proptest harness) can only be written after the library's public API is stable and verified.

The recommended stack is minimal: Rust 1.85+ (edition 2024) plus proptest 1.10 as a dev-dependency only. No new runtime dependencies are introduced. The module split follows the `src/module.rs` convention (not `src/module/mod.rs`), with `lib.rs` serving as a thin facade of `pub use` re-exports that preserves the existing public API surface exactly. The proptest strategy design must generate valid engine states by construction (via `prop_flat_map`) rather than filtering generated values with `prop_assume!`, which would degrade shrinking quality and make failures hard to diagnose.

The dominant risk is silent public API breakage during the module split: items that were public in the single-file `lib.rs` become invisible to callers if their `pub use` re-exports are omitted. The mitigation is mandatory: run `cargo test --examples` after every individual module extraction — `examples/tictactoe.rs` serves as the canary for the full public surface. A secondary risk is incomplete property test assertions: undo/redo tests that only check `engine.state` and skip `engine.replay_hash()` will miss an entire class of hash-related bugs. Both risks are avoidable with discipline and can be codified as phase exit criteria.

## Key Findings

### Recommended Stack

The stack is deliberately narrow. proptest 1.10 (MSRV 1.84, fully compatible with the project's Rust 1.85+) is the only new addition and belongs in `[dev-dependencies]` only — it must never appear in `[dependencies]`. The `src/module.rs` file convention (introduced in Rust 2018 edition) is the community standard and should be used for all five new module files. The Rust 2024 edition introduces no module-system changes, so the edition upgrade is safe. For shared integration test fixtures, `tests/common/mod.rs` is the correct pattern — `tests/common.rs` would pollute `cargo test` output with a "running 0 tests" entry.

**Core technologies:**
- Rust 1.85+ (edition 2024): language runtime — already in use, no regressions from edition upgrade
- proptest 1.10: property-based testing — hypothesis-style shrinking via value trees; superior to quickcheck for finding minimal counterexamples; dev-dependency only

**Supporting tools:**
- cargo test --examples: exit criterion for each module extraction step
- cargo clippy: catch dead_code and API visibility issues pre-commit
- cargo doc --no-deps: verify re-exported items surface correctly after split

### Expected Features

The correctness properties the engine must demonstrably satisfy fall into two priority tiers. P1 features establish that the engine's core guarantees (undo/redo correctness, hash determinism, preview isolation, rule lifecycle reversibility) are machine-verified. P2 features add depth but are not required to make the correctness claim credible. The backgammon example is a P1 delivery because it is the required harness for the headline undo/redo walk property test.

**Must have (table stakes — P1):**
- Operation round-trip property test (apply + undo returns to prior state) — the foundation all other undo properties rest on
- Hash determinism property test — proves FNV-1a accumulation is pure
- Undo restores state identity + replay hash round-trip — the core engine guarantee, dual-asserted
- `dispatch_preview` zero side effects test — proves preview is safe to call freely
- Cancelled transaction leaves state unchanged — proves cancellation does not corrupt state
- `RuleLifetime::Triggers` and `::Turns` undo correctness — proves rule lifecycle is reversible
- Arbitrary-length undo/redo walk using backgammon (proptest, N dispatches then N undos) — headline property
- Backgammon board conservation invariant — domain-level property
- Partial move undo with one die — exercises the non-determinism use case

**Should have (competitive — P2, add after P1 is green):**
- Undo/redo alternation invariant (dispatch, undo, redo repeated N times)
- Preview commutes with dispatch property
- State machine transition fuzz with `proptest-state-machine`
- Hash order-dependence test

**Defer (v2+):**
- Concurrent dispatch safety — only relevant if `Engine` ever becomes `Send`; out of scope
- Exhaustive backgammon legality testing — belongs in a separate game-layer suite

### Architecture Approach

The architecture is a five-module split of the existing `src/lib.rs`, with strict dependency ordering: hash -> operation -> transaction -> rule -> engine. Each module is a single `.rs` file using the `src/module.rs` convention. `lib.rs` becomes a pure facade with no logic, only `mod` declarations and explicit `pub use` re-exports. `CommitFrame` stays private inside `engine.rs` — it is never made public. The backgammon example (`examples/backgammon.rs`) is built after the library split is stable and green, mirroring the structure of the existing `examples/tictactoe.rs`. Dice randomness uses a local LCG or `std::time::SystemTime` seed — no new runtime crates.

**Major components:**
1. `src/hash.rs` — FNV-1a 64-bit hash function and constants, `pub(crate)` visibility only
2. `src/operation.rs`, `src/transaction.rs`, `src/rule.rs` — trait and type definitions, leaf nodes in dependency DAG
3. `src/engine.rs` — `Engine<S,O,E,P>` struct, `CommitFrame` (private), all dispatch/undo/redo logic; the integration point
4. `src/lib.rs` — thin facade: `mod` declarations + explicit `pub use` re-exports
5. `examples/backgammon.rs` — concrete game implementation using `use herdingcats::*;`, serves as both working example and proptest harness

### Critical Pitfalls

1. **Incomplete `pub use` re-exports break the public API** — After each module extraction, run `cargo test --examples` before continuing. `tictactoe.rs` compilation is the non-negotiable canary. Never use `pub use module::*` glob re-exports; every re-exported item must be named explicitly.

2. **`prop_assume!` filtering degrades shrinking quality** — Do not use rejection sampling to constrain game states. Build valid-state strategies by construction using `prop_flat_map`. Reserve `prop_assume!` only for conditions with <5% rejection rate.

3. **Undo tests that skip `replay_hash` assertion hide hash bugs** — Every undo/redo roundtrip assertion must check both `engine.read() == original_state` AND `engine.replay_hash() == original_hash`. Define a snapshot helper that captures both and use it as the baseline in all roundtrip tests.

4. **Single final assertion in stateful tests misses mid-sequence bugs** — Check invariants after each dispatch step, not only at the end. A rule lifetime double-decrement that cancels out over N steps is undetectable with only a terminal assertion.

5. **`Box<dyn Rule>` is not `Clone` — do not attempt `Arbitrary` for `Engine`** — Generate test scenarios as `(initial_state, Vec<Event>)`, build the engine in the test body via `Engine::new()` + `engine.dispatch()`, and use concrete test rule structs (not trait objects) inside test modules.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Module Split
**Rationale:** The module structure must be stable before tests can be placed correctly. Every subsequent phase (proptest, backgammon example) depends on knowing which file owns which code. This phase has a strict sequential sub-order dictated by the dependency DAG: hash -> operation -> transaction -> rule -> engine -> lib.rs facade.
**Delivers:** A compilable, fully-tested (existing tests pass), multi-module library crate with identical public API surface to the current single-file version.
**Addresses:** STACK.md module organization patterns; ARCHITECTURE.md component responsibilities and build order.
**Avoids:** Pitfall 1 (incomplete re-exports) and Pitfall 2 (glob shadow) — exit criterion for every module extraction step is green `cargo test --examples`.
**Research flag:** Standard patterns — skip phase-level research. Official Rust Book patterns apply directly.

### Phase 2: Engine Property Tests (inline `#[cfg(test)]`)
**Rationale:** Engine unit properties (Operation round-trip, hash determinism, undo/redo correctness, preview isolation, rule lifetime lifecycle) are independent of the backgammon game model. They test the generic engine against simple concrete types (e.g., `u8` state). These must be established and green before the backgammon harness is written, so failures can be attributed unambiguously to engine logic, not to game-layer complexity.
**Delivers:** Machine-verified correctness for all P1 engine mechanics: operation contracts, hash accumulation, undo/redo stack behavior, rule lifetime reversibility, preview isolation, cancelled transaction invariant.
**Uses:** proptest 1.10 with `proptest!` macro, `prop_compose!`, and inline `#[cfg(test)] mod tests { use super::*; }` pattern.
**Implements:** All P1 table-stakes features except the backgammon-dependent ones.
**Avoids:** Pitfall 3 (test visibility across modules), Pitfall 5 (single final assertion), Pitfall 6 (`Box<dyn Rule>` blocking Arbitrary), Pitfall 7 (missing hash assertion).
**Research flag:** Standard proptest patterns apply; skip phase-level research. The `prop_flat_map` strategy pattern is documented in the official proptest book.

### Phase 3: Backgammon Example and Integration Property Tests
**Rationale:** The backgammon example is gated on Phase 1 (stable public API) and Phase 2 (engine correctness confirmed). It serves two purposes: a runnable example demonstrating the engine for non-deterministic games with partial-move undo, and the harness for the headline undo/redo walk property test. This phase is more complex because it requires designing the board representation, op types, event types, and rule architecture — all of which must implement the engine's generic traits correctly.
**Delivers:** `examples/backgammon.rs`, `tests/common/mod.rs` shared fixture, and the integration property tests: arbitrary-length undo/redo walk, board conservation invariant, partial move undo with one die.
**Uses:** Backgammon board as `[i8; 26]` + bear-off counters, `prop_flat_map` for dice-constrained move generation, `tests/common/mod.rs` for shared fixtures.
**Implements:** All remaining P1 features (backgammon-dependent properties), plus lays groundwork for P2 state machine tests.
**Avoids:** Pitfall 4 (`prop_assume!` breaking shrinking) — board states are generated by construction. Architecture anti-patterns: rolling dice inside a Move event, one mega-Op for all move types.
**Research flag:** Moderate — backgammon data model is medium-confidence (inferred from rules + standard game theory). Validate the `[i8; 26]` board representation and rule architecture against the actual game mechanics early in this phase before writing proptest strategies.

### Phase 4: P2 Property Tests and Polish
**Rationale:** Once P1 correctness is green and the backgammon harness exists, P2 properties (undo/redo alternation invariant, preview commutes with dispatch, hash order-dependence) are straightforward additions. State machine fuzzing with `proptest-state-machine` is the highest-effort P2 item and should be added last, after simpler properties have exercised the engine adequately.
**Delivers:** Deeper confidence properties, final CI stability verification, and optional `proptest-state-machine` integration.
**Implements:** All P2 features from FEATURES.md.
**Research flag:** `proptest-state-machine` crate (version 0.3) may need a quick verification of current API compatibility at implementation time — the core pattern is documented but the crate is less mature than proptest core.

### Phase Ordering Rationale

- The module split comes first because it is a prerequisite for everything: test file placement, public API stability, and backgammon example compilation all depend on a clean module structure.
- Engine properties come before backgammon because the dependency graph in FEATURES.md is explicit: "Operation round-trip is the foundation; if it fails, all higher-level undo tests fail for the wrong reason."
- The backgammon phase comes third because it has a hard sequential dependency on both the stable API (Phase 1) and confirmed engine correctness (Phase 2).
- P2 polish comes last because it adds depth without being foundational.
- This ordering also minimizes the blast radius of bugs: failures in Phase 2 are cleanly attributable to engine logic, not game-layer implementation choices.

### Research Flags

Phases needing deeper research during planning:
- **Phase 3 (Backgammon):** Validate the backgammon board data model (`[i8; 26]` representation) and op/event/rule decomposition against actual game mechanics before writing proptest strategies. The board representation is MEDIUM-confidence (inferred, not from an authoritative source). Recommend one early spike to verify the bearing-off and hit/reenter operations implement correctly before building the strategy infrastructure around them.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Module Split):** Official Rust Book patterns, HIGH confidence, direct applicability.
- **Phase 2 (Engine Properties):** Official proptest documentation covers all required patterns at HIGH confidence.
- **Phase 4 (P2 Polish):** Extends Phase 2 and 3 patterns; only `proptest-state-machine` API needs a quick version-check at implementation time.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | proptest 1.10 verified on docs.rs; module conventions from official Rust Book and edition guide |
| Features | HIGH | Proptest strategy patterns from official docs; feature prioritization based on direct analysis of engine code |
| Architecture | HIGH | Module split patterns from official Rust Book; backgammon data model MEDIUM (standard game theory + multiple independent sources agree) |
| Pitfalls | HIGH | Re-export semver pitfalls from cargo-semver-checks maintainer + official Cargo Book; proptest filtering pitfall from official proptest book + 2024 practitioner case studies |

**Overall confidence:** HIGH

### Gaps to Address

- **Backgammon board representation validation:** The `[i8; 26]` signed-array model is the standard choice cited by multiple sources but is MEDIUM-confidence because no single authoritative Rust backgammon implementation was available for direct verification. Validate the bearing-off edge case (how borne-off checkers interact with the bar index encoding) before writing the proptest board strategy.
- **`proptest-state-machine` API stability:** Version 0.3 is referenced in research but it is a less-mature crate. At Phase 4 implementation time, verify the current API against the installed version to avoid surprises.
- **Doubles rule scope:** PROJECT.md limits doubles to 2 moves (not the standard 4). This simplification is noted in ARCHITECTURE.md but was not validated against any existing implementation. Confirm this is intentional before writing the dice strategy that generates doubles.

## Sources

### Primary (HIGH confidence)
- [docs.rs/proptest/latest](https://docs.rs/proptest/latest/proptest/) — proptest version, Cargo.toml setup, macro reference, filtering/shrinking behavior
- [Rust Book ch11-03 Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html) — `tests/common/mod.rs` pattern
- [Rust Book ch07-05 Separating Modules into Different Files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html) — module split conventions
- [Rust Edition Guide — 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) — no module-system changes confirmed
- [Proptest State Machine Testing — official proptest book](https://proptest-rs.github.io/proptest/proptest/state-machine.html) — state machine test patterns
- [Proptest Filtering Documentation — official proptest book](https://altsysrq.github.io/proptest-book/proptest/tutorial/filtering.html) — shrinking degradation via prop_assume
- [Breaking semver in Rust by adding a private type or import — predr.ag](https://predr.ag/blog/breaking-semver-in-rust-by-adding-private-type-or-import/) — pub use glob shadow pitfall
- [SemVer Compatibility — The Cargo Book](https://doc.rust-lang.org/cargo/reference/semver.html) — re-export visibility rules
- [Visibility and Privacy — The Rust Reference](https://doc.rust-lang.org/reference/visibility-and-privacy.html) — pub(crate) semantics
- [Stateful Property Testing in Rust — Readyset Engineering Blog, 2024](https://readyset.io/blog/stateful-property-testing-in-rust) — per-step invariant checking patterns

### Secondary (MEDIUM confidence)
- [Higher-Order Strategies — Proptest Book](https://altsysrq.github.io/proptest-book/proptest/tutorial/higher-order.html) — prop_compose / flat_map patterns
- [Exploring Round-trip Properties in Property-based Testing — PLClub (2023)](https://www.cis.upenn.edu/~plclub/blog/2023-12-07-round-trip-properties/) — round-trip property design
- [Property Testing Stateful Code in Rust — rtpg.co, 2024](https://rtpg.co/2024/02/02/property-testing-with-imperative-rust/) — practitioner stateful testing case study
- [Model-Based Stateful Testing with proptest-state-machine — Nikos Baxevanis, 2025](https://blog.nikosbaxevanis.com/2025/01/10/state-machine-testing-proptest/) — state machine test structure
- Board representation `[i8; 26]` — MEDIUM (multiple independent sources; standard in academic backgammon implementations)

### Tertiary (LOW confidence)
- [backgammon crate on docs.rs](https://docs.rs/backgammon/latest/backgammon/) — board model reference; docs incomplete, used as secondary validation only

---
*Research completed: 2026-03-08*
*Ready for roadmap: yes*
