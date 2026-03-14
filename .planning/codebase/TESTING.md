# Testing Patterns

**Analysis Date:** 2026-03-13

## Test Framework

**Runner:**
- `proptest = "1.10"` in `Cargo.toml` dev-dependencies
- Standard Rust test runner (via `cargo test`)
- No integration test framework (only unit tests within modules)

**Assertion Library:**
- Standard Rust assertions: `assert!`, `assert_eq!`, `assert_ne!`
- PropTest assertions: `prop_assert!`, `prop_assert_eq!`
- Manual comparison: `.is_none()`, `.is_some()`, `matches!()` pattern matching

**Run Commands:**
```bash
cargo test                    # Run all unit and property-based tests
cargo test -- --nocapture    # Run with output visible
cargo test -- --test-threads=1    # Run single-threaded for determinism
```

## Test File Organization

**Location:**
- Tests co-located with source code using `#[cfg(test)]` modules
- Each source file (`engine.rs`, `mutation.rs`, `behavior.rs`, etc.) contains its own test module
- Two test modules in `engine.rs`: `mod tests` (unit tests) and `mod props` (property-based tests)
- Separate submodule structure: `#[cfg(test)] mod tests { ... }`

**Naming:**
- Test functions use snake_case with descriptive names: `dispatch_returns_none_when_cancelled`
- Test fixtures (helper structs) named concisely: `CounterOp`, `NoRule`, `MixedOp`, `MutationInjector`
- Property tests prefixed with `prop_`: `prop_01_undo_roundtrip`, `prop_02_preview_isolation`
- Fixture-specific test modules: `mod tests` for unit tests, `mod props` for property tests

**Structure:**
```
src/engine.rs
├── #[cfg(test)]
│   └── mod tests
│       ├── CounterOp (fixture)
│       ├── NoRule (fixture)
│       ├── test_dispatch_basic
│       ├── test_undo_basic
│       └── dispatch() return value tests
└── #[cfg(test)]
    └── mod props
        ├── CounterOp (fixture - property variant)
        ├── NoRule (fixture)
        ├── MixedOp (fixture for reversibility)
        ├── mixed_op_strategy (strategy generator)
        ├── proptest! { prop_01_undo_roundtrip }
        ├── proptest! { prop_02_preview_isolation }
        └── proptest! { prop_06_reversible_after_irreversible_undoable }
```

## Test Structure

**Suite Organization:**
From `src/engine.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;

    // Fixture definitions (CounterOp, NoRule, etc.)
    #[derive(Clone, Debug, PartialEq)]
    enum CounterOp {
        Inc,
        Dec,
        Reset { prior: i32 },
    }

    impl Mutation<i32> for CounterOp { ... }

    struct NoRule;
    impl Behavior<i32, CounterOp, (), u8> for NoRule { ... }

    // Test functions
    #[test]
    fn test_name() {
        // arrange
        let mut engine = Engine::new(0i32);

        // act
        let _ = engine.dispatch(...);

        // assert
        assert_eq!(engine.read(), 1);
    }
}
```

**Patterns:**
- **Setup:** Fixtures defined at module level, reused across tests
- **Teardown:** Implicit; no explicit cleanup needed (fixtures are scoped test-local)
- **Assertion:** Direct equality checks with `assert_eq!` and boolean checks with `assert!`

## Mocking

**Framework:** No explicit mocking framework detected

**Patterns:**
Minimal, struct-based "mocks" defined inline within test module:

```rust
struct NoRule;
impl Behavior<i32, CounterOp, (), u8> for NoRule {
    fn id(&self) -> &'static str { "no_rule" }
    fn priority(&self) -> u8 { 0 }
    // other trait methods use defaults (no-op)
}

struct MutationInjector;
impl Behavior<i32, CounterOp, (), u8> for MutationInjector {
    fn id(&self) -> &'static str { "injector" }
    fn priority(&self) -> u8 { 0 }
    fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
        // Custom behavior for testing
        tx.mutations.push(CounterOp::Dec);
    }
}
```

**What to Mock:**
- Behavior implementations: create test-specific structs implementing the Behavior trait
- Mutation types: define simple enum variants for testing state transitions
- Event types: use `()` (unit type) when events irrelevant to test

**What NOT to Mock:**
- The `Engine` itself; use real engine instances in tests
- The mutation application pipeline; test it directly
- The undo/redo stack; verify its behavior with real operations

## Fixtures and Factories

**Test Data:**
Complete fixture example from `src/engine.rs`:

