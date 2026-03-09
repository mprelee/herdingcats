# Host Binding Contract

This document defines what consumer-owned state and event information the DSL may reference.

## Binding Philosophy

The DSL does not get arbitrary access to Rust types.

Instead, the consumer exposes a narrow binding surface the compiler is allowed to target. The binding surface exists so Phase 5 can generate code without reflection, runtime scripting, or engine trait redesign.

## State Bindings

The consumer may expose:
- named state fields
- named read-only helper values
- named read-only helper predicates

The DSL may not assume:
- arbitrary field traversal
- method discovery
- mutable references into state
- private implementation details

## Event Bindings

The consumer may expose:
- event variant names
- explicitly approved event fields for matching or guard checks

V1 recommendation:
- support variant matching by default
- allow field extraction only when the consumer declares it explicitly

## Operation Bindings

Generated rules need an operation target. The binding contract must say whether effects emit:

- generated operations only
- consumer-owned handwritten operations only
- a wrapper enum that can carry both handwritten and generated operations

V1 recommendation:
- assume a consumer-visible wrapper operation path is configured explicitly when mixed operation families are needed

## Transaction Flag Bindings

Only approved existing transaction fields may be surfaced:
- `cancelled`
- `deterministic`
- `irreversible`

No new transaction semantics are introduced in v1.1.

## Consumer-Provided Configuration

Phase 5 should expect explicit configuration for:
- state type path
- event type path
- operation type or wrapper path
- approved state bindings
- approved event bindings
- approved helper predicates or helper values

## Compiler Inference Limits

The compiler may infer:
- omitted lifetime -> `permanent`
- simple normalized rule ordering
- normalized binding references after configuration is loaded

The compiler should not infer:
- hidden state access
- arbitrary type relationships
- reversibility for lossy effects
- unsupported event field access

## Why This Contract Matters

Without a concrete binding surface:
- Phase 5 parser/codegen work would guess at consumer types
- the feature would drift toward runtime scripting
- implementation pressure would spill into engine API changes

This contract keeps the DSL additive and build-time only.
