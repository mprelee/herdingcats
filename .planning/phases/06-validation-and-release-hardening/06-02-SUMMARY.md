---
phase: 06-validation-and-release-hardening
plan: "02"
subsystem: release-hardening
tags: [rust, diagnostics, compatibility, docs, release]

requires:
  - phase: 06-validation-and-release-hardening
    provides: validation harness and compile-fail coverage from Plan 01
provides:
  - richer structured diagnostics with rule/file/help context
  - handwritten-only compatibility checks for library and examples
  - explicit build-time DSL boundary documentation in README and crate docs
  - warning-free generated consumer example path for release verification
affects: []

tech-stack:
  added: []
  patterns:
    - "diagnostics carry stable authored context and corrective help"
    - "compatibility treated as automated regression coverage, not just manual verification"
    - "top-level docs mirror the actual build.rs plus OUT_DIR integration architecture"
    - "generated runtime output avoids avoidable warning noise in the example consumer"

key-files:
  modified:
    - crates/herdingcats_codegen/src/diagnostics.rs
    - crates/herdingcats_codegen/src/lib.rs
    - crates/herdingcats_codegen/src/validate.rs
    - crates/herdingcats_codegen/src/codegen.rs
    - tests/phase5_codegen.rs
    - README.md
    - docs/dsl/README.md
    - src/lib.rs
    - examples/dsl_consumer/build.rs
    - examples/dsl_consumer/src/lib.rs

key-decisions:
  - "Diagnostics now carry optional source path, rule id, and help text so build failures stay assertable and downstream-facing."
  - "Handwritten-only compatibility is enforced through automated Cargo example/test invocations, not left as an implicit side effect of the main test suite."
  - "Release docs now treat the DSL path as a separate build-time companion flow and explicitly preserve handwritten-only usage as the default runtime story."
  - "Generated runtime code now pattern-matches unused event bindings with `_` and uses `match` arms so the example consumer does not emit avoidable generated-code warnings."

requirements-completed: [INT-03, INT-05]

completed: "2026-03-09"
---

# Phase 6 Plan 02 Summary

## What Was Built

Upgraded `herdingcats_codegen` diagnostics so parse, validation, and I/O failures can carry authored file context, rule ids, and stable help text. Validation errors now include corrective direction, and the consumer `build.rs` surfaces that richer diagnostic formatting directly through the real build path.

Added explicit handwritten-compatibility regression coverage in `tests/phase5_codegen.rs`, updated the top-level README, crate docs, and DSL docs to make the build-time-only boundary explicit, and cleaned up generated runtime output so the example consumer no longer emits avoidable warnings during release verification.

## Verification Run

Executed:

```bash
cargo test
cargo test --lib --examples
cargo run --quiet --example tictactoe
cargo run --quiet --example backgammon
cargo run --quiet --manifest-path examples/dsl_consumer/Cargo.toml
rg "build-time|runtime parser|runtime scripting|after\\(\\)|handwritten" README.md docs/dsl/README.md src/lib.rs
```

Result:
- full repository test suite green
- handwritten library/examples compatibility checks green
- nested generated consumer run green
- boundary language present in README, DSL docs, and crate docs

## Task Commits

1. **Task 1: diagnostics polish and compatibility coverage** — pending commit
2. **Task 2: docs/boundary hardening** — pending commit

## Deviations

No scope deviations from Plan 06-02.
