# Phase 3: Backgammon Example and Integration Properties - Research

**Researched:** 2026-03-08
**Domain:** Rust examples, proptest property strategies, backgammon board representation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Phase boundary:**
- Create `examples/backgammon.rs` — a runnable backgammon example with board state, move types, and a scripted game sequence demonstrating per-die undo. Add two proptest property tests (BACK-05: board conservation, BACK-06: per-die undo) inline in the same file. No changes to library source code; everything lives in `examples/backgammon.rs`.

**Example narrative:**
- `main()` demonstrates: 1 dice roll + 2 moves + 1 undo (short, focused)
- Includes per-move undo demonstration — not just play-forward. This is the architectural point of BACK-03 (separate RollDice/Move CommitFrames): you can undo a move but not the roll
- Print board state after every action (roll, each move, each undo)
- Labeled output: `println!` annotations at each step ("Rolling dice: 3, 5", "Moving checker from 8 to 5", "Undoing last move") — example is self-explanatory to a first reader

**Dice + reversibility design:**
- `RollDice` is non-undoable: `tx.irreversible = false` on the RollDice transaction — it applies `state.dice = [d1, d2]` but never pushes a CommitFrame. `engine.undo()` cannot undo a dice roll
- `Move` is undoable: `tx.irreversible = true` (default) on Move transactions — each move pushes its own CommitFrame, enabling per-die undo
- `BgState` stores: `dice: [u8; 2]` and `dice_used: [bool; 2]`
- `MoveOp` stores `die_index: usize` — `undo()` restores `state.dice_used[die_index] = false`

**BACK-05 property test approach:**
- `prop_compose!` generates `Vec<BackgammonOp>` directly (bypasses event/rule layer)
- Conservation check: `sum(board[0..=25].abs()) + white_home + black_home == 30`
- Both BACK-05 and BACK-06 tests live inline in `examples/backgammon.rs` as `#[cfg(test)] mod props { ... }`

### Claude's Discretion

- Exact `BackgammonOp` variant names and field layout (beyond the locked `MoveOp { from, to, captured, die_index }` shape)
- Board display format (how `BgState` is printed — compact text representation of points, bar, home)
- Which specific move the example script demonstrates — just needs to be a valid backgammon move
- How `RollDiceOp.apply()` and `RollDiceOp.undo()` are stubbed — `undo()` can `unreachable!()` or be a no-op
- Priority type for `BackgammonPriority` — simple enum like tictactoe's `GamePriority`

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BACK-01 | `examples/backgammon.rs` is a runnable example (`cargo run --example backgammon` succeeds) demonstrating a short game sequence | Example file structure from tictactoe.rs; Cargo auto-discovers files in examples/ |
| BACK-02 | Board representation: `[i8; 26]` where indices 0–23 are points (positive = White, negative = Black, magnitude = count), index 24 = White bar, 25 = Black bar, plus two `u8` bear-off counters | Board layout section below; verified against standard backgammon conventions |
| BACK-03 | `RollDice` event produces its own CommitFrame (committed separately from moves), enabling per-die undo | Engine source confirms: `tx.irreversible = false` skips CommitFrame; `irreversible = true` (default) pushes one. Dice roll uses `irreversible = false` — no CommitFrame pushed |
| BACK-04 | `Move` event covers: place checker on empty point, hit opponent blot (sends to bar), re-enter from bar, bear off to home | MoveOp variants section below with apply/undo logic for each case |
| BACK-05 | Property test (proptest): board conservation — total checker count across all points + bars + home is invariant | op-level strategy pattern from Phase 2; `prop_compose!` over valid move ops |
| BACK-06 | Property test (proptest): per-die undo — after dispatching a `Move` event, `engine.undo()` fully restores both `state` and `replay_hash` | Phase 2 PROP-01 pattern directly applied; assert both `engine.read()` and `engine.replay_hash()` |
</phase_requirements>

---

## Summary

Phase 3 is entirely self-contained in `examples/backgammon.rs`. It has zero changes to `src/`. The phase has three concerns: (1) a `BgState` board type implementing the `[i8; 26]` representation, (2) `BackgammonOp` and `BackgammonEvent` types that wire into `Engine`, and (3) two proptest property tests using op-level generation strategies.

