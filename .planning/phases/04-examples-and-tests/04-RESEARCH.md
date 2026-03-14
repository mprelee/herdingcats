# Phase 4: Examples and Tests - Research

**Researched:** 2026-03-13
**Domain:** Rust examples, proptest property testing, HerdingCats public API exercise
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Tic-tac-toe example structure**
- Run mode: Scripted demo — `main()` runs a pre-scripted sequence of moves with no user input
- Output style: Annotated step-by-step — each operation prints: `[dispatch] Move(0,0) by X => Committed(frame)`; `[undo] => Undone(frame)`, etc.
- Game rules: Full rules — 4 behaviors: `ValidateTurn` (Disallowed if wrong player), `ValidateCell` (InvalidInput if out of bounds, Disallowed if occupied), `PlaceMarker`, `CheckWin` (signals game-over with 3 in a row)
- Outcome coverage: Script is designed to exercise all 7 Outcome variants: `Committed`, `Undone`, `Redone`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted`
- Runs via: `cargo run --example tictactoe`

**Backgammon example structure**
- Scope: Focused demo — only `RollDice` (Irreversible) + `MovePiece` (Reversible) behaviors. No bearing off, hitting, re-entering, pip count, or doubling cube.
- Dice: Scripted/seeded — fixed dice values hard-coded in the script. Same output every run.
- Output style: Same annotated style as tictactoe — `[dispatch] RollDice(3,5) => Committed(frame)   [IRREVERSIBLE]`
- Key demo sequence: dispatch MovePiece, dispatch MovePiece, undo() => Undone, dispatch RollDice (Irreversible — history cleared), undo() => Disallowed(NothingToUndo)
- Runs via: `cargo run --example backgammon`

**Test placement**
- All new Phase 4 tests extend existing `#[cfg(test)]` modules in `src/engine.rs`
- No new `tests/` directory — consistent with the pattern from Phases 1-3
- `proptest!` macro used inline in the same `#[cfg(test)]` module

**Invariant test structure**
- 15 named test functions: `invariant_01_never_advances_without_input`, `invariant_02_dispatch_is_atomic`, ..., `invariant_15_engine_errors_distinct_from_outcomes`
- One test per ARCHITECTURE.md invariant by number — easy to audit against the list
- Tests are self-contained unit tests; they do not depend on game logic from the examples

**Property test coverage (proptest)**
- Suite 1 — Determinism: `prop_dispatch_is_deterministic` — same input sequence applied to two identically-constructed engines always produces the same sequence of outcomes. Uses `vec(any::<u8>(), 0..10)` strategy.
- Suite 2 — Undo correctness: `prop_undo_restores_exact_state` — after an arbitrary sequence of dispatch/undo/redo operations (0..20 operations), the final engine state is consistent with what those operations should produce.
- `proptest = "1.10"` already in `[dev-dependencies]`

### Claude's Discretion
- Exact behavior names and types for the examples (player enum, board struct, etc.)
- Whether `NoChange` and `Aborted` appear in tictactoe via a dedicated board-full check or a "game already over" guard
- Exact proptest strategy type for the undo/redo property test (`Op` enum with Dispatch/Undo/Redo variants is the natural approach)
- Order of invariants covered — can reuse existing test coverage for invariants already well-tested in Phases 1-3

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| EXAM-01 | Tic-tac-toe example compiles and runs, demonstrating dispatch, all outcome variants, and undo/redo | Full public API confirmed; 4-behavior structure designed; all 7 Outcome variants mapped to scripted moves |
| EXAM-02 | Backgammon example compiles and runs, demonstrating dice-roll irreversibility clearing undo history | `Reversibility::Irreversible` + `undo_depth()` API confirmed; key sequence identified |
| TEST-01 | Unit tests cover dispatch outcomes, undo/redo behavior, all 15 core invariants, and edge cases | 15 invariants from ARCHITECTURE.md enumerated; existing test coverage catalogued; gaps identified |
| TEST-02 | Property tests (proptest) verify determinism, atomicity, and undo/redo correctness across arbitrary operation sequences | proptest 1.10 already in dev-deps; `proptest!` macro syntax confirmed; `Op` enum strategy pattern identified |
</phase_requirements>

