# Phase 1: Core Types - Research

**Researched:** 2026-03-13
**Domain:** Rust trait design, associated types, enum modeling, rustdoc re-exports
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**EngineSpec design**
- Trait with associated types: `State`, `Input`, `Diff`, `Trace`, `NonCommittedInfo`, `OrderKey: Ord`
- Users define a unit struct and `impl EngineSpec for MySpec { type State = ...; ... }`
- Non-committed payload associated type is named `NonCommittedInfo`

**Behavior trait**
- `Behavior<E: EngineSpec>` — one type param, tied to a spec
- Signature: `fn name(&self) -> &'static str`, `fn order_key(&self) -> E::OrderKey`, `fn evaluate(&self, input: &E::Input, state: &E::State) -> BehaviorResult<E::Diff, E::NonCommittedInfo>`
- User implements: `impl Behavior<MySpec> for MyBehavior { ... }`

**Frame**
- Library-owned type: `pub struct Frame<E: EngineSpec> { pub input: E::Input, pub diff: E::Diff, pub trace: E::Trace }`
- Derives: `Debug, Clone, PartialEq`
- `Outcome<F, N>` in practice is `Outcome<Frame<E>, E::NonCommittedInfo>`

**Outcome**
- Generic `Outcome<F, N>` with 7 variants: `Committed(F)`, `Undone(F)`, `Redone(F)`, `NoChange`, `InvalidInput(N)`, `Disallowed(N)`, `Aborted(N)`
- Derives: `Debug, Clone, PartialEq`

**EngineError**
- Structured variants for MVP: `BehaviorPanic`, `InvalidState(String)`, `CorruptHistory`
- `#[non_exhaustive]`
- Derives: `Debug, Clone, PartialEq`
- No `std::error::Error` impl in this phase

**Module layout**
- Flat crate root: all types re-exported at `herdingcats::*`
- `src/spec.rs` — `EngineSpec` trait
- `src/behavior.rs` — `Behavior` trait + `BehaviorResult`
- `src/outcome.rs` — `Outcome`, `Frame`, `EngineError`
- `src/lib.rs` — `pub use` re-exports only
- Full rustdoc comments on every public type, trait, and method

**Type bounds on EngineSpec associated types**
- `State: Clone + Debug + Default`
- `Input: Clone + Debug`
- `Diff: Clone + Debug`
- `Trace: Clone + Debug`
- `NonCommittedInfo: Clone + Debug`
- `OrderKey: Ord`

### Claude's Discretion
- Exact rustdoc wording and link structure
- Whether to add `#[must_use]` to `Outcome` and `Result<Outcome, EngineError>`
- Whether `BehaviorResult` gets `Debug + Clone + PartialEq` derives (reasonable yes)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-01 | Library user can define engine type params via a single `EngineSpec` trait (bundles S, I, D, T, O, K) | Associated type trait pattern verified; type bounds confirmed |
| CORE-02 | Library user can implement `Behavior` trait with `name()`, `order_key()`, `evaluate()` — signature prevents mutation | `&self` + `&E::State` in evaluate signature structurally enforces immutability |
| CORE-03 | `BehaviorResult<D, O>` provides `Continue(Vec<D>)` and `Stop(O)` variants | Standard enum; dyn safety analysis confirms no issues |
| CORE-04 | `Outcome<F, N>` enum provides all 7 variants | Standard enum; `#[must_use]` recommendation researched |
| CORE-05 | `EngineError` is distinct from `Outcome` — reserved for engine-internal failures | `#[non_exhaustive]` mechanics verified; cross-crate match behavior confirmed |
</phase_requirements>

---

## Summary

Phase 1 defines all public type contracts for the `herdingcats` crate. The design is entirely Rust-idiomatic: `EngineSpec` is a trait-with-associated-types bundle, `Behavior<E>` is a parametric trait over the spec, and `Outcome`/`BehaviorResult`/`EngineError` are enums. No external dependencies are needed. All of Rust's standard tools — associated types, derive macros, `#[non_exhaustive]`, `#[must_use]` — directly support the locked decisions.

