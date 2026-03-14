# Project Research Summary

**Project:** HerdingCats
**Domain:** Rust deterministic turn-based game engine library
**Researched:** 2026-03-13
**Confidence:** HIGH

## Executive Summary

HerdingCats is a zero-dependency Rust library that provides a deterministic, ordered-behavior dispatch engine for turn-based games. Game developers implement the `Behavior` trait on their rule objects and register them with the engine; the engine drives ordered evaluation, atomic state commits, and undo/redo. The v0.4.0 release had three documented correctness bugs — behavior state stored outside the main state tree, memory-address-based ordering tiebreakers, and eager full state clones — and v0.5.0 must fix all three at the architectural level before adding any new features. The research consensus is clear: the fixes are design-level changes, not patches. Every pitfall that bit v0.4.0 has a compile-time-enforced prevention in Rust.

The recommended approach is to build the library in strict type-dependency order: `BehaviorResult` → `Behavior` trait → `Frame` → `Outcome`/`EngineError` → `WorkingState<S>` CoW wrapper → `History` → `Engine`. This order ensures each layer is independently testable before the next is layered on. The two highest-risk decisions that must be made in Phase 1 are (a) bundling the five engine type parameters into a single `EngineSpec` associated-type trait to prevent API ergonomic collapse, and (b) ensuring `Behavior::evaluate()` receives only `&S`, not `&mut S`, which structurally prevents all direct state mutation by behaviors.

The primary risk is over-complicating static dispatch. Research examined `Vec<Box<dyn Behavior>>` versus tuple-based static dispatch and found that `Vec<Box<dyn Behavior>>` is the correct MVP choice: fixed at construction, never modified at runtime, vtable overhead negligible for turn evaluation, and ergonomically straightforward. Tuple-based static dispatch is a post-MVP optimization if benchmarks demand it. The two working examples — tic-tac-toe (minimal) and backgammon (exercises dice-roll irreversibility) — must both be fully implemented in v0.5.0 to validate the public API against real game logic.

## Key Findings

### Recommended Stack

HerdingCats requires Rust 2024 edition (1.85+) which brings improved RPIT lifetime capture rules and cleaner library error messages — directly useful for a trait-heavy API. There are zero runtime dependencies; this is a hard architectural constraint. Dev dependencies are `proptest` (1.x) for invariant testing and `proptest-state-machine` (0.3) for modeling the engine as a reference state machine and verifying operation-sequence invariants. No other tooling is required.

The critical language-level patterns are: (1) `WorkingState<'a, S>` with a `Borrowed`/`Owned` enum (not `std::borrow::Cow`, which requires `ToOwned` and is designed for string/slice scenarios); (2) `(order_key, behavior_name)` as the deterministic composite sort key — never pointer addresses; (3) `&'static str` for behavior names to avoid heap allocation and enable stable ordering across compilations.

**Core technologies:**
- Rust 2024 edition (1.85+): language — 2024 edition RPIT and diagnostic attributes reduce noise in trait-heavy library APIs
- Cargo workspaces: build system — no alternative; use workspaces if examples grow into a separate crate
- proptest + proptest-state-machine: dev-only testing — property-based invariant testing; generates thousands of random inputs and shrinks failures automatically
- rustfmt + clippy: formatting and lint — enforce via CI; clippy `-D warnings` catches iterator anti-patterns and unnecessary clones before review

### Expected Features

The v0.5.0 feature set is authoritative and complete. The `Behavior` trait, `BehaviorResult`, `Outcome`, `EngineError`, `Frame`, `dispatch`, `undo`, `redo`, CoW working state, and irreversibility boundary are all P1 — none can be deferred. The two examples are also P1 because they validate the API against real game logic. Everything else (history replay API, `NeedsChoice`, derive macros, DSL) defers to v0.5.x or later.

