# Phase 02: Engine Property Tests - Research

**Researched:** 2026-03-08
**Domain:** proptest 1.x property-based testing in Rust, engine invariant verification
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Test file location:**
- All property tests inline in `engine.rs` as a separate `#[cfg(test)] mod props { ... }` block
- The existing `#[cfg(test)] mod tests { ... }` block keeps unit tests; the new `mod props` holds proptest-based tests
- All four property tests in one module — shared CounterOp fixture, shared NoRule, no submodule split

**Op fixture scope:**
- Property test strategies generate `Inc` and `Dec` only — no `Reset` in generated sequences
  - `Reset { prior }` requires knowing current state at generation time; excluded to keep strategies stateless
  - `Reset` remains in unit tests (mod tests) for targeted roundtrip coverage
- PROP-04 (cancelled transaction): generate arbitrary Vec<CounterOp> (Inc/Dec), push into tx.ops, then set `tx.cancelled = true` — confirms ops are suppressed, not just that an empty tx is a no-op
- PROP-03 (rule lifetimes): generate `n` from range `1..=10` (not arbitrary u32) — dispatch exactly n times, verify disabled at n
- All four property tests use the same fixture type: `Engine<i32, CounterOp, (), u8>`

**Rule presence:**
- PROP-01 and PROP-02: add `NoRule` (passthrough, no-op `before`/`after`) — tests that rule traversal doesn't affect invariants, slightly more realistic than bare engine
- PROP-03: use a dedicated `CountingRule` with a no-op `before()` hook — lifetime decrement logic is in the engine, not the rule; keep the invariant clean
- PROP-04: NoRule sufficient (cancelled tx suppresses ops regardless of rules)

**Test configuration:**
- Proptest case count: leave at default (256 cases per test) — override with `PROPTEST_CASES` env var if needed
- Generated op sequence length for PROP-01: `0..=20` ops per sequence
- No `proptest.toml` config file — leave all shrinking settings at proptest defaults

### Claude's Discretion
- Exact `prop_compose!` strategy names and signatures
- How to handle the `tx.irreversible` flag in generated transactions for PROP-01 (must be true for undo stack to record the frame — likely set statically in the strategy)
- `CountingRule` struct placement within `mod props`
- Whether to use `prop::collection::vec` or a custom strategy for op sequence generation

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PROP-01 | Property test: arbitrary sequences of `apply` then `undo` return the original state AND the original `replay_hash` | Engine stores `state_hash_before` in CommitFrame and restores it on undo; `tx.irreversible = true` required for undo stack entry; proptest `prop::collection::vec` generates op sequences |
| PROP-02 | Property test: `dispatch_preview` leaves all four engine mutable fields identical after return — `state`, `replay_hash`, `lifetimes`, `enabled` set | `dispatch_preview` snapshot/restore pattern in source is total; all four fields are saved and restored unconditionally |
| PROP-03 | Property test: `RuleLifetime::Turns(n)` rule disabled after exactly `n` dispatches; `RuleLifetime::Triggers(n)` rule disabled after exactly `n` `before()` calls | `Turns` decremented in post-dispatch loop (always runs); `Triggers` decremented inside `before` loop only when rule is enabled; both reach 0-check and call `enabled.remove` at that exact count |
| PROP-04 | Property test: cancelled transaction leaves `state` and `replay_hash` completely unchanged — verified by snapshot before and after dispatch | Engine only updates `replay_hash` and pushes CommitFrame when `tx.irreversible && !tx.cancelled`; ops are only applied when `!tx.cancelled`; snapshot-compare pattern confirms both fields unchanged |
</phase_requirements>

---

## Summary

Phase 02 adds four proptest-based property tests to `engine.rs` in a new `#[cfg(test)] mod props` block (sibling to the existing `mod tests`). The tests verify invariants of the engine's dispatch/undo/preview/cancellation machinery for arbitrary inputs. `proptest = "1.10"` is already in `[dev-dependencies]`; no new dependencies are needed.

The code structure is entirely additive: no existing files are modified except `engine.rs`, and only the test section grows. The four test functions share a `CounterOp` fixture (Inc/Dec only) and most share a `NoRule` passthrough rule. PROP-03 requires a dedicated `CountingRule` with no-op before/after to isolate the engine's own lifetime decrement logic.

