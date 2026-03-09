# AST and IR Contract

This document defines the semantic boundary between parsing and code generation for the planned `herdingcats` DSL.

## Pipeline

Phase 5 should implement the DSL as a layered pipeline:

1. parse authored files with `pest`
2. build an AST that preserves authored structure
3. lower the AST into a validated IR
4. generate Rust from the IR only

Code generation must not consume raw parser pairs directly.

## AST Goals

The AST should preserve authored intent and diagnostics structure.

Minimum AST concepts:
- rule definition
- rule id
- priority literal
- lifetime literal
- event match
- guard expressions
- effect list
- source location or span metadata for diagnostics

The AST is syntax-oriented:
- it preserves what the author wrote
- it may still contain unresolved names
- it may still contain invalid constructs

## Validated IR Goals

The IR is semantic and implementation-facing.

The IR should contain only constructs that Phase 5 codegen can safely turn into Rust.

Minimum IR concepts:
- `RuleSpec`
  - stable resolved id
  - normalized priority
  - normalized lifetime
  - resolved event matcher
  - resolved guard predicates
  - validated effect list
- `EventMatcher`
  - variant-only match or approved field match
- `GuardPredicate`
  - approved comparison over approved bindings
- `EffectSpec`
  - reversible op emission
  - transaction cancellation
  - approved transaction flag update

## Lowering Rules

AST to IR lowering must:
- resolve omitted lifetime to `permanent`
- normalize priority to one resolved numeric or consumer-mapped value
- resolve event references against the approved event surface
- resolve guard bindings against the approved binding surface
- reject unsupported syntax before IR emission

## Invariants the IR Must Guarantee

Once a rule reaches IR, Phase 5 codegen may assume:

- rule id is unique and stable within the authored set
- lifetime is valid and maps cleanly onto `RuleLifetime`
- event matching uses only approved event access
- guards use only approved state/event bindings
- every effect is representable as `before()`-phase transaction behavior
- no effect implies direct state mutation
- no effect implies mutating `after()` behavior
- emitted operations can be generated in a deterministic field order

## Normalization Expectations

The IR should remove ambiguity from authored syntax:

- equivalent guard forms should normalize to a consistent predicate structure
- equivalent lifetime forms should normalize to one representation
- effect order should be explicit and preserved
- binding references should be canonicalized so later codegen can emit stable names

## Why This Layer Exists

`GEN-01` requires a validated IR because:
- parse success is not semantic success
- good diagnostics should fail before Rust emission where possible
- codegen should be mechanical, not full of semantic branching
- reversibility and engine-compatibility constraints belong in validation, not in ad hoc emit logic