Two subtleties warrant attention. First, `Box<dyn Behavior<E>>` is dyn-safe when `E` is concrete (trait-level type parameters are permitted on trait objects; only method-level generics are prohibited). This is the intended storage pattern for Phase 2's dispatch loop. Second, `#[non_exhaustive]` on `EngineError` has no effect inside the defining crate — tests and examples in the repo can match `EngineError` exhaustively, but downstream library users cannot. This is the intended behavior.

The module layout (private impl modules, `pub use` at crate root) is supported directly by rustdoc: when implementation modules are private, rustdoc inlines re-exported items at the re-export site. Users see a flat `herdingcats::*` API, and documentation appears at crate root without any extra `#[doc(inline)]` annotations needed.

**Primary recommendation:** Implement exactly as specified in CONTEXT.md. No deviations required. Focus effort on thorough rustdoc comments and correct derive sets.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std only | (std) | All types, traits, derives | Zero-dep constraint from PROJECT.md |

### Supporting (dev only)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| proptest | 1.10 | Property-based testing | Phase 4 tests — already in Cargo.toml |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Outcome<F, N>` generic enum | Concrete `Outcome<Frame<E>>` | Generic form enables re-use in undo/redo; Phase 2/3 depend on it |
| `#[non_exhaustive]` on `EngineError` | Sealed trait pattern | `#[non_exhaustive]` is simpler for a pure enum; sealed traits add complexity |

**Installation:** No new dependencies needed. `Cargo.toml` already correct.

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── lib.rs          # pub use re-exports only — no logic
├── spec.rs         # EngineSpec trait
├── behavior.rs     # Behavior<E> trait + BehaviorResult<D, O> enum
└── outcome.rs      # Outcome<F, N> enum, Frame<E> struct, EngineError enum
```

### Pattern 1: Associated-Type Bundle Trait (EngineSpec)

**What:** A marker trait with only associated types. Users implement it on a unit struct to bundle all game-specific types behind a single type parameter.

**When to use:** Whenever a family of types must travel together through generic bounds (prevents "generic explosion" where every function requires `<S, I, D, T, O, K>` individually).

**Example:**
```rust
// Source: Rust Reference — associated items
pub trait EngineSpec {
    type State: Clone + Debug + Default;
    type Input: Clone + Debug;
    type Diff:  Clone + Debug;
    type Trace: Clone + Debug;
    type NonCommittedInfo: Clone + Debug;
    type OrderKey: Ord;
}

// User code:
struct TicTacToeSpec;
impl EngineSpec for TicTacToeSpec {
    type State = TicTacToeState;
    type Input = TicTacToeMove;
    // ...
}
```

### Pattern 2: Parametric Behavior Trait

**What:** `Behavior<E: EngineSpec>` takes the spec as a type parameter. Methods reference `E::Input`, `E::State`, etc. `Box<dyn Behavior<E>>` is valid because `E` is fixed at the call site — trait-level type parameters are dyn-safe.

**When to use:** Every concrete behavior the user defines. Phase 2 stores `Vec<Box<dyn Behavior<E>>>`.

**Example:**
```rust
// Source: verified against Rust Reference object safety rules
pub trait Behavior<E: EngineSpec> {
    fn name(&self) -> &'static str;
    fn order_key(&self) -> E::OrderKey;
    fn evaluate(
        &self,
        input: &E::Input,
        state: &E::State,
    ) -> BehaviorResult<E::Diff, E::NonCommittedInfo>;
}
```

**Dyn safety note:** `Box<dyn Behavior<E>>` compiles when `E` is a concrete type. All three methods take `&self` (no generic method params). This is valid dyn usage.

### Pattern 3: Two-Parameter Generic Enum (Outcome, BehaviorResult)

**What:** Parametric enums where type parameters carry semantic meaning (`F` = frame type, `N` = non-committed info).

**When to use:** Core result types. Keeping them generic means Phase 3 undo/redo can return `Outcome<Frame<E>, E::NonCommittedInfo>` naturally.

**Example:**
```rust
// Source: ARCHITECTURE.md + CONTEXT.md
#[must_use = "dispatch outcomes must be handled"]
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<F, N> {
    Committed(F),
    Undone(F),
    Redone(F),
    NoChange,
    InvalidInput(N),
    Disallowed(N),
    Aborted(N),
}
```

### Pattern 4: Non-Exhaustive Error Enum

**What:** `#[non_exhaustive]` on `EngineError` forces downstream library users to add a wildcard arm in match expressions. The crate itself (tests included) can match exhaustively.

