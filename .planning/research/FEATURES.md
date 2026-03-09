# Feature Research

**Domain:** Property-based testing of a generic Rust game rule engine with undo/redo
**Researched:** 2026-03-08
**Confidence:** HIGH (proptest docs verified via official sources and docs.rs)

---

## Feature Landscape

### Table Stakes (Users Expect These)

These are correctness properties the engine must demonstrably satisfy. A library claiming determinism and undo/redo soundness without these tests is not credible.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Operation round-trip: apply then undo returns to prior state | The core contract of `Operation<S>` — if `undo` is wrong, everything downstream breaks | LOW | Pure property: `op.apply(&mut s); op.undo(&mut s); assert!(s == original)`. Needs `PartialEq` on state. |
| Hash determinism: same `hash_bytes()` sequence always produces same replay hash | FNV-1a is deterministic by definition; test proves the implementation is correct | LOW | Generate arbitrary op sequences, run twice from same state, assert hashes match. |
| Undo restores state identity | `engine.dispatch(...)` then `engine.undo()` must leave state identical to before dispatch | MEDIUM | Requires generating valid transactions. State must implement `PartialEq + Clone`. |
| Redo re-applies state correctly | After `undo`, `redo` must produce the same post-dispatch state | MEDIUM | Builds on undo correctness. Test: dispatch, capture state A; undo; redo; assert state == A. |
| Undo/redo replay hash round-trip | `replay_hash` after undo must equal pre-dispatch hash; after redo must equal post-dispatch hash | MEDIUM | The `CommitFrame` stores `state_hash_before` and `state_hash_after` for exactly this purpose. |
| Cancelled transaction leaves state unchanged | When `tx.cancelled = true`, no ops are applied and engine state is unmodified | LOW | Unit test: cancel a transaction, assert state and hash unchanged. |
| Non-reversible transaction clears redo stack | After a fresh `dispatch`, `redo` must be a no-op (redo stack cleared in code) | LOW | Structural: dispatch, undo, dispatch again, verify redo does nothing. |
| `dispatch_preview` has zero lasting side effects | State, hash, lifetimes, enabled set all restored after preview | LOW | Capture all mutable fields before preview, assert all equal after. |
| Rule priority ordering is stable | Rules with lower priority value run before higher (sort_by_key on add_rule) | LOW | Add rules in reverse priority order, dispatch, verify `before` hooks fire in correct sequence via side-effect log. |

### Differentiators (Competitive Advantage)