---

## Summary

Phase 4 is a pure API exercise phase — no new engine features. The entire HerdingCats public API is already implemented and tested across Phases 1-3. The work is: (1) writing two tutorial-quality game examples that drive the real API end-to-end, and (2) adding structured invariant and property tests to `src/engine.rs`'s existing `#[cfg(test)]` module.

The public API surface is small and fully understood: `Engine::new`, `Engine::dispatch`, `Engine::undo`, `Engine::redo`, `Engine::state`, `Engine::undo_depth`, `Engine::redo_depth`, plus the type-system contracts (`EngineSpec`, `Behavior`, `Apply`, `Reversibility`, `BehaviorResult`, `Outcome`, `Frame`, `HistoryDisallowed`, `EngineError`). All bounds on associated types are documented and confirmed.

The 15 core invariants from ARCHITECTURE.md are the definitive test checklist. Approximately 8-10 of the 15 already have strong indirect test coverage from prior phases; the Phase 4 invariant tests add explicit, numbered functions so each invariant is traceable by name.

**Primary recommendation:** Implement tictactoe.rs first (it exercises all 7 Outcome variants and is the user-facing tutorial); then backgammon.rs (focused, short); then the 15 invariant tests (some are trivial one-liners reusing existing helper infrastructure); then the two proptest suites (the `Op` enum strategy is the key design decision).

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| proptest | 1.10 (already in dev-deps) | Property-based testing: generate arbitrary op sequences | Already in `Cargo.toml`; zero setup cost |
| Rust std | (edition 2024) | `println!`, `format!`, output annotation | No external deps needed for example output |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| proptest::prelude::* | (part of proptest 1.10) | Brings `proptest!`, `any`, `prop_assert`, `vec` into scope | Always import in `#[cfg(test)]` block |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `proptest!` macro | `#[proptest]` attribute macro | Attribute form exists but `proptest!` is the primary API; macro form integrates cleanly in `#[cfg(test)]` modules without extra crate |
| `vec(any::<u8>(), 0..10)` | `prop::collection::vec` (same thing) | Identical — `vec` is re-exported under `proptest::prelude` |

**Installation:** Nothing to install. `proptest = "1.10"` is already in `[dev-dependencies]`.

---

## Architecture Patterns

### Recommended Project Structure

No new directories. All new code goes in two existing locations:

```
examples/
├── tictactoe.rs    # Replace placeholder — full scripted demo
└── backgammon.rs   # Replace placeholder — focused irreversibility demo

src/
└── engine.rs       # Append to existing #[cfg(test)] mod tests { ... }
                    #   - 15 invariant_XX_* test functions
                    #   - 2 proptest suites (proptest! block)
```

### Pattern 1: EngineSpec Unit Struct for Examples

Both examples follow the same established pattern as existing tests:

```rust
// Source: src/spec.rs doc example + src/engine.rs TestSpec
struct TicTacToeSpec;

impl EngineSpec for TicTacToeSpec {
    type State     = TicTacToeState;
    type Input     = TicTacToeInput;
    type Diff      = TicTacToeDiff;
    type Trace     = String;
    type NonCommittedInfo = String;
    type OrderKey  = u32;
}
```

**When to use:** Every time an example or test needs a game-specific engine binding.

### Pattern 2: Behavior as Zero-Size Struct

```rust
// Source: src/behavior.rs established pattern
struct ValidateCell;

impl Behavior<TicTacToeSpec> for ValidateCell {
    fn name(&self) -> &'static str { "validate_cell" }
    fn order_key(&self) -> u32 { 0 }
    fn evaluate(
        &self,
        input: &TicTacToeInput,
        state: &TicTacToeState,
    ) -> BehaviorResult<TicTacToeDiff, String> {
        // validation logic returning Stop(reason) or Continue(vec![])
    }
}
```

**When to use:** All behaviors in both examples. Behavior state goes in the game state struct, not in the behavior itself.

### Pattern 3: Apply impl on the Diff type

