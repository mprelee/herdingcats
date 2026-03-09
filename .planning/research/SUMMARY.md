# Project Research Summary

**Project:** herdingcats
**Domain:** Build-time DSL and Rust code generation for additional rules
**Researched:** 2026-03-09
**Confidence:** MEDIUM

## Executive Summary

The recommended v1.1 direction is a build-time companion compiler around `pest`, not a runtime parser or proc-macro-first API. Consumers keep their core game state, events, and baseline operations in Rust, author additional rules in a constrained DSL, and compile those rules into ordinary Rust `Rule` and `Operation` implementations that plug into the current engine unchanged.

Research repeatedly pointed to the same risk: this is not mainly a parser feature. It is an invariants-preservation feature attached to a parser. The DSL has to compile into the actual `herdingcats` model of reversible operations, deterministic hashing, stable rule ids, and transaction-driven mutation. The current engine code also makes `before()` the safe target for mutating generated rules; `after()` mutation should stay out of v1.1 scope.

## Key Findings

### Recommended Stack

Keep the runtime crate lean and move parsing/codegen into a separate build-time crate. Use `pest` + `pest_derive` for parsing, lower into an IR, and emit Rust with `quote`/`proc-macro2`. Use `build.rs` in the consumer crate to write generated Rust into `OUT_DIR`, and add `trybuild` plus `proptest`-based invariant tests around the generated output.

**Core technologies:**
- `pest` / `pest_derive`: PEG grammar parsing for authored DSL files
- Build-time companion crate: parser, validator, IR, and Rust generator kept out of runtime dependencies
- `quote` / `proc-macro2`: structured Rust emission for generated rules and operations

### Expected Features

The narrow, credible v1.1 scope is: authored rule ids, priorities, lifetimes, event matching, guard conditions, and reversible op emission in `before()` hooks. Runtime loading, full game-definition language ambitions, and arbitrary scripting should stay out.

**Must have (table stakes):**
- Build-time DSL parsing and Rust generation
- Generated reversible ops and generated rule registration
- Constrained binding layer for host state/event access

**Should have (competitive):**
- Readable generated Rust
- Compile-fail diagnostics and behavioral equivalence tests

**Defer (v2+):**
- Runtime scripting/loading
- Broad `after()` mutation semantics
- Full game-definition language

### Architecture Approach

Use a separate codegen/compiler layer that emits Rust targeting the existing `Rule`, `Operation`, and `Transaction` traits. Consumers integrate through `build.rs`, generated modules, and rule registration helpers. The runtime engine should remain structurally unchanged for v1.1.

**Major components:**
1. Codegen crate — grammar, parser, validator, IR, code emission
2. Consumer build integration — `build.rs` plus generated `OUT_DIR` source
3. Validation harness — fixture example, compile-fail tests, and invariant/property tests

### Critical Pitfalls

1. **Non-deterministic codegen** — canonicalize ordering and centralize hash generation
2. **Non-reversible generated ops** — reject lossy DSL constructs and property-test apply/undo/redo
3. **DSL that hides the engine model** — keep semantics centered on `before()` and emitted transaction ops
4. **API leakage** — keep parser/codegen build-time only and avoid changing core traits
5. **Parser-only testing** — require engine invariant tests with generated code

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 4: DSL Scope and Semantic Contract
**Rationale:** The grammar cannot be designed safely until the engine-compatible semantic model is fixed
**Delivers:** DSL scope, AST/IR, reversibility rules, binding model
**Addresses:** Authoring Surface, Semantics and Generation
**Avoids:** DSL/engine mismatch and undo-contract drift

### Phase 5: Build-Time Compiler and Example Integration
**Rationale:** Once semantics are fixed, parser/codegen and consumer integration can be built end-to-end
**Delivers:** `pest` grammar, parser, codegen, `build.rs` integration, example fixture
**Uses:** `pest`, generated modules, companion codegen crate
**Implements:** Build-time compiler architecture

### Phase 6: Validation and Release Hardening
**Rationale:** Generated code must prove engine invariants before the feature can be trusted
**Delivers:** compile-fail tests, equivalence tests, generated-op property tests, docs and API-boundary hardening

### Phase Ordering Rationale

- Semantic constraints come before syntax to avoid designing a DSL the engine cannot honor
- End-to-end build integration belongs before release hardening because diagnostics and tests depend on a working pipeline
- Validation is a first-class phase because determinism and undo correctness are the core value of the crate

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4:** Binding/config design between generic host types and generated code
- **Phase 6:** Generated-code diagnostics and compile-fail ergonomics

Phases with standard patterns (skip research-phase):
- **Phase 5:** `pest` parser plus `build.rs` source generation is a standard Rust build-time pattern

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | Recommended structure is clear, but versions were not verified against external docs in this session |
| Features | MEDIUM | Scope is coherent and aligned with user intent |
| Architecture | MEDIUM | Integration path fits the current engine, with one important `before()`-only constraint |
| Pitfalls | HIGH | Risks are driven directly by current engine invariants and code shape |

**Overall confidence:** MEDIUM

### Gaps to Address

- Binding layer design: how generated code safely references consumer state/event helpers
- Whether v1.1 needs any engine fix around `after()` semantics or can simply scope generated mutation to `before()`
- Exact generated-code surface for consumer ergonomics without creating accidental public API

## Sources

### Primary (HIGH confidence)
- `.planning/PROJECT.md` — milestone scope and constraints
- `src/rule.rs`, `src/engine.rs`, `src/operation.rs`, `src/transaction.rs` — runtime semantics
- Local research synthesis from parallel explorer agents

### Secondary (MEDIUM confidence)
- `.planning/codebase/*.md` documents — current architecture, concerns, and stack posture

---
*Research completed: 2026-03-09*
*Ready for roadmap: yes*
