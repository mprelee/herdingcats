# Phase 4: Examples and Tests - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement two real-game examples that validate the public API under real conditions, and write unit/property tests that enforce all 15 core invariants. Both examples live in `examples/` (already placeholder files). Tests extend existing `#[cfg(test)]` modules in `src/`. No new game engine features — this phase is purely API demonstration and test coverage.

</domain>

<decisions>
## Implementation Decisions

### Tic-tac-toe example structure
- **Run mode:** Scripted demo — `main()` runs a pre-scripted sequence of moves with no user input
- **Output style:** Annotated step-by-step — each operation prints: `[dispatch] Move(0,0) by X => Committed(frame)`; `[undo] => Undone(frame)`, etc.
- **Game rules:** Full rules — 4 behaviors: `ValidateTurn` (Disallowed if wrong player), `ValidateCell` (InvalidInput if out of bounds, Disallowed if occupied), `PlaceMarker`, `CheckWin` (signals game-over with 3 in a row)
- **Outcome coverage:** Script is designed to exercise all 7 Outcome variants: `Committed`, `Undone`, `Redone`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted`
- **Runs via:** `cargo run --example tictactoe`

### Backgammon example structure
- **Scope:** Focused demo — only `RollDice` (Irreversible) + `MovePiece` (Reversible) behaviors. No bearing off, hitting, re-entering, bearing off, pip count, or doubling cube.
- **Dice:** Scripted/seeded — fixed dice values hard-coded in the script. Same output every run.
- **Output style:** Same annotated style as tictactoe — `[dispatch] RollDice(3,5) => Committed(frame)   [IRREVERSIBLE]`
- **Key demo sequence:** dispatch MovePiece, dispatch MovePiece, undo() => Undone, dispatch RollDice (Irreversible — history cleared), undo() => Disallowed(NothingToUndo)
- **Runs via:** `cargo run --example backgammon`

### Test placement
- All new Phase 4 tests extend existing `#[cfg(test)]` modules in `src/engine.rs`
- No new `tests/` directory — consistent with the pattern from Phases 1-3
- `proptest!` macro used inline in the same `#[cfg(test)]` module

### Invariant test structure
- 15 named test functions: `invariant_01_never_advances_without_input`, `invariant_02_dispatch_is_atomic`, ..., `invariant_15_engine_errors_distinct_from_outcomes`
- One test per ARCHITECTURE.md invariant by number — easy to audit against the list
- Tests are self-contained unit tests; they do not depend on game logic from the examples

### Property test coverage (proptest)
- **Suite 1 — Determinism:** `prop_dispatch_is_deterministic` — same input sequence applied to two identically-constructed engines always produces the same sequence of outcomes. Uses `vec(any::<u8>(), 0..10)` strategy.
- **Suite 2 — Undo correctness:** `prop_undo_restores_exact_state` — after an arbitrary sequence of dispatch/undo/redo operations (0..20 operations), the final engine state is consistent with what those operations should produce.
- `proptest = "1.10"` already in `[dev-dependencies]`

### Claude's Discretion
- Exact behavior names and types for the examples (player enum, board struct, etc.)
- Whether `NoChange` and `Aborted` appear in tictactoe via a dedicated board-full check or a "game already over" guard
- Exact proptest strategy type for the undo/redo property test (`Op` enum with Dispatch/Undo/Redo variants is the natural approach)
- Order of invariants covered — can reuse existing test coverage for invariants already well-tested in Phases 1-3

</decisions>

<specifics>
## Specific Ideas

- Both examples should be readable as tutorials — someone can read `tictactoe.rs` top to bottom and understand the entire HerdingCats API
- The annotated output format (`[dispatch] ... => ...`) is the key readability decision — it makes the Outcome type visible in the terminal without needing a debugger
- Backgammon's irreversibility demo is the killer use case: `undo()` returning `Disallowed(NothingToUndo)` after a dice roll is the moment that makes irreversibility click for users

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engine.rs` `#[cfg(test)]` module: 48 existing tests define the `TestSpec` + helper behaviors pattern — Phase 4 invariant tests and property tests follow the same pattern
- `proptest = "1.10"` already in `[dev-dependencies]` — no installation needed
- `examples/tictactoe.rs` and `examples/backgammon.rs` exist as empty placeholders (just `fn main() {}`)
- Full public API at crate root: `Engine`, `EngineSpec`, `Behavior`, `BehaviorResult`, `Outcome`, `Apply`, `Reversibility`, `HistoryDisallowed`, `Frame`, `EngineError`

### Established Patterns
- `EngineSpec` implemented on a unit struct: `struct TicTacToeSpec;` + `impl EngineSpec for TicTacToeSpec { type State = ...; ... }`
- Behaviors as zero-size structs implementing `Behavior<E>` — no state in the behavior itself; all state in `E::State`
- `Apply<E>` on the diff type: `impl Apply<TicTacToeSpec> for TicTacToeMove { fn apply(...) -> Vec<E::Trace> }`
- `#[cfg(test)] mod tests { use super::*; ... }` — all tests in same file

### Integration Points
- ARCHITECTURE.md §"Core Invariants" lists all 15 invariants to test against
- `Cargo.toml` `[[example]]` auto-discovery covers `examples/*.rs` — no config needed
- `src/engine.rs` receives the most additions: 15 invariant tests + 2 proptest suites

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-examples-and-tests*
*Context gathered: 2026-03-14*
