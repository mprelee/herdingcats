---
phase: 05-build-time-compiler-and-example-integration
plan: "01"
subsystem: compiler-front-end
tags: [rust, pest, parser, ast, ir, validation]

requires:
  - phase: 04-dsl-scope-and-semantic-contract
    provides: locked DSL syntax, semantic contract, validation rules, and binding model
provides:
  - `herdingcats_codegen` companion crate
  - parser front-end wired to the Phase 4 grammar contract
  - AST types with spans and validated IR types
  - binding/config resolution and semantic validation before codegen
  - front-end tests for happy path and failure modes
affects: [05-02-code-generation, 05-03-integration-example]

tech-stack:
  added:
    - "pest 2.8.6"
    - "pest_derive 2.8.6"
  patterns:
    - "parser -> AST -> validated IR"
    - "separate companion crate for build-time compiler code"
    - "semantic failures happen before Rust emission"
    - "binding validation resolves event/state/op access explicitly"

key-files:
  created:
    - crates/herdingcats_codegen/Cargo.toml
    - crates/herdingcats_codegen/src/lib.rs
    - crates/herdingcats_codegen/src/parser.rs
    - crates/herdingcats_codegen/src/ast.rs
    - crates/herdingcats_codegen/src/ir.rs
    - crates/herdingcats_codegen/src/validate.rs
    - crates/herdingcats_codegen/src/bindings.rs
    - crates/herdingcats_codegen/src/diagnostics.rs
    - tests/phase5_codegen.rs
  modified:
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Root Cargo.toml now acts as both package manifest and workspace root so the companion crate can live in-tree."
  - "Lifetime parsing follows the locked Phase 4 concrete syntax (`turns 2`, `triggers 1`) instead of changing the grammar contract."
  - "Validation rejects semantic violations before codegen rather than depending on emitted Rust errors."

requirements-completed: []

completed: "2026-03-09"
---

# Phase 5 Plan 01 Summary

## What Was Built

Created the `herdingcats_codegen` companion crate and implemented the full front-end pipeline through validated IR. The new crate parses authored DSL using `pest`, builds AST nodes with source spans, loads binding/config rules, and lowers only semantically valid input into IR for later code generation.

Added semantic validation for the Phase 4 contract: duplicate ids, invalid lifetime values, unapproved event or state bindings, and unapproved emitted operations all fail before any Rust generation is attempted. Added root integration tests proving one happy-path lowering flow and several failure modes.

## Verification Run

Executed:

```bash
cargo test
```

Result:
- root crate tests green
- new `tests/phase5_codegen.rs` suite green
- doctests green

## Task Commits

1. **Task 1: codegen front-end crate** — `87feaad`
2. **Task 2: validation layer and tests** — `fd1e42e`

## Deviations

No scope deviations from Plan 05-01.
