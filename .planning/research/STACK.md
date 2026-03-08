# Stack Research

**Domain:** Rust library crate — module refactoring with property-based testing
**Researched:** 2026-03-08
**Confidence:** HIGH (verified with official docs and docs.rs)

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.85+ (edition 2024) | Language runtime | Already in use; edition 2024 stable as of Feb 2025 via Rust 1.85.0. No module-system changes in 2024 edition — safe to upgrade without regressions. |
| proptest | 1.10 | Property-based testing (dev-only) | Hypothesis-style shrinking via value trees. Better shrinking than quickcheck's random walk. 75M+ downloads, MSRV 1.84. Actively maintained in proptest-rs/proptest. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| proptest-derive | 0.5+ | `#[derive(Arbitrary)]` for custom types | Use when writing `prop_compose!` strategies becomes verbose for structs; optional for this milestone but useful for backgammon board state |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo test | Run all unit + integration tests | `cargo test` compiles and runs `#[cfg(test)]` modules and `tests/` integration tests in one pass |
| cargo clippy | Lint for idiomatic Rust | Run before each commit; catches dead_code, missing `Default` impls, etc. |
| cargo doc | Generate and verify rustdoc | Verify re-exports surface correctly after module split; `pub use` re-exports inline into parent module docs by default |

## Installation

```toml
# Cargo.toml — add only this section; zero changes to [dependencies]
[dev-dependencies]
proptest = "1.10"
```

```bash
# Verify it resolves correctly
cargo add proptest --dev
```

No runtime dependencies are added. `proptest` is compile-time invisible to library consumers.

## Module Organization Pattern

### Use `src/module.rs` (not `src/module/mod.rs`)

The 2018 edition introduced the `module.rs` file convention. The community has standardized on it. `mod.rs` inside subdirectories is discouraged because it creates many files named `mod.rs` that are hard to navigate in editors.

**Correct file layout for this project:**

```
src/
  lib.rs          ← module declarations + pub use re-exports only
  hash.rs         ← fnv1a_hash (private fn) + constants
  operation.rs    ← Operation<S> trait
  transaction.rs  ← Transaction<O> struct + impl
  rule.rs         ← RuleLifetime enum + Rule<S,O,E,P> trait
  engine.rs       ← CommitFrame (private) + Engine<S,O,E,P> struct + impl
```

### `lib.rs` Pattern: Declare + Re-export

`lib.rs` should only contain `mod` declarations and `pub use` re-exports. No logic lives here.

```rust
// src/lib.rs
mod hash;          // private — fnv1a_hash is internal to engine.rs
pub mod operation;
pub mod transaction;
pub mod rule;
pub mod engine;

// Flatten the public API so callers use `herdingcats::Engine` not
// `herdingcats::engine::Engine`
pub use engine::Engine;
pub use operation::Operation;
pub use rule::{Rule, RuleLifetime};
pub use transaction::Transaction;
```

**Why private `mod hash`:** The FNV hash function is an implementation detail of `Engine`. It should not be part of the public API. `engine.rs` uses it via `crate::hash::fnv1a_hash`.

**Why `pub use` re-exports at crate root:** The current public API uses `herdingcats::Engine`, `herdingcats::Operation`, etc. Re-exporting from `lib.rs` preserves this without any breaking change. Downstream code using `use herdingcats::Engine` continues to compile unchanged.

### Inline `#[cfg(test)]` in Every Module File

Standard Rust convention: unit tests live in the same file as the code they test. No separate `src/tests/` directory needed for unit tests.

```rust
// src/transaction.rs (example pattern)
#[derive(Clone)]
pub struct Transaction<O> { ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transaction_is_not_cancelled() {
        let tx: Transaction<u8> = Transaction::new();
        assert!(!tx.cancelled);
    }
}
```

### Proptest Inside `#[cfg(test)]` Modules

proptest macros work inside `#[cfg(test)]` blocks. Import from `proptest::prelude::*` for convenience.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn hash_is_deterministic(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
            let h1 = fnv1a_hash(&bytes);
            let h2 = fnv1a_hash(&bytes);
            prop_assert_eq!(h1, h2);
        }
    }
}
```

**ProptestConfig for longer-running tests:**

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn undo_redo_roundtrip(...) { ... }
}
```

Default is 256 cases. For undo/redo correctness, 256 is sufficient. Use 1000 only for critical invariants.

### Shared Test Fixtures for Integration Tests

For test fixtures shared across integration test files (e.g., backgammon harness used by multiple `tests/*.rs` files), use `tests/common/mod.rs` — **not** `tests/common.rs`.

```
tests/
  common/
    mod.rs        ← shared helpers (BackgammonFixture, etc.)
  engine_props.rs ← integration property tests using common::
```

