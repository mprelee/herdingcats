# DSL v1.1 Contract

This document locks the Phase 4 authored-language boundary for `herdingcats` v1.1.

The goal is narrow: let a consumer define additional rules in external DSL files that lower into generated Rust implementing the existing `Rule` trait. This DSL is not a scripting language, not a runtime mod format, and not a replacement for handwritten engine or game code.

## Build-Time Integration Path

The v1.1 DSL is a build-time authoring tool.

- Parse authored `.cats` files from `build.rs`.
- Lower them through `herdingcats_codegen`.
- Write generated Rust into `OUT_DIR`.
- `include!` that generated module into the consumer crate and register generated rules beside handwritten ones.

There is no runtime parser, no runtime scripting entrypoint, and no expectation that authored rules are loaded dynamically after compilation.

Handwritten-only usage remains valid: consumers can ignore the DSL path entirely and keep using handwritten `Rule` and `Operation` implementations.

## Design Constraints

Every authored rule must map onto existing engine semantics from `src/rule.rs`, `src/transaction.rs`, and `src/operation.rs`:

- stable `id()` string
- ordered `priority()`
- `RuleLifetime`
- a `before()` hook that can inspect state and the incoming event
- transaction effects expressed through `Transaction<O>`
- reversible operations implementing `Operation<S>`

The DSL must not promise behavior that the engine cannot safely honor today.

## File Model

- Authoring happens in external files parsed from a `.pest` grammar.
- A file contains one or more `rule` blocks.
- Phase 4 locks the surface syntax and semantic scope only. Parser, AST, validation, and code generation come later.

## Top-Level Rule Shape

Each authored rule has this logical shape:

```text
rule "<stable-id>" {
  priority <integer>
  lifetime <lifetime-spec>
  on <event-match>
  when <guard-expression>
  before {
    <effect-statement>*
  }
}
```

Fields are defined as follows:

- `rule "<stable-id>"`: required. This is the engine rule key and must compile to `Rule::id() -> &'static str`.
- `priority <integer>`: optional. Defaults to `0`. Lower values run earlier in `before()`, matching the `Rule` contract.
- `lifetime <lifetime-spec>`: optional. Defaults to `permanent`.
- `on <event-match>`: required. Declares the event variant this rule can inspect.
- `when <guard-expression>`: optional. Zero or more guard lines may appear. All must pass for the rule to emit effects.
- `before { ... }`: required. All generated behavior for v1.1 lives here.

There is no authored `after { ... }` block in v1.1.

## Required Fields and Defaults

### `id`

- Required.
- Must be stable across builds and releases if the consumer wants rule enablement and lifetime state to remain meaningful.
- Must be unique within the authored rule set.

### `priority`

- Optional.
- Default: `0`.
- Maps directly to `Rule::priority()`.
- The DSL uses an integer surface in Phase 4 because the engine trait only requires `Copy + Ord`. Phase 5 can decide how the generated code binds that number into the consumer’s chosen priority type.

### `lifetime`

- Optional.
- Default: `permanent`.
- Maps directly to `RuleLifetime`:
  - `permanent` -> `RuleLifetime::Permanent`
  - `turns <n>` -> `RuleLifetime::Turns(n)`
  - `triggers <n>` -> `RuleLifetime::Triggers(n)`

Counts must be positive integers.

## Event Matching Scope

Event matching is intentionally constrained.

Allowed v1.1 event match forms:

- event variant only
- event variant plus named field pattern bindings
- event variant plus literal equality checks on bound fields through `when`

Examples:

```text
on ScoreCommitted
on PlayScored(kind, team)
on CardPlayed(card, controller)
```

Phase 4 does not allow:

- nested destructuring
- matching multiple event variants in one rule
- arbitrary boolean logic inside the `on` clause
- calling methods while pattern matching

The event surface must stay small enough that Phase 5 can validate it into a closed, static mapping against the consumer’s event enum.

## Guard Expression Scope

Guards exist to decide whether a matched event should emit effects.

Allowed guard surface:

- comparisons against approved bindings
- boolean flags
- integer equality and ordering checks
- string or enum-literal equality
- conjunction via repeated `when` lines or `&&`
- negation of a boolean binding

Bindings may come from:

- event fields named in `on`
- a constrained approved state binding surface
- consumer-exposed constants or enum literals

Representative guard forms:

```text
when state.scoring_mode == "touchdown_plus_one"
when team == "home"
when points > 0
when !state.mercy_rule_active
```

