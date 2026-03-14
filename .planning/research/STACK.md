# Stack Research

**Domain:** Rust deterministic turn-based game engine library
**Researched:** 2026-03-13
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 2024 edition (1.85+) | Language | 2024 edition is now stable; brings improved RPIT lifetime capture rules (less `+ '_` noise in trait-heavy APIs), `#[diagnostic::do_not_recommend]` for cleaner library error messages, and async closure traits in prelude. Zero-cost abstractions and ownership semantics are the right tool for a zero-dependency state engine. |
| Cargo | Bundled with toolchain | Build system | No alternative. Use workspaces if examples grow. |
| rustfmt | Bundled | Code formatting | Enforce via `cargo fmt --check` in CI. Use 2024 edition-aware config. |
| clippy | Bundled | Lint pass | `cargo clippy -- -D warnings` catches iterator anti-patterns, unnecessary clones, and redundant trait bounds before review. |

### Supporting Libraries (dev-dependencies only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| proptest | 1.x | Property-based testing | Testing dispatch determinism invariants: same input + same state always produces same outcome. Generates thousands of random inputs and shrinks failures automatically. |
| proptest-state-machine | 0.x | State machine property testing | Models the engine as a reference state machine and checks invariants (undo/redo stack size, committed state consistency) across arbitrary operation sequences. Use when unit tests cover known cases but you want coverage of unknown interaction sequences. |

No runtime dependencies. Zero. This is a hard constraint per PROJECT.md.

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo test | Unit and integration test runner | Prefer `#[cfg(test)]` modules inside `src/` for unit tests; `tests/` for integration tests that exercise the public API as a consumer would. |
| cargo doc | API documentation | Write doc comments on all public types. Doc-tests serve as usage examples and are run by `cargo test`. |
| cargo bench / criterion | Microbenchmarks (optional) | Only if CoW performance needs validation against eager-clone baseline. criterion is a dev-dependency only. |

## Rust Pattern Recommendations

### Static Dispatch via Generics — Use This

The behavior set is statically known at compile time. Use generic parameters, not `Box<dyn Behavior>`.

```rust
pub struct Engine<S, I, D, O, K, B>
where
    B: BehaviorSet<S, I, D, O, K>,
{ ... }
```

**Why:** Monomorphization eliminates vtable dispatch overhead, enables inlining across behavior calls, and lets the compiler verify at compile time that the behavior set satisfies all required trait bounds. The ARCHITECTURE.md is explicit: "minimize dynamic dispatch." A benchmark in the ecosystem shows static dispatch running ~3x faster than dynamic dispatch over large iteration counts — this matters for AI look-ahead that dispatches the same input thousands of times.

**Tradeoff:** Binary size grows with monomorphization. Acceptable here because the behavior set is small and fixed.

### Trait Definition for Behavior

```rust
pub trait Behavior<S, I, D, O, K: Ord> {
    fn name(&self) -> &'static str;
    fn order_key(&self) -> K;
    fn evaluate(&self, input: &I, state: &S) -> BehaviorResult<D, O>;
}
```

`&'static str` for `name()` avoids allocation and makes the name usable as a `HashMap` key or sort key without cloning. `K: Ord` enables the `(order_key, name)` sort without any runtime infrastructure.

**Do not** make `evaluate` take `&mut self`. Behaviors read state through the working state parameter; they do not own mutable state. This is central to CoW correctness.

### BehaviorSet as a Trait Over a Tuple

To dispatch over a heterogeneous but statically known collection of behaviors, represent the set as a trait implemented on tuples:

```rust
pub trait BehaviorSet<S, I, D, O, K: Ord> {
    fn dispatch(
        &self,
        input: &I,
        working: &mut WorkingState<S>,
        trace: &mut Vec<T>,
    ) -> DispatchResult<D, O>;
}

impl<S, I, D, O, K, B1, B2> BehaviorSet<S, I, D, O, K> for (B1, B2)
where
    B1: Behavior<S, I, D, O, K>,
    B2: Behavior<S, I, D, O, K>,
    K: Ord,
{ ... }
```

Implement this for tuples up to a reasonable arity (8–12 behaviors covers most games). Macros can generate the impls. This is 100% static dispatch with no HList complexity or external dependencies.

**Why not frunk/HList:** frunk adds a dependency and significant complexity for a gain (arbitrary arity) that the use case does not require. Most games have fewer than 12 behaviors. Tuple-based dispatch is idiomatic, dependency-free, and produces clear error messages.

**Why not `Vec<Box<dyn Behavior>>`:** Runtime registration, heap allocation per behavior, vtable overhead on every dispatch call, and non-deterministic ordering unless explicitly maintained. All of these contradict the architecture.

### CoW Working State — How to Model It

`std::borrow::Cow` is not the right tool here. `Cow<'a, T>` handles the `Borrowed` / `Owned` split but requires `T: ToOwned` and is designed for string/slice optimization, not arbitrarily structured composite state.