**Primary recommendation:** Use `prop::collection::vec(prop_oneof![Just(CounterOp::Inc), Just(CounterOp::Dec)], 0..=20)` for op sequence generation; use `prop_compose!` for structured composites (e.g., n + dispatch count for PROP-03). All tests fit naturally into a single `proptest! { ... }` block per test function.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| proptest | 1.10 (already in Cargo.toml) | Property-based test generation and shrinking | Standard PBT crate for Rust; already declared; no change needed |

### Supporting
| Macro / Item | Source | Purpose | When to Use |
|-------------|--------|---------|-------------|
| `proptest! { }` | `proptest::prelude::*` | Define parameterized test functions | All four PROP tests |
| `prop_compose!` | `proptest::prelude::*` | Build named strategy functions | Op-sequence strategy, n+count strategy for PROP-03 |
| `prop::collection::vec` | `proptest::prelude::prop` | Generate `Vec<T>` of specified length range | Op sequence generation in PROP-01, PROP-04 |
| `prop_oneof!` | `proptest::prelude::*` | Choose uniformly among a set of strategies | Selecting Inc vs Dec variant |
| `Just(value)` | `proptest::prelude::*` | Lift a concrete value to a Strategy | Wrapping enum variants that have no data |
| `prop_assert_eq!` | `proptest::prelude::*` | Equality assertion inside proptest body | Comparing state/hash before and after |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `prop::collection::vec` with `prop_oneof!` | `proptest-derive` `#[derive(Arbitrary)]` | derive is OUT OF SCOPE per REQUIREMENTS.md; manual strategies required |
| Inline `proptest! { }` strategies | Separate strategy functions with `prop_compose!` | Both work; `prop_compose!` preferred for PROP-03 where n and dispatch-count are linked |

**Installation:** No changes needed — `proptest = "1.10"` already in `[dev-dependencies]`.

## Architecture Patterns

### Recommended Module Structure (within engine.rs)

```
engine.rs
├── CommitFrame (private struct)
├── Engine<S,O,E,P> (pub struct)
├── impl Engine { ... }
├── #[cfg(test)] mod tests { ... }   ← existing unit tests (no changes)
└── #[cfg(test)] mod props {         ← NEW: all four property tests
    ├── use super::*;
    ├── use proptest::prelude::*;
    ├── CounterOp (re-declare or use super::tests::CounterOp via re-export)
    ├── NoRule (struct + impl Rule)
    ├── CountingRule (struct + impl Rule) — for PROP-03 only
    ├── fn counter_op_strategy() -> impl Strategy<Value=CounterOp>
    ├── fn op_sequence_strategy() -> impl Strategy<Value=Vec<CounterOp>>
    ├── proptest! { fn prop_01_undo_roundtrip ... }
    ├── proptest! { fn prop_02_preview_isolation ... }
    ├── proptest! { fn prop_03_rule_lifetimes ... }
    └── proptest! { fn prop_04_cancelled_tx ... }
    }
```

### Pattern 1: Op Sequence Strategy

**What:** Generate `Vec<CounterOp>` containing only Inc and Dec, length 0 to 20.
**When to use:** PROP-01 and PROP-04 — anywhere an arbitrary sequence of ops is needed.

```rust
// Source: proptest docs — prop::collection::vec + prop_oneof!
fn op_sequence_strategy() -> impl Strategy<Value = Vec<CounterOp>> {
    prop::collection::vec(
        prop_oneof![Just(CounterOp::Inc), Just(CounterOp::Dec)],
        0..=20,
    )
}
```

### Pattern 2: PROP-01 Test Body Structure

**What:** Take op sequence, snapshot state+hash, dispatch with `irreversible=true`, undo all, compare.
**When to use:** PROP-01.

```rust
// Source: engine.rs dispatch + undo contract (verified from source)
proptest! {
    #[test]
    fn prop_01_undo_roundtrip(ops in op_sequence_strategy()) {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_rule(NoRule, RuleLifetime::Permanent);

        let state_before = engine.read();
        let hash_before = engine.replay_hash();

        // Dispatch each op in its own irreversible tx so undo stack grows
        for op in &ops {
            let mut tx = Transaction::new(); // irreversible=true by default
            tx.ops.push(op.clone());
            engine.dispatch((), tx);
        }

        // Undo all frames
        for _ in &ops {
            engine.undo();
        }

        prop_assert_eq!(engine.read(), state_before);
        prop_assert_eq!(engine.replay_hash(), hash_before);
    }
}
```

### Pattern 3: PROP-02 Preview Isolation

