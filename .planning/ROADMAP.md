# Roadmap: herdingcats

## Milestones

- ✅ **v1.0 Refactor and Test** — Phases 1-3 (shipped 2026-03-09)
- ○ **v1.1 Pest Feature (PEG Parser)** — Phases 4-6 (planned)

## Overview

`herdingcats` shipped v1.0 as a stable, well-tested generic rule engine. v1.1 adds a new build-time authoring path: consumers define additional rules in a `pest`-backed DSL, compile them into Rust code, and integrate those generated rules into the existing `Rule`/`Operation` engine model without introducing runtime scripting.

## Phases

<details>
<summary>✅ v1.0 Refactor and Test (Phases 1-3) — SHIPPED 2026-03-09</summary>

- [x] Phase 1: Module Split and Foundation (3/3 plans) — completed 2026-03-09
- [x] Phase 2: Engine Property Tests (1/1 plan) — completed 2026-03-09
- [x] Phase 3: Backgammon Example and Integration Properties (2/2 plans) — completed 2026-03-09

Full details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

### Phase 4: DSL Scope and Semantic Contract
**Goal**: Define an engine-compatible DSL and semantic model that can safely compile authored rules into reversible `herdingcats` behavior
**Depends on**: Phase 3
**Requirements**: DSL-01, DSL-02, DSL-03, DSL-04, GEN-01
**Success Criteria** (what must be TRUE):
1. A `.pest` grammar and AST cover the initial authored rule surface: ids, priorities, lifetimes, event matches, and guards
2. A validated IR exists between parsing and Rust emission, rejecting unsupported or non-reversible constructs before codegen
3. The milestone documents and implementation clearly scope generated mutating rules to `before()` semantics under the current engine model
4. The binding/config approach for consumer state/event access is concrete enough for code generation without redesigning engine traits
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md — Language design: grammar scope, AST, authored rule examples, and `before()`-only semantic contract
- [ ] 04-02-PLAN.md — Semantic validation: IR, reversibility checks, stable rule-id policy, and binding/config model

### Phase 5: Build-Time Compiler and Example Integration
**Goal**: Build the end-to-end `pest` parser and Rust code generator, then prove a consumer can compile authored rules into working engine integrations
**Depends on**: Phase 4
**Requirements**: GEN-02, GEN-03, GEN-05, INT-01, INT-02
**Success Criteria** (what must be TRUE):
1. A build-time companion compiler parses authored DSL files and emits Rust source into `OUT_DIR`
2. Generated Rust compiles into engine-compatible `Rule` implementations and reversible operations without changing the core runtime traits
3. Consumer integration path is real: `build.rs`, generated-module inclusion, and generated rule registration all work end-to-end
4. At least one example or fixture demonstrates authored DSL -> generated Rust -> dispatch through `Engine`
**Plans**: 3 plans

Plans:
- [ ] 05-01-PLAN.md — Parser implementation: `pest` grammar wiring, AST construction, and parse-time diagnostics
- [ ] 05-02-PLAN.md — Code generation: Rust emission for generated ops/rules plus registration helper
- [ ] 05-03-PLAN.md — Integration example: consumer `build.rs`, generated module inclusion, and end-to-end fixture/example

### Phase 6: Validation and Release Hardening
**Goal**: Prove generated rules preserve `herdingcats` invariants and harden the public integration surface for release
**Depends on**: Phase 5
**Requirements**: GEN-04, INT-03, INT-04, INT-05
**Success Criteria** (what must be TRUE):
1. Generated operations are verified to preserve apply/undo roundtrip correctness and stable replay-hash behavior
2. Invalid DSL and unsupported semantics fail with actionable diagnostics covered by compile-fail or equivalent tests
3. Existing handwritten library usage and current examples still compile unchanged when the DSL path is unused
4. Documentation makes the v1.1 boundary explicit: build-time generation only, no runtime scripting, no broad `after()` mutation support
**Plans**: 2 plans

Plans:
- [ ] 06-01-PLAN.md — Validation suite: equivalence tests, property tests, and deterministic codegen assertions
- [ ] 06-02-PLAN.md — Release hardening: diagnostics polish, compatibility checks, docs/examples update, and final API-boundary review

## Progress

**Execution Order:**
Phases execute in numeric order: 4 → 5 → 6

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Module Split and Foundation | v1.0 | 3/3 | Complete | 2026-03-09 |
| 2. Engine Property Tests | v1.0 | 1/1 | Complete | 2026-03-09 |
| 3. Backgammon Example and Integration Properties | v1.0 | 2/2 | Complete | 2026-03-09 |
| 4. DSL Scope and Semantic Contract | v1.1 | 1/2 | In Progress | - |
| 5. Build-Time Compiler and Example Integration | v1.1 | 0/3 | Pending | - |
| 6. Validation and Release Hardening | v1.1 | 0/2 | Pending | - |
