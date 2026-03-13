# Coding Conventions

**Analysis Date:** 2026-03-13

## Naming Patterns

**Files:**
- Rust module files use snake_case: `engine.rs`, `behavior.rs`, `mutation.rs`, `action.rs`, `hash.rs`
- Each trait or major struct concept gets its own module file
- No `mod.rs` pattern observed; inline module organization in `lib.rs`

**Functions:**
- Function names use snake_case: `dispatch`, `dispatch_with`, `dispatch_preview`, `add_behavior`, `replay_hash`
- Internal helper functions prefixed with descriptive names: `op_sequence_strategy`, `mixed_op_strategy`
- Accessor methods follow convention: `read()`, `new()`, `id()`, `priority()`

**Variables:**
- Local variables and parameters use snake_case: `state`, `behavior`, `event`, `tx`, `engine`
- Abbreviations used for conciseness in specific contexts: `tx` for transaction/action, `op` for operation
- Mutable bindings explicitly declared with `mut`: `let mut engine`, `let mut tx`

**Types:**
- PascalCase for struct/enum/trait names: `Engine`, `Behavior`, `Mutation`, `Action`, `CommitFrame`
- Generic type parameters use single uppercase letters or descriptive short names: `S` (state), `M` (mutation), `I` (input), `P` (priority)
- Trait implementations follow `impl Trait<S, M, I, P> for Type` pattern

## Code Style

**Formatting:**
- No explicit formatter detected in Cargo.toml configuration
- Code uses standard Rust formatting conventions
- 4-space indentation
- Comments use `//` for single-line and `/* */` for multi-line documentation
- File: `rustfmt.toml` contains `edition = "2024"` indicating rustfmt configuration

**Linting:**
- `#![warn(missing_docs)]` crate-level lint at top of `src/lib.rs` enforces documentation
- All public items must have documentation comments or build will warn
- No explicit clippy configuration detected in checked files

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. Internal crate imports (`use crate::...`)
3. External crate imports (via `use external_crate::...`)

**Example from `src/engine.rs`:**
```rust
use crate::action::Action;
use crate::behavior::Behavior;
use crate::hash::{FNV_OFFSET, FNV_PRIME, fnv1a_hash};
use crate::mutation::Mutation;
```

**Path Aliases:**
- No path aliases or alternative module paths observed in codebase
- Direct module paths used: `crate::behavior`, `crate::mutation`, `crate::action`

## Error Handling

**Patterns:**
- No dedicated error type or custom Result wrapper observed in current codebase
- Outcome types are domain-specific: `Option<Action<M>>` returned from dispatch operations
- `is_none()` / `is_some()` checks used to inspect results
- No panic-heavy approach; methods return `None` for non-committed states
- Example from dispatch: cancelled actions and empty mutations both return `None`

## Logging

**Framework:** None detected

**Patterns:**
- No structured logging framework present
- Codebase is infrastructure/library focused; logging responsibility deferred to library user
- Documentation emphasizes behavior transparency through `Action` return types and replay history
- Tests use simple `println!` equivalents implicit in assertions

## Comments

**When to Comment:**
- /// style documentation comments for all public items (enforced by `#![warn(missing_docs)]`)
- Comments above key sections within implementations to organize code blocks
- Section markers used: `// ============================================================` style dividers
- Test fixtures documented inline within test modules

**JSDoc/TSDoc:**
- Rust uses `///` for doc comments (not JSDoc)
- Documentation style observed in `src/behavior.rs`, `src/mutation.rs`:
  - Triple-slash comments immediately precede the item they document
  - Markdown formatting supported in doc comments
  - `# Examples` section with runnable code blocks in many functions
  - Generic explanation followed by concrete `# Examples` section

**Example from `src/mutation.rs`:**
```rust
/// Return a deterministic byte sequence that uniquely identifies this
/// mutation variant and its data.
///
/// The engine concatenates the `hash_bytes()` of all mutations...
///
/// # Examples
///
/// ```
/// use herdingcats::Mutation;
/// // ... code example ...
/// ```
fn hash_bytes(&self) -> Vec<u8>;
```

## Function Design

**Size:**
- Functions kept relatively small and focused
- Complex dispatch logic broken into phases: `before()` hooks, mutation application, `after()` hooks
- Helper functions created for repeated logic (e.g., `op_sequence_strategy` in tests)

**Parameters:**
- Trait methods accept borrowed references when inspecting: `&self`, `&state`, `&event`
- Mutable references when mutation needed: `&mut state`, `&mut tx`, `&mut event`
- Owned values used only for types implementing `Clone` and when transfer of ownership is intentional
- Generic parameters constrained with bounds: `S: Clone`, `M: Mutation<S>`

**Return Values:**
- `Option<T>` for nullable results: `Option<Action<M>>`
- Unit type `()` for operations with no meaningful return
- Owned types returned when ownership transfer needed: `Engine<S, M, I, P>` returned from constructors
- Functions that inspect state return clones: `read()` returns `S`

## Module Design

**Exports:**
- Public items exported via `pub use` statements at `src/lib.rs` module level
- Current exports (from previous commit): `Action`, `Behavior`, `Engine`, `Mutation`
- Internal modules not re-exported: `action`, `behavior`, `engine`, `hash`, `mutation` are private mod declarations
- Users access traits/structs through `herdingcats::` namespace

**Barrel Files:**
- `src/lib.rs` serves as barrel file aggregating all public exports
- Each concept (Engine, Behavior, Mutation, Action) lives in separate module file
- Clear separation between public API (`lib.rs` exports) and implementation details (private module internals)

## Generic Patterns Observed

**Trait Parameterization:**
- Heavy use of generics to maintain type safety: `Engine<S, M, I, P>`, `Behavior<S, M, I, P>`, `Mutation<S>`
- Bounds carefully chosen to enable implementation without forcing unnecessary constraints
- `PhantomData<S>` markers used for variance control when struct doesn't own the type parameter

**Copy vs Clone:**
- Traits use `Clone` for stateful types: `S: Clone` required for state snapshots
- Simple types use `Copy` when possible: `P: Copy + Ord` for priority values
- Mutation types required to be `Clone` to support undo stack storage

---

*Convention analysis: 2026-03-13*
