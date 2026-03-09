# Architecture Research

**Domain:** Rust library crate — module refactor + game example
**Researched:** 2026-03-08
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        src/lib.rs (facade)                       │
│   pub mod hash; pub mod operation; pub mod transaction;          │
│   pub mod rule; pub mod engine;                                  │
│   pub use engine::Engine;  pub use rule::{Rule, RuleLifetime};  │
│   pub use operation::Operation;  pub use transaction::Transaction│
└──────────────────────────────┬──────────────────────────────────┘
                               │ re-exports (pub use)
     ┌─────────────────────────┼──────────────────────────┐
     │             │           │           │              │
┌────▼───┐  ┌──────▼──┐  ┌────▼────┐  ┌──▼────┐  ┌──────▼────┐
│hash.rs │  │operation│  │transact │  │rule.rs│  │engine.rs  │
│        │  │.rs      │  │ion.rs   │  │       │  │           │
│fnv1a   │  │Operation│  │Transact-│  │Rule   │  │Engine<S,O │
│hash fn │  │<S> trait│  │ion<O>   │  │<S,O,E,│  │,E,P>      │
│FNV     │  │         │  │struct   │  │P>trait│  │CommitFrame│
│consts  │  │         │  │         │  │RuleLi-│  │undo/redo  │
│        │  │         │  │         │  │fetime │  │dispatch   │
└────────┘  └─────────┘  └─────────┘  └───────┘  └───────────┘
     ▲                                                  │
     └──────────────────────────────────────────────────┘
          engine.rs uses hash internally (private dep)
```

```
examples/
├── tictactoe.rs    (existing, deterministic, must not break)
└── backgammon.rs   (new, non-deterministic dice, partial-move undo)
         │
         ├── uses herdingcats::* (Engine, Operation, Rule, etc.)
         ├── defines local: Board, Op, Event, Priority
         └── exercises: dice rolls, hit/reenter, bearing off, undo
```

### Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| `hash.rs` | FNV-1a 64-bit constants and hash function | `src/hash.rs` |
| `operation.rs` | `Operation<S>` trait definition | `src/operation.rs` |
| `transaction.rs` | `Transaction<O>` struct and `impl Default` | `src/transaction.rs` |
| `rule.rs` | `Rule<S,O,E,P>` trait, `RuleLifetime` enum | `src/rule.rs` |
| `engine.rs` | `Engine<S,O,E,P>` struct, `CommitFrame`, all dispatch logic | `src/engine.rs` |
| `lib.rs` | Module declarations (`mod X;`) + `pub use` re-exports only | `src/lib.rs` |

## Recommended Project Structure

```
src/
├── lib.rs              # thin facade: mod declarations + pub use re-exports
├── hash.rs             # fnv1a_hash(), FNV_OFFSET, FNV_PRIME (pub(crate))
├── operation.rs        # pub trait Operation<S>
├── transaction.rs      # pub struct Transaction<O>
├── rule.rs             # pub trait Rule<S,O,E,P>, pub enum RuleLifetime
└── engine.rs           # pub struct Engine<S,O,E,P>, CommitFrame (private)

examples/
├── tictactoe.rs        # unchanged
└── backgammon.rs       # new

tests/                  # optional integration test dir (proptest lives here or inline)
```

### Structure Rationale

- **hash.rs:** The FNV implementation is an internal concern. Exports `pub(crate)` only — `fnv1a_hash` and the two constants are never part of the public API. Engine uses them directly; no consumer needs them.
- **operation.rs:** The `Operation<S>` trait has no dependencies on other internal modules. Cleanest leaf node — put it first in the dependency chain.
- **transaction.rs:** Depends only on `Operation<S>` being imported as a type parameter bound. No direct imports required beyond standard library.
- **rule.rs:** Depends on `operation.rs` (imports `Operation`) and `transaction.rs` (imports `Transaction`). Also defines `RuleLifetime` here because it only exists to be passed to the engine alongside a rule.
- **engine.rs:** The integration point. Imports `hash`, `operation`, `transaction`, `rule`. This is the only module that is "fat." `CommitFrame` stays private in engine.rs — it is an implementation detail, not a public type.
- **lib.rs as facade:** Declares all modules with `mod hash; mod operation; mod transaction; mod rule; mod engine;` and re-exports every currently-public item. The public API surface must be identical post-split so `tictactoe.rs`'s `use herdingcats::*;` continues to work without change.

## Architectural Patterns

### Pattern 1: Thin Facade lib.rs

**What:** `lib.rs` contains only `mod` declarations and `pub use` re-exports. Zero logic lives in it.

**When to use:** Always for library crates being split into modules. This is the idiomatic Rust approach documented in The Book.

**Trade-offs:** Slightly more boilerplate in lib.rs vs. logic spread across it, but gives consumers a stable import path independent of internal file structure.

**Example:**
```rust
// src/lib.rs
mod hash;
mod operation;
mod transaction;
mod rule;
mod engine;

