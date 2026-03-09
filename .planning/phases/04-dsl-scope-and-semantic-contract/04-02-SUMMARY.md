---
phase: 04-dsl-scope-and-semantic-contract
plan: "02"
subsystem: docs
tags: [dsl, semantics, ir, validation, bindings]

requires:
  - phase: 04-dsl-scope-and-semantic-contract
    provides: language contract, grammar scope, and example boundary from Plan 01
provides:
  - AST and validated IR contract for later parser/codegen work
  - semantic mapping from authored rules to Rule, Transaction, and Operation behavior
  - pre-codegen validation failures for unsafe or out-of-scope constructs
  - host binding contract for approved state/event access
affects: [05-build-time-compiler-and-example-integration]

tech-stack:
  added: []
  patterns:
    - "Parser -> AST -> validated IR -> codegen"
    - "before()-only authored mutation surface"
    - "Semantic validation rejects non-reversible and after()-mutating constructs before Rust emission"
    - "Consumer-exposed binding surface instead of arbitrary Rust access"

key-files:
  created:
    - docs/dsl/ir.md
    - docs/dsl/semantics.md
    - docs/dsl/validation.md
    - docs/dsl/bindings.md
  modified: []

key-decisions:
  - "Code generation consumes validated IR, never raw parser output."
  - "Validation failures are semantic and explicit, especially for direct mutation and after()-mutating constructs."
  - "The consumer must expose a narrow binding surface for state, event, and operation access."
  - "Determinism constraints are locked at the semantic layer before hash_bytes() implementation exists."

requirements-completed: [DSL-03, DSL-04, GEN-01]

completed: "2026-03-09"
---

# Phase 4 Plan 02 Summary

## What Was Built

Defined the semantic enforcement contract for the planned DSL. The new docs lock the parser-to-codegen boundary as `parser -> AST -> validated IR -> codegen`, and explain how authored rules map onto existing `Rule`, `Transaction`, and reversible `Operation` behavior.

Added a validation contract that forces unsafe constructs to fail before Rust emission, including duplicate ids, unsupported bindings, direct state mutation, non-reversible effects, and any authored form that implies mutating `after()` semantics. Added a host binding contract that keeps consumer state/event access narrow and explicit instead of drifting into arbitrary Rust access.

## Key Files

- `docs/dsl/ir.md`
- `docs/dsl/semantics.md`
- `docs/dsl/validation.md`
- `docs/dsl/bindings.md`

## Verification Run

Executed:

```bash
cd /Users/mprelee/herdingcats && test -f docs/dsl/ir.md && test -f docs/dsl/semantics.md && rg "AST|IR|before\\(\\)|reversible|hash_bytes|after\\(\\)" docs/dsl/ir.md docs/dsl/semantics.md
cd /Users/mprelee/herdingcats && test -f docs/dsl/validation.md && test -f docs/dsl/bindings.md && rg "duplicate|reversible|binding|event|after\\(\\)|direct state mutation" docs/dsl/validation.md docs/dsl/bindings.md
cd /Users/mprelee/herdingcats && test -f docs/dsl/ir.md && test -f docs/dsl/semantics.md && test -f docs/dsl/validation.md && test -f docs/dsl/bindings.md && rg "AST|IR|duplicate|binding|before\\(\\)|after\\(\\)|reversible|hash_bytes" docs/dsl/*.md
cargo test
```

## Deviations

### Auto-fixed during phase verification

- Updated `docs/dsl/semantics.md` to describe the authored rule surface consistently as `on` + `when` rather than `when` + `guard`.

This was a contract-alignment fix so the semantics doc matched the locked Phase 4 syntax.
