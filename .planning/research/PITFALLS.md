# Pitfalls Research

**Domain:** Rust deterministic turn-based game engine library (CoW working state, static generics, trait-based behaviors, undo/redo)
**Researched:** 2026-03-13
**Confidence:** HIGH (grounded in v0.4.0 known issues + verified Rust ecosystem patterns)

---

## Critical Pitfalls

### Pitfall 1: Behavior State Living Outside the Main State Tree

**What goes wrong:**
Behavior lifecycle data (enabled/disabled flags, counters, cooldowns, remaining uses) lives as mutable engine-internal fields rather than inside the user's `State` struct. Undo/redo restores committed state but leaves behavior internal state at the post-undo value. The behavior and the game state are now diverged. Serializability breaks entirely — you cannot snapshot the game and restore it faithfully.

**Why it happens:**
It feels natural to put "engine machinery" inside the engine struct. Behavior management feels like infrastructure, not game state. The mistake is treating behavior configuration as separate from the state it affects.

**How to avoid:**
Enforce this at the type level: the engine should hold no mutable behavior configuration. All behavior-meaningful state lives in `S` (the user's state type). Behaviors are stateless objects whose only identity is their `name()` and `order_key()` values. Any effect they have on their own "lifetime" or "enabled" status must flow through a `Diff` applied to `S`.

**Warning signs:**
- Engine struct has fields like `enabled: HashSet<&'static str>`, `lifetimes: HashMap<...>`, or similar
- Undo test passes for domain state but behavior counters are wrong after undo
- State snapshot/restore produces different behavior outcomes than the original run

**Phase to address:**
Phase 1 (Core types and engine skeleton) — this invariant must be encoded in the type structure from the first commit; retrofitting is a rewrite.

---

### Pitfall 2: Memory Address as Ordering Tiebreaker

**What goes wrong:**
When two behaviors share the same `order_key`, execution order is determined by memory address (pointer comparison on boxed trait objects). This is non-deterministic across runs, compilations, and platforms. Replay, save/load, and cross-machine tests all silently produce different results. The engine claims to be deterministic but is not.

**Why it happens:**
When implementing sorting on trait objects, comparing `Box<dyn Behavior>` pointers is the path of least resistance — it compiles and appears to produce stable ordering within a single run. The non-determinism only manifests across runs.

**How to avoid:**
Use `(order_key, behavior_name)` as the composite sort key. `behavior_name()` must return a `&'static str` that is stable across compilations (string literal, not runtime-computed). Sort by `(order_key, name)` using `Ord`/`PartialOrd` on a tuple. Never touch pointer values for ordering.

**Warning signs:**
- Behavior trait lacks a `name()` method, or `name()` returns a heap-allocated `String` rather than `&'static str`
- Sort comparator accesses pointer values
- Replay test fails intermittently across runs
- Two behaviors with same `order_key` produce different outcomes after `cargo build --release` vs debug

**Phase to address:**
Phase 1 (Core types) — the `Behavior` trait definition must include `name() -> &'static str` and `order_key() -> K` from the start.

---

### Pitfall 3: Eager Full State Clone Instead of True CoW

**What goes wrong:**
The engine clones the entire state on every dispatch to create the working state, even for large state trees. AI look-ahead (calling dispatch speculatively dozens of times per turn) becomes catastrophically slow. For a game with a large board + many entities + behavior state, each speculative evaluation clones several kilobytes unnecessarily.

**Why it happens:**
`let mut working = self.state.clone()` is one line and obviously correct. True CoW requires more design: choosing what counts as a "substate," managing shared-vs-owned references. The eager clone is the MVP shortcut that stays forever.

**How to avoid:**
Design the CoW boundary explicitly at the type level. The recommended approach:
- Define `WorkingState<'a, S>` that holds a reference to committed state and a `Option<S>` for the dirty copy
- On first write, clone committed into the `Option` and all subsequent reads/writes use the clone
- On abort, drop the `Option` — committed state is untouched
- On commit, `take()` the `Option` and replace committed state

The substate granularity is user-defined but the engine API should make the CoW boundary visible as a type, not just a semantic convention.

**Warning signs:**
- `WorkingState` is just `S` (no reference to committed, no dirty tracking)
- `dispatch()` contains `state.clone()` before any behavior is evaluated
- Preview/speculative dispatch benchmarks scale linearly with state size rather than with the number of mutations

**Phase to address:**
Phase 2 (CoW working state) — this is a dedicated implementation concern and must not be deferred to "optimize later."

---

### Pitfall 4: `dispatch_preview()` with Side Effects (The Dirty Preview Anti-Pattern)

**What goes wrong:**
A "preview" or "dry run" dispatch that mutates behavior state (enabled flags, lifetimes, counters) even though it does not commit domain state. The preview appears to be read-only from the domain perspective but silently advances behavior bookkeeping. AI systems that call preview extensively corrupt behavior state without any indication. The bug is invisible until behavior interactions produce wrong results.

**Why it happens:**
The distinction between "speculative domain state" and "behavior metadata" is not enforced at the type level. If behavior management data lives in engine internals (see Pitfall 1), a preview can update those internals without touching the committed `State`. This looks like isolation at the domain level but is contamination at the behavior level.

**How to avoid:**
Eliminate the concept of a side-effecting preview. With Pitfall 1 already solved (behavior state in `S`), a true preview is just a dispatch that discards its `WorkingState` instead of committing it. There is no separate preview path — failed or discarded dispatches are the preview. The engine has one dispatch algorithm; commit or discard is the only variation.

**Warning signs:**
- A `dispatch_preview()` method exists separately from `dispatch()`
- Engine has separate "preview" vs "real" dispatch code paths
- Behavior state differs between test runs that use preview vs test runs that don't

**Phase to address:**
Phase 2 (Dispatch algorithm) — preview semantics must be defined before any dispatch code is written.

---

### Pitfall 5: Generic Type Parameter Explosion (The Five-Type-Param Engine)

**What goes wrong:**
The `Engine<S, I, D, O, K>` struct accumulates type parameters until instantiation requires specifying all five (or more). Library users write `Engine::<MyState, MyInput, MyDiff, MyOutcome, u32>::new(behaviors)` and the compiler demands turbofish syntax everywhere. Type inference fails in common patterns. The library becomes unusable without a macro or newtype wrapper. Third-party code that wraps the engine must re-propagate all type params.

**Why it happens:**
Each generic parameter feels independently justified. `S` is the state type. `I` is input. `D` is diff. `O` is non-committed outcome. `K` is order key. All legitimate. The explosion happens because they appear individually on every method, impl block, and trait bound. As each is added, the others feel fixed, so the accumulation isn't noticed until the API is tried from the consumer side.

**How to avoid:**
Use an associated-type trait to bundle related types. Define a `GameSpec` or `EngineSpec` trait:

```rust
pub trait EngineSpec {
    type State;
    type Input;
    type Diff;
    type Outcome;
    type OrderKey: Ord;
}
```

Then `Engine<G: EngineSpec>` has one type parameter. The user implements `EngineSpec` once for their game and never writes turbofish again. This also makes trait bounds readable: `where G: EngineSpec` rather than `where S: ..., I: ..., D: ..., O: ...`.

**Warning signs:**
- `Engine` struct has more than two type parameters
- Example code requires turbofish or explicit type annotations on every engine construction
- `impl<S, I, D, O, K> Behavior<S, I, D, O, K> for MyBehavior` appears in user code
- Compiler error messages contain five unresolved type variables simultaneously

**Phase to address:**
Phase 1 (Core types) — the `EngineSpec` bundling decision must happen before any other type is defined. Retrofitting associated-type bundling onto an existing five-param API is a breaking change.

---

### Pitfall 6: Undo Stack Exposed as Public Field

**What goes wrong:**
`undo_stack` and `redo_stack` are public fields on the engine struct. Tests and user code depend on direct field access. When the internal representation changes (from `Vec` to `VecDeque`, from frames to frame indices, from full copies to diffs), every consumer breaks. The "public field" becomes a defacto stable API that cannot be changed without breakage.

**Why it happens:**
Exposing fields is the quickest way to write tests that check stack depth or frame contents. There is no immediate visible cost. The cost only materializes when the implementation needs to change.

**How to avoid:**
Make stacks private from the first commit. Provide query methods: `undo_depth() -> usize`, `redo_depth() -> usize`, `can_undo() -> bool`, `can_redo() -> bool`. If tests need to inspect frame contents, expose `undo_peek() -> Option<&Frame>` rather than the entire stack. Tests written against these methods survive any internal representation change.

**Warning signs:**
- Tests contain `engine.undo_stack.len()` or `engine.redo_stack[0]`
- `undo_stack` or `redo_stack` appear in `pub` struct fields
- `clippy` shows `pub field` warnings on history-related fields

**Phase to address:**
Phase 3 (Undo/redo) — define the query API surface first, implement internals behind it.

---

### Pitfall 7: Irreversibility Designation with No Compile-Time Enforcement

**What goes wrong:**
The library user designates certain frames as irreversible by implementing a trait method or passing a flag, but nothing stops them from forgetting. A dice-roll outcome that should erase undo history does not, and players can undo past the randomness reveal. The game appears to support undo correctly but silently allows cheating. The library cannot enforce irreversibility semantics on behalf of the user.

**Why it happens:**
Irreversibility is a domain concept the engine cannot know about. The natural solution is a trait method with a default return value of `false` (reversible). Developers implement their `Frame` or `Input` types and never override the method because the default "works" — undo just happens to function for their test cases.

**How to avoid:**
Make irreversibility explicit at the call site rather than implicit via default. Design `dispatch()` to accept a `Reversibility` parameter or require the user to specify it per dispatch. Consider:

```rust
pub enum Reversibility {
    Reversible,
    Irreversible,
}

fn dispatch(&mut self, input: I, reversibility: Reversibility) -> Result<Outcome<...>, EngineError>
```

This forces every `dispatch()` call to make a deliberate choice. Users cannot forget — the code will not compile without specifying reversibility. Document clearly: "dice rolls and public information reveals must be `Irreversible`."

**Warning signs:**
- `is_reversible()` is a trait method with a default implementation of `true`
- No tests verify that undo history is cleared after an irreversible commit
- Tic-tac-toe and backgammon examples do not demonstrate the irreversibility boundary

**Phase to address:**
Phase 3 (Undo/redo) — irreversibility must be designed into the `dispatch()` signature, not retrofitted.

---

### Pitfall 8: Outcome/EngineError Conflation

**What goes wrong:**
Domain-level non-committed outcomes (`Disallowed`, `InvalidInput`, `Aborted`) are returned as `Err(EngineError::...)` rather than `Ok(Outcome::...)`. Library users cannot distinguish "the engine is broken" from "the move was illegal." Error handling code catches genuine engine panics and valid game rejections in the same branch. Callers write `if let Err(e) = engine.dispatch(...)` and handle illegal moves the same way they handle engine bugs.

**Why it happens:**
`Result<T, E>` maps naturally to "success / failure." A rejected move feels like failure, so `Err` feels right. The subtlety that `Disallowed` is a successful, expected, domain-level outcome — not an error — requires deliberate design.

**How to avoid:**
Hold the line: `dispatch()` returns `Result<Outcome<F, N>, EngineError>`. `Outcome` holds all domain results including non-committed ones. `EngineError` is strictly reserved for impossible internal states, corrupted engine invariants, or bugs. Document this distinction explicitly in rustdoc on `EngineError`. Add `#[non_exhaustive]` to `EngineError` so future engine failure modes remain additive.

**Warning signs:**
- `EngineError` variants include names like `Disallowed`, `InvalidInput`, `NothingToUndo`
- `match engine.dispatch(...)` arms handling normal game logic appear in `Err(_)` branches
- `EngineError` documentation describes game-rule rejection scenarios

**Phase to address:**
Phase 1 (Core types) — the `Outcome` and `EngineError` type definitions establish this boundary. Get it right before any dispatch logic is written.

---

### Pitfall 9: Behaviors Mutating Working State Directly

**What goes wrong:**
A `Behavior` implementation receives `&mut WorkingState` and writes to it directly instead of returning diffs. The engine cannot accumulate the diff record for the `Frame`. Trace cannot be generated at the moment of mutation. The invariant "any diff that mutates state must append a trace entry" is silently violated. Undo/redo becomes incorrect because the `Frame` diff record is incomplete.

**Why it happens:**
Passing `&mut state` to a behavior is simpler than designing a `Diff` enum and wiring diff application through the engine. If the `evaluate()` signature accepts a mutable reference "temporarily for convenience," the diff pathway atrophies and gets removed.

**How to avoid:**
The `evaluate()` method signature must be `fn evaluate(&self, input: &I, state: &S) -> BehaviorResult<D, O>`. Both `input` and `state` are immutable references. Behaviors are structurally prevented from mutating state — they can only return diffs. This is not a convention or a documentation rule; it is enforced by the type system.

**Warning signs:**
- `evaluate()` signature contains `state: &mut S` or `working: &mut WorkingState<S>`
- Behavior implementations contain `state.something = new_value` rather than `Continue(vec![Diff::Something(new_value)])`
- Frame diff list is empty for dispatches that visibly changed state

**Phase to address:**
Phase 1 (Core types) — the `Behavior` trait signature is the prevention. If `evaluate()` takes `&mut`, this pitfall is already present.

---

### Pitfall 10: HashMap Iteration Non-Determinism in Behavior Storage

**What goes wrong:**
Behaviors or behavior state are stored in a `HashMap<&'static str, Box<dyn Behavior>>` (or similar). Iteration order is random across runs due to Rust's randomized SipHash seed. The engine iterates the map to produce the ordered behavior sequence, but iteration order varies. Games appear deterministic within a single run but differ across runs. Replay tests intermittently fail.

**Why it happens:**
`HashMap` is the standard Rust collection for key-based lookup. It feels natural to store behaviors keyed by name. The non-determinism is subtle — `HashMap` documentation warns about it, but the link between storage choice and ordering-based dispatch is easy to miss.

**How to avoid:**
Store behaviors in a `Vec` sorted by `(order_key, name)` at construction time. Never use `HashMap` for anything that feeds into dispatch ordering. If name-based lookup is needed, build a secondary `HashMap<&'static str, usize>` index pointing into the `Vec`. The `Vec` is the canonical ordered sequence; the map is a lookup cache only.

**Warning signs:**
- Behavior storage uses `HashMap`, `BTreeMap` (safer but still a design smell), or any collection where iteration order is not explicitly sorted
- Behavior ordering is derived from iteration rather than from an explicitly sorted sequence
- Replay determinism tests pass locally but fail in CI with different build environments

**Phase to address:**
Phase 1 (Core types) — the behavior storage data structure must be a sorted `Vec` by design.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Eager full state clone instead of CoW | Correct, simple, one line | O(state_size) per speculative dispatch; AI look-ahead unusable | Never for this project; CoW is a stated requirement |
| Public undo/redo stack fields | Tests can check depth directly | All representation changes break callers | Never; provide query methods instead |
| Default `is_reversible() -> true` | User only overrides when needed | Silent undo-past-randomness bugs in games with dice/hidden info | Never for irreversibility; force explicit call-site choice |
| `dyn Behavior` over static generics | Simpler homogeneous collection | vtable overhead per dispatch; prevents monomorphization optimizations | Only if compile times become intolerable; measure first |
| Five separate type parameters on Engine | Each param independently justified | Turbofish required everywhere; impossible ergonomics | Never; bundle via `EngineSpec` associated-type trait |
| Behavior evaluation takes `&mut State` | Fewer intermediary types needed | Breaks diff accumulation, trace generation, Frame correctness | Never; immutable evaluate is a core invariant |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full state clone on every dispatch | AI look-ahead benchmarks degrade with state size; profiler shows `Clone::clone` dominating dispatch time | Implement CoW `WorkingState<'a, S>` that lazily clones substates on first write | At ~5+ speculative dispatches per turn with state > 1KB |
| Monomorphization explosion from five type params | Compilation times grow superlinearly as behaviors are added; binary size inflates | Bundle types via `EngineSpec`; apply the inner-function extraction pattern for non-generic logic | At ~10+ distinct `Engine` instantiation sites in user code |
| Sorting behaviors on every dispatch | Dispatch latency increases with behavior count; profiler shows `sort` in dispatch hot path | Sort behaviors once at engine construction; store as pre-sorted `Vec` | At ~50+ behaviors per engine instance |
| Re-applying full diff list for undo | Undo latency grows linearly with diff count per frame | Store state snapshots at irreversibility boundaries; diff lists stay bounded between boundaries | At ~100+ diffs per frame or ~1000+ frames in undo history |

---

## "Looks Done But Isn't" Checklist

- [ ] **CoW working state:** Dispatch appears to work but uses eager clone — verify with a large state benchmark that shows flat dispatch time vs state size
- [ ] **Deterministic ordering:** Behaviors execute in correct order in happy-path tests, but equal-priority behaviors use pointer tiebreaker — verify with two behaviors at same `order_key` that produce conflicting diffs, check order matches `(order_key, name)` not pointer ordering
- [ ] **Undo correctness:** Undo restores domain state but behavior counters/cooldowns remain at post-undo values — verify behavior state in `S` changes after undo, not just domain fields
- [ ] **Irreversibility boundary:** `dispatch()` commits correctly but irreversible flag has no effect — verify that undo history is empty after an irreversible dispatch
- [ ] **Atomicity:** Dispatch returns a result but failed mid-dispatch dispatches leave partial working state visible — verify that a `Stop(Disallowed)` from behavior N does not affect committed state even if behaviors 1..N-1 returned diffs
- [ ] **Frame diff completeness:** Frame is committed but diff list is empty or partial — verify that replaying frame diffs from a prior committed state reaches the same committed state
- [ ] **Trace coupling:** Trace is produced but not synchronized with diff application order — verify trace entry order matches the order diffs were applied, not the order behaviors were evaluated
- [ ] **EngineError isolation:** Rejected dispatches are recoverable, but they return `Err(EngineError::...)` instead of `Ok(Outcome::...)` — verify that `Disallowed` and `InvalidInput` outcomes appear in `Ok`, not `Err`

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Behavior state in engine internals | HIGH | Rewrite: move all engine-internal behavior data into user `State`; redesign diff types to carry behavior mutations; update all tests |
| Memory address ordering tiebreaker | MEDIUM | Add `name() -> &'static str` to `Behavior` trait; update sort comparator to use `(order_key, name)` tuple; verify all tests still pass |
| Eager full clone | MEDIUM | Design `WorkingState<'a, S>` wrapper; thread it through dispatch; replace `state.clone()` call sites |
| Generic type parameter explosion | HIGH | Introduce `EngineSpec` trait; migrate all existing type parameters to associated types; this is a breaking API change |
| Public undo/redo stacks | LOW | Make fields private; add query methods; update test assertions to use methods |
| Irreversibility defaults | MEDIUM | Change `dispatch()` signature to require explicit `Reversibility` arg; update all call sites; add tests for history erasure |
| `Outcome`/`EngineError` conflation | MEDIUM | Audit `EngineError` variants; move domain-level outcomes to `Outcome` enum; update all `match` arms in user code |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Behavior state outside main state tree | Phase 1: Core types | `Engine` struct fields contain no behavior metadata; behavior state mutations flow through `Diff` |
| Memory address ordering tiebreaker | Phase 1: Core types | `Behavior::name()` returns `&'static str`; sort uses `(order_key, name)` tuple; two equal-priority behaviors execute in alphabetical order |
| Eager full state clone | Phase 2: CoW working state | Benchmark: dispatch time flat vs state size; `dispatch()` source contains no `state.clone()` |
| Dirty preview side effects | Phase 2: Dispatch algorithm | No `dispatch_preview()` method exists; discarded dispatches leave committed state identical to pre-dispatch |
| Generic type parameter explosion | Phase 1: Core types | `Engine` has exactly one type param (`G: EngineSpec`); example code requires no turbofish |
| Public undo/redo stacks | Phase 3: Undo/redo | `undo_stack` and `redo_stack` do not appear in `pub` struct fields; tests use `undo_depth()` |
| Irreversibility with no enforcement | Phase 3: Undo/redo | `dispatch()` requires explicit `Reversibility` argument; irreversible dispatch erases history in test |
| Outcome/EngineError conflation | Phase 1: Core types | `Disallowed`, `InvalidInput`, `Aborted` appear only in `Outcome`; `EngineError` variants are engine-internal only |
| Behaviors mutating working state | Phase 1: Core types | `Behavior::evaluate()` signature takes `&S` not `&mut S`; clippy confirms no interior mutability on state in evaluate |
| HashMap non-determinism | Phase 1: Core types | Behavior storage is `Vec` sorted at construction; no `HashMap` in dispatch-ordering codepath |

---

## Sources

- CONCERNS.md — v0.4.0 known issues: behavior lifetimes as engine internals, memory-address tiebreaker, eager clone, public stacks, `Option<Action<M>>` divergence
- ARCHITECTURE.md — v0.5.0 design spec: CoW semantics, `(order_key, behavior_name)` ordering, `Outcome`/`EngineError` split, behavior state in main state tree
- [Improving overconstrained Rust library APIs — LogRocket Blog](https://blog.logrocket.com/improving-overconstrained-rust-library-apis/) — generic overconstraint and monomorphization tradeoffs (MEDIUM confidence)
- [Generic associated types to be stable in Rust 1.65 — Rust Blog](https://blog.rust-lang.org/2022/10/28/gats-stabilization/) — GAT stabilization, required bounds pitfalls (HIGH confidence)
- [Rusty Garbage: My HashMap is non-deterministic — Medium](https://medium.com/@draft1967/rusty-garbage-my-hashmap-is-non-deterministic-0e518be0c5c6) — HashMap non-determinism in game state (MEDIUM confidence)
- [Reducing generics bloat — Rust Internals](https://internals.rust-lang.org/t/reducing-generics-bloat/6337) — monomorphization and binary bloat from generic proliferation (MEDIUM confidence)
- [Item 12: Understand the trade-offs between generics and trait objects — Effective Rust](https://www.lurklurk.org/effective-rust/generics.html) — static vs dynamic dispatch ergonomics (HIGH confidence)
- [Transactional Operations in Rust](https://fy.blackhats.net.au/blog/2021-11-14-transactional-operations-in-rust/) — ACI properties, partial-update atomicity pitfalls (MEDIUM confidence)
- [undo crate — docs.rs](https://docs.rs/undo) — existing Rust undo/redo patterns (HIGH confidence)

---
*Pitfalls research for: Rust deterministic turn-based game engine library (HerdingCats v0.5.0)*
*Researched: 2026-03-13*
