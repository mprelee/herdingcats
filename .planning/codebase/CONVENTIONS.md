# Coding Conventions

**Analysis Date:** 2026-03-08

## Naming Patterns

**Files:**
- `snake_case` for all Rust source files: `lib.rs`, `tictactoe.rs`
- Single-file library: `src/lib.rs` is the sole library entry point

**Structs and Enums:**
- `PascalCase` for all types: `Transaction`, `CommitFrame`, `RuleLifetime`, `Engine`
- `PascalCase` for enum variants: `RuleLifetime::Permanent`, `RuleLifetime::Turns(u32)`, `Op::SwitchTurn`
- Named fields in enum variants for clarity: `Op::Place { idx: usize, player: Player }`

**Traits:**
- `PascalCase` with descriptive role names: `Operation`, `Rule`

**Functions and Methods:**
- `snake_case` for all functions and methods: `apply`, `undo`, `hash_bytes`, `dispatch_preview`, `add_rule`, `replay_hash`
- Short, verb-oriented names: `apply`, `undo`, `read`, `write`, `dispatch`

**Variables:**
- `snake_case`: `state_snapshot`, `lifetime_snapshot`, `hash_before`, `enabled_snapshot`
- Single-letter variables used minimally and only in local scope: `a`, `b`, `c` for board indices within a `for` loop

**Constants:**
- `SCREAMING_SNAKE_CASE`: `FNV_OFFSET`, `FNV_PRIME`
- Type annotated explicitly: `const FNV_OFFSET: u64 = ...`

**Generic Type Parameters:**
- Single uppercase letters: `S` (state), `O` (operation), `E` (event), `P` (priority)
- Consistent across the entire `Engine<S, O, E, P>` and `Rule<S, O, E, P>` boundary

## Code Style

**Formatting:**
- `rustfmt` 1.8.0-stable (default settings; no `rustfmt.toml` present)
- Trailing commas in struct and enum definitions
- Long function signatures broken across multiple lines with each parameter on its own line:
  ```rust
  pub fn add_rule<R>(
      &mut self,
      rule: R,
      lifetime: RuleLifetime,
  ) where
      R: Rule<S, O, E, P> + 'static,
  { ... }
  ```

**Linting:**
- No `clippy.toml` present; standard Clippy rules apply
- No `#![deny(...)]` or `#![allow(...)]` attributes observed in `src/lib.rs`

## Section Delimiters

Sections within files are separated by prominent ASCII banner comments to aid navigation:

```rust
//
// ============================================================
// Engine
// ============================================================
//
```

Within `impl` blocks, subsections use a shorter form:

```rust
//
// --------------------------------------------------------
// Commit Dispatch
// --------------------------------------------------------
//
```

This is a strict convention used consistently throughout `src/lib.rs` and `examples/tictactoe.rs`. New sections must follow this pattern.

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate imports (`use herdingcats::*`)
3. No internal module imports (single-file library)

**Style:**
- Use glob imports from the library in consumer code: `use herdingcats::*;`
- Grouped standard library imports kept together: `use std::collections::{HashMap, HashSet};`
- No path aliases used

## Visibility

- `pub` on all types and methods intended for library consumers: `Engine`, `Transaction`, `Rule`, `Operation`, `RuleLifetime`, `dispatch`, `add_rule`, `undo`, `redo`, `read`, `write`
- Internal types default to private (no `pub`): `CommitFrame`, `fnv1a_hash`
- Public struct fields used when the field is a primary interface: `Engine::state`, `Transaction::ops`, `Transaction::irreversible`, `Transaction::deterministic`, `Transaction::cancelled`

## Error Handling

**Pattern:**
- No `Result` or `?` operator used in the library core — operations are infallible by design
- Invalid states are handled by mutating `tx.cancelled = true` inside rules rather than returning errors:
  ```rust
  if state.winner.is_some() {
      tx.cancelled = true;
      return;
  }
  ```
- `unreachable!()` used for exhaustive match arms that logically cannot occur:
  ```rust
  Cell::_ => unreachable!(),
  ```
- No `panic!`, `unwrap()`, or `expect()` calls in library code

## Trait Design

**Required vs Default Methods:**
- Trait methods that must always be defined: `id()`, `priority()`, `apply()`, `undo()`, `hash_bytes()`
- Trait methods with no-op defaults (override only when needed): `before()`, `after()` on `Rule`
- Default implementations use underscore-prefixed parameters to silence unused warnings:
  ```rust
  fn before(
      &self,
      _state: &S,
      _event: &mut E,
      _tx: &mut Transaction<O>,
  ) {}
  ```

## Generics and Bounds

- `where` clauses on separate lines from `impl` or `fn` signatures for readability
- Bounds stated explicitly and minimally: `S: Clone`, `O: Operation<S>`, `P: Copy + Ord`
- `PhantomData` used when a type parameter is logically tied to a struct but not stored directly: `CommitFrame::_marker: std::marker::PhantomData<S>`

## Derive Macros

- `#[derive(Clone)]` applied to all types that must be snapshot-able or stored on stacks
- `#[derive(Clone, Copy, Debug)]` for lightweight value types like `RuleLifetime`
- `#[derive(Clone, Copy, Debug, PartialEq)]` for domain enums (`Player`, `Cell`)
- Derive macros are the only attributes used; no proc macros or custom derives

## Constructors

- `fn new() -> Self` is the canonical constructor pattern for both library types (`Transaction`, `Engine`) and consumer types (`Game`)
- No builder pattern used; structs initialized with field literals inside `new()`

## Comments

**When to Comment:**
- Section banners for every logical group of code (mandatory, see Section Delimiters above)
- No inline comments on individual statements — code is expected to be self-explanatory
- No JSDoc-style doc comments (`///`) observed in the library code; documentation expected via `README.md` and `docs/`

## Architectural Constraints (Enforced by Convention)

These are enforced through documentation (`docs/ARCHITECTURAL_INVARIANTS.md`, `AI.md`) and code review, not compiler enforcement:

- All state mutation must occur through `Operation::apply()` — never by direct field assignment outside an `Operation`
- Priority type `P` must be a closed enum (never a raw integer)
- Events must be closed enums with static dispatch — no `dyn Any` or runtime type inspection
- Rule durations must use `RuleLifetime`, not fields on rule structs
- RNG state (if added) must live inside `S` and mutate through `Operation`

---

*Convention analysis: 2026-03-08*