**Must have (table stakes):**
- `Behavior` trait with `name() -> &'static str`, `order_key() -> K`, `evaluate(&I, &S) -> BehaviorResult<D, O>` — the user's only extension point
- `BehaviorResult<D, O>`: `Continue(Vec<D>)` / `Stop(O)` — behavior contract
- `dispatch(input, reversibility) -> Result<Outcome, EngineError>` — atomic, ordered, immediate-diff, trace-generating
- `Outcome` enum: `Committed`, `Undone`, `Redone`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted`
- `EngineError` strictly for engine-internal failures — never domain outcomes
- `Frame<I, D, T>`: canonical committed transition record with input, diff, trace
- CoW `WorkingState<S>`: no eager clone; lazy copy on first write
- `undo()` / `redo()` with explicit `Reversibility` argument on dispatch
- Tic-tac-toe example (minimal, full API surface)
- Backgammon example (exercises dice-roll irreversibility)

**Should have (competitive):**
- Irreversibility boundary: explicit `Reversibility` param on `dispatch()`, erases undo/redo history on irreversible commit — compile-time forced choice, not an opt-in default
- Diff-as-record in Frame: enables replay, network sync, and time-travel debugging
- `EngineError` distinct from domain outcomes: `Result<Outcome, EngineError>` with `#[non_exhaustive]` on `EngineError`
- Live working state visible to later behaviors: immediate diff application, not deferred batching
- Zero runtime dependencies: embeddable in WASM, server-side AI workers, embedded systems

**Defer (v0.5.x+):**
- `NeedsChoice` outcome / interactive dispatch branching — requires suspending and resuming dispatch state; needs concrete use case before design
- History replay / frame iterator API — add when replay or time-travel debugging is requested
- Derive macros for `Diff` — add when users report boilerplate fatigue
- DSL / card-text compilation — long-term direction; requires mature behavior model first

### Architecture Approach

The engine is a thin orchestrator: it owns dispatch logic, behavior ordering, CoW working state, and history stacks. All domain types are supplied via generic type parameters bundled under a single `EngineSpec` associated-type trait. The module structure is flat — `lib.rs` (re-exports only), `behavior.rs`, `outcome.rs`, `frame.rs`, `working_state.rs`, `history.rs`, `engine.rs` — with no subfolders needed at MVP scale. The type dependency graph is acyclic and each layer is independently testable.

The critical architectural decision is the undo strategy: snapshot-based (store `prior_state: S` alongside each `Frame`) rather than reverse-diff (requires a `Reversible` trait on user diff types). Snapshot undo has no additional trait requirements on user types, is trivially correct, and state size for turn-based games is small. Behaviors are stored in a `Vec<Box<dyn Behavior>>` sorted once at `Engine::new()` — dynamic dispatch at the behavior collection is the accepted MVP tradeoff.

**Major components:**
1. `Engine<G: EngineSpec>` — owns committed state + history; exposes `dispatch`, `undo`, `redo`
2. `WorkingState<'a, S>` — CoW wrapper: `Borrowed(&S)` until first write, `Owned(S)` after; dropped on failure, moved into committed on success
3. `Behavior` trait — `name()`, `order_key()`, `evaluate(&I, &S)` — user extension point; immutable evaluate enforced by signature
4. `History<I, D, T>` — two `Vec<HistoryEntry>` stacks (undo + redo); private; query-only API
5. `Frame<I, D, T>` — canonical committed record: input + diff vec + trace vec
6. `Outcome<F, N>` + `EngineError` — `Result<Outcome, EngineError>` where `Ok` carries all domain outcomes including rejected ones

### Critical Pitfalls

1. **Generic type parameter explosion** — bundle `S, I, D, T, O, K` into a single `EngineSpec` associated-type trait from the first commit; retrofitting is a breaking API change
2. **Behavior state outside the main state tree** — the `Engine` struct must hold no mutable behavior configuration; enforce at the type level: behaviors are stateless rule objects
3. **Memory address as ordering tiebreaker** — sort must use `(order_key, behavior_name)` tuple; `name()` returns `&'static str`; never use pointer values for comparison
4. **Eager full state clone instead of CoW** — `WorkingState<'a, S>` with `Borrowed`/`Owned` enum; `dispatch()` must not contain `state.clone()` before the first diff application
5. **Outcome/EngineError conflation** — `Disallowed`, `InvalidInput`, `Aborted` belong in `Outcome`; `EngineError` is strictly for engine-internal impossible states; mark `#[non_exhaustive]`
6. **Irreversibility with no compile-time enforcement** — `dispatch()` takes an explicit `Reversibility` parameter; callers cannot forget to declare it

## Implications for Roadmap

Based on research, the type dependency graph is the natural phase boundary. Each phase must be fully correct before the next begins because later types depend on earlier ones and retrofitting is expensive (some changes are breaking API changes).