Not allowed in v1.1:

- arbitrary arithmetic
- arbitrary function or method calls
- loops
- collection iteration
- closures
- user-defined expressions
- host-language escape hatches

The reason is semantic control: guards must remain statically understandable and validation-friendly before code generation.

## Allowed `before()` Effect Categories

All authored effects must compile into engine-compatible `before()` behavior.

Allowed effect categories:

- emit one or more reversible operations
- cancel the transaction
- mark the transaction as non-undoable by setting `tx.irreversible = false`
- mark the transaction as non-deterministic by setting `tx.deterministic = false`

Representative authored forms:

```text
before {
  emit AwardPoints(team: team, points: 1)
  cancel
  set tx.irreversible = false
  set tx.deterministic = false
}
```

Semantic constraints:

- `emit ...` must target an approved operation family that lowers to `O: Operation<S>`.
- Emitted operations must remain reversible through `apply`/`undo`.
- `set tx.irreversible = false` is allowed because it maps directly to `Transaction::irreversible`.
- `set tx.deterministic = false` is allowed because it maps directly to `Transaction::deterministic`.
- `set tx.cancelled = true` is represented by `cancel`.

Phase 4 does not permit arbitrary writes to other `Transaction` fields or any mutation outside those explicit categories.

## Direct State Mutation Is Prohibited

The authored DSL must not mutate state directly.

Rejected categories:

- `state.score.home += 1`
- `state.players[active].hp = 0`
- `event.points = 99`

Reason:

- `herdingcats` requires state changes to flow through `Operation<S>`.
- Undo/redo correctness depends on each mutation having an explicit inverse.
- Replay hashing depends on deterministic operation bytes, not hidden side effects.

Any authored change to state must lower to emitted reversible operations, never direct writes.

## `after()` Mutation Is Prohibited in v1.1

The engine has an `after()` hook, but Phase 4 intentionally excludes authored mutating `after()` behavior.

Rejected categories:

- `after { emit ... }`
- `after { cancel }`
- `after { state... }`

Reason:

- v1.1 is scoped to semantics that safely compile into `before()` behavior.
- The current milestone does not establish generated `after()` mutation semantics.
- Allowing authored `after()` effects now would over-promise behavior the parser/codegen phases cannot safely implement without wider engine design work.

## Out of Scope

The following are explicitly out of scope for v1.1:

- runtime parsing or runtime rule loading
- runtime scripting or embedded host-language code
- direct state mutation
- authored `after()` blocks with mutating behavior
- arbitrary expressions or function calls
- loops, iteration, or reusable macros/fragments
- full game-definition language ambitions
- public redesign of `Rule`, `Transaction`, `Operation`, or `RuleLifetime`
- implicit side effects outside emitted operations or explicit transaction flags

## Semantic Mapping to Engine Concepts

| DSL concept | Engine target |
| --- | --- |
| `rule "foo"` | `Rule::id()` |
| `priority 10` | `Rule::priority()` |
| `lifetime permanent` | `RuleLifetime::Permanent` |
| `lifetime turns 3` | `RuleLifetime::Turns(3)` |
| `lifetime triggers 1` | `RuleLifetime::Triggers(1)` |
| `on Event(...)` | typed event match inside generated `before()` |
| `when ...` | generated guard checks before effects |
| `emit SomeOp(...)` | `tx.ops.push(...)` |
| `cancel` | `tx.cancelled = true` |
| `set tx.irreversible = false` | `tx.irreversible = false` |
| `set tx.deterministic = false` | `tx.deterministic = false` |

## Minimal Authoring Example

```text
rule "scoring.touchdown_bonus" {
  priority 10
  lifetime permanent
  on TouchdownScored(team)
  when state.scoring_mode == "touchdown_plus_one"
  before {
    emit AwardPoints(team: team, points: 1)
  }
}
```

This is in scope because it:

- has a stable id
- has explicit priority and lifetime
- matches a constrained event shape
- uses a simple guard
- emits a reversible operation from `before()`
- does not mutate state directly
- does not rely on `after()`

## Implementation Guidance for Phase 5

Phase 5 should treat this document as a contract, not inspiration.

- If a syntax form is not described here, it is out of scope.
- If a semantic capability does not map directly to `Rule`, `Transaction`, `RuleLifetime`, or emitted `Operation`s, it is out of scope.
- Validation must reject authored constructs that imply hidden mutation, scripting, or `after()`-phase mutation.
