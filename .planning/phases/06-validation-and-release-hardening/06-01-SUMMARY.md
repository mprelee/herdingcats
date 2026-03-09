---
phase: 06-validation-and-release-hardening
plan: "01"
subsystem: validation-suite
tags: [rust, validation, proptest, diagnostics, integration]

requires:
  - phase: 05-build-time-compiler-and-example-integration
    provides: real consumer fixture and generated runtime integration path
provides:
  - dedicated Phase 6 validation harness in root integration tests
  - generated-op determinism and roundtrip coverage through the real consumer fixture
  - generated-rule undo/replay-hash validation through nested consumer tests
  - failing-build diagnostics coverage using build.rs against overridden authored rule files
affects: [06-02-release-hardening]

tech-stack:
  added: []
  patterns:
    - "real nested consumer tests invoked from root integration harness"
    - "generated-op and generated-rule invariants validated through engine behavior, not source snapshots"
    - "build.rs accepts rule-path override for fixture-driven compile-fail coverage"
    - "consumer fixture split into reusable library plus demo binary"

key-files:
  created:
    - tests/phase6_validation.rs
    - examples/dsl_consumer/src/lib.rs
  modified:
    - examples/dsl_consumer/Cargo.toml
    - examples/dsl_consumer/build.rs
    - examples/dsl_consumer/src/main.rs
    - examples/dsl_consumer/rules/example.cats

key-decisions:
  - "Phase 6 validates generated behavior by running tests inside the real nested consumer crate instead of inventing a separate mock fixture."
  - "The consumer build script now accepts `HERDINGCATS_DSL_RULES_PATH` so compile-fail diagnostics can exercise the real build pipeline with temporary DSL files."
  - "The example consumer was split into a reusable library plus thin binary so generated operations and generated rules can be tested directly without duplicating fixture code."

requirements-completed: [GEN-04, INT-03, INT-04]

completed: "2026-03-09"
---

# Phase 6 Plan 01 Summary

## What Was Built

Added `tests/phase6_validation.rs` as the Wave 1 harness for generated-operation determinism, generated-operation roundtrip, generated-rule undo/replay-hash behavior, and failing-build diagnostics. The harness drives the real `examples/dsl_consumer` crate rather than a mocked fixture, so the tests cover the actual `build.rs` and `OUT_DIR` integration path.

Refactored `examples/dsl_consumer` into a small library plus thin binary, added nested tests for generated-op and generated-rule invariants, and extended the DSL fixture with a cancel rule so Phase 6 can prove no-op/cancel behavior leaves state and replay hash unchanged. The build script now accepts an override rules path and emits more assertable failure text for compile-fail coverage.

## Verification Run

Executed:

```bash
cargo test --test phase6_validation
cargo test --test phase5_codegen
cargo test generated_rule -- --nocapture
test -f tests/phase6_validation.rs
```

Result:
- Phase 6 validation harness green
- existing Phase 5 codegen suite still green
- generated-rule filtered verification green
- required validation artifact present

## Task Commits

1. **Task 1: generated-op and generated-rule invariant coverage** — pending commit
2. **Task 2: failing-build diagnostics coverage** — pending commit

## Deviations

No scope deviations from Plan 06-01.
