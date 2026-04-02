# Design: Effect, Delta & Rule

**Status:** Draft
**Date:** 2026-04-02

---

## Overview

This document describes three foundational types — `Effect`, `Delta`, and `Rule` — that
together provide a symbolic, auditable, and traceable model of state change throughout
the simulation.

The core insight: **separate the description of a change from the record of its
application, and trace both back to the rule that decided it.**

- An `Effect` says *what should happen*
- A `Delta` says *what did happen, and proves it*
- A `Rule` says *why it happened*

All rules, effects, and their associated state types are **known at compile time**. This
rules out dynamic dispatch and runtime type erasure — the compiler verifies exhaustiveness
at every boundary.

---

## Effect

### Concept

An `Effect` is a trait that symbolically describes a state change. Concrete effects are
almost always enums, where each variant carries the data needed to fully describe one
kind of change.

```rust
// Example: health-related effects
enum HealthEffect {
    Heal(u32),
    Damage(u32),
    SetMax(u32),
    Revive,
    Kill,
}
```

The key property: an `Effect` is a **value**, not a mutation. It can be stored,
serialized, logged, compared, and replayed independently of any live state.

### Trait Interface

```rust
pub trait Effect<S> {
    /// Apply this effect to a state, returning the new state.
    /// Pure — does not mutate in place.
    fn apply(&self, state: &S) -> S;

    /// Human-readable description, e.g. "Damage(10)" or "Healed for 5 HP".
    fn describe(&self) -> String;

    /// The logical inverse of this effect, if one exists.
    /// Enables undo/rollback without storing full snapshots.
    /// Returns None when no meaningful inverse exists (e.g. Kill).
    fn inverse(&self) -> Option<Self> where Self: Sized {
        None
    }
}
```

**Notes:**
- `apply` is pure and returns a new state. This keeps effects composable and testable,
  and means effects can be evaluated speculatively without touching live state.
- `inverse` returns `Option<Self>` — since all effect types are compile-time enums,
  there is no need for trait objects here. The inverse of a `HealthEffect` is a
  `HealthEffect`.