The critical architectural demonstration is the `tx.irreversible` flag contrast between `RollDice` (non-undoable, `irreversible = false`) and `Move` (undoable, `irreversible = true`). From reading `engine.rs`, the condition `if tx.irreversible && !tx.cancelled` controls whether a `CommitFrame` is pushed. Setting `irreversible = false` on a dice roll transaction means it is applied to state but never pushed onto the undo stack — `engine.undo()` will skip it and go directly to the last Move's CommitFrame.

The proptest strategy design for BACK-05 must operate at the op level (not event level) to avoid needing a full rule system to generate valid inputs. The conservation invariant `sum(abs(board[0..=26])) + white_home + black_home == 30` needs careful handling of the bar indices (24 and 25): bar values are stored with the same i8 sign convention as points, so `abs()` applies uniformly across all 26 entries.

**Primary recommendation:** Implement in a single file following the exact tictactoe.rs structure, with the `irreversible` flag as the organizing architectural idea. The proptest `mod props` section should parallel Phase 2's inline-in-source approach, using `prop_compose!` to build op sequences that preserve board conservation.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| herdingcats | 0.2.0 (local) | Engine, Operation, Rule, Transaction, RuleLifetime | This crate — the entire point of the phase |
| proptest | 1.10 | Property-based test generation and shrinking | Already in Cargo.toml dev-dependencies; Phase 2 established the pattern |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::fmt | stdlib | Display trait for BgState board output | Board printing in main() and undo demo |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| inline `#[cfg(test)] mod props` | Separate test file | Separate file would break the Phase 1/2 consistency pattern; inline is the locked pattern |
| op-level generation (BACK-05) | Event-level generation | Event-level would require a full Rule system and valid game state management; op-level is simpler and focuses on the conservation invariant |

**Installation:** No new dependencies — `proptest = "1.10"` already present in `[dev-dependencies]`.

---

## Architecture Patterns

### Recommended File Structure

```
examples/
└── backgammon.rs     # All of phase 3 lives here
    ├── BackgammonPriority (enum)
    ├── BgState (struct)
    ├── BackgammonOp (enum) — implements Operation<BgState>
    ├── BackgammonEvent (enum)
    ├── Rules (structs implementing Rule<BgState, BackgammonOp, BackgammonEvent, BackgammonPriority>)
    ├── Display for BgState
    ├── fn main()
    └── #[cfg(test)] mod props { ... }  — BACK-05 and BACK-06
```

### Pattern 1: Non-Undoable RollDice Transaction

**What:** The RollDice dispatch sets `tx.irreversible = false` before or during rules, so the engine applies the state mutation but never pushes a CommitFrame. `engine.undo()` cannot reach this transaction.

**When to use:** Any event that the game rules say cannot be reversed (dice rolls in backgammon, coin flips, random draws).

**Engine mechanism (from engine.rs):**
```rust
// Source: src/engine.rs line 374
if tx.irreversible && !tx.cancelled {
    // ... update replay_hash ...
    self.undo_stack.push(CommitFrame { ... });
    self.redo_stack.clear();
}
```
When `tx.irreversible = false`, the block is skipped entirely. State is still mutated (ops were applied), but no CommitFrame is pushed.

**Example:**
```rust
// Source: pattern established in CONTEXT.md
struct RollDiceRule;

impl Rule<BgState, BackgammonOp, BackgammonEvent, BackgammonPriority> for RollDiceRule {
    fn id(&self) -> &'static str { "roll_dice" }
    fn priority(&self) -> BackgammonPriority { BackgammonPriority::Default }

    fn before(
        &self,
        _state: &BgState,
        event: &mut BackgammonEvent,
        tx: &mut Transaction<BackgammonOp>,
    ) {
        if let BackgammonEvent::RollDice { d1, d2 } = event {
            tx.ops.push(BackgammonOp::SetDice { d1: *d1, d2: *d2 });
            tx.irreversible = false;  // Dice roll is not undoable
        }
    }
}
```

### Pattern 2: Undoable Move Transaction (Per-Die CommitFrame)

**What:** Each Move event dispatches with the default `tx.irreversible = true`, causing a separate CommitFrame to be pushed for every die's move. `engine.undo()` reverses one die's move at a time, restoring both board position and `dice_used[die_index]`.

**When to use:** Any game action that the player should be able to take back (within the current turn).