These go beyond basic unit tests. They represent the engine-level properties that proptest's shrinking makes practically discoverable — bugs that manual tests miss.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Arbitrary-length undo/redo walk: N dispatches, N undos returns to initial state | Proves correctness holds across any sequence, not just 1 dispatch | MEDIUM | Generate `Vec<BgEvent>` of length 1..20, dispatch all, undo all, assert state == initial. The key property for a game engine. |
| Undo/redo alternation invariant | `dispatch, undo, redo` repeated N times always converges to same final state | MEDIUM | Proptest generates N, runs the cycle; verifies hash and state are equal each time. |
| Preview commutes with dispatch | `dispatch_preview(e)` then `dispatch(e)` must produce same state as `dispatch(e)` alone | MEDIUM | Proves preview truly has no side effects — a subtle correctness property. |
| `RuleLifetime::Triggers` countdown is undoable | After dispatch triggers a rule's lifetime to zero and disables it, undo must restore it to enabled | MEDIUM | The `enabled_snapshot` in `CommitFrame` handles this; test proves it. Critical for rule lifecycle correctness. |
| `RuleLifetime::Turns` countdown is undoable | Turns-based rules decrement on each dispatch; undo must restore the countdown | MEDIUM | Same mechanism as Triggers. Property: dispatch N times, undo N times, lifetime counter is back to start. |
| Hash order-dependence: different op sequences produce different hashes | XOR-fold with FNV_PRIME means op order matters; verify two distinct valid move sequences have distinct hashes | LOW | Use Backgammon concrete moves; this catches hash collisions and ordering bugs. |
| Backgammon board conservation: total checkers never change | 15 checkers per player must be conserved across all moves; no checkers created or destroyed | MEDIUM | Domain invariant enforced by proptest over arbitrary legal move sequences. Catches off-by-one errors in move ops. |
| Backgammon: dice values constrain legal moves | After rolling dice (d1, d2), the moves applied must use exactly those pip values | HIGH | Requires generating dice + board state together, then generating only moves consistent with those dice. `prop_compose` needed. |
| Partial move undo: use one die, undo it, see original board | Models the real gameplay flow: roll dice, move one checker, reconsider, undo that checker | MEDIUM | Generates die value, move op, applies it, undos it, asserts board restored. Core non-determinism test case. |
| State machine transition fuzz: arbitrary event sequences do not panic | Dispatch arbitrary events including illegal ones (cancelled transactions); engine must never panic or corrupt state | MEDIUM | Use `proptest-state-machine` `StateMachineTest` with `preconditions` to exclude illegal states, or just feed all events and check panic-freedom + conservation invariants. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Testing rule hook ordering via mutable global state | Seems like a straightforward way to record execution order across rules | Global mutable state in tests is non-deterministic under proptest's parallel execution; causes flaky tests | Return execution log as part of the transaction or use a dedicated test-only rule that appends to a `Vec` held in the game state |
| Property tests on the full backgammon legal move generator | Tempting to test the game logic exhaustively at this layer | This is integration/game testing, not engine testing; backgammon legal move generation is complex and belongs in the game layer, not the engine layer tests | Test board conservation and dice-constraint properties instead; let the game layer own legality |
| Testing `write()` preserves semantic state meaning | `write()` is explicitly a reset — it clears undo/redo stacks and resets hash | Testing that write "correctly" sets state conflates the engine with game-level semantics | Unit test only: assert that after `write(snapshot)`, undo is no-op, redo is no-op, hash is FNV_OFFSET |
| Concurrent/parallel dispatch testing | Sounds thorough | The engine is single-threaded by design (`Engine` is not `Send + Sync`); concurrent testing adds no value and may require unsafe workarounds | If concurrency is ever needed, that is a new design milestone, not a test task |
| Testing the FNV-1a hash function itself | Developers sometimes unit test the primitive for completeness | FNV-1a is a public-domain algorithm with published test vectors; reinventing those tests adds noise | Use one known-value test per module as a smoke check; the real value is in testing the engine's use of the hash, not the hash itself |
| `dispatch_preview` property tests with side-effecting rules | Seems like a natural extension to test | If a rule's `before`/`after` hooks have external side effects (IO, global state), preview cannot truly be side-effect-free — that is a game-layer problem, not an engine problem | Document that `dispatch_preview` only undoes engine-internal state; rules are responsible for not having external side effects |

---

## Feature Dependencies

```
[Operation round-trip unit test]
    └──required by──> [Undo restores state identity]
                          └──required by──> [Arbitrary-length undo/redo walk]
                          └──required by──> [Undo/redo alternation invariant]
                          └──required by──> [Triggers/Turns lifetime undo]

[Hash determinism unit test]
    └──required by──> [Undo/redo replay hash round-trip]
                          └──required by──> [Hash order-dependence test]

[Cancelled transaction test]
    └──required by──> [Non-reversible tx clears redo stack]

[dispatch_preview zero side effects]
    └──required by──> [Preview commutes with dispatch]

[Backgammon board setup (State + Op + Event types)]
    └──required by──> [Board conservation invariant]
    └──required by──> [Dice-constraint move generation]
                          └──required by──> [Partial move undo]
    └──required by──> [State machine transition fuzz]
```

### Dependency Notes