```rust
// Source: src/apply.rs Apply trait contract
impl Apply<TicTacToeSpec> for TicTacToeDiff {
    fn apply(&self, state: &mut TicTacToeState) -> Vec<String> {
        match self {
            TicTacToeDiff::PlaceMarker { row, col, player } => {
                state.board[*row][*col] = Some(*player);
                vec![format!("placed {:?} at ({},{})", player, row, col)]
            }
            // ...
        }
    }
}
```

**When to use:** One `Apply` impl per Diff type per EngineSpec. Trace entries returned here appear in `Frame::traces`.

### Pattern 4: Annotated Output in main()

The scripted demo pattern that makes Outcome variants visible:

```rust
// Convention established by CONTEXT.md decisions
fn print_dispatch(label: &str, outcome: &Result<Outcome<Frame<TicTacToeSpec>, String>, EngineError>) {
    match outcome {
        Ok(Outcome::Committed(frame)) => println!("[dispatch] {} => Committed(frame #{:?})", label, frame.input),
        Ok(Outcome::Disallowed(reason)) => println!("[dispatch] {} => Disallowed({})", label, reason),
        Ok(Outcome::InvalidInput(reason)) => println!("[dispatch] {} => InvalidInput({})", label, reason),
        Ok(Outcome::NoChange) => println!("[dispatch] {} => NoChange", label),
        Ok(Outcome::Aborted(reason)) => println!("[dispatch] {} => Aborted({})", label, reason),
        _ => {}
    }
}
```

### Pattern 5: proptest! with Op Enum Strategy

The canonical approach for undo/redo property testing (per CONTEXT.md discretion note):

```rust
// Source: proptest docs + CONTEXT.md
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Dispatch(u8),
    Undo,
    Redo,
}

fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        any::<u8>().prop_map(Op::Dispatch),
        Just(Op::Undo),
        Just(Op::Redo),
    ]
}

proptest! {
    #[test]
    fn prop_undo_restores_exact_state(ops in prop::collection::vec(arb_op(), 0..20)) {
        // Apply ops to engine, verify state consistency
        // Key invariant: after any sequence, engine.state() matches
        // what replaying only the committed (non-undone) dispatches would produce
    }
}
```

**When to use:** For the undo correctness property test. The `Op` enum lets proptest generate arbitrary sequences of dispatch/undo/redo.

### Anti-Patterns to Avoid

- **Game logic in behavior state:** All behavior-local state (whose turn, game-over flag) goes in `E::State`, not in the behavior struct. Behaviors are zero-size structs.
- **Direct state mutation in behaviors:** Behaviors return `BehaviorResult` — they never call `state.board[r][c] = ...` directly. The `Apply` impl does that.
- **`use super::*` in example files:** Examples import from the crate root via `use herdingcats::*`, not from internal modules.
- **Panicking on `Outcome::Committed` mismatch:** Use `assert!(matches!(...))` or explicit `if let` — don't `.unwrap()` on Outcome variants; `Result::unwrap()` is fine on the outer `Result`.
- **Testing EngineError variants that the engine never actually produces in MVP:** `BehaviorPanic` and `CorruptHistory` are documented but the MVP engine never generates them from normal use. Invariant 15 just needs to verify the types are distinct, not that the engine emits them.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arbitrary op sequence generation | Manual random seed + loop | `proptest!` + `Op` enum strategy | Shrinking, reproducible failures, seed reporting |
| Output formatting | Custom display trait impls | `format!` / `println!` with `{:?}` Debug | Examples are terminal demos; Debug is sufficient |
| Behavior ordering verification | Manual sort + compare | Rely on existing test `engine_new_sorts_behaviors_by_order_key_then_name` | Already passing; invariant 3 test just re-asserts the contract, doesn't re-test mechanism |

**Key insight:** The engine is already built and tested. Phase 4 is consumption, not construction. Don't add new engine abstractions.

---

## Common Pitfalls

### Pitfall 1: EngineSpec::State Requires Default

**What goes wrong:** `type State = TicTacToeState` fails to compile if `TicTacToeState` doesn't implement `Default`.
**Why it happens:** `EngineSpec::State` has the bound `Clone + std::fmt::Debug + Default`.
**How to avoid:** Derive or impl `Default` on all game state types (`TicTacToeState`, `BackgammonState`).
**Warning signs:** `the trait bound TicTacToeState: Default is not satisfied`