**Example:**
```rust
// Source: pattern established in CONTEXT.md
struct MoveRule;

impl Rule<BgState, BackgammonOp, BackgammonEvent, BackgammonPriority> for MoveRule {
    fn id(&self) -> &'static str { "move" }
    fn priority(&self) -> BackgammonPriority { BackgammonPriority::Default }

    fn before(
        &self,
        state: &BgState,
        event: &mut BackgammonEvent,
        tx: &mut Transaction<BackgammonOp>,
    ) {
        if let BackgammonEvent::Move { from, to, die_index } = event {
            // tx.irreversible = true (default) — CommitFrame will be pushed
            let captured = state.board[*to] * state.turn_sign() == -1;
            tx.ops.push(BackgammonOp::MoveOp {
                from: *from,
                to: *to,
                captured,
                die_index: *die_index,
            });
        }
    }
}
```

### Pattern 3: Op-Level Proptest Strategy (BACK-05)

**What:** Generate `Vec<BackgammonOp>` directly using `prop_compose!`, bypassing the event/rule layer. Apply ops directly to a BgState and assert conservation after each sequence.

**When to use:** When the property being tested (conservation) is a state-level invariant that does not depend on the rule system being correct.

**Example:**
```rust
// Source: Phase 2 established pattern in engine.rs mod props
prop_compose! {
    fn bg_move_op_strategy(state: BgState)
        (from in 0usize..24, die_index in 0usize..2)
    -> BackgammonOp {
        // generate valid move ops based on current state
        // ...
    }
}
```

Note: The CONTEXT.md locks "op-level generation" for BACK-05 specifically. Because ops are generated directly, the strategy does not need to produce only game-legal positions — it only needs to produce ops where apply+undo roundtrips are conservative.

### Pattern 4: Assert Both State and Hash (BACK-06)

**What:** After dispatch and undo, assert both `engine.read()` (state equality) AND `engine.replay_hash()` (hash equality). This is the established standard from PROP-01.

**Example:**
```rust
// Source: engine.rs prop_01_undo_roundtrip — established in Phase 2
prop_assert_eq!(engine.read(), state_before);
prop_assert_eq!(engine.replay_hash(), hash_before);
```

### Anti-Patterns to Avoid

- **Only asserting state on undo:** `replay_hash` must also be asserted — state match alone is insufficient to detect hash corruption. This is locked in from Phase 2.
- **Sharing one CommitFrame for roll+move:** Each die's move must be a separate dispatch with its own CommitFrame. Do not batch RollDice and Move ops into a single transaction.
- **Making RollDice reversible:** `tx.irreversible` must be `false` for RollDice. The whole point of BACK-03 is that dice rolls are not on the undo stack.
- **Calling undo on RollDice:** The `RollDiceOp.undo()` is unreachable in practice. Use `unreachable!()` to make the intent explicit. Do not implement a real inverse.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Property strategy shrinking | Custom shrinker | proptest's built-in `Vec` shrinking | proptest automatically shrinks `Vec<Op>` sequences to minimal failing cases |
| Undo stack management | Custom frame stack | `engine.undo()` | CommitFrame already captures everything; manually managing frames is redundant |
| Hash correctness | Custom fingerprint | `engine.replay_hash()` | Engine already computes FNV running hash over irreversible+deterministic ops |
| Test isolation | Cloning engine state manually | `engine.read()` snapshot + `engine.replay_hash()` snapshot | These are the public API for taking before/after snapshots |

**Key insight:** The proptest tests for BACK-05 and BACK-06 should NOT test the rule system — they test state-level conservation and undo-correctness at the operation level. This avoids coupling the property test to game rule validity and keeps strategies simple.

---

## Common Pitfalls

### Pitfall 1: `irreversible` Naming Confusion

**What goes wrong:** `tx.irreversible = false` sounds like "this cannot be reversed" but actually means "do not push a CommitFrame" (i.e., the engine will not reverse it). The naming is from the engine's perspective: `irreversible = true` means the commit IS reversible (can be undone); `irreversible = false` means it is NOT reversible (no undo available).

**Why it happens:** The boolean reads backwards from plain English — "irreversible" in English means "cannot undo," but in the Transaction API it means "push to undo stack" when `true`.

**How to avoid:** Read the field as "put this on the undo stack" when `true`. From `transaction.rs` source: "Whether this commit is recorded on the undo stack. Defaults to `true`."