- **Operation round-trip is the foundation:** Every undo-correctness property depends on `apply` and `undo` being inverses. This must be tested first; if it fails, all higher-level undo tests fail for the wrong reason.
- **Hash unit test enables hash round-trip:** The replay hash properties (undo restores hash, redo restores hash) are not testable without first confirming the hash accumulation logic is correct.
- **Backgammon types must exist before backgammon property tests:** The example game (`examples/backgammon.rs`) must compile and expose `proptest`-derivable strategies before any game-specific properties can run. This is a hard sequential dependency.
- **Dice-constraint depends on board state strategy:** To test that moves respect dice values, you need a strategy that generates `(board_state, dice_roll)` jointly — the `prop_compose` flat_map pattern. The board state strategy must come first.

---

## MVP Definition

### Launch With (v1 — this milestone)

Minimum set to make the claim "determinism and undo/redo correctness are machine-verified" credible.

- [ ] Operation round-trip property test (per-module, inline `#[cfg(test)]`) — proves the `Operation<S>` contract
- [ ] Hash determinism property test — proves FNV-1a accumulation is pure
- [ ] Undo restores state identity property test — the core engine guarantee
- [ ] Undo/redo replay hash round-trip property test — proves hash tracks state correctly
- [ ] `dispatch_preview` zero side effects unit test — proves preview is safe to call freely
- [ ] Cancelled transaction leaves state unchanged unit test — proves cancellation is clean
- [ ] `RuleLifetime::Triggers` undo correctness test — proves lifecycle is reversible
- [ ] `RuleLifetime::Turns` undo correctness test — proves lifecycle is reversible
- [ ] Arbitrary-length undo/redo walk (proptest, N dispatches then N undos) using Backgammon — the headline property test
- [ ] Backgammon board conservation invariant (proptest) — domain-level property
- [ ] Partial move undo with one die (proptest) — exercises the non-determinism use case

### Add After Validation (v1.x)

These add confidence but are not needed to establish core correctness.

- [ ] Undo/redo alternation invariant (dispatch, undo, redo repeated N times) — add when undo walk is green
- [ ] Preview commutes with dispatch property — add after preview side-effect test is green
- [ ] State machine transition fuzz with `proptest-state-machine` — add if edge cases surface from the walk tests
- [ ] Hash order-dependence test — add once board conservation and move generation are stable

### Future Consideration (v2+)

- [ ] Concurrent dispatch safety — only if `Engine` ever becomes `Send`; out of scope for this library's design
- [ ] Exhaustive backgammon legality testing — belongs in a separate game-layer test suite, not the engine

---

## Feature Prioritization Matrix

| Feature | Value to Engine Correctness | Implementation Cost | Priority |
|---------|--------------------------|---------------------|----------|
| Operation round-trip | HIGH | LOW | P1 |
| Hash determinism | HIGH | LOW | P1 |
| Undo restores state | HIGH | LOW | P1 |
| Undo/redo hash round-trip | HIGH | LOW | P1 |
| dispatch_preview side effects | HIGH | LOW | P1 |
| Cancelled tx unchanged | HIGH | LOW | P1 |
| Triggers/Turns undo | HIGH | MEDIUM | P1 |
| Arbitrary-length undo/redo walk | HIGH | MEDIUM | P1 |
| Backgammon board conservation | HIGH | MEDIUM | P1 |
| Partial move undo | HIGH | MEDIUM | P1 |
| Undo/redo alternation invariant | MEDIUM | MEDIUM | P2 |
| Preview commutes with dispatch | MEDIUM | MEDIUM | P2 |
| State machine transition fuzz | MEDIUM | HIGH | P2 |
| Hash order-dependence | LOW | LOW | P2 |
| Exhaustive legality testing | LOW | HIGH | P3 |
| Concurrent dispatch safety | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for this milestone — engine correctness claims require these
- P2: Should have — adds depth but not foundational
- P3: Defer — out of scope or belongs to a different layer

---

## Implementation Notes by Test Category

### proptest Strategy Patterns Needed