### Pitfall 2: Frame<E> Manual Clone Bounds

**What goes wrong:** Trying to clone a `Frame<TicTacToeSpec>` without ensuring all associated types are `Clone`.
**Why it happens:** `Frame<E>` has a manual `Clone` impl with where-clause `E::Input: Clone, E::Diff: Clone, E::Trace: Clone` — not a derive.
**How to avoid:** Ensure all example `Input`, `Diff`, `Trace` types derive `Clone`. `String` is already `Clone`.
**Warning signs:** `Clone is not implemented for Frame<TicTacToeSpec>`

### Pitfall 3: Exhaustive Match on Outcome Without Wildcard

**What goes wrong:** Matching on `Outcome<Frame<TicTacToeSpec>, String>` and missing variants causes compile error.
**Why it happens:** `Outcome` is NOT `#[non_exhaustive]` — all 7 variants are the stable contract and must be matched.
**How to avoid:** Either match all 7 variants, or use `_` arm if some variants are intentionally unhandled in demo output.

### Pitfall 4: Stop() Maps to Aborted, Not Disallowed/InvalidInput

**What goes wrong:** A behavior returning `BehaviorResult::Stop(info)` results in `Outcome::Aborted(info)`, not `Outcome::Disallowed(info)`.
**Why it happens:** `engine.rs` dispatch loop maps `Stop` → `Aborted` unconditionally. To get `Disallowed` or `InvalidInput`, the behavior must... also return `Stop`. The distinction between `Disallowed` and `InvalidInput` comes from the caller's semantic intent — the engine doesn't distinguish them. The CONTEXT.md 4-behavior design for tictactoe uses `Stop` to return these variants.
**How to avoid:** To produce `Disallowed` and `InvalidInput` as distinct `Outcome` variants, the behaviors must return distinct payload strings — but critically, the current engine maps all `Stop` → `Aborted`.

**CRITICAL FINDING:** Reviewing `engine.rs` dispatch loop: `BehaviorResult::Stop(info) => return Ok(Outcome::Aborted(info))`. There is NO mechanism to produce `Outcome::Disallowed` or `Outcome::InvalidInput` from dispatch. These variants exist in the `Outcome` enum but the engine's dispatch only produces `Aborted`, `NoChange`, or `Committed`. Undo/redo produce `Disallowed(HistoryDisallowed::...)`.

This means EXAM-01's "exercise all 7 Outcome variants" constraint requires:
- `Committed` — from successful dispatch
- `NoChange` — from dispatch with no diffs
- `Aborted` — from `BehaviorResult::Stop(...)` in dispatch
- `Undone` — from `engine.undo()`
- `Redone` — from `engine.redo()`
- `Disallowed` — from `engine.undo()` when stack empty OR `engine.redo()` when stack empty
- `InvalidInput` — **this variant cannot be produced by the current engine dispatch loop**

`InvalidInput` is a named variant in `Outcome` but the MVP engine dispatch never produces it — `Stop` always maps to `Aborted`. The planner must decide: either (a) note that `InvalidInput` is demonstrated by showing it as a named concept in the example with a comment explaining it would come from a future engine extension, or (b) interpret "exercise all 7 variants" to mean the example code handles all 7 arms in a match statement, with `InvalidInput` being unreachable in current MVP but structurally covered. Option (b) is cleaner — an exhaustive match arm for `InvalidInput` can appear in the output handler even if the current dispatch never generates it.

**Warning signs:** Trying to return `Outcome::InvalidInput` from dispatch — impossible without engine changes.

### Pitfall 5: proptest in #[cfg(test)] mod — Need `use proptest::prelude::*`

**What goes wrong:** `prop_assert!`, `any::<u8>()`, `prop_oneof!`, `Just` not found.
**Why it happens:** proptest's API lives under `proptest::prelude`.
**How to avoid:** Add `use proptest::prelude::*;` inside the `#[cfg(test)]` module.

### Pitfall 6: Backgammon Placeholder Comment References Non-Existent Variant

**What goes wrong:** `examples/backgammon.rs` placeholder says `Reversibility::Permanent` which does not exist — the variant is `Reversibility::Irreversible`.
**How to avoid:** Use `Reversibility::Irreversible` throughout.