```rust
#[derive(Clone, Debug, PartialEq)]
enum CounterOp {
    Inc,
    Dec,
    Reset { prior: i32 },
}

impl Mutation<i32> for CounterOp {
    fn apply(&self, state: &mut i32) {
        match self {
            CounterOp::Inc => *state += 1,
            CounterOp::Dec => *state -= 1,
            CounterOp::Reset { .. } => *state = 0,
        }
    }
    fn undo(&self, state: &mut i32) {
        match self {
            CounterOp::Inc => *state -= 1,
            CounterOp::Dec => *state += 1,
            CounterOp::Reset { prior } => *state = *prior,
        }
    }
    fn hash_bytes(&self) -> Vec<u8> {
        match self {
            CounterOp::Inc => vec![0],
            CounterOp::Dec => vec![1],
            CounterOp::Reset { prior } => {
                let mut v = vec![2];
                v.extend_from_slice(&prior.to_le_bytes());
                v
            }
        }
    }
}
```

**Location:**
- Fixtures defined within `#[cfg(test)]` modules
- At module level, above test functions
- Separate fixture sets for unit tests (`mod tests`) and property tests (`mod props`)
- Property tests define strategy generators: `fn op_sequence_strategy() -> impl Strategy<...>`

## Coverage

**Requirements:** None enforced by configuration

**View Coverage:**
No coverage tooling configured. Manual coverage determined by reading test suite.

## Test Types

**Unit Tests:**
- Scope: Single engine operation with controlled inputs
- Approach: Create minimal engine, call one operation, assert result
- Example: `dispatch_returns_none_when_cancelled` — tests that cancellation prevents state change
- Files: `src/engine.rs`, `src/mutation.rs`, `src/behavior.rs` contain focused unit tests

**Property-Based Tests:**
- Framework: `proptest` crate
- Scope: Sequences of operations with generated inputs, invariant verification across all variants
- Approach: Define strategy generator, run many randomized test iterations
- Example: `prop_01_undo_roundtrip` — verifies that undoing N operations restores state and hash exactly

**Integration Tests:**
- Not used; engine is a library with no external dependencies to integrate

## Common Patterns

**Async Testing:**
Not applicable; engine is synchronous and no async code present.

**Error Testing:**
No explicit error types tested; the engine uses `Option<T>` for outcomes:

```rust
#[test]
fn dispatch_returns_none_when_cancelled() {
    struct Canceller;
    impl Behavior<i32, CounterOp, (), u8> for Canceller {
        fn id(&self) -> &'static str { "canceller" }
        fn priority(&self) -> u8 { 0 }
        fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
            tx.cancelled = true;
        }
    }
    let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
    engine.add_behavior(Canceller);

    let mut tx = Action::new();
    tx.mutations.push(CounterOp::Inc);
    let result = engine.dispatch_with((), tx);
    assert!(result.is_none(), "cancelled dispatch should return None");
}
```

**Property Test Example:**
From `src/engine.rs`:

```rust
proptest! {
    #[test]
    fn prop_01_undo_roundtrip(ops in op_sequence_strategy()) {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(NoRule);

        let state_before = engine.read();
        let hash_before = engine.replay_hash();

        for op in &ops {
            let mut tx = Action::new();
            tx.mutations.push(op.clone());
            let _ = engine.dispatch_with((), tx);
        }

        for _ in &ops {
            engine.undo();
        }

        prop_assert_eq!(engine.read(), state_before);
        prop_assert_eq!(engine.replay_hash(), hash_before);
    }
}
```

**Strategy Generators:**
```rust
fn op_sequence_strategy() -> impl Strategy<Value = Vec<CounterOp>> {
    prop::collection::vec(
        prop_oneof![Just(CounterOp::Inc), Just(CounterOp::Dec)],
        0..=20,
    )
}

fn reversible_irrev_reversible_strategy() -> impl Strategy<Value = (Vec<MixedOp>, Vec<MixedOp>)> {
    (
        prop::collection::vec(Just(MixedOp::Rev), 0..=5usize),
        prop::collection::vec(Just(MixedOp::Rev), 1..=5usize),
    )
}
```

## Test Coverage Summary

**Core Coverage Areas:**
1. **Dispatch behavior** — returns `Some(action)` for valid mutations, `None` for cancelled/empty
2. **Preview isolation** — `dispatch_preview` leaves state and replay_hash untouched
3. **Undo/redo roundtrips** — undoing N operations restores original state and hash
4. **Irreversibility barriers** — actions containing irreversible mutations clear undo stack
5. **Reversible after irreversible** — operations after barrier are individually undoable but can't undo past barrier
6. **Behavior lifecycle** — behaviors execute in priority order, `before` then `after` in correct phases

**Named Property Tests:**
- `prop_01_undo_roundtrip` — N operations + full undo = original state
- `prop_02_preview_isolation` — preview doesn't mutate any observable state
- `prop_04_cancelled_tx_isolation` — cancelled actions have no side effects
- `prop_05_irreversible_clears_undo_stack` — irreversible mutations barrier history
- `prop_06_reversible_after_irreversible_undoable` — reversible ops after barrier individually undoable

---

*Testing analysis: 2026-03-13*