**Warning signs:** If `engine.undo()` after a dice roll reverses the dice values, `tx.irreversible` was incorrectly left at `true` for the RollDice transaction.

### Pitfall 2: Board Conservation with Bar Indices

**What goes wrong:** The bar indices (24 = White bar, 25 = Black bar) use the same sign convention as point indices. White bar count is positive, Black bar count is negative at index 25. Taking `abs()` on all 26 board entries works correctly, but only if bar indices are not treated differently.

**Why it happens:** The `[i8; 26]` representation packs bars into the same array as points. The conservation sum `sum(board[0..=25].abs()) + white_home + black_home` works only when `abs()` is applied to the full slice including bars.

**How to avoid:** Use `board.iter().map(|x| x.unsigned_abs() as u32).sum::<u32>()` across the full `[i8; 26]` slice. Do not add bar indices separately with different logic.

**Warning signs:** Conservation sum off by 1 or 2 when a checker is on the bar.

### Pitfall 3: MoveOp Undo Missing `dice_used` Restoration

**What goes wrong:** `MoveOp.undo()` reverses the board change but forgets to set `state.dice_used[die_index] = false`, leaving the die marked as consumed after undo.

**Why it happens:** `dice_used` is a companion field to the board. It is easy to focus on board index manipulation and overlook the die-state restoration.

**How to avoid:** The `MoveOp` struct stores `die_index: usize`. The `undo()` implementation must unconditionally set `state.dice_used[die_index] = false` in addition to reversing the board change.

**Warning signs:** BACK-06 proptest failing because `engine.read().dice_used` does not match pre-move snapshot even though board positions match.

### Pitfall 4: Proptest in `examples/` Requires `--test` Flag

**What goes wrong:** Running `cargo run --example backgammon` does not execute `#[cfg(test)]` blocks. The property tests in `mod props` are only reachable via `cargo test --example backgammon`.

**Why it happens:** Examples are binary targets; `#[cfg(test)]` is only compiled when building test binaries.

**How to avoid:** Run verification as `cargo test --example backgammon` to exercise BACK-05 and BACK-06. The `cargo run --example backgammon` command only exercises BACK-01 (main() succeeds).

**Warning signs:** Tests "pass" in CI because the command was `cargo run` not `cargo test --example`.

### Pitfall 5: Bear-Off Op Overshoot

**What goes wrong:** When bearing off, the `to` index is typically encoded as a sentinel (e.g., `26` or `usize::MAX`) that is outside the `[i8; 26]` board. Writing to this index panics or corrupts memory.

**Why it happens:** Bear-off moves a checker off the board entirely — there is no `to` point in the `[i8; 26]` array.

**How to avoid:** `BearOffOp` must handle the `to` position as a special case. It decrements `board[from]` by the appropriate sign and increments the `white_home` or `black_home` counter (separate `u8` fields), never writing to `board[26]`.

**Warning signs:** Index out of bounds panic at runtime during bear-off move.

---

## Code Examples

Verified patterns from project source code:

### BgState Board Layout

```rust
// Board encoding (locked in BACK-02):
// board[0..=23]: points (positive = White checkers, negative = Black checkers, magnitude = count)
// board[24]: White bar (positive, White checkers on bar)
// board[25]: Black bar (negative, Black checkers on bar)
// white_home: u8 — White checkers borne off
// black_home: u8 — Black checkers borne off
//
// Standard starting position (White moves 1->24, Black moves 24->1):
// - White: 2 on point 24, 5 on point 13, 3 on point 8, 5 on point 6
// - Black: mirror image (negated values at mirrored indices)

#[derive(Clone, Debug, PartialEq)]
struct BgState {
    board: [i8; 26],       // indices 0-23: points, 24: White bar, 25: Black bar
    white_home: u8,        // White checkers borne off
    black_home: u8,        // Black checkers borne off
    dice: [u8; 2],         // current dice roll
    dice_used: [bool; 2],  // which dice have been consumed
}
```

### Conservation Invariant (BACK-05)

```rust
// Total checkers = 30 (15 per side)
// Conservation: sum of absolute values of all board entries + home counters == 30
fn checker_count(state: &BgState) -> u32 {
    let board_sum: u32 = state.board
        .iter()
        .map(|x| x.unsigned_abs() as u32)
        .sum();
    board_sum + state.white_home as u32 + state.black_home as u32
}
```

### Non-Undoable Dispatch (RollDice)