- Not all effects are invertible. `Kill` has no meaningful inverse without additional
  context (what was the entity's HP before?). The default implementation returns `None`.
- The `S` type parameter makes `Effect` reusable across any state domain: health,
  position, inventory, rule state, etc.

### Clamping & Overkill

Effects describe *intent*, not outcome. `Damage(100)` applied to an entity with 10 HP
is still `Damage(100)` — the clamping happens inside `apply()`. This means:

- The `Delta` records the *intended* effect symbolically
- The before/after hashes capture the *actual* resulting state
- Overkill, overflow, and immune cases are handled at application time, not encoded
  in the effect variant

---

## Delta

### Concept

A `Delta` is the record of an `Effect` being applied to a specific state. It pairs the
symbolic description of the change with evidence of what state existed before and after,
and a reference to the rule that caused it.

```rust
pub struct Delta<E> {
    /// The effect that was applied.
    pub effect: E,

    /// Hash of the state before the effect was applied.
    pub before: StateHash,

    /// Hash of the state after the effect was applied.
    pub after: StateHash,

    /// The rule that generated this delta, if any.
    /// None for changes applied manually or from external sources.
    pub source: Option<RuleId>,
}
```

### StateHash

```rust
/// Opaque content hash of a state value.
/// Cheap to compare; produced by hashing the serialized state.
pub struct StateHash(u64); // or [u8; 32] for cryptographic strength
```

A `StateHash` uniquely identifies a particular state value. Hash granularity — whether
this covers the full simulation state or a per-entity slice — is an open question (see
**Open Design Questions**).

### Key Properties

**Integrity verification:**
Given a state `s` and a `Delta`, you can verify the delta is consistent:
```
hash(s) == delta.before                      →  delta was produced from this state
hash(effect.apply(s)) == delta.after         →  the recorded outcome is correct
```

**Chain composition:**
A sequence of Deltas is *consistent* if each delta's `before` matches the previous
delta's `after`. Gaps and conflicts are trivially detectable:
```
Delta(A→B) + Delta(B→C) + Delta(C→D)   →  valid chain
Delta(A→B) + Delta(X→C)                →  conflict: X ≠ B
```

**Conflict detection:**
Two Deltas with the same `before` hash but different `after` hashes represent a fork —
both claim to follow from the same state but diverge. Useful for concurrent or
multiplayer scenarios.

---

## Changeset

A `Changeset` is an ordered, atomic group of `Delta`s representing all changes that
occurred in a single evaluation pass (e.g. one tick). It is the primary unit handed
to the history log.

```rust
pub struct Changeset<E> {
    pub deltas: Vec<Delta<E>>,
}

impl<E> Changeset<E> {
    /// True if every delta forms a valid chain (before[n] == after[n-1]).
    pub fn is_consistent(&self) -> bool { ... }

    /// The before-hash of the first delta — the state this changeset was applied to.
    pub fn origin(&self) -> Option<&StateHash> { ... }

    /// The after-hash of the last delta — the state this changeset produced.
    pub fn terminus(&self) -> Option<&StateHash> { ... }

    /// All deltas produced by a specific rule, in order.
    pub fn trace(&self, rule: RuleId) -> impl Iterator<Item = &Delta<E>> {
        self.deltas.iter().filter(move |d| d.source == Some(rule))
    }
}
```

---

## Rule

### Concept

A `Rule` observes state and emits `Effect`s when its conditions are met. Rules are the
*decision layer*: they encode the logic of the simulation ("when X is true, cause Y").

Rules do **not** produce `Delta`s directly. The `RuleEngine` is responsible for
applying effects to state, computing hashes, and wrapping everything into a stamped
`Delta`. Rules remain ignorant of the audit machinery and are pure functions of state.

Rules may be stateless (pure functions of observed state) or stateful (they track
private data: cooldowns, charge counters, firing history). See **Rule State** for how
stateful rules integrate cleanly with the broader system.

### Supporting Types

```rust
/// All rules are known at compile time, so RuleId is a closed enum.
/// Variants ARE the identifiers — no strings, no integers, exhaustiveness is
/// compiler-checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleId {
    Regeneration,
    Poison,
    AmbientHeal,
    // one variant per rule in the simulation
}

/// Determines evaluation order and conflict resolution.
/// Higher value = higher priority. Signed to allow rules below a default baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub i32);

impl Priority {
    pub const LOW: Priority     = Priority(-100);
    pub const DEFAULT: Priority = Priority(0);
    pub const HIGH: Priority    = Priority(100);
}
```

### Trait Interface

The `Rule` trait defines the interface each concrete rule implements. It is generic over
the effect type `E` and state type `S` to keep individual rules decoupled from the
full simulation state when possible.

```rust
pub trait Rule<E, S> {
    /// Stable identifier. Used to stamp Deltas and access rule state.
    fn id(&self) -> RuleId;

    /// Human-readable name for logging and debugging.
    fn name(&self) -> &str;

    /// Evaluation priority. Higher values run first.
    fn priority(&self) -> Priority;

    /// Returns true if this rule is applicable to the current state.
    /// Should be cheap — a field comparison, a threshold check.
    /// Called before `generate`; rules that don't match skip `generate` entirely.
    fn matches(&self, state: &S) -> bool;

    /// Generate the effects this rule wants to apply.
    /// Only called when `matches` returns true.
    /// May return zero effects if the rule matched but finds nothing to do.
    fn generate(&self, state: &S) -> Vec<E>;
}
```

### Priority & Conflict Resolution

Rules are evaluated in descending priority order. When multiple rules produce
**conflicting** Deltas (same `before` hash, different `after` hashes for the same
state component), the **highest-priority rule wins** and the lower-priority delta is
discarded.

For rules at equal priority, conflicts are broken deterministically by `RuleId`
discriminant order. This guarantees reproducibility given the same rule set and initial
state.

```
Priority 100: [ RegenerationRule ]   →  Heal(5)
Priority   0: [ PoisonRule ]         →  Damage(3)
Priority   0: [ CurseRule ]          →  Damage(10)  ← wins tie over PoisonRule
Priority -50: [ AmbientHealRule ]    →  Heal(1)
```

Non-conflicting effects from multiple rules all apply, each producing its own Delta.

### Enum Dispatch

Since all rules are known at compile time, the engine uses a `SimRule` enum rather than
`Box<dyn Rule>`. No heap allocation per rule, no vtable, fully monomorphised:

```rust
/// Closed enum over all rules in the simulation.
/// Enum dispatch replaces dyn trait — zero-cost, exhaustiveness-checked.
pub enum SimRule {
    Regeneration(RegenerationRule),
    Poison(PoisonRule),
    AmbientHeal(AmbientHealRule),
}

impl SimRule {
    pub fn id(&self) -> RuleId {
        match self {
            Self::Regeneration(_) => RuleId::Regeneration,
            Self::Poison(_)       => RuleId::Poison,
            Self::AmbientHeal(_)  => RuleId::AmbientHeal,
        }
    }

    pub fn priority(&self) -> Priority {
        match self {
            Self::Regeneration(r) => r.priority(),
            Self::Poison(r)       => r.priority(),
            Self::AmbientHeal(r)  => r.priority(),
        }
    }

    pub fn matches(&self, state: &SimState) -> bool {
        match self {
            Self::Regeneration(r) => r.matches(state),
            Self::Poison(r)       => r.matches(state),
            Self::AmbientHeal(r)  => r.matches(state),
        }
    }

    pub fn generate(&self, state: &SimState) -> Vec<SimEffect> {
        match self {
            Self::Regeneration(r) => r.generate(state),
            Self::Poison(r)       => r.generate(state),
            Self::AmbientHeal(r)  => r.generate(state),
        }
    }
}
```

The match arms are mechanical boilerplate and a natural candidate for a proc macro if
it becomes burdensome.

### RuleEngine

```rust
pub struct RuleEngine {
    /// Rules sorted by Priority descending at construction time.
    rules: Vec<SimRule>,
}

impl RuleEngine {
    /// Evaluate all applicable rules against the current state.
    /// Returns a Changeset of stamped, ordered Deltas.
    pub fn evaluate(&self, state: &SimState) -> Changeset<SimEffect> {
        let mut deltas  = vec![];
        let mut current = state.clone();

        for rule in &self.rules {
            if rule.matches(&current) {
                for effect in rule.generate(&current) {
                    let before = hash_of(&current);
                    let next   = effect.apply(&current);
                    let after  = hash_of(&next);

                    deltas.push(Delta {
                        effect,
                        before,
                        after,
                        source: Some(rule.id()),
                    });

                    current = next;
                }
            }
        }

        Changeset { deltas }
    }
}
```

---

## Rule State

### The Problem

Rules often need private state that persists across ticks: a cooldown timer, a charge
counter, a record of recent firings. This state must live *somewhere*, and that choice
has deep consequences for the design.

Three options were considered:

| Option | Where state lives | Trade-offs |
|---|---|---|
| **A: Inside the Rule struct** | `&mut self` on each firing | Simple, but breaks immutability, invisible to audit trail |
| **B: Parallel engine-managed map** | `HashMap<RuleId, Box<dyn Any>>` alongside `S` | Pragmatic, but rule state changes produce no Deltas — invisible to the audit trail |
| **C: Part of total state `S`** | A `RuleStateStore` struct embedded in `S` | Fully consistent: all state changes are Effects and produce Deltas |

**Option C is the chosen design.** The motivating principle: a cooldown expiring is a
simulation event just like HP changing. It should be traceable, replayable, and
hashable. A second class of state invisible to the Delta system would undermine the
audit trail.

Because all rules are known at compile time, Option C has no dynamic dispatch cost.
`RuleStateStore` is a plain struct — no `HashMap`, no `Box<dyn Any>`, no downcasting.

### RuleStateStore

```rust
/// All per-rule persistent state, embedded in the total simulation state.
/// Only rules that require persistent state have a field here.
/// Stateless rules (e.g. AmbientHeal) have no entry.
#[derive(Debug, Clone, Hash)]
pub struct RuleStateStore {
    pub regeneration: RegenState,
    pub poison:       PoisonState,
}
```

Fields are accessed by name — the compiler verifies correctness. Adding a new stateful
rule means adding a field here, a variant to `RuleId`, and a variant to
`RuleStateEffect`; the compiler then flags every match that needs updating.

`RuleStateStore` is part of `SimState` and is included in its hash. Any tick that
changes rule state is reflected in the `before`/`after` hashes of the relevant Deltas.

### Effects on Rule State

Rule state changes are `Effect`s like any other — they flow through the same
Effect/Delta pipeline. Because all rule state types are known at compile time, the
`RuleStateEffect` is a closed, typed enum:

```rust
// Per-rule state effect types, co-located with each rule's definition
enum RegenRuleEffect {
    SetCooldown(u32),
    DecrementCooldown,
}

enum PoisonRuleEffect {
    Activate,
    Deactivate,
}

// Closed enum over all rule-state effects — exhaustiveness checked by compiler
enum RuleStateEffect {
    Regeneration(RegenRuleEffect),
    Poison(PoisonRuleEffect),
}

// Top-level simulation effect: all domains unified in one type
enum SimEffect {
    Health(HealthEffect),
    Position(PositionEffect),
    RuleState(RuleStateEffect),
}
```

When `RegenerationRule` fires, it produces two Deltas in the same tick:

1. `Delta { effect: SimEffect::Health(Heal(5)),                                source: Some(RuleId::Regeneration), ... }`
2. `Delta { effect: SimEffect::RuleState(Regeneration(SetCooldown(10))),       source: Some(RuleId::Regeneration), ... }`

Both carry the same `source`. `Changeset::trace(RuleId::Regeneration)` returns both,
giving the complete causal picture: *it healed 5 HP and put itself on a 10-tick cooldown*.

### Rule Access Pattern

A stateful rule reads its named field directly from `state.rule_store` — no runtime
lookup, no unwrap:

```rust
struct RegenerationRule;

#[derive(Default, Clone, Hash)]
struct RegenState {
    cooldown_remaining: u32,
}

impl Rule<SimEffect, SimState> for RegenerationRule {
    fn id(&self)       -> RuleId   { RuleId::Regeneration }
    fn name(&self)     -> &str     { "Regeneration" }
    fn priority(&self) -> Priority { Priority::DEFAULT }

    fn matches(&self, state: &SimState) -> bool {
        state.rule_store.regeneration.cooldown_remaining == 0
            && state.health.current < state.health.max
    }

    fn generate(&self, _state: &SimState) -> Vec<SimEffect> {
        vec![
            SimEffect::Health(HealthEffect::Heal(5)),
            SimEffect::RuleState(RuleStateEffect::Regeneration(
                RegenRuleEffect::SetCooldown(10),
            )),
        ]
    }
}
```

The rule is a pure function of `&SimState` — no `&mut self`, no hidden state, no
runtime lookup. The compiler verifies both the field access and the effect variant.

### Cooldown Decrement

Decrementing cooldowns each tick is itself an `Effect`, producing its own `Delta`.
This can be emitted by a dedicated low-priority housekeeping rule or as a built-in
engine step. Either way, the decrement appears in the Delta log and is fully traceable.

```rust
// Emitted every tick for any rule with cooldown_remaining > 0
SimEffect::RuleState(RuleStateEffect::Regeneration(RegenRuleEffect::DecrementCooldown))
```

Nothing is implicit. The audit trail is complete.

---

## Open Design Questions

| Question | Options | Notes |
|---|---|---|
| **Hash granularity** | Full `SimState` vs. per-entity vs. per-component | Full state is simple but expensive to recompute; per-component is efficient but requires defining component boundaries up front |
| **Hash strength** | `u64` (fast, structural) vs. `[u8; 32]` (cryptographic) | `u64` is sufficient for a local simulation; cryptographic strength is only needed in networked or adversarial contexts |
| **apply() ownership** | `&S -> S` (clone) vs. `S -> S` (consume) vs. `&mut S` (in-place) | Clone is safest for composability and speculative evaluation; in-place mutation may be necessary for performance at scale |
| **Multi-entity effects** | `Effect<S>` targets one state vs. `Effect<World>` targets many | A "Reflect" or AoE effect needs to touch multiple entities. May require either a `World`-scoped effect type or a multi-target effect wrapper |
| **Rule conflict resolution** | Highest priority wins vs. merge function | "Highest wins" is simple and deterministic. A merge function (e.g. sum all concurrent `Heal`s) allows richer semantics but adds complexity |
| **Cooldown decrement ownership** | Dedicated housekeeping rule vs. engine built-in tick step | A housekeeping rule is consistent with the design (all changes are Effects) but adds a rule that isn't "domain logic". An engine built-in is pragmatic but slightly special-cased |
| **`Rule` trait `id()` method** | On the trait vs. only on `SimRule` enum | Individual rule structs don't strictly need `id()` since `SimRule` can return it. Keeping it on the trait makes rules testable in isolation without `SimRule` |
| **Effect invertibility** | `Option<Self>` on the trait vs. a separate `Invertible` supertrait | `Option<Self>` is simple. A supertrait would let you express "this effect domain supports undo" as a type-level constraint |

---

## Example: Health System

```rust
#[derive(Debug, Clone, PartialEq)]
enum HealthEffect {
    Heal(u32),
    Damage(u32),
    SetMax(u32),
}

#[derive(Debug, Clone)]
struct HealthState {
    current: u32,
    max: u32,
}

impl Effect<HealthState> for HealthEffect {
    fn apply(&self, state: &HealthState) -> HealthState {
        match self {
            HealthEffect::Heal(n) => HealthState {
                current: (state.current + n).min(state.max),
                max: state.max,
            },
            HealthEffect::Damage(n) => HealthState {
                current: state.current.saturating_sub(*n),
                max: state.max,
            },
            HealthEffect::SetMax(n) => HealthState {
                current: state.current.min(*n),
                max: *n,
            },
        }
    }

    fn describe(&self) -> String {
        match self {
            HealthEffect::Heal(n)   => format!("Heal({n})"),
            HealthEffect::Damage(n) => format!("Damage({n})"),
            HealthEffect::SetMax(n) => format!("SetMax({n})"),
        }
    }

    fn inverse(&self) -> Option<Self> {
        // Damage and Heal are not true inverses without knowing the pre-application
        // value (due to clamping). A richer undo system would store the original
        // state in the Delta rather than relying on effect inversion.
        None
    }
}
```

---

## Architecture

```
  ┌─────────────────────────────────────────────────────┐
  │  SimState                                           │
  │   ├── WorldState   (health, position, inventory…)  │
  │   └── RuleStateStore  (per-rule persistent state)  │
  └─────────────────────────────────────────────────────┘
       │
       │  observed read-only (&SimState)
       ▼
  [ SimRule ]  ─────────────────────────────► why it happened
       │  matches(&S)    → bool             (cheap predicate, may read rule state)
       │  generate(&S)   → Vec<SimEffect>   (world effects + self-state effects)
       │  id(), name(), priority()
       ▼
  [ SimEffect ]  ───────────────────────────► what should happen (symbolic intent)
       │   ├── Health(HealthEffect)
       │   ├── Position(PositionEffect)
       │   └── RuleState(RuleStateEffect)   (rule state changes flow here too)
       │
       │  RuleEngine: effect.apply(&S) → S'
       │              stamp Delta with source RuleId
       ▼
  [ Delta<SimEffect> ]  ────────────────────► proof it happened + who caused it
       │  effect + before + after + source: Option<RuleId>
       │
       │  collected into
       ▼
  [ Changeset<SimEffect> ]  ────────────────► full record of one evaluation pass
       │  trace(RuleId)     → deltas by cause (world changes + rule self-changes)
       │  is_consistent()   → verify hash chain integrity
       ▼
  [ History ]  ─────────────────────────────► append-only audit log / replay log
```

Every change in the simulation — to world state *or* rule state — flows through the
same Effect/Delta pipeline. Any `Delta` can be traced to its originating `Rule` and
verified against state via its hashes. The simulation is fully inspectable: not just
*what* changed, but *why*, and *what the rule did to itself as a result*.