### Pitfall 7: Invariants Already Covered — Avoid Redundant Test Logic

**What goes wrong:** Writing a long test for invariant 3 (deterministic ordering) that duplicates `engine_new_sorts_behaviors_by_order_key_then_name`.
**How to avoid:** The invariant tests should be thin assertions that call the same helpers and assert the invariant's specific guarantee — not re-test the mechanism. Reference existing tests in comments.

---

## Code Examples

Verified patterns from official sources and existing codebase:

### TicTacToe EngineSpec skeleton

```rust
// Source: src/spec.rs + src/engine.rs TestSpec pattern
use herdingcats::{Apply, Behavior, BehaviorResult, Engine, EngineSpec, Outcome, Reversibility};

struct TicTacToeSpec;

#[derive(Debug, Clone, PartialEq, Default)]
enum Player { #[default] X, O }

#[derive(Debug, Clone, Default)]
struct TicTacToeState {
    board: [[Option<Player>; 3]; 3],  // None = empty
    current_player: Player,
    winner: Option<Player>,
    game_over: bool,
}

#[derive(Debug, Clone)]
enum TicTacToeInput {
    Place { row: usize, col: usize },
}

#[derive(Debug, Clone)]
enum TicTacToeDiff {
    PlaceMarker { row: usize, col: usize, player: Player },
    SwitchPlayer,
    SetWinner(Player),
    SetGameOver,
}

impl Apply<TicTacToeSpec> for TicTacToeDiff { ... }

impl EngineSpec for TicTacToeSpec {
    type State = TicTacToeState;
    type Input = TicTacToeInput;
    type Diff = TicTacToeDiff;
    type Trace = String;
    type NonCommittedInfo = String;
    type OrderKey = u32;
}
```

### Outcome coverage for all 7 variants in tictactoe

```
Committed  — successful place move
Undone     — engine.undo() after a successful place
Redone     — engine.redo() after the undo
NoChange   — dispatch Place when game_over AND CheckWin behavior returns Continue([]) for all
Aborted    — BehaviorResult::Stop("...") from ValidateCell (out of bounds) OR ValidateTurn (wrong turn)
Disallowed — engine.undo() on empty stack (returns Disallowed(NothingToUndo))
InvalidInput — exhaustive match arm only; current engine dispatch never produces this variant
```

Note: To produce both `Aborted` AND have the example comment/print-statement use "InvalidInput" semantics, the `ValidateCell` behavior can return `Stop("InvalidInput: out of bounds")` (the payload is just a String) and the print can label it `[InvalidInput]`. The `Outcome` variant will be `Aborted` at the type level but the annotated output can display the semantic distinction.

Alternatively: The CONTEXT.md says to exercise "Disallowed if wrong player" and "InvalidInput if out of bounds" — this suggests these were intended as distinct Outcome variants. The planner must reconcile this with the actual engine dispatch which only produces `Aborted` from `Stop`. The cleanest resolution: use `Aborted` for both cases (they are semantically different payloads) and note in comments that `InvalidInput` and `Disallowed` are available as outcomes from undo/redo operations.

### proptest! basic structure

```rust
// Source: proptest docs + existing dev-dep in Cargo.toml
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_dispatch_is_deterministic(
            inputs in prop::collection::vec(any::<u8>(), 0..10)
        ) {
            let mut engine1 = Engine::<TestSpec>::new(
                vec![],
                vec![Box::new(EchoBehavior { key: 0, behavior_name: "echo" })],
            );
            let mut engine2 = Engine::<TestSpec>::new(
                vec![],
                vec![Box::new(EchoBehavior { key: 0, behavior_name: "echo" })],
            );
            for input in &inputs {
                let o1 = engine1.dispatch(*input, Reversibility::Reversible).unwrap();
                let o2 = engine2.dispatch(*input, Reversibility::Reversible).unwrap();
                prop_assert_eq!(engine1.state(), engine2.state(),
                    "engines diverged on input {:?}", input);
                // Check outcome type matches (Frame equality requires PartialEq on Frame)
                let _ = (o1, o2);
            }
        }
    }
}
```

### Op enum for undo/redo property test