**When to use:** Error types that may grow new variants in future releases without breaking downstream code.

**Example:**
```rust
// Source: Rust Reference — type_system attributes
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    BehaviorPanic,
    InvalidState(String),
    CorruptHistory,
}
```

### Pattern 5: Private Module + `pub use` Re-export

**What:** Implementation files (`spec.rs`, `behavior.rs`, `outcome.rs`) are `mod` (not `pub mod`) in `lib.rs`. Types are surfaced via `pub use`. Rustdoc inlines documentation automatically when source module is private.

**When to use:** Library crates that want a flat public API without exposing internal module structure.

**Example:**
```rust
// src/lib.rs
mod spec;
mod behavior;
mod outcome;

pub use spec::EngineSpec;
pub use behavior::{Behavior, BehaviorResult};
pub use outcome::{Frame, Outcome, EngineError};
```

Users write `use herdingcats::{EngineSpec, Behavior, Outcome};` — no module paths needed.

Rustdoc renders all items as if they live at crate root (automatic when source module is private). No `#[doc(inline)]` required.

### Anti-Patterns to Avoid

- **`pub mod spec` (public modules):** Leaks module structure into the public API. Users would see both `herdingcats::EngineSpec` and `herdingcats::spec::EngineSpec`. Keep implementation modules private.
- **Concrete type in `Outcome` during Phase 1:** Do not specialize `Outcome` to `Frame<E>` in this phase. Keep `Outcome<F, N>` generic — downstream phases specialize.
- **Adding `std::error::Error` impl to `EngineError` now:** Explicitly deferred from this phase. Adding it later (with `#[non_exhaustive]` in place) is backward-compatible.
- **`where Self: Sized` on Behavior methods:** This would break dyn usage. Do not add it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Compile-time mutation prevention | Runtime checks or `RefCell` | `&self` + `&E::State` in signature | Rust's borrow checker enforces immutability structurally |
| Forward-compatibility on `EngineError` | Sealed trait | `#[non_exhaustive]` | Built-in language support; no boilerplate |
| Flat API surface | Proc-macro re-export | `pub use` in `lib.rs` | Rustdoc inlines automatically for private modules |
| Derive impls | Manual `Debug`/`Clone`/`PartialEq` | `#[derive(...)]` | Standard; no edge cases for these simple types |

**Key insight:** All complexity in this phase is handled by Rust's type system itself. The work is authoring, not engineering — writing the correct type signatures and derives, not building mechanisms.

---

## Common Pitfalls

### Pitfall 1: `pub mod` Instead of `mod`

**What goes wrong:** Making implementation modules `pub mod` means users see both `herdingcats::Outcome` and `herdingcats::outcome::Outcome`. Rustdoc generates duplicate entries. `use herdingcats::outcome::Outcome` imports work, cluttering the API surface.

**Why it happens:** Habit from application code where module visibility doesn't matter.

**How to avoid:** All three implementation modules (`spec`, `behavior`, `outcome`) must be private `mod`, not `pub mod`. Re-export via `pub use` in `lib.rs`.

**Warning signs:** `cargo doc --open` shows items nested under module paths in addition to crate root.

### Pitfall 2: `#[non_exhaustive]` Misapplied to `Outcome`

**What goes wrong:** If `Outcome` gets `#[non_exhaustive]`, tests inside the crate can still match exhaustively (no effect within defining crate), but the intent is wrong — `Outcome` variants are a stable public contract, not a forward-extension point.

**Why it happens:** Conflating error types (which may grow) with result types (which have a fixed semantic contract).

**How to avoid:** Apply `#[non_exhaustive]` only to `EngineError`. Do not apply to `Outcome` or `BehaviorResult`.

### Pitfall 3: Breaking Dyn Safety on `Behavior`

**What goes wrong:** Adding a method with a generic type parameter to `Behavior<E>` makes the trait not dyn-compatible. Phase 2's `Vec<Box<dyn Behavior<E>>>` fails to compile.

**Why it happens:** Adding a convenience method like `fn describe<T: Display>(&self) -> T` without considering dyn impact.

**How to avoid:** All methods on `Behavior<E>` must use only `&self`, types from `E::*`, or concrete types. No method-level generic type parameters.

