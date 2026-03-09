# Requirements: herdingcats v1.1 Pest Feature (PEG Parser)

**Defined:** 2026-03-09
**Core Value:** The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest make this machine-verifiable, not just manually checked.

## v1 Requirements

### DSL Authoring Surface

- [x] **DSL-01**: Library consumer can define additional rules in external DSL files parsed from a `.pest` grammar
- [x] **DSL-02**: Each authored rule can declare a stable rule id, priority, and lifetime compatible with `herdingcats::Rule`
- [ ] **DSL-03**: Each authored rule can match on a constrained event surface and guard on approved state/event bindings
- [x] **DSL-04**: DSL scope is explicitly limited to semantics that compile into engine-compatible `before()` behavior

### Semantic Lowering and Code Generation

- [x] **GEN-01**: Authored DSL lowers into a validated intermediate representation before Rust code generation
- [ ] **GEN-02**: Build-time generation emits Rust code that compiles into `Rule` implementations compatible with the existing engine trait surface
- [ ] **GEN-03**: Generated mutations flow through reversible `Operation` implementations rather than direct state mutation
- [ ] **GEN-04**: Generated operations produce stable deterministic `hash_bytes()` output suitable for replay hashing
- [ ] **GEN-05**: Generated output includes a registration path that allows consumers to add generated rules alongside handwritten rules

### Consumer Integration and Validation

- [ ] **INT-01**: Consumer crate can compile authored rules during `build.rs` and include generated Rust from `OUT_DIR`
- [ ] **INT-02**: Repository contains at least one end-to-end example or fixture proving authored DSL -> generated Rust -> engine dispatch integration
- [ ] **INT-03**: Compile-fail or equivalent diagnostics tests cover invalid DSL and codegen failures with actionable feedback
- [ ] **INT-04**: Tests verify generated rules and operations preserve undo/redo and replay-hash invariants
- [ ] **INT-05**: Existing handwritten library usage, including current examples, continues to compile without requiring the DSL feature

## v2 Requirements

### Extended DSL Semantics

- **DSL-V2-01**: DSL can express safe `after()`-phase effects once engine semantics are clarified or updated
- **DSL-V2-02**: DSL can support richer composition helpers or reusable rule fragments without exposing arbitrary scripting

### Authoring Ergonomics

- **ERG-01**: Generated-code diagnostics map more directly back to authored DSL locations and concepts
- **ERG-02**: Consumer integration can be reduced further with optional helper macros or wrappers if the build-time API proves too noisy

## Out of Scope

| Feature | Reason |
|---------|--------|
| Runtime parsing or runtime compilation of rules | This milestone is explicitly for design-time iteration and static Rust compilation |
| Full game-definition language | The goal is additional rules on top of handwritten engine/game logic, not replacing core game implementation |
| Arbitrary direct state mutation from DSL | Violates the engine’s reversible `Operation` contract |
| Broad `after()` mutation support | Current engine dispatch semantics make generated mutating `after()` rules unsafe without further design work |
| Public engine trait redesign | v1.1 should fit the current `Rule`/`Operation`/`Transaction` model additively |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DSL-01 | Phase 4 | Complete |
| DSL-02 | Phase 4 | Complete |
| DSL-03 | Phase 7 | Pending |
| DSL-04 | Phase 4 | Complete |
| GEN-01 | Phase 4 | Complete |
| GEN-02 | Phase 7 | Pending |
| GEN-03 | Phase 7 | Pending |
| GEN-04 | Phase 8 | Pending |
| GEN-05 | Phase 7 | Pending |
| INT-01 | Phase 8 | Pending |
| INT-02 | Phase 8 | Pending |
| INT-03 | Phase 8 | Pending |
| INT-04 | Phase 8 | Pending |
| INT-05 | Phase 8 | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-09*
*Last updated: 2026-03-09 after milestone research*
