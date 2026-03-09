# Pitfalls Research

**Domain:** Build-time generated rules for `herdingcats`
**Researched:** 2026-03-09
**Confidence:** MEDIUM

## Major Pitfalls

### Non-deterministic Generated Behavior

- Unstable iteration order in codegen can change emitted rule/op ordering
- Generated identifiers or hash bytes can accidentally depend on paths, timestamps, or unordered maps
- This directly threatens replay-hash determinism

Prevention:
- Canonicalize ordering everywhere in the generator
- Centralize `hash_bytes()` emission
- Keep deterministic fixture tests

### Undo Logic That Is Not Actually Reversible

- DSL-authored mutations can compile while still failing the strict `apply`/`undo` inverse contract
- Lossy assignments and deletions are especially dangerous

Prevention:
- Reject DSL constructs that cannot prove an inverse
- Require prior-state capture for lossy mutations
- Property-test apply/undo/redo for generated operations

### DSL Semantics That Do Not Match the Engine Model

- The engine works as `event -> rule hooks -> transaction ops -> committed mutations`
- A DSL that feels like “mutate state directly” will generate awkward or incorrect Rust
- `after()` is not a safe target for new mutating generated ops under current dispatch semantics

Prevention:
- Center the DSL on `before()`-phase guards and emitted reversible ops
- Keep the language small and explicit about transaction effects

### Public API Leakage

- Generated types or parser concerns can become accidental stable API if exposed too directly
- Runtime parser dependencies would broaden the crate surface unnecessarily

Prevention:
- Keep parser/codegen build-time only
- Keep runtime trait surface unchanged if possible
- Treat generated Rust as an integration detail

### Testing the Parser but Not the Invariants

- It is easy to stop at “grammar parses” and “generated code compiles”
- That misses the engine’s actual risk surface: determinism, lifetime behavior, undo/redo, preview isolation

Prevention:
- Add parser/codegen tests, behavioral equivalence tests, and engine property tests using generated rules
- Keep invariant tests as release blockers, not cleanup

## Phase Guidance

- Phase 4 should lock deterministic codegen and engine-compatible DSL semantics first
- Phase 5 should prove end-to-end generated integration with a fixture/example
- Phase 6 should harden tests, diagnostics, and public integration boundaries

## Sources

- `.planning/PROJECT.md`
- `.planning/codebase/CONCERNS.md`
- `.planning/codebase/TESTING.md`
- `src/rule.rs`
- `src/operation.rs`
- Local research synthesis from parallel explorer agents

---
*Pitfalls research for: herdingcats generated-rule DSL*
*Researched: 2026-03-09*
