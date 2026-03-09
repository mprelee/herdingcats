---
phase: 05-build-time-compiler-and-example-integration
plan: "03"
subsystem: integration-example
tags: [rust, build-rs, out-dir, example, integration]

requires:
  - phase: 05-build-time-compiler-and-example-integration
    provides: validated IR and deterministic Rust emission from Plans 01-02
provides:
  - real consumer fixture with build-time DSL compilation
  - generated runtime module included from `OUT_DIR`
  - mixed handwritten and generated rule registration in one engine
  - end-to-end dispatch proof from authored DSL to engine state change
  - integration regression coverage for the consumer path
affects: [06-01-validation-suite, 06-02-release-hardening]

tech-stack:
  added: []
  patterns:
    - "consumer build.rs invokes codegen crate"
    - "generated runtime module wrapped in stable `generated_rules` namespace"
    - "generated module references consumer types via parent-scope paths"
    - "integration test executes nested Cargo consumer as a real fixture"

key-files:
  created:
    - examples/dsl_consumer/Cargo.toml
    - examples/dsl_consumer/build.rs
    - examples/dsl_consumer/rules/example.cats
    - examples/dsl_consumer/src/main.rs
  modified:
    - crates/herdingcats_codegen/src/codegen.rs
    - tests/phase5_codegen.rs

key-decisions:
  - "Runtime codegen now emits a nested `generated_rules` module so included output has a predictable namespace in consumer crates."
  - "Generated runtime code uses fully qualified `herdingcats` paths plus parent-scope consumer types to avoid import collisions in `include!` sites."
  - "The integration proof uses a nested example crate executed by test, which validates real `build.rs` behavior instead of a mocked call path."

requirements-completed: [INT-01, INT-02]

completed: "2026-03-09"
---

# Phase 5 Plan 03 Summary

## What Was Built

Added a real example consumer crate under `examples/dsl_consumer` that compiles authored DSL from `build.rs`, writes generated Rust into `OUT_DIR`, includes the generated module, and registers generated rules beside a handwritten rule in the same engine instance.

Adjusted runtime code generation to emit a stable `generated_rules` module with valid enum field syntax and parent-scoped type references so the included code compiles cleanly inside a consumer crate. Added an end-to-end test that runs the nested example through Cargo and proves the authored DSL rule changes engine behavior during a real dispatch.

## Verification Run

Executed:

```bash
cargo test
test -f examples/dsl_consumer/build.rs
test -f examples/dsl_consumer/src/main.rs
test -f examples/dsl_consumer/rules/example.cats
```

Result:
- root crate tests green
- codegen tests green
- nested consumer end-to-end test green
- required integration artifacts present

## Task Commits

1. **Task 1: example consumer fixture and build integration** — pending commit
2. **Task 2: mixed registration path and end-to-end proof** — pending commit

## Deviations

No scope deviations from Plan 05-03.