**Warning signs:** Compiler error: "the trait `Behavior` is not dyn-compatible."

### Pitfall 4: Missing `Default` Bound on `State`

**What goes wrong:** Phase 2 needs `E::State::default()` for engine construction. If `State: Clone + Debug` only, Phase 2 code cannot call `Default::default()`.

**Why it happens:** Bounds look complete but `Default` is missed.

**How to avoid:** `State: Clone + Debug + Default` — locked in CONTEXT.md. Verify the bound is on `EngineSpec`, not just added where needed later.

### Pitfall 5: `OrderKey` Missing `Clone` or `PartialEq`

**What goes wrong:** Phase 2 needs to sort behaviors by `(order_key, name)`. Sorting requires `Ord` (locked in CONTEXT.md) but comparison and storage also benefit from `Clone`. If `OrderKey: Ord` only, `order_key()` returns a value that can be compared but not stored without moves.

**Why it happens:** `Ord` implies `PartialOrd + Eq + PartialEq` but not `Clone`.

**How to avoid:** The CONTEXT.md locks `OrderKey: Ord`. Phase 2 stores `order_key()` return values in a sort key tuple — the caller owns the value from the getter. This is fine: `order_key(&self) -> E::OrderKey` returns by value. Phase 2 can call `b.order_key()` directly in the sort comparator without needing `Clone` on `OrderKey`. No additional bound needed beyond `Ord`. If `Clone` is needed later, it can be added then. Do not over-constrain in Phase 1.

---

## Code Examples

Verified patterns from official sources and architecture document:

### EngineSpec with All Required Bounds
```rust
// Source: CONTEXT.md locked decisions + Rust Reference associated items
pub trait EngineSpec {
    type State: Clone + std::fmt::Debug + Default;
    type Input: Clone + std::fmt::Debug;
    type Diff:  Clone + std::fmt::Debug;
    type Trace: Clone + std::fmt::Debug;
    type NonCommittedInfo: Clone + std::fmt::Debug;
    type OrderKey: Ord;
}
```

### Behavior Trait — Dyn-Safe Pattern
```rust
// Source: verified dyn safety rules — trait-level type params are allowed on trait objects
pub trait Behavior<E: EngineSpec> {
    fn name(&self) -> &'static str;
    fn order_key(&self) -> E::OrderKey;
    fn evaluate(
        &self,
        input: &E::Input,
        state: &E::State,
    ) -> BehaviorResult<E::Diff, E::NonCommittedInfo>;
}
// Box<dyn Behavior<ConcreteSpec>> is valid.
```

### BehaviorResult Enum
```rust
// Source: ARCHITECTURE.md BehaviorResult section
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorResult<D, O> {
    Continue(Vec<D>),
    Stop(O),
}
```

### Outcome with #[must_use]
```rust
// Source: Rust Reference diagnostics attributes + CONTEXT.md
#[must_use = "dispatch outcomes must be handled"]
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<F, N> {
    Committed(F),
    Undone(F),
    Redone(F),
    NoChange,
    InvalidInput(N),
    Disallowed(N),
    Aborted(N),
}
```

### Frame Struct
```rust
// Source: CONTEXT.md + ARCHITECTURE.md Frame section
#[derive(Debug, Clone, PartialEq)]
pub struct Frame<E: EngineSpec> {
    pub input: E::Input,
    pub diff:  E::Diff,
    pub trace: E::Trace,
}
```

### EngineError with #[non_exhaustive]
```rust
// Source: Rust Reference type_system attributes
// #[non_exhaustive] has NO effect inside this crate.
// Downstream users MUST add `_ => {}` wildcard to match arms.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    BehaviorPanic,
    InvalidState(String),
    CorruptHistory,
}
```

