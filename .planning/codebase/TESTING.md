# Testing Patterns

**Analysis Date:** 2026-03-08

## Test Framework

**Runner:**
- Rust's built-in test harness (`cargo test`)
- Rust edition 2024, cargo 1.93.1
- No third-party test runner (e.g., `nextest` not configured)
- No `Cargo.toml` dev-dependencies for test libraries

**Assertion Library:**
- Rust standard `assert!`, `assert_eq!`, `assert_ne!` macros only
- No third-party assertion crate (`pretty_assertions`, `assert_matches`, etc.)

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test -- --nocapture  # Run with stdout visible
cargo test <name>       # Run a specific test by name
```

## Test File Organization

**Current State:**
- No tests exist anywhere in the codebase
- No `#[cfg(test)]` blocks in `src/lib.rs`
- No `tests/` directory for integration tests
- No test modules in `examples/tictactoe.rs`
- The `CONTRIBUTING.md` mentions "Run tests and ensure reproducibility" but no tests are present

**Expected Patterns (based on Rust idioms for a library crate):**

**Unit Tests (when added):**
- Location: inline in `src/lib.rs` inside a `#[cfg(test)]` module at the bottom of the file
- Naming: `tests` module name, individual test functions prefixed with the behavior they verify

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_applies_ops_to_state() {
        // ...
    }
}
```

**Integration Tests (when added):**
- Location: `tests/` directory at the project root (e.g., `tests/engine.rs`)
- These use the public API only (`use herdingcats::*`)
- Appropriate for testing `Engine` behavior end-to-end, undo/redo correctness, replay hash invariants

## Test Structure

**Suite Organization (recommended pattern):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Group by feature under test
    mod dispatch {
        use super::*;

        #[test]
        fn applies_ops_when_not_cancelled() { ... }

        #[test]
        fn skips_ops_when_cancelled() { ... }
    }

    mod undo_redo {
        use super::*;

        #[test]
        fn undo_restores_state() { ... }

        #[test]
        fn redo_reapplies_ops() { ... }
    }
}
```

## Mocking

**Framework:**
- No mocking framework present (no `mockall`, `mockito`, etc.)
- No external dependencies at all — `[dependencies]` is empty in `Cargo.toml`

**Pattern:**
- Mocking is not needed: the library is pure logic with no I/O, no async, and no external calls
- Test doubles (fake implementations of `Operation` and `Rule`) are written inline as minimal structs:

```rust
#[derive(Clone)]
struct Noop;

impl Operation<i32> for Noop {
    fn apply(&self, _state: &mut i32) {}
    fn undo(&self, _state: &mut i32) {}
    fn hash_bytes(&self) -> Vec<u8> { vec![] }
}
```

**What to Mock:**
- `Operation` implementations — use minimal no-op or simple increment/decrement variants
- `Rule` implementations — use structs that push a known op or set `tx.cancelled = true`

**What NOT to Mock:**
- `Engine` itself — test through its public API directly
- `Transaction` — construct with `Transaction::new()` and set fields directly

## Fixtures and Factories

**Test Data:**
- No fixture library; construct state directly in each test using `::new()` constructors
- Recommended pattern for an `Engine` fixture:

```rust
fn make_engine() -> Engine<i32, Noop, (), ()> {
    Engine::new(0)
}
```

**Location:**
- Helper functions defined inside the `#[cfg(test)]` module where they are used
- No shared test utilities directory exists

## Coverage

**Requirements:**
- No coverage tooling configured
- No minimum coverage threshold enforced
- The `CONTRIBUTING.md` requires that tests pass ("Run tests and ensure reproducibility") but does not specify coverage targets

**View Coverage (if desired):**
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Test Types

**Unit Tests:**
- Scope: individual methods on `Engine` — `dispatch`, `dispatch_preview`, `undo`, `redo`, `read`, `write`, `add_rule`
- Approach: construct minimal `Operation` and `Rule` implementations inline; assert on `engine.state` and `engine.replay_hash()` after each call

**Integration Tests:**
- Scope: full game simulation using the public API, verifying architectural invariants hold end-to-end
- Key invariants to verify:
  - `dispatch_preview` does not alter `replay_hash()` or undo stack
  - `undo` restores state, lifetimes, and `replay_hash()` to pre-dispatch values
  - `redo` re-applies and restores the same post-dispatch values
  - Cancelled transactions do not mutate state or hash
  - Irreversible transactions clear the redo stack

**E2E Tests:**
- Not applicable — this is a library crate with no binary or HTTP interface

## Critical Areas Requiring Test Coverage

The following behaviors are documented as architectural invariants in `docs/ARCHITECTURAL_INVARIANTS.md` and have zero test coverage:

1. **Replay hash integrity** — hash updated only on irreversible commit, restored on undo/redo, unchanged during preview
2. **Preview isolation** — `dispatch_preview` must leave `undo_stack`, `redo_stack`, `lifetimes`, `enabled`, and `replay_hash` unchanged
3. **Undo/redo correctness** — state, lifetimes, enabled rule set, and replay hash all restored correctly
4. **Rule ordering** — `before` hooks fire low-to-high priority; `after` hooks fire high-to-low
5. **Trigger lifetime decrement** — `RuleLifetime::Triggers(n)` disables the rule after `n` fires
6. **Turn lifetime decrement** — `RuleLifetime::Turns(n)` disables the rule after `n` turns
7. **Cancellation** — cancelled transaction does not apply ops and does not update replay hash
8. **Irreversible flag** — non-irreversible transaction does not push a `CommitFrame` or update hash

---

*Testing analysis: 2026-03-08*