```rust
// Source: CONTEXT.md discretion note + proptest docs
#[derive(Debug, Clone)]
enum Op { Dispatch(u8), Undo, Redo }

fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        any::<u8>().prop_map(Op::Dispatch),
        Just(Op::Undo),
        Just(Op::Redo),
    ]
}

proptest! {
    #[test]
    fn prop_undo_restores_exact_state(
        ops in prop::collection::vec(arb_op(), 0..20)
    ) {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![Box::new(EchoBehavior { key: 0, behavior_name: "echo" })],
        );
        // Apply all ops; engine must never panic
        for op in ops {
            match op {
                Op::Dispatch(b) => { let _ = engine.dispatch(b, Reversibility::Reversible); }
                Op::Undo        => { let _ = engine.undo(); }
                Op::Redo        => { let _ = engine.redo(); }
            }
        }
        // The state after any op sequence is always well-defined (no corruption)
        // Verify structural consistency: undo_depth and state cohere
        // (exact consistency check: undo to bottom, state should be initial vec![])
    }
}
```

### Invariant test naming pattern

```rust
// Source: CONTEXT.md locked decision
#[test]
fn invariant_01_never_advances_without_input() {
    // Engine state does not change without a dispatch/undo/redo call.
    let engine = Engine::<TestSpec>::new(vec![1u8], vec![]);
    let before = engine.state().clone();
    // No operation called — state must be identical
    assert_eq!(engine.state(), &before);
}

#[test]
fn invariant_02_dispatch_is_atomic() {
    // If Stop fires mid-dispatch, no prior diffs in that dispatch are committed.
    // Covered by existing `stop_halts_dispatch` test — assert here that state == initial.
    // ...
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `#[derive(Clone)]` on Frame<E> | Manual `Clone` impl with associated type bounds | Phase 3 | Allows unit struct EngineSpecs that don't satisfy `E: Clone` |
| v0.4.0: `Reversibility::Permanent` | `Reversibility::Irreversible` | v0.5.0 Phase 2 | Backgammon placeholder has wrong name — must use Irreversible |
| Separate test files (`tests/`) | `#[cfg(test)]` modules inline | Established Phase 1-3 | All Phase 4 tests follow the inline pattern |

**Deprecated/outdated:**
- `Reversibility::Permanent`: The backgammon.rs placeholder comment references this — it does not exist in v0.5.0. Use `Reversibility::Irreversible`.

---

## Open Questions

1. **InvalidInput vs Aborted distinction in dispatch**
   - What we know: `BehaviorResult::Stop(info)` always maps to `Outcome::Aborted(info)` in the current engine. The CONTEXT.md references `InvalidInput(N)` as a distinct outcome from `ValidateCell`.
   - What's unclear: Whether the user intended `InvalidInput` to be produced by dispatch (requires engine changes) or whether the examples just need to match on `InvalidInput` arm in output handlers.
   - Recommendation: The planner should generate invariant-15 explicitly confirming `EngineError` vs `Outcome` distinction. For EXAM-01, produce `Aborted` for both "out of bounds" and "wrong player" cases, and label the output differently in `println!`. Cover `InvalidInput` with an exhaustive match arm in the output formatter that prints `[InvalidInput]` even though it's currently unreachable from dispatch — this satisfies "exercises all 7 Outcome variants" at the type-system level (the match arm exists and compiles). Alternatively the user could interpret "exercise" to mean the variant appears in the output. The planner should note this ambiguity and pick the safer approach (exhaustive match + comment).

2. **Frame PartialEq in determinism proptest**
   - What we know: `Frame<E>` has a manual `PartialEq` impl requiring `E::Input: PartialEq, E::Diff: PartialEq, E::Trace: PartialEq`. The `TestSpec` in `engine.rs` uses `u8` / `String` — both satisfy these bounds.
   - What's unclear: Whether the proptest determinism test should use `prop_assert_eq!` on the full `Outcome` or just on `engine.state()`.
   - Recommendation: Compare `engine.state()` (simpler, no Frame equality needed). Only compare Frame equality if both outcomes are `Committed` — guard with `if let`.