**What:** Snapshot all four engine fields, call dispatch_preview, assert all four unchanged.
**When to use:** PROP-02.

```rust
// Source: engine.rs dispatch_preview — saves/restores state, lifetimes, enabled, replay_hash
proptest! {
    #[test]
    fn prop_02_preview_isolation(ops in op_sequence_strategy()) {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_rule(NoRule, RuleLifetime::Permanent);

        // Dispatch a few ops to establish non-trivial pre-state
        for op in &ops {
            let mut tx = Transaction::new();
            tx.ops.push(op.clone());
            engine.dispatch((), tx);
        }

        let state_before = engine.read();
        let hash_before = engine.replay_hash();

        // dispatch_preview with an arbitrary ops tx
        let mut preview_tx = Transaction::new();
        for op in &ops {
            preview_tx.ops.push(op.clone());
        }
        engine.dispatch_preview((), preview_tx);

        prop_assert_eq!(engine.read(), state_before,
            "state changed after dispatch_preview");
        prop_assert_eq!(engine.replay_hash(), hash_before,
            "replay_hash changed after dispatch_preview");
    }
}
```

Note: `lifetimes` and `enabled` are private fields — their restoration is validated indirectly via a second dispatch after preview (state and hash must equal what a dispatch without preview produces). Alternatively, because `NoRule` is `Permanent`, its presence in `enabled` is stable; the test confirms isolation of the observable interface.

### Pattern 4: PROP-03 Lifetime Off-by-One

**What:** Use `prop_compose!` to link `n` (the lifetime count) with the dispatch count. Add rule with `Turns(n)` or `Triggers(n)`, dispatch exactly `n` times, verify disabled; add with `Turns(n)` dispatch `n-1` times verify still enabled.
**When to use:** PROP-03.

```rust
// Source: engine.rs dispatch loop — Turns decremented post-loop always, Triggers inside before-loop
prop_compose! {
    fn n_and_ops()(n in 1u32..=10u32) -> u32 { n }
}

proptest! {
    #[test]
    fn prop_03_turns_lifetime(n in 1u32..=10u32) {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_rule(CountingRule, RuleLifetime::Turns(n));

        // Dispatch n-1 times: rule must still be enabled (reachable via before/after)
        for _ in 0..n.saturating_sub(1) {
            engine.dispatch((), Transaction::new());
        }
        // After n-1 dispatches the rule should still be tracked
        // (We can't access engine.enabled directly, but we can add a second rule
        //  that only fires when CountingRule is still enabled, or simply dispatch
        //  one more and confirm counting behavior changed)

        // Dispatch the nth time: rule must now be disabled
        engine.dispatch((), Transaction::new());

        // Verify by adding an event where CountingRule would have incremented a counter
        // — since rule is disabled, before() no longer fires, counter stays 0
        // Use a side-channel: CountingRule wraps an Rc<Cell<u32>> trigger count
        // See CountingRule design note in Code Examples section.
    }

    #[test]
    fn prop_03_triggers_lifetime(n in 1u32..=10u32) {
        // Same structure, add rule with RuleLifetime::Triggers(n)
        // Triggers decrements inside the before() loop when rule.id() is in enabled
        // After n before() calls, rule is removed from enabled
    }
}
```

### Pattern 5: PROP-04 Cancelled Transaction Isolation

**What:** Snapshot `state` and `replay_hash`, dispatch a tx with `cancelled = true` and arbitrary ops, assert both unchanged.
**When to use:** PROP-04.

```rust
// Source: engine.rs dispatch — ops only applied when !tx.cancelled;
//         hash only updated and frame only pushed when irreversible && !tx.cancelled
proptest! {
    #[test]
    fn prop_04_cancelled_tx(ops in op_sequence_strategy()) {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_rule(NoRule, RuleLifetime::Permanent);

        let state_before = engine.read();
        let hash_before = engine.replay_hash();

        let mut tx = Transaction::new();
        for op in ops {
            tx.ops.push(op);
        }
        tx.cancelled = true;

        engine.dispatch((), tx);

        prop_assert_eq!(engine.read(), state_before);
        prop_assert_eq!(engine.replay_hash(), hash_before);
    }
}
```

### Anti-Patterns to Avoid