### lib.rs Re-export Pattern
```rust
// Source: Rust rustdoc re-exports guide
// Private modules — rustdoc inlines automatically
mod spec;
mod behavior;
mod outcome;

pub use crate::spec::EngineSpec;
pub use crate::behavior::{Behavior, BehaviorResult};
pub use crate::outcome::{EngineError, Frame, Outcome};
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate generic params `<S, I, D, T, O, K>` | `EngineSpec` associated-type bundle | v0.5.0 redesign | Eliminates generic explosion across function signatures |
| Behavior state stored engine-internally | Behavior state in `E::State` tree | v0.5.0 redesign | Undo/redo correctness, CoW semantics |
| Memory address as ordering tiebreaker | `(order_key, behavior_name)` | v0.5.0 redesign | Deterministic behavior ordering |

**Deprecated/outdated:**
- v0.4.0 `Behavior` trait structure: had internal state, different ordering contract — do not port forward

---

## Open Questions

1. **`OrderKey: Clone` — needed in Phase 2?**
   - What we know: `Ord` is sufficient for comparison; `order_key()` returns by value
   - What's unclear: Whether Phase 2's sort implementation will need to store keys separately or compare inline
   - Recommendation: Keep `OrderKey: Ord` only in Phase 1. Phase 2 researcher/planner can add `Clone` if sorting pattern requires it.

2. **`std::error::Error` on `EngineError` — timing**
   - What we know: Explicitly deferred from Phase 1
   - What's unclear: Whether Phase 4 examples want `?` propagation on `EngineError`
   - Recommendation: Add in Phase 4 or as a Phase 1 amendment when examples are written

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-01 | `EngineSpec` with all 6 associated types compiles with no warnings | unit (compile) | `cargo test --lib -- core::spec` | ❌ Wave 0 |
| CORE-02 | `Behavior<E>` impl compiles; `evaluate` cannot mutate `state` | unit (compile) | `cargo test --lib -- core::behavior` | ❌ Wave 0 |
| CORE-03 | `BehaviorResult::Continue(vec![])` and `BehaviorResult::Stop(x)` constructable and matchable | unit | `cargo test --lib -- core::behavior` | ❌ Wave 0 |
| CORE-04 | All 7 `Outcome` variants constructable and pattern-matchable | unit | `cargo test --lib -- core::outcome` | ❌ Wave 0 |
| CORE-05 | `EngineError` is a separate type; `#[non_exhaustive]` present; `BehaviorPanic`, `InvalidState`, `CorruptHistory` exist | unit | `cargo test --lib -- core::outcome` | ❌ Wave 0 |

**Note on CORE-02 mutation prevention:** The "test" is that the code compiles — the compiler enforces `&E::State` immutability structurally. A doc-test or unit test that tries to mutate `state` in `evaluate` should fail to compile (negative test). This is best expressed as a `compile_fail` doc-test in the `Behavior` trait rustdoc.

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** `cargo test` green + `cargo clippy -- -D warnings` clean before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/core_types.rs` — integration-level compile tests for CORE-01 through CORE-05
- [ ] `src/lib.rs` doc-test for `compile_fail` mutation attempt (CORE-02)
- [ ] Individual `#[cfg(test)]` modules inside `src/spec.rs`, `src/behavior.rs`, `src/outcome.rs`

*(No test framework installation needed — `cargo test` is built-in. No `proptest` setup needed for Phase 1.)*

---

## Sources

### Primary (HIGH confidence)
- Rust Reference — type_system attributes (`#[non_exhaustive]`): https://doc.rust-lang.org/reference/attributes/type_system.html
- Rust Reference — diagnostics attributes (`#[must_use]`): https://doc.rust-lang.org/reference/attributes/diagnostics.html
- Rust Reference — traits, object safety: https://doc.rust-lang.org/reference/items/traits.html#object-safety
- Rustdoc guide — re-exports and inlining: https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html
- ARCHITECTURE.md (authoritative design document, in-repo)
- 01-CONTEXT.md (locked decisions, in-repo)

### Secondary (MEDIUM confidence)
- quinedot.github.io dyn safety guide — verified against Rust Reference: https://quinedot.github.io/rust-learning/dyn-safety.html
- Rust 1.85.0 release notes (Rust 2024 edition changes): https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero external deps; all built-in Rust features
- Architecture: HIGH — locked in CONTEXT.md, verified against Rust reference
- Pitfalls: HIGH — verified against Rust Reference (dyn safety, `#[non_exhaustive]`, module visibility)
- Dyn safety of `Behavior<E>`: HIGH — verified against Rust Reference object safety rules

**Research date:** 2026-03-13
**Valid until:** 2026-09-13 (stable Rust features; not fast-moving)
