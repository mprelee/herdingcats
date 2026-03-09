# Phase 05: Build-Time Compiler and Example Integration - Research

**Researched:** 2026-03-09
**Domain:** Build-time parser, validated IR, Rust code generation, and consumer integration for the `herdingcats` DSL
**Confidence:** HIGH

<user_constraints>
## User Constraints (from completed Phase 4 and milestone requirements)

### Locked Decisions

**Compiler model:**
- The feature remains build-time source generation, not runtime interpretation
- Phase 5 must work from the locked Phase 4 DSL contract, not invent new syntax
- Generated code must target the current `Rule`, `Operation`, `Transaction`, and `Engine` contracts additively

**Semantic model:**
- Code generation must consume validated IR, never raw parser output
- Authored mutating behavior remains `before()`-only in v1.1
- Authored effects must lower to reversible operations or explicit approved transaction edits

**Integration model:**
- The consumer path must be real in this phase: `build.rs`, `OUT_DIR`, generated module inclusion, and rule registration
- Phase 5 must prove a real example or fixture consumer, not just unit-test internals

### Claude's Discretion
- Exact crate/module names for the companion codegen implementation
- Whether to use an example crate, fixture crate, or both for the end-to-end proof
- Detailed split of parser vs validation responsibilities, as long as the validated-IR boundary is preserved

### Deferred Ideas (OUT OF SCOPE)

- Phase 6 invariant hardening beyond smoke/compile validation
- runtime parser dependencies in the main `herdingcats` crate
- expanding the authored language beyond the locked Phase 4 contract
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GEN-02 | Build-time generation emits Rust code that compiles into engine-compatible `Rule` implementations | Phase 5 needs a dedicated codegen module and deterministic Rust emission path |
| GEN-03 | Generated mutations flow through reversible `Operation` implementations rather than direct state mutation | Plans must preserve the `Operation<S>` model and emitted-op flow through `Transaction::ops` |
| GEN-05 | Generated output includes a registration path for generated and handwritten rules | Phase 5 should emit registration helpers and define operation wrapper integration |
| INT-01 | Consumer crate can compile authored rules during `build.rs` and include generated Rust from `OUT_DIR` | Phase 5 must include a real build script integration path |
| INT-02 | Repository contains an end-to-end example or fixture proving authored DSL -> generated Rust -> engine dispatch | The final plan must build a concrete consumer example, not a mock |
</phase_requirements>

---

## Summary

Phase 5 is the compiler phase. Phase 4 already locked the authored syntax and semantic model, so the job here is to turn that contract into a working build-time pipeline with three clean layers: parser + validation, deterministic Rust generation, and a real integration consumer. The central rule is unchanged from the milestone research: the generated system must compile into the existing engine model, not around it.

The recommended split is exactly the roadmap split: Plan 05-01 builds the front-end and validated IR; Plan 05-02 builds deterministic Rust emission and generated registration helpers; Plan 05-03 proves the consumer path end-to-end through `build.rs` and a fixture/example crate. That order avoids the two biggest risks: emitting Rust before normalization, and delaying the integration example until architectural mistakes are expensive to fix.

## Standard Stack

### Core
| Component | Purpose | Why Standard Here |
|----------|---------|-------------------|
| Companion crate such as `crates/herdingcats_codegen` | Keeps parser/codegen out of runtime crate surface | Matches Phase 4 additive architecture and avoids runtime dependency drift |
| `pest` grammar front-end | Parse external authored DSL files | Already chosen by milestone direction |
| AST -> validated IR -> codegen pipeline | Separates syntax from semantics | Required by `GEN-01` and necessary for deterministic codegen |

### Supporting
| Component | Purpose | When to Use |
|----------|---------|-------------|
| Binding/config loader | Resolve approved state/event/op paths | Needed before IR validation can succeed |
| Generated registration helper | Consumer-friendly rule loading path | Needed for `GEN-05` and the fixture consumer |
| Consumer fixture/example | Real build integration proof | Needed for `INT-01` and `INT-02` |

## Architecture Patterns

### Pattern 1: Companion Codegen Crate

Recommended layout:

```text
crates/herdingcats_codegen/
  src/lib.rs
  src/parser.rs
  src/ast.rs
  src/ir.rs
  src/validate.rs
  src/bindings.rs
  src/codegen.rs
  src/render.rs
  src/diagnostics.rs
```

This keeps the compiler pipeline modular and aligns directly with the Phase 4 contract artifacts.

### Pattern 2: IR-First Codegen

The generator should only see validated IR that already guarantees:
- unique ids
- resolved lifetimes
- approved bindings
- deterministic effect order
- no forbidden `after()` or direct-mutation semantics

This makes codegen mechanical and keeps semantic failures out of Rust emission.

### Pattern 3: Real Consumer Integration

The repository should contain a real integration target, such as:

```text
examples/dsl_consumer/
  Cargo.toml
  build.rs
  rules/*.cats
  src/main.rs
```

That fixture should:
- call the codegen crate from `build.rs`
- write generated Rust into `OUT_DIR`
- include the generated module
- register generated rules into an engine
- dispatch at least one event that proves the authored rule path works

## Validation Architecture

Phase 5 validates compiler-pipeline correctness at the boundaries:

1. Parse validation
   - grammar errors
   - span capture
2. AST validation
   - malformed sections or literals
3. Semantic validation during AST -> IR lowering
   - duplicate ids
   - invalid lifetimes
   - unapproved bindings
   - forbidden `after()` semantics
   - non-reversible effects
4. IR readiness validation
   - canonical rule/effect ordering
   - resolved binding names
   - emission-safe identifiers
5. Generated-source smoke validation
   - emitted Rust compiles in a consumer fixture

Phase boundary:
- Phase 5 proves the compiler pipeline works
- Phase 6 proves generated behavior preserves deeper engine invariants

## Risks and Watchouts

### Risk 1: Binding Model Drift

If Plan 05-01 does not load and resolve binding/config data early, later plans will re-invent implicit rules that Phase 4 explicitly forbids.

### Risk 2: Emitting Before Normalizing

If Plan 05-02 consumes parser output or partially validated structures, deterministic generation and diagnostics quality will degrade quickly.

### Risk 3: Fake Integration

If Plan 05-03 uses only mocked tests instead of a real consumer fixture with `build.rs`, it will not satisfy `INT-01` or reveal the actual integration pain points.

## Recommended Split

### Plan 05-01 — Parser, AST, Binding Load, and IR Validation
- build file loader and grammar wiring
- define AST and parser diagnostics
- load consumer binding/config data
- lower AST to validated IR with semantic failures

### Plan 05-02 — Rust Emission and Registration Helpers
- deterministic Rust generator from validated IR
- generated operation and rule types
- `before()` logic generation
- generated registration helper and `OUT_DIR` writer

### Plan 05-03 — Consumer Fixture and Build Integration
- build.rs invocation path
- generated module inclusion
- operation wrapper integration
- rule registration + one real dispatch flow

## Sources

- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/phases/04-dsl-scope-and-semantic-contract/04-RESEARCH.md`
- `.planning/phases/04-dsl-scope-and-semantic-contract/04-VERIFICATION.md`
- `docs/dsl/*.md`
- `src/rule.rs`
- `src/operation.rs`
- `src/transaction.rs`
- `src/engine.rs`

---
*Phase researched: 05-build-time-compiler-and-example-integration*
*Ready for planning: yes*