- **Asserting only state, not hash:** Every undo test MUST assert both `engine.read()` and `engine.replay_hash()`. The STATE.md decision is explicit: "Every undo/redo property test must assert both engine.read() (state) AND engine.replay_hash() — not just state equality."
- **Using `tx.irreversible = false` in PROP-01:** If `irreversible` is false, dispatch does NOT push to the undo stack and does NOT update `replay_hash`. The undo call becomes a no-op. Set `irreversible = true` (which is already the Transaction::new() default).
- **Re-using `Reset` variant in generated sequences:** `Reset { prior }` embeds state at generation time, making the strategy stateful. Inc/Dec are stateless and sufficient.
- **Directly accessing private fields for PROP-02/PROP-03:** `engine.lifetimes` and `engine.enabled` are private. Validate their restoration through observable behavior (dispatch a second time, confirm state/hash track correctly).
- **Asserting Turns lifetime for PROP-04:** Cancelled transactions still decrement `Turns` lifetimes (the Turns loop runs unconditionally in dispatch). PROP-04 only claims `state` and `replay_hash` are unchanged — lifetime side-effects on cancelled dispatch are NOT part of the PROP-04 invariant.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random test input generation | Manual PRNG + test vectors | `proptest::prelude::*` | Proptest handles shrinking, seed replay, and coverage; hand-rolled random loses all of that |
| Enum strategy for CounterOp | Match + random u8 dispatch | `prop_oneof![Just(CounterOp::Inc), Just(CounterOp::Dec)]` | Proptest shrinks enum choices; raw random cannot shrink |
| Variable-length op sequences | for loop with rand | `prop::collection::vec(strategy, range)` | Handles length shrinking (shrinks toward shorter sequence on failure) |
| Dependent strategy (n + count) | Nested proptest!/closure | `prop_compose!` with two argument lists | Proptest's shrinking understands the dependency relationship |

**Key insight:** Proptest's shrinking is the primary value-add over unit tests. Every layer of hand-rolled randomness removes shrinking ability, making failures harder to diagnose.

## Common Pitfalls

### Pitfall 1: tx.irreversible Confusion in PROP-01

**What goes wrong:** Test dispatches ops but undo is a no-op, so the roundtrip always trivially passes (nothing was committed to undo stack).
**Why it happens:** `Transaction::new()` defaults `irreversible = true`, but if a test accidentally sets it false, no CommitFrame is pushed.
**How to avoid:** Use `Transaction::new()` without modification for undo tests. The default `irreversible = true` is correct. A trivially-passing PROP-01 (empty undo stack) would be a false green.
**Warning signs:** PROP-01 passes instantly with 0 shrink steps even on intentionally broken undo logic.

### Pitfall 2: PROP-03 CountingRule Observability

**What goes wrong:** Cannot assert "rule fired" or "rule was skipped" because `engine.enabled` is private and `before()`/`after()` have no observable return value.
**Why it happens:** Rule hooks are side-effect-only; there's no built-in "did this rule fire" output.
**How to avoid:** Use an `Rc<Cell<u32>>` (or `Arc<AtomicU32>`) counter in `CountingRule` that increments in `before()`. Compare trigger count to expected count after n dispatches. Alternatively: use the lifetime expiry itself as the observable — after n dispatches with `Triggers(n)`, an op pushed by the rule no longer appears in state.
**Warning signs:** PROP-03 tests rules but never actually distinguishes "enabled before dispatch n" from "disabled after dispatch n."

### Pitfall 3: PROP-04 Turns Lifetime Side-Effect Misunderstanding

**What goes wrong:** Test adds a `Turns(n)` rule and then asserts after a cancelled dispatch that ALL engine state is identical including lifetimes — this will fail because `Turns` IS decremented even on cancelled transactions.
**Why it happens:** The `Turns` decrement loop in `dispatch` runs unconditionally (after the `if !tx.cancelled` ops block, before the `if tx.irreversible && !tx.cancelled` hash/stack block).
**How to avoid:** PROP-04 uses `NoRule` with `Permanent` lifetime. The invariant is state+hash only — not lifetime/enabled state. Keep PROP-04 focused on what it claims.
**Warning signs:** PROP-04 fails when a `Turns` rule is added and lifetime changes are asserted.

### Pitfall 4: Shared Fixture Name Collision Between mod tests and mod props