3. **Invariants 6, 7, 8, 9 — mostly compile-time enforced**
   - What we know: Invariants 6 (behaviors don't mutate state directly), 7 (engine applies diffs centrally), 8 (diff must append trace), 9 (trace in execution order) are partially enforced structurally by the type system.
   - What's unclear: How "thin" the invariant tests should be for structurally-enforced properties.
   - Recommendation: For compile-time invariants, the test is a `// structural: enforced by &E::State borrow` comment plus a single behavioral check that confirms the observable outcome (e.g., traces length == diffs length for invariant 8).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` + `proptest 1.10` |
| Config file | none (inline `#[cfg(test)]` modules) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |
| Examples run command | `cargo run --example tictactoe && cargo run --example backgammon` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXAM-01 | tictactoe compiles and runs, all 7 Outcome arms exercised | smoke (manual run) | `cargo run --example tictactoe` | ✅ placeholder exists |
| EXAM-02 | backgammon compiles and runs, irreversibility demo | smoke (manual run) | `cargo run --example backgammon` | ✅ placeholder exists |
| TEST-01 | 15 invariant unit tests pass | unit | `cargo test invariant_` | ❌ Wave 0: add in engine.rs |
| TEST-02 | 2 proptest suites pass | property | `cargo test prop_` | ❌ Wave 0: add in engine.rs |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo run --example tictactoe && cargo run --example backgammon`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/engine.rs` — append 15 `invariant_XX_*` test functions to existing `#[cfg(test)]` module
- [ ] `src/engine.rs` — append 2 `proptest!` suites: `prop_dispatch_is_deterministic`, `prop_undo_restores_exact_state`
- [ ] `examples/tictactoe.rs` — replace placeholder `fn main() {}` with full implementation
- [ ] `examples/backgammon.rs` — replace placeholder `fn main() {}` with full implementation

*(No new test infrastructure needed — cargo test already discovers `#[test]` and `proptest!` in `#[cfg(test)]` blocks)*

---

## Sources

### Primary (HIGH confidence)
- `src/engine.rs` (read in full) — Engine struct, dispatch/undo/redo implementation, existing 48 tests, TestSpec + helper behavior pattern
- `src/outcome.rs` (read in full) — All 7 Outcome variants, Frame<E> fields (input, diffs, traces, reversibility), EngineError, HistoryDisallowed
- `src/spec.rs` (read in full) — EngineSpec associated type bounds (State: Clone+Debug+Default, Input: Clone+Debug, Diff: Clone+Debug+Apply<Self>, etc.)
- `src/behavior.rs` (read in full) — Behavior<E> trait: name(), order_key(), evaluate()
- `src/apply.rs` (read in full) — Apply<E> trait: apply(&self, &mut E::State) -> Vec<E::Trace>
- `src/reversibility.rs` (read in full) — Reversibility::Reversible, Reversibility::Irreversible
- `src/lib.rs` (read in full) — Confirmed flat public API: Engine, EngineSpec, Behavior, BehaviorResult, Apply, Reversibility, Outcome, Frame, HistoryDisallowed, EngineError
- `Cargo.toml` (read in full) — `proptest = "1.10"` confirmed in dev-dependencies; edition 2024
- `.planning/phases/04-examples-and-tests/04-CONTEXT.md` (read in full) — All locked decisions
- `ARCHITECTURE.md` (read in full) — All 15 core invariants verbatim
- `examples/tictactoe.rs`, `examples/backgammon.rs` (read) — Confirmed placeholders; backgammon has wrong variant name in comment

### Secondary (MEDIUM confidence)
- proptest docs.rs (WebFetch on 1.6.0 docs, syntax consistent with 1.x) — `proptest!` macro syntax, `any::<T>()`, `prop::collection::vec`, `prop_oneof!`, `Just`, `prop_assert_eq!`

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — proptest already in Cargo.toml; no new dependencies; full API read from source
- Architecture: HIGH — entire public API read from source; existing test patterns directly observable
- Pitfalls: HIGH — `InvalidInput` pitfall discovered by reading actual dispatch loop; `Reversibility::Permanent` confirmed by reading backgammon.rs placeholder

**Research date:** 2026-03-13
**Valid until:** Stable (no external dependencies changing; entire codebase is self-contained)