pub use engine::Engine;
pub use operation::Operation;
pub use rule::{Rule, RuleLifetime};
pub use transaction::Transaction;
```

Consumers see `herdingcats::Engine`, `herdingcats::Operation`, etc. — same as today.

### Pattern 2: pub(crate) for Internal Dependencies

**What:** Items used across modules within the crate but not part of the public API use `pub(crate)` visibility, not `pub`.

**When to use:** `hash.rs` exports `fnv1a_hash`, `FNV_OFFSET`, `FNV_PRIME` as `pub(crate)`. `CommitFrame` in `engine.rs` stays fully private (no `pub` at all — only `engine.rs` constructs it).

**Trade-offs:** More precise than making everything `pub`. Prevents accidental public API expansion during refactor.

**Example:**
```rust
// src/hash.rs
pub(crate) const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub(crate) const FNV_PRIME: u64 = 0x100000001b3;

pub(crate) fn fnv1a_hash(bytes: &[u8]) -> u64 { ... }
```

### Pattern 3: Inline #[cfg(test)] per Module

**What:** Each module file contains its own `#[cfg(test)] mod tests { ... }` block testing only that module's behavior.

**When to use:** Standard Rust convention. Keeps tests adjacent to the code they test. Integration tests (multi-module) go in `tests/` or in `engine.rs` tests that exercise full dispatch.

**Trade-offs:** Tests are compiled as part of the module, not a separate crate, giving access to private items. This is desirable for testing `CommitFrame` internals if needed.

## Backgammon Data Model

### Board Representation

Use a signed integer array of length 26. This is the standard compact representation:

```
indices 0..23  — the 24 points (standard backgammon numbering)
index 24       — White's bar
index 25       — Black's bar

Positive value at point N: White has that many checkers there
Negative value at point N: Black has that many checkers there
Zero: point is empty

Separate bear-off counters:
white_off: u8   — checkers borne off by White
black_off: u8   — checkers borne off by Black
```

This representation (one `[i8; 26]` plus two `u8` counters) is compact, clone-cheap, and directly encodes occupancy + ownership in a single value per point.

### Dice Representation

```rust
struct Dice {
    d1: u8,   // 1-6
    d2: u8,   // 1-6
    used: [bool; 2],   // which dice have been consumed
}
```

For doubles, both dice show the same value and both must be used (standard rules allow 4 moves on doubles — out of scope per PROJECT.md, implement as 2 moves only for engine testing purposes).

Dice rolls are non-deterministic (the point of choosing backgammon). They are generated via the event and wrapped in a non-deterministic, reversible transaction (`deterministic: false`).

### Operation Types

```rust
enum Op {
    // Move a checker from src point to dst point
    MoveChecker { player: Player, from: usize, to: usize },

    // Hit an opponent's blot: move it to bar
    HitChecker { opponent: Player, point: usize },

    // Reenter from bar to a point
    Reenter { player: Player, to: usize },

    // Bear off a checker from a point
    BearOff { player: Player, from: usize },

    // Consume a die (mark it used)
    UseDie { die_index: usize },

    // Record the dice roll result (for undo of die roll)
    SetDice { d1: u8, d2: u8 },

    // Advance turn
    SwitchTurn,
}
```

Every `Op` must implement `apply`, `undo`, and `hash_bytes`. The `undo` contract for `HitChecker` must restore the blot to its original point (not just the bar). This requires encoding the source point in the op.