```rust
// Source: Transaction API (transaction.rs) + engine.rs CommitFrame push condition
let mut tx = Transaction::new();
tx.irreversible = false;  // No CommitFrame pushed — dice roll cannot be undone
engine.dispatch(BackgammonEvent::RollDice { d1: 3, d2: 5 }, tx);
// engine.undo() will NOT undo this — undo stack unchanged
```

### Undoable Dispatch (Move) + Undo Verification

```rust
// Source: PROP-01 pattern from engine.rs mod props
let state_before = engine.read();
let hash_before = engine.replay_hash();

// Move uses default tx (irreversible = true) — CommitFrame is pushed
engine.dispatch(BackgammonEvent::Move { from: 8, to: 5, die_index: 0 }, Transaction::new());

engine.undo();  // Reverses the Move CommitFrame

assert_eq!(engine.read(), state_before);      // state fully restored
assert_eq!(engine.replay_hash(), hash_before); // hash fully restored
```

### Proptest Module Structure (inline in examples/backgammon.rs)

```rust
// Source: engine.rs mod props — established Phase 2 pattern
#[cfg(test)]
mod props {
    use super::*;
    use proptest::prelude::*;

    // BACK-05: board conservation
    proptest! {
        #[test]
        fn prop_board_conservation(ops in bg_op_sequence_strategy()) {
            let state = BgState::new();
            let total_before = checker_count(&state);
            // apply ops...
            prop_assert_eq!(checker_count(&state), total_before);
        }
    }

    // BACK-06: per-die undo roundtrip
    proptest! {
        #[test]
        fn prop_per_die_undo(/* ... */) {
            // ... capture state_before + hash_before
            // dispatch Move
            // engine.undo()
            prop_assert_eq!(engine.read(), state_before);
            prop_assert_eq!(engine.replay_hash(), hash_before);
        }
    }
}
```

### MoveOp Variants and their Undo Logic

```rust
// The four Move event sub-cases (BACK-04):

// 1. Place on empty point
//    apply:  board[to] += sign;  dice_used[die_index] = true
//    undo:   board[to] -= sign;  dice_used[die_index] = false

// 2. Hit opponent blot (captured = true)
//    apply:  board[to] = sign;  board[opponent_bar] += -sign;  dice_used[die_index] = true
//    undo:   board[to] = -sign; board[opponent_bar] -= -sign;  dice_used[die_index] = false
//    NOTE:   MoveOp must store `captured: bool` to know which undo branch to take

// 3. Re-enter from bar
//    apply:  board[bar] -= sign;  board[to] += sign;  dice_used[die_index] = true
//    undo:   board[bar] += sign;  board[to] -= sign;  dice_used[die_index] = false

// 4. Bear off to home
//    apply:  board[from] -= sign;  *home_counter += 1;  dice_used[die_index] = true
//    undo:   board[from] += sign;  *home_counter -= 1;  dice_used[die_index] = false
//    NOTE:   No `to` index — uses home counter, not board[to]
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single transaction per turn | Per-die CommitFrames via separate dispatch calls | Phase 3 design decision | Enables undo of individual dice moves without reversing the roll |
| Full state snapshots for undo | Operation-level inverse (op.undo()) with hash snapshots in CommitFrame | Established in Phase 1 | Memory-efficient undo; hash correctness guaranteed |
| Separate test files | Inline `#[cfg(test)] mod props` in source/example files | Phase 1/2 pattern | Consistency; tests co-located with implementation |

**Deprecated/outdated:**
- None — this is a new file. All patterns are current.

---

## Open Questions

1. **BACK-05 op strategy: valid-only vs any op**
   - What we know: CONTEXT.md says "op-level generation" and the conservation invariant is `sum(abs(board)) + homes == 30`
   - What's unclear: Whether the proptest strategy should only generate valid backgammon moves (checkers that actually exist at `from`), or arbitrary board mutations that happen to be conservative
   - Recommendation: Generate only structurally valid ops (checkers present at `from`, to/bar within bounds) to avoid panics from `board[from]` underflow. The strategy can use `Just(...)` ops constructed from a fixed starting position, or generate bounded indices and guard with `prop_assume!`.