**Why `tests/common/mod.rs` over `tests/common.rs`:** Files directly in `tests/` are each compiled as separate integration test crates and appear in `cargo test` output. A file at `tests/common.rs` would produce a "running 0 tests" section — noise. Files in `tests/common/mod.rs` are subdirectory modules, not separate crates, so they don't pollute test output.

Within `tests/common/mod.rs`, import from the library using `use herdingcats::*;` — integration tests exercise the public API only.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| proptest 1.10 | quickcheck | Never for new Rust code. quickcheck uses random walk shrinking (often produces large, hard-to-read failures). proptest value-tree shrinking almost always finds the minimal counterexample. |
| proptest 1.10 | bolero | If you need coverage-guided fuzzing (libFuzzer/AFL integration). Bolero wraps proptest for pure property tests anyway. Overkill for this milestone. |
| `src/module.rs` | `src/module/mod.rs` | Only if a module has submodules (e.g., `src/engine/dispatch.rs`). This codebase's modules are single-file; no subdirectory nesting needed. |
| `pub use` re-exports in lib.rs | `pub mod` with direct path imports | Use direct paths if you want consumers to be explicit about which submodule they're importing from. Not appropriate here — breaks existing `use herdingcats::Engine` patterns. |
| Inline `#[cfg(test)]` | `tests/` only | Use `tests/` for integration-level tests that treat the library as a black box. Use inline for unit tests of private logic (e.g., `fnv1a_hash` internals, `CommitFrame` behavior). This project needs both. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `src/module/mod.rs` layout | Creates many identically-named files; editor tab titles show `mod.rs` for every module, making navigation difficult. The pattern was deprecated in favor of `module.rs` in Rust 2018. | `src/module.rs` |
| `tests/common.rs` (flat file) | Cargo treats top-level `tests/*.rs` files as independent integration test crates. `common.rs` appears as "running 0 tests" in output — misleading and noisy. | `tests/common/mod.rs` |
| `proptest-derive` for simple types | Adds a proc-macro dependency and compile overhead. For the types in this codebase (enums with a handful of variants, structs with 3-4 fields), `prop_compose!` strategies are clearer and require no derive machinery. | `prop_compose!` macro |
| Adding proptest to `[dependencies]` | Bloats library consumers; proptest is large and has its own dependency tree. Library consumers should never pay the compile cost of test tooling. | `[dev-dependencies]` only |
| Glob `pub use hash::*` in lib.rs | Exposes private implementation details unintentionally. Glob re-exports make it impossible to tell what is and isn't part of the public API without reading source. | Explicit `pub use` per item |

## Stack Patterns by Variant

**For the `hash` module (private implementation detail):**
- Declare as `mod hash;` (not `pub mod`) in `lib.rs`
- Access from `engine.rs` via `use crate::hash::fnv1a_hash;`
- Unit tests inline in `hash.rs` can test the private function directly via `super::fnv1a_hash`

**For property tests of undo/redo correctness:**
- Use `prop_compose!` to generate a sequence of `Operation` values
- Drive the engine through N dispatches, then N undos, assert state equals initial
- This is the critical invariant: `dispatch N ops; undo N times; state == initial`

**For backgammon as test harness:**
- Implement in `examples/backgammon.rs` (not `tests/`) so it's runnable via `cargo run --example backgammon`
- Import as a test fixture from `tests/common/mod.rs` for property-based tests
- Non-determinism (dice rolls) lives in the event, not the engine — generate dice values as proptest inputs

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| proptest 1.10 | Rust 1.84+ (MSRV) | Project is Rust 1.85+, fully compatible |
| proptest 1.10 | edition 2024 | No known incompatibilities; proptest is edition-agnostic |
| proptest-derive 0.5 | proptest 1.x | If added later — check proptest-derive crate for exact 1.10 compatibility at time of use |

## Sources

- [docs.rs/proptest/latest](https://docs.rs/proptest/latest/proptest/) — current version (1.10), Cargo.toml setup, macro reference (HIGH confidence)
- [Rust Book ch11-03 Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html) — `tests/common/mod.rs` pattern, authoritative (HIGH confidence)
- [Rust Edition Guide — 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) — confirmed no module-system changes in 2024 edition (HIGH confidence)
- [Rust Internals — module scheme discussion](https://internals.rust-lang.org/t/the-module-scheme-module-rs-file-module-folder-instead-of-just-module-mod-rs-introduced-by-the-2018-edition-maybe-a-little-bit-more-confusing/21977) — community consensus on `module.rs` vs `mod.rs` (MEDIUM confidence, community forum)
- [proptest CHANGELOG](https://github.com/proptest-rs/proptest/blob/main/proptest/CHANGELOG.md) — version history (HIGH confidence, official repo)
- [proptest ProptestConfig docs](https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html) — `cases`, `max_shrink_iters` fields (HIGH confidence)

---
*Stack research for: herdingcats — Rust library module refactoring with property-based testing*
*Researched: 2026-03-08*