### Event Types

```rust
enum Event {
    RollDice,
    Move { from: usize, die_index: usize },  // player chooses which die to consume
    BearOff { from: usize, die_index: usize },
}
```

Separating `RollDice` from `Move` lets the engine exercise exactly the pattern in PROJECT.md: "undo of single-die usage." A player can undo a single checker move (one die consumed) without undoing the roll.

### Rule Architecture

```
RollRule      — handles RollDice event, generates SetDice op,
                marks tx.deterministic = false
MoveRule      — validates checker movement, generates MoveChecker + UseDie ops,
                generates HitChecker op if landing on blot
ReenterRule   — handles Move event when player has checkers on bar,
                must reenter before any other move
BearOffRule   — handles BearOff event, validates all checkers in home board,
                generates BearOff op
WinRule (after hook) — checks if white_off == 15 or black_off == 15
```

### State Structure

```rust
#[derive(Clone)]
struct BackgammonState {
    board: [i8; 26],    // points 0-23, index 24 = white bar, 25 = black bar
    white_off: u8,
    black_off: u8,
    current: Player,
    dice: Option<Dice>,
    winner: Option<Player>,
}
```

## Data Flow

### Dispatch Flow (Normal Move)

```
Player chooses move
    ↓
engine.dispatch(Event::Move { from, die_index }, Transaction::new())
    ↓
RollRule.before() — skips (not RollDice event)
    ↓
MoveRule.before():
    - validates from point has current player's checker
    - validates die value matches distance to destination
    - if destination has exactly 1 opponent checker: push HitChecker op
    - push MoveChecker op
    - push UseDie op
    ↓
ops applied to state in order
    ↓
WinRule.after():
    - checks white_off == 15 or black_off == 15
    - if so: game over (set winner via op or tx.cancelled)
    ↓
CommitFrame pushed to undo_stack (transaction is irreversible by default)
```

### Non-Deterministic Dice Roll Flow

```
engine.dispatch(Event::RollDice, Transaction::new())
    ↓
RollRule.before():
    - rolls dice: d1 = rand, d2 = rand
    - tx.deterministic = false   ← marks this as non-deterministic
    - tx.irreversible = true     ← but still committed
    - push SetDice { d1, d2 }
    ↓
SetDice.apply() stores dice in state
    ↓
CommitFrame pushed — tx.deterministic = false recorded
    ↓
engine.undo() reverses: SetDice.undo() clears dice from state
```

The `Transaction.deterministic` flag on the frame signals to proptest and test harnesses that replaying this exact sequence requires providing the same dice values explicitly.

### Undo Sequence (Single Die Undo)

```
After two dice consumed (two Move dispatches):

engine.undo()  ← undoes second Move (UseDie + MoveChecker reversed)
engine.undo()  ← undoes first Move (UseDie + MoveChecker reversed)
engine.undo()  ← undoes RollDice (SetDice reversed, dice = None)
```

This works because each Move is its own CommitFrame. Undo is per-frame, not per-turn.

## Build Order

The modules have a strict dependency DAG. Build (implement) in this order:

1. **hash.rs** — no dependencies, pure functions. Write tests for hash determinism here.
2. **operation.rs** — no dependencies. Trait definition only, no tests needed beyond compile check.
3. **transaction.rs** — no dependencies beyond `operation.rs` type parameter. Add `impl Default for Transaction<O>`.
4. **rule.rs** — imports `Operation`, `Transaction`. Trait + `RuleLifetime` enum. Test `RuleLifetime` clone/copy semantics.
5. **engine.rs** — imports all of the above. Tests for dispatch, undo/redo, rule lifetime decrement. This is where proptest runs.
6. **lib.rs** — wire up `mod` declarations and `pub use`. Run `cargo test` to confirm zero regressions.
7. **examples/backgammon.rs** — implement after lib.rs is stable. Use `use herdingcats::*;`. Follow tictactoe.rs structure exactly.

Implement and test each module before moving to the next. Do not write backgammon until the library split compiles clean.

## Anti-Patterns

### Anti-Pattern 1: Splitting by file size rather than concept

