# Semantic Mapping to `herdingcats`

This document explains how authored DSL rules map onto existing engine behavior.

## Core Mapping

Each authored rule becomes generated Rust that fits the current `Rule` model:

- authored `rule id` -> `Rule::id()`
- authored `priority` -> `Rule::priority()`
- authored `lifetime` -> `RuleLifetime`
- authored `when` + `guard` -> logic inside `before()`
- authored `emit` -> pushes operations into `Transaction::ops`
- authored `cancel` -> sets `tx.cancelled = true`
- authored transaction flags -> update approved `Transaction` fields in `before()`

## `before()`-Only Mutation

V1 semantics allow only `before()`-phase mutation.

Reason:
- the engine applies transaction ops after `before()` hooks
- this is the natural place for generated mutation to participate in one dispatch
- it keeps authored semantics aligned with the current engine contract

Generated rules may still observe the current state and incoming event in `before()`.

## Reversible Effects Only

Authored effects must lower to reversible operations or explicit transaction edits.

Allowed effect categories:
- emit a generated operation
- emit a wrapper operation that carries prior state where needed
- cancel the transaction
- set explicitly approved transaction flags

Forbidden effect categories:
- direct state assignment
- hidden snapshots
- side-effecting callbacks
- non-reversible mutations with no stored prior value

## Deterministic Operation Shape

Phase 6 will verify `hash_bytes()` behavior in generated code, but Phase 4 still locks the semantic preconditions:

- emitted operations must have canonical field ordering
- equal authored effects must lower to equal IR/effect shapes
- effect order within a rule is preserved
- generated identifiers must not depend on unstable input ordering

## Lifetime Semantics

Authored lifetimes map directly onto existing engine semantics:

- `permanent` -> `RuleLifetime::Permanent`
- `turns(n)` -> `RuleLifetime::Turns(n)`
- `triggers(n)` -> `RuleLifetime::Triggers(n)`

The DSL does not invent new lifecycle behavior in v1.1.

## Guard Semantics

Guards are pure predicates over approved bindings.

They:
- read approved state/event values
- evaluate before effects are emitted
- never mutate anything
- decide whether the rule contributes transaction behavior for the current dispatch

## Why `after()` Is Rejected

The DSL does not expose mutating `after()` semantics in v1.1.

This is a semantic decision, not just a syntax omission:
- the current milestone promises only behavior that safely maps to one-dispatch transaction mutation
- `after()` mutation would require broader guarantees than Phase 4 is establishing
- the generated language should not imply engine behavior that later phases cannot defend

## Consumer Responsibilities

The consumer still owns:
- state type definitions
- event type definitions
- core handwritten operations and rules
- choosing which bindings and helpers are exposed to the DSL

The DSL is additive. It extends handwritten systems; it does not replace them.