**Use a custom `WorkingState<S>` wrapper instead:**

```rust
pub struct WorkingState<'a, S> {
    committed: &'a S,
    dirty: Option<S>,
}

impl<'a, S: Clone> WorkingState<'a, S> {
    pub fn new(committed: &'a S) -> Self {
        Self { committed, dirty: None }
    }

    pub fn read(&self) -> &S {
        self.dirty.as_ref().unwrap_or(self.committed)
    }

    pub fn write(&mut self) -> &mut S {
        if self.dirty.is_none() {
            self.dirty = Some(self.committed.clone());
        }
        self.dirty.as_mut().unwrap()
    }

    pub fn into_dirty(self) -> Option<S> {
        self.dirty
    }
}
```

**Why this pattern:**
- Reads are zero-cost until first write (no clone on the happy path for `NoChange` dispatches)
- First write triggers exactly one clone of the whole state, or a substate if you add substate-level granularity
- No `unsafe`, no external crates
- `into_dirty()` returning `Option<S>` naturally represents the `NoChange` case: if `None`, nothing was written
- The lifetime `'a` ties the borrow of committed state to the dispatch call, preventing committed state from being mutated through the working state

**Substate granularity extension:** For large states where AI look-ahead dispatches thousands of times, add substate-level CoW by wrapping each substate:

```rust
pub struct SubCow<'a, T> {
    committed: &'a T,
    dirty: Option<T>,
}
```

Apply to `domain` and `behaviors` substates independently. Only the touched substate is cloned per dispatch.

### Associated Types vs Generic Parameters

Use **associated types** for the diff and outcome types on `Engine` because there is exactly one diff type and one outcome type for a given engine instantiation — not multiple. Generic parameters are appropriate when a function or trait is generic over any type satisfying a bound; associated types express a 1:1 relationship.

```rust
// Good: associated types for single-valued relationships
pub trait ApplyDiff {
    type State;
    fn apply(&self, state: &mut Self::State);
}

// Good: generic parameter when truly polymorphic
pub fn sorted_behaviors<K: Ord>(behaviors: &mut Vec<impl Behavior<K>>) { ... }
```

### GATs — Not Needed for MVP

Generic Associated Types (stable since Rust 1.65) are the right tool for lending iterators, zero-copy parsers, and lifetime-parameterized associated types. This engine does not need them for the MVP. If a future `NeedsChoice` feature requires returning references into behavior state from `evaluate`, revisit GATs then.

### const Generics — Limited Use

Stable const generics (since Rust 1.51) can encode the number of behaviors in a fixed-size behavior array. However, the tuple approach above is simpler and more ergonomic. Only reach for const generics if you need fixed-size arrays with compile-time length guarantees (e.g., a fixed-size history ring buffer). Complex arithmetic over const generics (`generic_const_exprs`) remains unstable — do not use it.

### PhantomData — Use for Marker Safety, Not Complexity

`PhantomData<K>` is useful to tie a phantom ordering key type to a struct that does not actually store a `K` at runtime, preserving variance and drop-check correctness. Use it when:
- A struct is generic over a type it does not own
- You need to enforce invariants about unused type parameters

Do not use it to build elaborate type-state machines for the dispatch flow. The engine's state transitions are runtime values, not type-state transitions; attempting to encode them in types creates ergonomic burden without safety benefit.

### Ordering Implementation

The `(order_key, behavior_name)` sort must be deterministic. Implement it by collecting behaviors into a `Vec` of `(K, &'static str, &dyn Fn(...) -> ...)` tuples sorted by `(K, &'static str)`, or drive the sort through the `BehaviorSet` trait implementation itself. Since the behavior set is static, the sort can be done once at engine construction time and the sorted order cached.

**Do not** use pointer addresses as tiebreakers. Pointer-address ordering is non-deterministic across builds, runs, and platforms. The v0.4.0 bug was exactly this.

## Installation

