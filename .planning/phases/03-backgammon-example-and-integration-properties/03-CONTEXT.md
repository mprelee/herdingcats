# Phase 3: Backgammon Example and Integration Properties - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Create `examples/backgammon.rs` — a runnable backgammon example with board state, move types, and a scripted game sequence demonstrating per-die undo. Add two proptest property tests (BACK-05: board conservation, BACK-06: per-die undo) inline in the same file. No changes to library source code; everything lives in `examples/backgammon.rs`.

</domain>

<decisions>
## Implementation Decisions

### Example narrative
- `main()` demonstrates: 1 dice roll + 2 moves + 1 undo (short, focused)
- Includes per-move undo demonstration — not just play-forward. This is the architectural point of BACK-03 (separate RollDice/Move CommitFrames): you can undo a move but not the roll
- Print board state after every action (roll, each move, each undo)
- Labeled output: `println!` annotations at each step ("Rolling dice: 3, 5", "Moving checker from 8 to 5", "Undoing last move") — example is self-explanatory to a first reader

### Dice + reversibility design
- **RollDice is non-undoable**: `tx.irreversible = false` on the RollDice transaction — it applies `state.dice = [d1, d2]` but never pushes a CommitFrame. `engine.undo()` cannot undo a dice roll (correct backgammon rule: you must play what you rolled)
- **Move is undoable**: `tx.irreversible = true` on Move transactions — each move pushes its own CommitFrame, enabling per-die undo
- `BgState` stores the current dice roll: `dice: [u8; 2]` and `dice_used: [bool; 2]`
- `MoveOp` stores `die_index: usize` — `undo()` restores `state.dice_used[die_index] = false`, making the die available again after undoing a move

### BACK-05 property test approach
- **Op-level generation**: `prop_compose!` generates `Vec<BackgammonOp>` directly (bypasses event/rule layer). Simpler strategies, better shrinking, focused on the conservation invariant
- **Conservation check**: `sum(board[0..=25].abs()) + white_home + black_home == 30` before and after any generated sequence — total checker count is constant at 30
- Both BACK-05 and BACK-06 tests live inline in `examples/backgammon.rs` as `#[cfg(test)] mod props { ... }` — consistent with Phase 1/2 pattern

### Claude's Discretion
- Exact `BackgammonOp` variant names and field layout (beyond the locked `MoveOp { from, to, captured, die_index }` shape)
- Board display format (how `BgState` is printed — compact text representation of points, bar, home)
- Which specific move the example script demonstrates (e.g., which point to move from/to) — just needs to be a valid backgammon move
- How `RollDiceOp.apply()` and `RollDiceOp.undo()` are stubbed (since undo is never called, `undo()` can `unreachable!()` or be a no-op)
- Priority type for `BackgammonPriority` — simple enum like tictactoe's `GamePriority`

</decisions>

<specifics>
## Specific Ideas

- The architectural insight to highlight: `engine.undo()` on a Move restores both `state` (board position + die availability) AND `replay_hash` — consistent with the established "assert both" pattern from Phase 2
- `RollDiceOp.undo()` is never reachable in practice (transaction is non-reversible), but must satisfy the `Operation<BgState>` trait — `unreachable!()` macro is the clean choice
- BACK-03 requirement language says "enabling per-die undo" — this means undoing individual Move transactions back to the post-roll state, NOT undoing the roll itself

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `examples/tictactoe.rs`: reference pattern for example file structure — State + Op + Event + Rule(s) + main()
- Phase 2 proptest patterns: `prop_compose!`, `prop_oneof!`, inline `#[cfg(test)] mod props` — same approach for BACK-05/BACK-06

### Established Patterns
- `tx.irreversible = true` for undoable transactions (established in Phase 2 — must be true for CommitFrame to push)
- `tx.irreversible = false` for non-undoable transactions (dice roll)
- All undo property tests assert both `engine.read()` (state) AND `engine.replay_hash()` — locked in from Phase 2 research

### Integration Points
- `examples/backgammon.rs` is a new file — zero changes to `src/` code
- `examples/tictactoe.rs` must continue to compile and run unchanged (MOD-03 is complete, keep it that way)
- Uses `herdingcats::*` public API: `Engine`, `Operation`, `Rule`, `Transaction`, `RuleLifetime`

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-backgammon-example-and-integration-properties*
*Context gathered: 2026-03-08*
