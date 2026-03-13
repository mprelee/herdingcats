# Feature Research

**Domain:** Deterministic turn-based game engine library (Rust)
**Researched:** 2026-03-13
**Confidence:** HIGH (architecture spec authoritative; ecosystem survey via boardgame.io, Asmodee rules engine, Rust game dev community)

---

## Context: What "Users" Means Here

Users of herdingcats are **game developers writing Rust**, not end players. The feature surface is the library's API and trait contracts. A game developer using herdingcats needs to:

1. Define their game state and rules (as behaviors, diffs, outcomes)
2. Drive the engine with inputs
3. Read results back (outcomes, frames, traces)
4. Undo/redo moves
5. Reason about history and state

Features are evaluated against this developer experience.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features that any rules-engine library must have. Missing these means the library cannot be used to build a real game.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `dispatch(input) -> Result<Outcome, EngineError>` | Core operation. Without it there is no engine. | MEDIUM | Atomic: either commits fully or leaves state unchanged |
| Ordered behavior evaluation | The library's central value. Without ordering, rule interactions are ambiguous. | MEDIUM | `(order_key, behavior_name)` deterministic ordering |
| `BehaviorResult::Continue(Vec<D>)` / `Stop(O)` | Behaviors need to either contribute or halt. The two-branch return is the fundamental behavior contract. | LOW | Halt-on-Stop must immediately discard working state |
| Atomic dispatch (all-or-nothing) | Without atomicity, partial failures corrupt state. Every comparable system (boardgame.io, Asmodee rules engine) enforces this. | MEDIUM | Working state is speculative; only committed on success |
| `Outcome` enum with `Committed`, `NoChange`, `InvalidInput`, `Disallowed`, `Aborted` | Game logic must distinguish "move was rejected" from "move changed nothing" from "engine broke". boardgame.io and the Asmodee rules engine both enforce input validation at the engine layer. | LOW | `EngineError` is separate — engine malfunction, not domain outcome |
| `Frame { input, diff, trace }` | Canonical transition record. Without it, undo/redo has no substrate, replay is impossible, and debugging is blind. | LOW | Produced only on successful non-empty dispatch |
| `undo() -> Result<Outcome, EngineError>` | Any game with meaningful choices needs undo. Users asked for it in boardgame.io (issue #95). Mobile games need it for mis-tap recovery. | MEDIUM | Returns `Undone(frame)` or `Disallowed(NothingToUndo)` |
| `redo() -> Result<Outcome, EngineError>` | Undo without redo is incomplete. Standard pairing in every command-pattern implementation. | LOW | Returns `Redone(frame)` or `Disallowed(NothingToRedo)` |
| Trace generation synchronized with dispatch | UIs need to know *why* state changed (e.g., "monster died") without re-deriving it from state. Asmodee rules engine explicitly recommends this. | MEDIUM | Trace is generated at diff-apply time, not reconstructed |
| Behavior state lives in main state tree | Without this, undo/redo breaks (behavior state diverges from committed state). This is a correctness requirement, not a feature. | LOW | Enforced by architecture; no hidden engine-internal mutable state |
| Copy-on-write working state | Without CoW, every dispatch eagerly clones the full game state. For AI look-ahead (many speculative dispatches) this is prohibitive. | HIGH | Read from committed until first write; clone substate on first write |
| Static behavior set (compile-time) | Dynamic registration destroys type safety and determinism guarantees. The library's value proposition depends on statically known, ordered behaviors. | LOW | No runtime `register_behavior()` API |
| `Behavior` trait with `name()`, `order_key()`, `evaluate()` | The user-facing extension point. Without it, users cannot define rules. | LOW | `name()` must return `&'static str` for deterministic ordering |

### Differentiators (Competitive Advantage)

Features that separate herdingcats from "roll your own command pattern" or a generic FSM crate.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Irreversibility boundary designation | Game domains have points of no return (dice revealed, hidden info made public). No existing general-purpose library handles this cleanly. The library user designates; the engine erases undo/redo history across that boundary. | LOW | Single policy for MVP: erase on irreversible commit |
| Diff-as-record (diffs stored in frame, not just applied) | Replay, network sync, and debugging all benefit from having the mutation log as first-class data. Most roll-your-own command patterns apply-and-discard. herdingcats stores diffs in the Frame. | MEDIUM | Enables time-travel debugging, deterministic replay from any checkpoint |
| `EngineError` distinct from domain outcomes | Most libraries conflate "engine bug" with "invalid input". herdingcats surfaces the distinction explicitly, enabling library users to handle them differently in production. | LOW | `Result<Outcome, EngineError>` — the `Err` variant means the library malfunctioned |
| Later behaviors see earlier diffs (live working state) | The "higher-priority behavior fires first and changes state that lower-priority behavior then observes" pattern is the core reason ordered behaviors exist. Generic event systems don't provide this. | MEDIUM | Requires immediate diff application during dispatch loop, not deferred |
| Trace ordering guaranteed to match execution | UI frameworks that react to trace entries depend on ordering. Reconstructed-after-the-fact traces can be wrong when side effects are interleaved. | LOW | Enforced by generating trace at diff-apply moment |
| Zero runtime dependencies | Embeddable in any Rust project, including WASM targets, server-side AI workers, embedded systems. Competing options (Bevy, boardgame.io) pull in large dependency trees. | LOW | Already established by PROJECT.md constraints |
| Tic-tac-toe + Backgammon examples | Two examples spanning trivial-to-moderate complexity. Backgammon specifically exercises irreversibility (dice roll) and multiple legal move generation. Shows the library is usable on real games, not just toy cases. | MEDIUM | Both are placeholder files; need full implementation |

### Anti-Features (Deliberately NOT in v0.5.0)

Features that seem useful but are excluded with intention.

| Feature | Why Requested | Why Problematic for v0.5.0 | Alternative |
|---------|---------------|----------------------------|-------------|
| `NeedsChoice` outcome / interactive branching | Many games require mid-dispatch player input (target selection, mulligan decisions). Users will ask for it. | Fundamentally changes the dispatch contract. Requires suspending and resuming dispatch state, interaction with async or callback models, and deeper design work. Not solvable cleanly at the library level without understanding concrete use cases first. | Defer to v0.6.0+. Model as a sequence of inputs: dispatch returns `Disallowed` or a domain signal, caller queries what choices are available, caller dispatches a `ChooseOption` input. |
| Runtime behavior registration | Mod systems, dynamically loaded content, scripting layers all seem to want this. | Destroys type safety and determinism guarantees. The library's ordering contract is only enforceable over a statically known set. Also prevents future DSL compilation path. | Static behaviors composed at engine construction time. Scripted behaviors are a future concern, likely via DSL. |
| Dynamic substates / dynamic dispatch as design center | ECS-style component lookup at runtime is familiar from Bevy. | Adds indirection that undermines the compile-time guarantees the library provides. `dyn Behavior` boxes everywhere would break zero-overhead and complicate the CoW granularity story. | User defines substates statically in their `State` struct. Library provides guidance but no forced layout. |
| Autonomous turn advancement / real-time scheduling | Games with "AI thinking" phases or timers appear to need engine-driven progression. | Fundamentally changes the engine model. herdingcats is input-driven by design. Autonomous advancement requires threading, async runtimes, or timer infrastructure — none of which belong in a zero-dependency library. | Caller drives the loop. AI thinks, then calls `dispatch`. External timer fires, caller calls `dispatch(TimerExpired)`. |
| Separate global validation pass before behavior evaluation | Early-return validation seems like a performance optimization. | Circumvents priority-based resolution. A high-priority behavior might transform input into something valid that a naive pre-check would reject. The architecture explicitly forbids this. | Validation happens naturally in ordered behavior evaluation. First behavior to return `Stop(InvalidInput(...))` short-circuits dispatch. |
| Built-in phase/turn management (like boardgame.io phases) | boardgame.io's phase system is convenient for typical board games. | Opinionated turn structure would conflict with games that don't fit the mold (e.g., simultaneous resolution, action points, reaction windows). herdingcats models turn structure as domain state and behaviors, not engine primitives. | Users model `CurrentPhase`, `ActivePlayer`, etc. in their `State`. Behaviors enforce phase-appropriate rules by inspecting state. |
| Multiplayer / network sync | Users building multiplayer games will want built-in sync. | Out of scope for a pure rules engine. The deterministic input-log model (dispatch from same inputs produces same state) is what enables network sync — but the transport is the caller's problem. | Determinism guarantee is the foundation. Caller records inputs, replicates input log to all peers, each peer dispatches identically. |

---

## Feature Dependencies

```
dispatch(input)
    └──requires──> Behavior trait (name, order_key, evaluate)
    └──requires──> BehaviorResult<D, O> (Continue / Stop)
    └──requires──> CoW working state
    └──requires──> Diff application (immediate, ordered)
    └──requires──> Trace generation (at diff-apply time)
    └──requires──> Frame construction (input + diff + trace)
    └──requires──> Outcome enum (Committed, NoChange, InvalidInput, Disallowed, Aborted)

undo()
    └──requires──> Frame (stored in undo stack)
    └──requires──> dispatch() (undo stack only populated by dispatch)
    └──enhances──> Irreversibility boundary (determines what can be undone)

redo()
    └──requires──> undo() (redo stack populated by undo)

Irreversibility boundary
    └──requires──> undo/redo stacks (to erase)
    └──requires──> Frame (to identify boundary frames)

Trace
    └──requires──> Diff application (trace entries generated at apply time)
    └──enhances──> Frame (frame carries trace for replay/debugging)

CoW working state
    └──requires──> Static substate layout (CoW granularity is per-substate)
    └──conflicts──> Dynamic substates (CoW with dynamic dispatch makes granularity undefined)

Static behavior set
    └──conflicts──> Runtime behavior registration
    └──enables──> Deterministic (order_key, behavior_name) ordering
```

### Dependency Notes

- **dispatch requires Behavior trait**: The trait is the user's only extension point. Without it there is no dispatch loop.
- **undo requires dispatch**: The undo stack only has content after successful dispatches. Both must be designed together.
- **CoW conflicts with dynamic substates**: CoW granularity is defined by the static substate layout. A dynamic layout would require either full-clone fallback (defeating the purpose) or runtime reflection (adding complexity).
- **Static behavior set enables deterministic ordering**: The `(order_key, behavior_name)` tiebreaker only works if `behavior_name` is stable at compile time. Runtime-registered behaviors would need runtime name assignment, which could be non-deterministic.
- **Trace requires diff application to be immediate**: If diffs were batched and applied at end of dispatch, trace entries could not be generated in real execution order. Immediate application is a hard architectural dependency of correct trace ordering.

---

## MVP Definition

### Launch With (v0.5.0)

Minimum viable product — what a user needs to build tic-tac-toe and backgammon with this library.

- [ ] `Behavior` trait with `name() -> &'static str`, `order_key() -> K`, `evaluate(&I, &S) -> BehaviorResult<D, O>` — the user's extension point
- [ ] `BehaviorResult<D, O>` enum: `Continue(Vec<D>)` and `Stop(O)` — the behavior contract
- [ ] `Outcome<F, N>` enum: `Committed(F)`, `Undone(F)`, `Redone(F)`, `NoChange`, `InvalidInput(N)`, `Disallowed(N)`, `Aborted(N)` — full result surface
- [ ] `EngineError` type distinguishing engine malfunction from domain outcomes
- [ ] `Frame<I, D, T>` struct: `input`, `diff`, `trace` — canonical transition record
- [ ] `dispatch(input) -> Result<Outcome, EngineError>` — atomic, ordered, immediate-diff, trace-generating
- [ ] CoW working state — no eager full-state clone on dispatch
- [ ] `undo() -> Result<Outcome, EngineError>` — with `Undone(frame)` and `Disallowed` variants
- [ ] `redo() -> Result<Outcome, EngineError>` — with `Redone(frame)` and `Disallowed` variants
- [ ] Irreversibility boundary: user-designated, erases undo/redo history on commit
- [ ] Tic-tac-toe example — full API demonstration, minimal state
- [ ] Backgammon example — full API demonstration, exercises dice roll irreversibility

### Add After Validation (v0.5.x)

Features to add once the core is validated with real game implementations.

- [ ] `NeedsChoice` outcome — add when a concrete use case (e.g., backgammon bearing-off target selection, card game targeting) proves the dispatch-suspend model is needed
- [ ] History access API (`engine.history()` or frame iterator) — add when replay or time-travel debugging is requested by users
- [ ] Derive macros or helper traits for common Diff patterns — add when users report boilerplate fatigue writing diff enums

### Future Consideration (v0.6.0+)

Features to defer until the library has real-world usage data.

- [ ] DSL / card-text compilation — long-term direction per PROJECT.md; requires mature behavior model and real game experience
- [ ] Multiplayer/network sync helpers — determinism is already there; transport layer is user concern until a concrete pattern emerges
- [ ] Runtime behavior registration — only if a scripting or mod use case proves it necessary; would require new design work to preserve ordering guarantees

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `dispatch` + `Behavior` trait + `BehaviorResult` | HIGH | MEDIUM | P1 |
| `Outcome` enum + `EngineError` | HIGH | LOW | P1 |
| `Frame<I, D, T>` | HIGH | LOW | P1 |
| CoW working state | HIGH | HIGH | P1 |
| Ordered behavior evaluation (immediate diff apply) | HIGH | MEDIUM | P1 |
| Trace generation synchronized with dispatch | HIGH | MEDIUM | P1 |
| `undo()` / `redo()` | HIGH | MEDIUM | P1 |
| Irreversibility boundary | MEDIUM | LOW | P1 |
| Tic-tac-toe example | MEDIUM | LOW | P1 |
| Backgammon example | HIGH | MEDIUM | P1 |
| History/replay access API | MEDIUM | LOW | P2 |
| `NeedsChoice` outcome | MEDIUM | HIGH | P2 |
| Derive macros for Diff | LOW | MEDIUM | P3 |
| DSL compilation | HIGH | HIGH | P3 |

**Priority key:**
- P1: Must have for v0.5.0 launch
- P2: Should have, add after validation
- P3: Future consideration

---

## Competitor Feature Analysis

| Feature | boardgame.io (JS) | Asmodee Rules Engine | herdingcats v0.5.0 |
|---------|------------------|---------------------|-------------------|
| Core dispatch model | Moves as pure functions `(G, ctx) -> G` | Input validation + state transition | Ordered behaviors, immediate diffs, atomic commit |
| Behavior ordering | Implicit (move order, phase priority) | Not exposed directly | Explicit `(order_key, behavior_name)`, user-controlled |
| Undo/redo | Via game log time-travel; limited native undo | Not documented | Native `undo()`/`redo()`, undo stack with irreversibility |
| Trace/log | Game log as first-class feature | Event sourcing for replay | Trace generated at diff-apply time, stored in Frame |
| Phase management | Built-in phase system, turn order, stage system | Not documented | Anti-feature: modeled in user state |
| Multiplayer | Built-in server sync | Client/server model | Out of scope; determinism enables it |
| Runtime behavior registration | Yes (move definitions) | Not applicable | Anti-feature: static only |
| Type safety | JavaScript (runtime) | Not Rust | Rust, compile-time, zero dynamic dispatch |
| Dependencies | Large (React, server, etc.) | Unknown | Zero runtime dependencies |
| Speculative/preview dispatch | No native support | Unknown | CoW working state supports cheap speculation |

---

## Sources

- [boardgame.io documentation](https://boardgame.io/documentation/) — phases, events, turn order, game log features
- [boardgame.io GitHub: undo/redo issue #95](https://github.com/boardgameio/boardgame.io/issues/95) — community evidence that undo is expected
- [Asmodee Rules Engine Architecture](https://doc.asmodee.net/rules-engine) — input validation, event sourcing, data encapsulation patterns
- [Making a turn-based multiplayer game in Rust (herluf-ba)](https://herluf-ba.github.io/making-a-turn-based-multiplayer-game-in-rust-01-whats-a-turn-based-game-anyway.html) — reducer pattern, validate-then-apply
- [Architecture discussion: Card game rules engine in Rust](https://users.rust-lang.org/t/architecture-discussion-writing-a-card-game-rules-engine-in-rust/41569) — effect objects over direct mutation
- [Game Programming Patterns: Command](https://gameprogrammingpatterns.com/command.html) — command pattern for undo/redo
- [Are we game yet? Rust ecosystem](https://arewegameyet.rs/) — Rust game development library landscape
- herdingcats ARCHITECTURE.md — authoritative for v0.5.0 scope and constraints
- herdingcats PROJECT.md — validated requirements, active requirements, out-of-scope items

---
*Feature research for: deterministic turn-based game engine library (Rust)*
*Researched: 2026-03-13*