**For engine unit properties (Operations, hash):**
Use `proptest! { #[test] fn ... }` with `any::<T>()` or explicit ranges. No special strategy infrastructure needed. These are the simplest tests.

**For undo/redo walk (arbitrary event sequences):**
Use `prop::collection::vec(backgammon_event_strategy(), 1..20)` to generate a sequence of events. Apply all, undo all. The `prop_compose!` macro builds `backgammon_event_strategy()` from dice ranges and point indices.

```rust
prop_compose! {
    fn arb_dice()(d1 in 1u8..=6, d2 in 1u8..=6) -> (u8, u8) { (d1, d2) }
}
prop_compose! {
    fn arb_point()(p in 0usize..24) -> usize { p }
}
```

**For board conservation and dice-constrained moves:**
Requires generating `(board_state, dice_roll)` jointly via `prop_flat_map` — one value constrains the next. This is the `vec_and_index` pattern from the proptest book applied to game state.

**For state machine fuzzing (optional v1.x):**
`proptest-state-machine` crate (`proptest-state-machine = "0.3"` in `[dev-dependencies]`). Implement `ReferenceStateMachine` as a simple model (just undo stack depth, checker counts) and `StateMachineTest` as the real `Engine`. Use `prop_state_machine!` macro. Strongest for finding minimal failing sequences via shrinking.

### What Makes Backgammon the Right Test Game

Backgammon stresses the engine in ways tic-tac-toe cannot:

1. **Non-determinism:** Dice rolls are part of the event. `Engine` is deterministic; the dice are external entropy. Tests must model this correctly — generate dice as part of test input, not inside rules.
2. **Partial moves:** A player rolls (3, 5) and wants to move one checker 3 pips then reconsider. This is undo in the middle of a "turn" — the engine sees two separate dispatchable events within one logical turn.
3. **Board complexity:** 24 points + bar + home per player gives enough state space that conservation invariants are non-trivial to accidentally satisfy.
4. **Doubles:** Rolling doubles gives 4 moves of the same value, not 2 — the event carries `(d1, d2)` and the number of usable moves changes. This stresses any move-generation strategy.

### Scope Boundary: Engine vs Game Layer

Tests **in `src/`** (inline `#[cfg(test)]`): engine mechanics — operation contracts, hash, undo/redo stack behavior, lifetime lifecycle, preview isolation.

Tests **in `examples/backgammon.rs`** or a `tests/` integration file: domain properties — board conservation, dice constraint, partial move undo. These use the engine as a black box via its public API.

Do NOT mix: engine module tests should not import `backgammon` types. Backgammon tests should not reach into engine internals.

---

## Sources

- [Proptest State Machine Testing — Official Proptest Book](https://proptest-rs.github.io/proptest/proptest/state-machine.html) — HIGH confidence, official docs
- [proptest-state-machine crate — crates.io](https://crates.io/crates/proptest-state-machine) — HIGH confidence
- [proptest docs.rs — Strategy trait and prop_compose](https://docs.rs/proptest/latest/proptest/) — HIGH confidence, official docs
- [Model-Based Stateful Testing with proptest-state-machine — Nikos Baxevanis (2025)](https://blog.nikosbaxevanis.com/2025/01/10/state-machine-testing-proptest/) — MEDIUM confidence, verified practitioner post
- [State Machine Properties — propertesting.com](https://propertesting.com/book_state_machine_properties.html) — MEDIUM confidence, community reference
- [Higher-Order Strategies — Proptest Book](https://altsysrq.github.io/proptest-book/proptest/tutorial/higher-order.html) — HIGH confidence, official docs (prop_compose / flat_map patterns)
- [Exploring Round-trip Properties in Property-based Testing — PLClub (2023)](https://www.cis.upenn.edu/~plclub/blog/2023-12-07-round-trip-properties/) — MEDIUM confidence, academic source

---
*Feature research for: herdingcats proptest milestone*
*Researched: 2026-03-08*