**What people do:** Move code to new files when lib.rs gets "too long," splitting arbitrarily.
**Why it's wrong:** Results in modules with no clear ownership. `CommitFrame` ends up in `engine.rs` — that's correct, not a module of its own.
**Do this instead:** Split by the abstraction boundary. Each module name matches a concept in the library's mental model (hash, operation, transaction, rule, engine).

### Anti-Pattern 2: Making CommitFrame public

**What people do:** Assume anything crossing a module boundary needs to be `pub`.
**Why it's wrong:** `CommitFrame` is an engine implementation detail. Making it public expands the API surface and locks in internal structure.
**Do this instead:** Keep `CommitFrame` in `engine.rs` with no `pub`. It does not need to be visible from `lib.rs`.

### Anti-Pattern 3: One mega-Op for backgammon

**What people do:** Create a single `MoveOp { from, to, hit_point: Option<usize> }` that handles all cases inline.
**Why it's wrong:** Undo becomes coupled — `HitChecker` undo (restore blot to point) must be independently undoable from `MoveChecker` undo. If combined, a partial undo breaks the invariant.
**Do this instead:** Separate ops for each atomic state change. Each op's undo is trivially correct in isolation. The transaction batches them in order.

### Anti-Pattern 4: Rolling dice inside a Move event

**What people do:** Generate dice roll inside the `MoveRule.before()` for the first move of a turn.
**Why it's wrong:** Breaks the engine's undo model. The dice roll would be embedded in the same CommitFrame as the first move — undoing the move also destroys knowledge of what was rolled.
**Do this instead:** `Event::RollDice` is its own dispatch call producing its own CommitFrame. Engine can undo the roll independently from undoing moves.

### Anti-Pattern 5: Re-declaring mod in multiple files

**What people do:** Write `mod operation;` in both `lib.rs` and `engine.rs`.
**Why it's wrong:** Rust treats this as two separate modules, causing duplicate type errors at compile time. The Rust Book is explicit: declare each module exactly once.
**Do this instead:** `mod operation;` appears only in `lib.rs`. `engine.rs` uses `crate::operation::Operation` or `use crate::operation::Operation;` to import it.

## Integration Points

### Internal Module Boundaries

| Boundary | Communication | Visibility |
|----------|---------------|------------|
| `engine.rs` → `hash.rs` | Direct function call to `fnv1a_hash` | `pub(crate)` |
| `engine.rs` → `rule.rs` | `Box<dyn Rule<...>>` trait objects | `pub` (trait is public) |
| `engine.rs` → `transaction.rs` | `Transaction<O>` fields and mutation | `pub` (struct fields are `pub`) |
| `lib.rs` → all modules | `pub use` re-exports | N/A |

### Example Boundary (examples/ to library)

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `backgammon.rs` → library | `use herdingcats::*;` | Must match tictactoe.rs pattern exactly |
| `backgammon.rs` → stdlib | `use std::...` for rand | No new runtime crate deps — use stdlib or manual LCG for dice |

Note on dice randomness: The project has zero external runtime dependencies. For dice rolls in `backgammon.rs`, use a simple approach: seed from `std::time::SystemTime` and implement a minimal LCG in the example file itself, or use `getrandom` via `#[cfg(test)]` only. The simplest path is a local function in `backgammon.rs` that reads time as a seed — no crate required.

## Sources

- [Separating Modules into Different Files — The Rust Book](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html) — HIGH confidence (official)
- [Re-exports — The rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html) — HIGH confidence (official)
- [Re-Exporting and Privacy in Rust — Rheinwerk Computing](https://blog.rheinwerk-computing.com/re-exporting-and-privacy-in-rust) — MEDIUM confidence (verified against official docs)
- [backgammon crate on docs.rs](https://docs.rs/backgammon/latest/backgammon/) — LOW confidence for data model (docs incomplete; structure inferred from rules + standard game theory)
- Board representation: `[i8; 26]` pattern — MEDIUM confidence (multiple independent sources agree on signed-array approach; standard in academic backgammon implementations)

---
*Architecture research for: herdingcats Rust library module split + backgammon example*
*Researched: 2026-03-08*
