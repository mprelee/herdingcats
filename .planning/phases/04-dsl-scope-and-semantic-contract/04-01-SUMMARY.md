---
phase: 04-dsl-scope-and-semantic-contract
plan: "01"
subsystem: docs
tags: [dsl, pest, docs, rules, transaction, semantics]

requires:
  - phase: 04-dsl-scope-and-semantic-contract
    provides: research, plan, and validation constraints for the DSL boundary
provides:
  - Narrow v1.1 DSL contract locked to engine-compatible before() semantics
  - Grammar scope skeleton for Phase 5 parser work
  - Concrete accepted examples tied to Rule, RuleLifetime, Transaction, and reversible Operation semantics
  - Concrete rejected examples covering direct state mutation, after() mutation, scripting, and runtime loading
affects: []

tech-stack:
  added: []
  patterns:
    - "File-based rule blocks parsed from a .pest grammar"
    - "before()-only authored mutation surface"
    - "Direct state mutation rejected in favor of emitted reversible operations"
    - "Transaction flags limited to cancelled, irreversible=false, and deterministic=false semantics already present in Transaction"

key-files:
  created:
    - docs/dsl/README.md
    - docs/dsl/grammar-scope.pest
    - docs/dsl/examples/accepted-rules.md
    - docs/dsl/examples/rejected-rules.md
  modified: []

key-decisions:
  - "The Phase 4 DSL surface is rule-centric and file-based: id, priority, lifetime, on, when, before."
  - "Authored mutation is limited to before() and must lower into emitted reversible operations or explicit Transaction flags."
  - "The grammar scope intentionally excludes after blocks, direct assignments, loops, and arbitrary expressions."
  - "Examples are written against engine semantics rather than aspirational language features."

requirements-completed: [DSL-01, DSL-02, DSL-03, DSL-04]

completed: "2026-03-09"
---

# Phase 4 Plan 01 Summary

## What Was Built

Created the Phase 4 DSL contract docs from scratch under `docs/dsl/`. The new README locks the smallest useful v1.1 authored rule surface and ties every allowed concept back to existing engine semantics in `Rule`, `RuleLifetime`, `Transaction`, and `Operation`.

Added a `grammar-scope.pest` skeleton that shows the intended file-based `.pest` syntax without overreaching into unsupported semantics. It includes rule blocks, priority and lifetime declarations, constrained event matching, simple guards, and a `before` block with only the approved effect categories.

Added accepted and rejected example sets to make the boundary concrete. Accepted examples show useful patterns that remain reversible and `before()`-compatible. Rejected examples explain exactly why direct state mutation, mutating `after()`, arbitrary evaluation, runtime loading, and loop constructs are not part of v1.1.

## Key Files

- `docs/dsl/README.md`
- `docs/dsl/grammar-scope.pest`
- `docs/dsl/examples/accepted-rules.md`
- `docs/dsl/examples/rejected-rules.md`

## Verification Run

Executed:

```bash
cd /Users/mprelee/herdingcats && test -f docs/dsl/README.md && test -f docs/dsl/grammar-scope.pest && rg "before\\(\\)|RuleLifetime|priority|guard|out of scope" docs/dsl/README.md
cd /Users/mprelee/herdingcats && test -f docs/dsl/examples/accepted-rules.md && test -f docs/dsl/examples/rejected-rules.md && rg "accepted|rejected|after\\(\\)|reversible|RuleLifetime" docs/dsl/examples/*.md
cd /Users/mprelee/herdingcats && test -f docs/dsl/README.md && test -f docs/dsl/grammar-scope.pest && test -f docs/dsl/examples/accepted-rules.md && test -f docs/dsl/examples/rejected-rules.md && rg "before\\(\\)|out of scope|direct state mutation|after\\(\\)" docs/dsl/README.md docs/dsl/examples/*.md
cargo test
```

## Deviations

### Auto-fixed during phase verification

- Normalized accepted examples to match the locked DSL contract exactly:
  - quoted rule ids
  - explicit `on ...` event matches
  - `when ...` guards instead of ad hoc `guard` lines
  - required `before { ... }` block
  - `turns 2` / `triggers 1` lifetime syntax
  - `set tx.deterministic = false` transaction flag syntax

This was a consistency fix across Phase 4 artifacts, not a scope change.