2. **Board display format**
   - What we know: CONTEXT.md marks this as Claude's discretion — "compact text representation of points, bar, home"
   - What's unclear: Whether a linear point-by-point display or a two-row "real backgammon board" layout is clearer
   - Recommendation: Linear single-row format showing point indices and values is simpler to implement and sufficient for demonstrating the engine concepts. A "point 8: +2 White" style or compact `[+2, -3, 0, ...]` format both work.

3. **BearOff sentinel encoding**
   - What we know: BACK-02 specifies `[i8; 26]` plus two `u8` home counters. Bear-off is one of the four BACK-04 move types.
   - What's unclear: How `to` is encoded for bear-off in the `MoveOp` struct vs in `BackgammonEvent::Move`
   - Recommendation: Use a dedicated `BackgammonOp::BearOffOp { from, die_index, player }` variant rather than overloading `MoveOp { to }` with a sentinel. This avoids the out-of-bounds pitfall (Pitfall 5 above) and makes the undo logic unambiguous.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | proptest 1.10 (already in Cargo.toml dev-dependencies) |
| Config file | none — default proptest settings |
| Quick run command | `cargo test --example backgammon` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BACK-01 | `cargo run --example backgammon` succeeds | smoke | `cargo run --example backgammon` | Wave 0 |
| BACK-02 | Board `[i8; 26]` + home counters correctly represents all positions | unit (inline) | `cargo test --example backgammon` | Wave 0 |
| BACK-03 | RollDice uses `irreversible = false`, Move uses default `irreversible = true` | unit + prop | `cargo test --example backgammon` | Wave 0 |
| BACK-04 | All four move variants apply and undo correctly | unit (inline) | `cargo test --example backgammon` | Wave 0 |
| BACK-05 | Board conservation invariant across op sequences | property (BACK-05 proptest) | `cargo test --example backgammon prop_board_conservation` | Wave 0 |
| BACK-06 | Per-die undo restores state and replay_hash | property (BACK-06 proptest) | `cargo test --example backgammon prop_per_die_undo` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --example backgammon`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green (`cargo test`) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `examples/backgammon.rs` — the entire phase; does not exist yet (confirmed: `cargo run --example backgammon` returns "no example target named backgammon")
- [ ] No additional test infrastructure needed — proptest is already in dev-dependencies, no new fixtures required

---

## Sources

### Primary (HIGH confidence)

- `src/engine.rs` (read directly) — CommitFrame push condition at line 374 confirms `tx.irreversible` semantics; `undo()` at line 433 confirms ops reversed in order + hash restored from `state_hash_before`
- `src/transaction.rs` (read directly) — `Transaction::new()` confirms `irreversible: true` default; field documentation confirms "Whether this commit is recorded on the undo stack"
- `src/operation.rs` (read directly) — `Operation<S>` trait signature: `apply`, `undo`, `hash_bytes`
- `examples/tictactoe.rs` (read directly) — Reference pattern: Priority enum, State struct, Op enum, Event enum, Rule structs, Display, main()
- `src/engine.rs mod props` (read directly) — Phase 2 proptest patterns: `prop_compose!` strategy, `prop_oneof!`, inline `#[cfg(test)] mod props`, `prop_assert_eq!(engine.read(), ...)` + `prop_assert_eq!(engine.replay_hash(), ...)`
- `Cargo.toml` (read directly) — `proptest = "1.10"` confirmed in dev-dependencies; no new dependencies needed

### Secondary (MEDIUM confidence)

- `.planning/STATE.md` — Records MEDIUM-confidence concern about `[i8; 26]` bearing-off edge cases; research above addresses this with the dedicated `BearOffOp` recommendation
- `.planning/phases/02-engine-property-tests/02-01-SUMMARY.md` — Confirms Phase 2 established patterns: "Both engine.read() and engine.replay_hash() asserted in undo roundtrip tests — state-only assertion is insufficient"
- `.planning/phases/03-backgammon-example-and-integration-properties/03-CONTEXT.md` — All locked decisions sourced from here (HIGH confidence for implementation intent)

### Tertiary (LOW confidence)

- None — all findings are backed by project source code directly.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified from Cargo.toml; all API patterns verified from source
- Architecture: HIGH — engine.rs and tictactoe.rs read directly; no speculation
- Pitfalls: HIGH for 1/3/4/5 (directly derived from source code reading); MEDIUM for pitfall 2 (bar sign convention is an inference from `[i8; 26]` spec, not tested in existing code)

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (30 days — stable internal codebase)