### Phase 1: Core Types and Trait Contracts

**Rationale:** The entire library depends on these types. `EngineSpec`, `BehaviorResult`, `Behavior` trait, `Frame`, `Outcome`, and `EngineError` must be designed together because getting them wrong requires breaking API changes. Five of the ten pitfalls are prevention-phase 1: generic explosion, behavior state placement, memory-address ordering, Outcome/EngineError conflation, behaviors mutating working state, and HashMap non-determinism. Fix them all in the type signatures before any logic is written.

**Delivers:** A compilable crate with all public types defined, documented with rustdoc, and exercised in doc-tests. No dispatch logic yet.

**Addresses:** `Behavior` trait, `BehaviorResult`, `Outcome`, `EngineError`, `Frame<I, D, T>`, `EngineSpec` bundling

**Avoids:** Generic type parameter explosion (Pitfall 5), Outcome/EngineError conflation (Pitfall 8), behaviors mutating working state (Pitfall 9), HashMap non-determinism (Pitfall 10), memory address ordering (Pitfall 2)

### Phase 2: CoW Working State and Dispatch Algorithm

**Rationale:** `WorkingState<'a, S>` is the most mechanically complex component and is a prerequisite for a correct dispatch loop. Dispatch must be implemented together with CoW to avoid the eager-clone anti-pattern becoming permanent. The `dispatch_preview()` dirty-preview anti-pattern (Pitfall 4) must also be addressed here: there is no separate preview path; discarded dispatches are the preview.

**Delivers:** `dispatch(input, reversibility) -> Result<Outcome, EngineError>` — atomic, ordered, immediate-diff, trace-generating. `WorkingState` fully implemented and unit tested.

**Uses:** Rust 2024 edition lifetime capture rules (relevant to `WorkingState<'a, S>` lifetime); `proptest` for dispatch determinism invariants

**Implements:** `WorkingState`, `Engine::dispatch()`, behavior ordering (sorted once at construction), `Apply<S>` and `Traced<T>` traits on user diff types

**Avoids:** Eager full state clone (Pitfall 3), dirty preview side effects (Pitfall 4)

### Phase 3: Undo, Redo, and Irreversibility

**Rationale:** Undo/redo are mechanical inverses of dispatch but introduce new design decisions: snapshot vs reverse-diff, public vs private stack API, and the compile-time enforcement of irreversibility. All three must be decided together because they affect `dispatch()`'s signature (the `Reversibility` parameter). The undo stack must be private from the first commit (Pitfall 6).

**Delivers:** `undo()` and `redo()` with full `Undone`/`Redone`/`Disallowed` outcomes. `Reversibility` parameter on `dispatch()`. `History` struct with private stacks and query-only API. Irreversibility boundary clears both stacks on irreversible commit.

**Implements:** `History<I, D, T>`, `HistoryEntry` with `prior_state` snapshot, `Engine::undo()`, `Engine::redo()`, `Engine::mark_irreversible()` / dispatch signature update

**Avoids:** Public undo/redo stacks (Pitfall 6), irreversibility with no enforcement (Pitfall 7)

### Phase 4: Working Examples (Tic-Tac-Toe + Backgammon)

**Rationale:** Examples are P1, not documentation afterthoughts. They are the integration test that validates whether the public API is actually usable for real games. Tic-tac-toe exercises the minimal happy path; backgammon exercises dice-roll irreversibility, multiple legal move generation, and the full undo/redo contract. These cannot be deferred — the API's ergonomic problems only surface when a real game is implemented.

**Delivers:** Two fully working example games in `examples/`. Both compile, run, and demonstrate the full API surface. Backgammon demonstrates irreversibility clearing undo history after a dice roll.

**Addresses:** Tic-tac-toe example (P1), Backgammon example (P1)

### Phase Ordering Rationale