**What goes wrong:** `CounterOp` is defined in `mod tests` and `mod props` — the compiler sees a conflict or a test in `mod props` tries to reference `super::tests::CounterOp` which is not a `pub` type.
**Why it happens:** `mod tests` inner items are private to that module; `mod props` is a sibling, not a child.
**How to avoid:** Re-declare `CounterOp`, `NoRule`, and `CountingRule` directly inside `mod props`. They're small structs with trivial impls. Duplication is fine in test code; it avoids visibility coupling between test modules.
**Warning signs:** `error[E0603]: struct 'CounterOp' is private` when `mod props` references `tests::CounterOp`.

### Pitfall 5: dispatch_preview lifetimes/enabled Observability

**What goes wrong:** PROP-02 tries to assert `engine.lifetimes == snapshot` but the field is private. Test either fails to compile or doesn't verify the full isolation claim.
**Why it happens:** `lifetimes` and `enabled` are private Engine fields.
**How to avoid:** Verify isolation of `lifetimes`/`enabled` indirectly: after calling `dispatch_preview`, call `dispatch` on the same ops and verify the result matches what a fresh engine (without preview) would produce. If lifetimes were mutated by preview, subsequent dispatches would diverge.
**Warning signs:** PROP-02 only asserts `state` and `hash`, missing the `lifetimes`/`enabled` claim.

## Code Examples

Verified patterns from source code and official proptest docs:

### CounterOp fixture for mod props
```rust
// Source: engine.rs mod tests — re-declare here for visibility isolation
#[derive(Clone, Debug)]
enum CounterOp {
    Inc,
    Dec,
}

impl Operation<i32> for CounterOp {
    fn apply(&self, state: &mut i32) {
        match self {
            CounterOp::Inc => *state += 1,
            CounterOp::Dec => *state -= 1,
        }
    }
    fn undo(&self, state: &mut i32) {
        match self {
            CounterOp::Inc => *state -= 1,
            CounterOp::Dec => *state += 1,
        }
    }
    fn hash_bytes(&self) -> Vec<u8> {
        match self {
            CounterOp::Inc => vec![0],
            CounterOp::Dec => vec![1],
        }
    }
}
```

### NoRule fixture for mod props
```rust
// Source: engine.rs mod tests — re-declare here
struct NoRule;
impl Rule<i32, CounterOp, (), u8> for NoRule {
    fn id(&self) -> &'static str { "no_rule" }
    fn priority(&self) -> u8 { 0 }
}
```

### CountingRule for PROP-03 (with observable trigger counter)
```rust
// Design pattern for making rule firing observable
use std::cell::Cell;
use std::rc::Rc;

struct CountingRule {
    trigger_count: Rc<Cell<u32>>,
}

impl Rule<i32, CounterOp, (), u8> for CountingRule {
    fn id(&self) -> &'static str { "counting_rule" }
    fn priority(&self) -> u8 { 0 }
    fn before(&self, _state: &i32, _event: &mut (), _tx: &mut Transaction<CounterOp>) {
        self.trigger_count.set(self.trigger_count.get() + 1);
    }
}
```

Note: `Rc<Cell<u32>>` is not `Send` but that is fine — proptest by default runs tests single-threaded. If proptest is configured with fork mode, use `Arc<AtomicU32>` instead. Default single-threaded mode is confirmed correct here.

### Op sequence strategy
```rust
// Source: proptest docs — prop::collection::vec + prop_oneof!
fn op_sequence_strategy() -> impl Strategy<Value = Vec<CounterOp>> {
    prop::collection::vec(
        prop_oneof![Just(CounterOp::Inc), Just(CounterOp::Dec)],
        0..=20,
    )
}
```

### Transaction construction for PROP-01 undo loop
```rust
// Key: Transaction::new() defaults irreversible=true — do not override
for op in &ops {
    let mut tx = Transaction::new(); // irreversible=true, deterministic=true
    tx.ops.push(op.clone());
    engine.dispatch((), tx);
}
for _ in &ops {
    engine.undo();
}
```

