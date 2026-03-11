# Phase 6: Tests and Examples - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Verify the new reversibility model and behavior lifecycle with property-based and unit tests; update both examples to compile and run correctly under the final v1.1 API. No new API features or documentation (Phase 7).

</domain>

<decisions>
## Implementation Decisions

### TEST-01: Existing test audit
- Grep for old names (`Operation`, `Rule`, `Transaction`, `RuleLifetime`) in all test code — if none found, TEST-01 is satisfied
- Document explicitly that prop_03 (deleted in Phase 4 — tested RuleLifetime::Turns/Triggers) is intentionally replaced by TEST-04 (stateful behavior unit test)
- Trust `cargo test` passing as the compilation proof; grep audit is a confirmation step, not a full rewrite

### TEST-02 and TEST-03: Reversibility proptests
- Two separate proptests: `prop_05_irreversible_clears_undo_stack` (TEST-02) and `prop_06_reversible_after_irreversible_undoable` (TEST-03)
- Both live in `engine.rs` `mod props` block, alongside existing prop_01/02/04
- Strategy: `Vec<MixedOp>` (Rev or Irrev variants) — reuse the `MixedOp` fixture already defined in the unit tests section; move or redeclare it in props as needed
- For prop_06: structured 3-part generated sequence — prefix of reversible ops, one irreversible op, suffix of reversible ops; verify suffix is individually undoable and undo halts at barrier

### TEST-04: Stateful behavior unit test
- Parameterized over N in 1..=10 (proptest or loop) — behavior is active for N dispatches, then `is_active()` returns false
- **Verify sleeping behavior**: after deactivation, assert `on_dispatch()` still increments the internal counter (dispatch_count > N) — validates the Phase 5 locked decision that on_dispatch fires on ALL behaviors regardless of is_active()
- **Verify hooks skipped**: after deactivation, add a second behavior with a `before()` hook that cancels actions; confirm actions go through (hook is not called) when is_active()=false — ties TEST-04 to LIFE-01

### TEST-05: Backgammon example update
- `RollDiceOp` gains `fn is_reversible(&self) -> bool { false }` — dice rolls are non-undoable
- `RollDiceOp.undo()` keeps `unreachable!("RollDiceOp cannot be undone")` — safety net; engine never calls it, but explicit panic if invariant is bypassed
- `RollDiceRule` gains an internal `rolls_dispatched: u32` counter incremented in `on_dispatch()` — demonstrates lifecycle hooks in a real game context (no behavioral change to the rule; stays active forever)
- Demo sequence in `main()`: Roll dice → print `[irreversible commit — undo barrier set]` → dispatch checker move → undo the move → dispatch another checker move
- Add short header comment block at top of `main()` listing what the demo demonstrates: `is_reversible()`, undo barrier, `on_dispatch` lifecycle counter
- Add a proptest in backgammon.rs `mod props` asserting that after a dice roll dispatch, the undo stack is empty (integration-level TEST-02 coverage in game context)

### TEST-06: Tictactoe example
- Already uses new names (`Behavior`, `add_behavior`, etc.) — confirm with grep, no changes needed
- `cargo run --example tictactoe` continues to compile and run unchanged

### Claude's Discretion
- Whether `MixedOp` fixture is shared between `mod tests` and `mod props` (e.g., moved to a common inner module or redeclared) — implementation detail
- Exact proptest case count / shrinking config for prop_05 and prop_06
- How the parameterized TEST-04 loop is structured (for loop vs proptest)
- Exact wording of the backgammon demo print statements

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engine.rs` `mod tests` — `MixedOp` (Rev/Irrev variants with `is_reversible()`), `CounterOp`, `MixedNoRule` — fixtures ready for reuse in `mod props`
- `src/engine.rs` `mod props` — existing `op_sequence_strategy()` and `CounterOp` — prop_05/06 can use same pattern
- `examples/backgammon.rs` `mod props` — existing board conservation and move/undo props — new dice-roll irreversibility prop fits naturally here

### Established Patterns
- `mod tests` (unit) + `mod props` (proptest) sibling pattern in `engine.rs` — required
- Section banners (`// ============================================================`) in every file
- All undo property tests assert both `engine.state` (read) AND `engine.replay_hash()` — maintain this
- `where` clause on separate lines from `impl`/`fn` signatures

### Integration Points
- `engine.rs` props module: add prop_05 and prop_06 after existing prop_04
- `examples/backgammon.rs`: update `RollDiceOp` (add `is_reversible`), update `RollDiceRule` (add counter field + `on_dispatch`), update `main()` sequence and add header comment, add prop in `mod props`
- `examples/tictactoe.rs`: no changes expected; confirm with grep

</code_context>

<specifics>
## Specific Ideas

- Backgammon `main()` demo sequence: Roll → `[irreversible commit — undo barrier set]` print → Move → Undo move → Move again. Shows barrier is enforced but reversible moves after it still work normally.
- TEST-04 "sleeping behavior" verification: dispatch_count should be N + (however many dispatches happen after deactivation), confirming on_dispatch continues to fire.
- prop_06 strategy: explicitly generate a "before" sequence, one Irrev action, and an "after" sequence — not random interleaving — so the test can assert exact undo semantics on the after-sequence.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 06-tests-and-examples*
*Context gathered: 2026-03-10*