- Phase 1 must precede all other phases because type changes after the API is in use are breaking changes
- Phase 2 (dispatch) must follow Phase 1 (types) because the dispatch algorithm is parameterized by those types
- Phase 3 (undo/redo) must follow Phase 2 (dispatch) because the undo stack is populated by dispatch; both must share the `Reversibility` parameter on `dispatch()`
- Phase 4 (examples) comes last to validate all prior phases; early examples would need to be rewritten as the API evolves
- This ordering exactly matches the type dependency graph in ARCHITECTURE.md — no circular dependencies, each phase independently testable

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2:** The `Apply<S>` and `Traced<T>` trait design has ergonomic implications for users; the exact trait bounds and method signatures need validation against the backgammon use case before being finalized
- **Phase 3:** The snapshot undo strategy is recommended but the memory implications for long-running sessions should be quantified; if AI lookahead dispatches thousands of times, history pruning may be needed

Phases with standard patterns (skip research-phase):
- **Phase 1:** Type design in Rust is well-documented; `EngineSpec` associated-type bundling is a standard library pattern with no unknowns
- **Phase 4:** Example implementation follows directly from API; no research needed

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Rust 2024 edition, proptest, zero-dependency constraint all verified against official sources; no speculative dependencies |
| Features | HIGH | Feature set derived from authoritative ARCHITECTURE.md and PROJECT.md specs; competitor analysis (boardgame.io, Asmodee) corroborates feature choices |
| Architecture | HIGH | Architecture spec is authoritative in repository; all Rust patterns verified against std docs; CoW and static dispatch patterns are well-established |
| Pitfalls | HIGH | Nine of ten pitfalls are grounded in v0.4.0 known issues from CONCERNS.md; one (generic explosion) is derived from Rust community consensus |

**Overall confidence:** HIGH

### Gaps to Address

- **`NeedsChoice` design:** Deferred to v0.5.x but the backgammon bearing-off example may surface this requirement; flag during backgammon implementation and decide whether to design the suspension model before v0.5.0 ships
- **`EngineSpec` ergonomics:** The associated-type bundling approach eliminates turbofish at construction but may make trait bounds verbose in user impl blocks; validate against real game code in Phase 4 and document type alias patterns
- **History pruning policy:** Snapshot undo stores one state clone per committed frame; long sessions or AI-heavy games may need a max-history-depth setting; out of scope for MVP but worth noting as a v0.5.x concern

## Sources

### Primary (HIGH confidence)
- ARCHITECTURE.md (repository root) — authoritative v0.5.0 design spec
- PROJECT.md (`.planning/PROJECT.md`) — validated requirements and out-of-scope items
- CONCERNS.md — v0.4.0 known issues (behavior lifetimes, address tiebreaker, eager clone, public stacks)
- [Announcing Rust 1.85.0 and Rust 2024 — Rust Blog](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) — 2024 edition features
- [std::borrow::Cow — Rust std docs](https://doc.rust-lang.org/std/borrow/enum.Cow.html) — Cow semantics and ToOwned requirements
- [PhantomData — Rust std docs](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) — variance and drop-check semantics
- [Generic associated types stable in Rust 1.65 — Rust Blog](https://blog.rust-lang.org/2022/10/28/gats-stabilization/) — GAT stability and pitfalls

### Secondary (MEDIUM confidence)
- [proptest-state-machine — crates.io](https://crates.io/crates/proptest-state-machine) — state machine testing API
- [Rust Static vs. Dynamic Dispatch — SoftwareMill](https://softwaremill.com/rust-static-vs-dynamic-dispatch/) — performance benchmark (~3x difference; illustrative)
- [boardgame.io documentation](https://boardgame.io/documentation/) — competitor feature analysis
- [Asmodee Rules Engine Architecture](https://doc.asmodee.net/rules-engine) — input validation and event sourcing patterns
- [Item 12: Understand the trade-offs between generics and trait objects — Effective Rust](https://www.lurklurk.org/effective-rust/generics.html) — static vs dynamic dispatch ergonomics
- [undo crate — docs.rs](https://docs.rs/undo) — existing Rust undo/redo patterns
- [Transactional Operations in Rust](https://fy.blackhats.net.au/blog/2021-11-14-transactional-operations-in-rust/) — atomicity pitfalls
- [Rusty Garbage: My HashMap is non-deterministic — Medium](https://medium.com/@draft1967/rusty-garbage-my-hashmap-is-non-deterministic-0e518be0c5c6) — HashMap non-determinism in game state

### Tertiary (LOW confidence)
- [WebSearch results on HList/frunk] — confirmed frunk is a real dependency; recommendation to avoid is based on zero-dependency project constraint

---
*Research completed: 2026-03-13*
*Ready for roadmap: yes*
