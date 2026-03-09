# Codebase Structure

**Analysis Date:** 2026-03-08

## Directory Layout

```
herdingcats/
├── src/
│   └── lib.rs              # Entire library: Engine, traits, Transaction, RuleLifetime
├── examples/
│   └── tictactoe.rs        # Reference implementation / runnable example
├── docs/
│   ├── ARCHITECTURAL_INVARIANTS.md   # Hard constraints the engine must never violate
│   └── EXTENSION_GUIDELINES.md       # How to safely add new mechanics
├── .planning/
│   └── codebase/           # GSD mapping documents (this directory)
├── target/                 # Cargo build output (generated, not committed)
├── AI.md                   # AI modification policy (read before any changes)
├── Cargo.toml              # Package manifest, no external dependencies
├── Cargo.lock              # Lockfile
├── README.md               # Project overview and core model description
├── CONTRIBUTING.md         # Contribution guidelines
├── LICENSE-MIT             # MIT license
└── LICENSE-APACHE          # Apache 2.0 license
```

## Directory Purposes

**`src/`:**
- Purpose: The entire library source
- Contains: One file (`lib.rs`) with all public types and traits
- Key files: `src/lib.rs`

**`examples/`:**
- Purpose: Runnable reference implementations demonstrating library usage
- Contains: Standalone `.rs` files that are Cargo examples (each has its own `main()`)
- Key files: `examples/tictactoe.rs`

**`docs/`:**
- Purpose: Architectural governance documents — mandatory reading before modifying the engine
- Contains: `ARCHITECTURAL_INVARIANTS.md`, `EXTENSION_GUIDELINES.md`
- Key files: `docs/ARCHITECTURAL_INVARIANTS.md`, `docs/EXTENSION_GUIDELINES.md`

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes
- Committed: No (in `.gitignore`)

**`.planning/codebase/`:**
- Purpose: GSD codebase map documents
- Generated: Yes (by GSD mapping commands)
- Committed: Yes

## Key File Locations

**Entry Points:**
- `src/lib.rs`: Library root — all public API defined here
- `examples/tictactoe.rs`: Runnable example binary with `fn main()`

**Configuration:**
- `Cargo.toml`: Package name (`herdingcats`), version (`0.2.0`), edition (`2024`), no `[dependencies]`
- `Cargo.lock`: Lockfile

**Core Logic:**
- `src/lib.rs` lines 1–19: FNV-1a 64-bit hash implementation (`fnv1a_hash`)
- `src/lib.rs` lines 27–31: `Operation<S>` trait definition
- `src/lib.rs` lines 39–56: `Transaction<O>` struct
- `src/lib.rs` lines 64–69: `RuleLifetime` enum
- `src/lib.rs` lines 77–99: `Rule<S, O, E, P>` trait definition
- `src/lib.rs` lines 107–115: `CommitFrame<S, O>` internal struct
- `src/lib.rs` lines 123–327: `Engine<S, O, E, P>` struct and all methods

**Governance:**
- `docs/ARCHITECTURAL_INVARIANTS.md`: Nine invariants that must never be weakened
- `docs/EXTENSION_GUIDELINES.md`: Eight extension patterns for safely growing the engine
- `AI.md`: Hard requirements and extension policy for AI-assisted modifications

## Naming Conventions

**Files:**
- Library source: `snake_case.rs` (e.g., `lib.rs`)
- Examples: `snake_case.rs` named after the game/scenario (e.g., `tictactoe.rs`)
- Docs: `SCREAMING_SNAKE_CASE.md` for governance documents

**Structs and Enums:**
- `PascalCase` for all types: `Engine`, `Transaction`, `CommitFrame`, `RuleLifetime`, `GameEvent`, `PlayRule`, `WinRule`

**Traits:**
- `PascalCase`: `Operation`, `Rule`

**Functions and Methods:**
- `snake_case`: `dispatch`, `dispatch_preview`, `add_rule`, `replay_hash`, `fnv1a_hash`

**Type Parameters:**
- Single uppercase letters following the convention: `S` (state), `O` (operation), `E` (event), `P` (priority)

**Constants:**
- `SCREAMING_SNAKE_CASE`: `FNV_OFFSET`, `FNV_PRIME`

**Enum variants:**
- `PascalCase`: `Permanent`, `Turns`, `Triggers`, `Default`, `Place`, `SetWinner`, `SwitchTurn`

## Where to Add New Code

**New Game (example):**
- Create: `examples/<game_name>.rs`
- Structure: Define state struct, `Op` enum implementing `Operation<State>`, event enum, `Rule` impls, `fn main()`
- Run with: `cargo run --example <game_name>`

**New Operation Type (within a game):**
- Add variant to the game's `Op` enum in its `examples/<game>.rs`
- Implement `apply()`, `undo()`, `hash_bytes()` for the new variant
- Must be deterministic and fully reversible

**New Rule (within a game):**
- Define a struct (zero-size or data-carrying) in `examples/<game>.rs`
- Implement `Rule<S, O, E, P>` for it — override only `before` and/or `after` as needed
- Register via `engine.add_rule(MyRule, RuleLifetime::Permanent)` in `main()`

**New Engine Feature (modifying the library):**
- Edit `src/lib.rs` only
- Must not violate any invariant in `docs/ARCHITECTURAL_INVARIANTS.md`
- Must follow patterns in `docs/EXTENSION_GUIDELINES.md`
- Feature-specific logic must not be embedded in the engine core

**New Transaction Flags:**
- Add fields to `Transaction<O>` in `src/lib.rs`
- Update `dispatch()` and `dispatch_preview()` to respect the flag deterministically

## Special Directories

**`docs/`:**
- Purpose: Architectural governance (not API docs — those are on docs.rs)
- Generated: No
- Committed: Yes — treat as source of truth for engine constraints

**`target/`:**
- Purpose: Compiled artifacts, test binaries, example binaries
- Generated: Yes (by `cargo build` / `cargo run`)
- Committed: No

---

*Structure analysis: 2026-03-08*