### Engine type and imports for mod props
```rust
#[cfg(test)]
mod props {
    use super::*;
    use crate::transaction::{RuleLifetime, Transaction};
    use proptest::prelude::*;
    // ... fixtures, strategies, tests
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual test vectors | proptest property tests | Already decided (v1 requirements) | Machine-verifiable invariants for arbitrary inputs |
| proptest-derive for Arbitrary | Manual prop_compose! strategies | v1 decision (OUT OF SCOPE per REQUIREMENTS.md) | Slightly more verbose but no proc-macro dependency |

**Deprecated/outdated:**
- `proptest-derive`: explicitly out of scope per REQUIREMENTS.md — "Added complexity for marginal gain; write prop_compose! strategies manually"

## Open Questions

1. **CountingRule observability approach for PROP-03**
   - What we know: `engine.enabled` is private; Rule hooks have no return value; `Rc<Cell<u32>>` is the idiomatic single-threaded solution
   - What's unclear: Whether the planner prefers the Rc<Cell> observable approach vs. a state-impact approach (CountingRule pushes an op and state change is the observable proxy)
   - Recommendation: Use `Rc<Cell<u32>>` — cleaner, direct, and doesn't require the rule to push ops (which would entangle the lifetime test with op correctness)

2. **PROP-02 lifetimes/enabled indirect verification**
   - What we know: Direct field access is impossible; indirect verification via subsequent dispatch is possible but adds complexity
   - What's unclear: How thorough PROP-02 needs to be for private fields — the source code is a simple snapshot/restore and the claim is strong
   - Recommendation: Planner should decide whether PROP-02 only asserts observable fields (state, hash) or adds a second-dispatch verification. The CONTEXT.md says "leaves all four engine mutable fields identical" — plan should document how `lifetimes`/`enabled` isolation is demonstrated.

3. **PROP-01 empty sequence handling**
   - What we know: `0..=20` includes 0 ops; an empty sequence dispatches a tx with no ops; undo on an empty undo stack is a no-op
   - What's unclear: With 0 ops, the tx is dispatched but no ops are applied, and `irreversible=true` pushes a CommitFrame with zero ops; undo pops it and restores nothing — this is technically a no-op and still passes
   - Recommendation: 0-length sequence is valid and tests the edge case of empty transactions; include in range as decided.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | proptest 1.10 |
| Config file | None — leave at proptest defaults, no proptest.toml |
| Quick run command | `cargo test --lib engine::props` |
| Full suite command | `cargo test --lib` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROP-01 | Arbitrary apply+undo sequences restore original state and replay_hash | property | `cargo test --lib engine::props::prop_01` | ❌ Wave 0 |
| PROP-02 | dispatch_preview leaves state, replay_hash, lifetimes, enabled unchanged | property | `cargo test --lib engine::props::prop_02` | ❌ Wave 0 |
| PROP-03 | Turns(n) disabled after n dispatches; Triggers(n) disabled after n before() calls | property | `cargo test --lib engine::props::prop_03` | ❌ Wave 0 |
| PROP-04 | Cancelled tx leaves state and replay_hash bitwise identical | property | `cargo test --lib engine::props::prop_04` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib engine::props`
- **Per wave merge:** `cargo test --lib`
- **Phase gate:** Full suite green (`cargo test`) before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `mod props` block in `src/engine.rs` — covers PROP-01, PROP-02, PROP-03, PROP-04 (all four tests in one block)

*(No separate test files needed — all test code lives inline in engine.rs per locked decision)*

## Sources

### Primary (HIGH confidence)
- `src/engine.rs` (read directly) — complete dispatch, undo, redo, dispatch_preview, add_rule implementations
- `src/transaction.rs` (read directly) — Transaction::new() defaults, RuleLifetime variants
- `src/operation.rs` (read directly) — Operation trait contract
- `Cargo.toml` (read directly) — confirms `proptest = "1.10"` in dev-dependencies
- `https://docs.rs/proptest/1.6.0/proptest/macro.proptest.html` — proptest! macro syntax and configuration
- `https://docs.rs/proptest/1.6.0/proptest/collection/fn.vec.html` — prop::collection::vec signature
- `https://docs.rs/proptest/1.6.0/proptest/macro.prop_compose.html` — prop_compose! syntax and multi-layer example
- `https://docs.rs/proptest/1.6.0/proptest/prelude/index.html` — prelude re-exports: Just, prop_oneof!, prop_assert_eq!, etc.

### Secondary (MEDIUM confidence)
- `cargo test --lib` output (run directly) — confirms 14 existing tests pass; engine compiles cleanly on edition 2024

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — proptest 1.10 is in Cargo.toml; API verified against docs.rs
- Architecture: HIGH — all engine source code read directly; no assumptions about behavior
- Pitfalls: HIGH — pitfalls derived directly from source code reading (Turns/Triggers decrement paths, private field access constraints, module visibility rules)

**Research date:** 2026-03-08
**Valid until:** 2026-09-08 (proptest stable; engine source is project-local and won't drift)