```toml
# Cargo.toml
[package]
edition = "2024"

[dependencies]
# None — zero runtime dependencies

[dev-dependencies]
proptest = "1"
proptest-state-machine = "0.3"
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Tuple-based static `BehaviorSet` | `Vec<Box<dyn Behavior>>` | Only if behaviors are dynamically registered at runtime (explicitly out of scope for this project) |
| Custom `WorkingState<S>` CoW | `std::borrow::Cow<'a, S>` | `std::Cow` is appropriate for string/slice optimization; not designed for composite structured state |
| `proptest` | `quickcheck` | quickcheck is a viable alternative; proptest has better shrinking and a more composable strategy API. Either works for invariant testing. |
| Tuple-based dispatch | `frunk` HList | HList is justified when arity is truly unbounded and you need type-level recursion. Adds a dependency and significant learning curve. |
| `&'static str` for behavior name | `String` | `String` requires heap allocation; behavior names are compile-time constants. |
| Associated types on `ApplyDiff` | Multiple generic params | Multiple generic params express "any combination" relationships; associated types express "for this type, there is exactly one" — cleaner for Diff. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `Box<dyn Behavior>` as design center | Heap allocation per behavior, vtable lookup on every dispatch call, requires `dyn`-safe trait (restrictions on associated types), prevents inlining, breaks deterministic ordering guarantees unless manually maintained | Tuple-based `BehaviorSet<...>` with static dispatch |
| `enum_dispatch` crate | Adds a dependency for a pattern that is achievable with native Rust enums and match. The behavior set is not a closed enum of fixed variants the library knows about — it is user-defined. | Implement `BehaviorSet` on user-defined tuples |
| `Arc<dyn Behavior>` / shared behavior references | Introduces interior mutability temptation, reference counting overhead, and makes behavior state ambiguous | Keep behavior state in the main state tree; behaviors are zero-sized or immutable rule structs |
| Mutable `&mut self` in `evaluate` | Allows behaviors to bypass the CoW working state and mutate directly, breaking undo/redo correctness and serializability | `&self` in `evaluate`; all state changes go through emitted diffs |
| Memory address ordering tiebreaker | Non-deterministic across runs, builds, and platforms (the v0.4.0 regression) | `(order_key, behavior_name)` with `&'static str` name |
| `generic_const_exprs` (nightly feature) | Unstable, API will change | Use stable const generics or runtime sorting |
| `unsafe` for CoW performance | Not needed; `WorkingState<S>` pattern achieves CoW without unsafe, and the performance gain does not justify the safety cost in a library | `WorkingState<'a, S>` with `Option<S>` dirty slot |

## Stack Patterns by Variant

**If the state struct is large (many substates, large board representation):**
- Use substate-level `SubCow<'a, T>` wrappers on each substate field
- Only clone the substates actually written during a dispatch
- Keep the top-level `WorkingState` as a struct of `SubCow` fields, not a single `Option<S>`

**If the number of behaviors exceeds 12:**
- Consider a macro to generate `BehaviorSet` impls for larger tuples
- Or use a `Vec` of behavior references sorted at construction time, accepting the dynamic dispatch cost as a conscious tradeoff
- Do not add frunk as a dependency just to handle large arity

**If behaviors need to be composed from multiple sources (game + expansion pack):**
- Nest tuples: `((CoreBehavior1, CoreBehavior2), (ExpansionBehavior1,))`
- Implement `BehaviorSet` for nested tuples via blanket impl
- This preserves static dispatch without requiring runtime registration

**If no_std compatibility is desired in future:**
- Avoid `std::collections::HashMap` in the engine core; prefer sorted `Vec` for behavior ordering
- Use `alloc` types (`Vec`, `String`) rather than `std`-only types
- The `WorkingState` pattern as described requires only `Clone` and `Option`, both in `core`

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| Rust 2024 edition | Rust 1.85.0+ | 2024 edition shipped in 1.85 (Feb 2025). Requires `edition = "2024"` in Cargo.toml. |
| proptest 1.x | Rust stable 1.65+ | GATs stabilized in 1.65 are used internally by proptest. No conflict. |
| proptest-state-machine 0.3 | proptest 1.x | Re-exports proptest; pin to matching major version. |

## Sources

- [Announcing Rust 1.85.0 and Rust 2024 — Rust Blog](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) — Rust 2024 edition feature list; RPIT lifetime capture, diagnostic attributes, async closures in prelude. HIGH confidence.
- [std::borrow::Cow — Rust std docs](https://doc.rust-lang.org/std/borrow/enum.Cow.html) — Confirmed Cow semantics and ToOwned requirements. HIGH confidence.
- [Generic associated types stable in Rust 1.65 — Rust Blog](https://blog.rust-lang.org/2022/10/28/gats-stabilization/) — GAT stability and use cases. HIGH confidence.
- [proptest-state-machine — crates.io](https://crates.io/crates/proptest-state-machine) — State machine testing API; ReferenceStateMachine trait. MEDIUM confidence (API details from crates.io, not official docs).
- [Rust Static vs. Dynamic Dispatch — SoftwareMill](https://softwaremill.com/rust-static-vs-dynamic-dispatch/) — Performance benchmark (64ms vs 216ms); vtable overhead explanation. MEDIUM confidence (single source, benchmark is illustrative not definitive).
- [PhantomData — Rust std docs](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) — Variance and drop-check semantics. HIGH confidence.
- [Rust 2024 edition RFC 3498 — lifetime capture rules](https://rust-lang.github.io/rfcs/3498-lifetime-capture-rules-2024.html) — RPIT lifetime capture semantics for `impl Future` return types. HIGH confidence.
- WebSearch results on HList/frunk — confirmed frunk is a real dependency, not std. MEDIUM confidence; recommendation to avoid it is based on project constraint of zero dependencies.

---
*Stack research for: HerdingCats — Rust deterministic turn-based state engine library*
*Researched: 2026-03-13*
