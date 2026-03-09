# Phase 06: Validation and Release Hardening - Research

**Researched:** 2026-03-09
**Domain:** invariant validation, diagnostics hardening, compatibility checks, and release-boundary documentation for the v1.1 DSL pipeline
**Confidence:** HIGH

<repo_notes>
## Repo Notes

- No `CLAUDE.md` was present in `/Users/mprelee/herdingcats`.
- No repo-local `skills` directory was present in `/Users/mprelee/herdingcats`.
</repo_notes>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GEN-04 | Generated operations produce stable deterministic `hash_bytes()` output suitable for replay hashing | Phase 6 must prove generated op hashing at the engine level, not just source-string determinism |
| INT-03 | Diagnostics tests cover invalid DSL and codegen failures with actionable feedback | Phase 6 needs a fixture-driven failing-build harness plus richer diagnostic shape than the current plain strings |
| INT-04 | Tests verify generated rules and operations preserve undo/redo and replay-hash invariants | Phase 6 should reuse the existing engine and backgammon proptest patterns with generated-op fixtures |
| INT-05 | Existing handwritten usage continues to compile without requiring the DSL feature | Phase 6 must explicitly test the handwritten-only path and preserve the runtime crate's no-codegen dependency boundary |
</phase_requirements>

---

## Summary

Phase 5 proved that the DSL pipeline exists: parse -> validate -> generate -> `build.rs` -> `OUT_DIR` -> dispatch in a real consumer. Phase 6 is about proving that this new path does not weaken the core engine contract and does not blur the public boundary of the crate.

The most important planning fact is that the repo already contains the invariant patterns Phase 6 should reuse. `src/engine.rs` establishes the canonical property-test contract for undo/hash roundtrips and cancelled transactions. `examples/backgammon.rs` shows the preferred split between op-level invariant tests and engine-level integration property tests. Phase 6 should not invent a new style; it should apply those same patterns to generated operations and generated rules.

The other major finding is that diagnostics and release hardening are currently behind the implementation. The codegen crate stores spans in the AST, but `Diagnostic` only carries `{ kind, message }`, so authored-location feedback is not yet surfaced. The root crate still has an empty `[dependencies]` section and the DSL consumer keeps `herdingcats_codegen` in `[build-dependencies]`, which is the correct INT-05 architecture, but that boundary is not yet made explicit in the top-level docs or compatibility tests.

## Current Codebase Findings

### What already exists and should be reused

- `tests/phase5_codegen.rs` already proves deterministic generated source strings, runtime module shape, file writing, and a nested `dsl_consumer` end-to-end run.
- `src/engine.rs` `mod props` establishes the repository standard for invariant testing:
  - generate simple op sequences
  - snapshot both `engine.read()` and `engine.replay_hash()`
  - assert both after undo or no-op scenarios
- `examples/backgammon.rs` adds the Phase 3 pattern that matters here:
  - use direct op strategies for low-level invariants
  - use engine dispatch only for properties that specifically need rule participation
- `crates/herdingcats_codegen/src/validate.rs` already canonicalizes rule ordering by sorting IR rules by id, which is a good base for deterministic generation.
- `examples/dsl_consumer` already demonstrates the intended architecture for build-time-only usage:
  - `herdingcats` is a normal dependency
  - `herdingcats_codegen` is a build dependency
  - generated code is included from `OUT_DIR`
  - handwritten and generated rules coexist in the same engine

### Gaps Phase 6 still must close

- Generated-operation correctness is not yet verified with engine invariant tests.
- `hash_bytes()` determinism is only demonstrated in handwritten examples and by generated source determinism, not by generated-op behavior under dispatch/undo/replay hashing.
- Invalid DSL/build failures are only unit-tested at parse/lower layers; there is no failing consumer-build test asserting user-facing diagnostics.
- `Diagnostic` lacks file path, span, and help text despite AST spans already existing.
- README and crate docs do not yet explain the v1.1 boundary:
  - build-time generation only
  - no runtime parser/runtime scripting
  - no broad generated `after()` mutation support
  - handwritten-only usage is still first-class and unchanged

## How To Validate GEN-04 and INT-04

### Reuse the existing invariant contracts exactly

The planner should treat these as locked repository patterns, not optional ideas:

1. Undo roundtrip tests must assert both state and replay hash.
2. Preview/cancel/no-op style tests should also assert both state and replay hash.
3. Low-level invariants should prefer op-level strategies.
4. Rule-dispatch integration should only be used when the property depends on generated rule behavior.

That comes directly from:

- `src/engine.rs` PROP-01 and PROP-04
- `examples/backgammon.rs` BACK-05 and BACK-06
- `.planning/STATE.md` decision: every undo/redo property test must assert both `engine.read()` and `engine.replay_hash()`

### Recommended validation target

Do not try to property-test arbitrary DSL text generation first. The simpler and more stable Phase 6 target is:

- author one or two fixed DSL fixtures
- lower them through the real codegen pipeline
- include the generated module in a test-only fixture harness
- drive proptest over the consumer event/operation values around that generated code

This keeps the property tests focused on engine invariants instead of parser fuzzing.

### Practical test shape for generated code

Use a dedicated integration test module or fixture crate that mirrors `examples/dsl_consumer`, but adds proptest-based assertions.

Recommended layers:

#### Layer A: generated-op `hash_bytes()` determinism

Use the generated operation wrapper from the consumer fixture and assert:

- identical generated op values produce identical `hash_bytes()`
- distinct generated op variants/payloads produce distinct `hash_bytes()` where shape differs
- repeated build/generate/include cycles produce the same bytes for the same logical op payload

This satisfies the narrow GEN-04 concern directly.

#### Layer B: engine roundtrip with generated rules

Create a minimal consumer fixture state/event/op set, dispatch generated-rule events, then:

- capture `state_before` and `hash_before`
- dispatch event(s) that cause generated ops to be emitted
- undo all committed transactions
- assert `engine.read() == state_before`
- assert `engine.replay_hash() == hash_before`

This is the direct INT-04 analogue of PROP-01 / BACK-06.

#### Layer C: cancellation and deterministic/irreversible flags

Because the DSL permits:

- `cancel`
- `set tx.deterministic = false`
- `set tx.irreversible = false`

Phase 6 should add focused fixtures proving those generated effects do not violate the engine contract. The highest-value checks are:

- a generated `cancel` rule leaves state and replay hash unchanged
- a generated `irreversible = false` rule mutates state but does not create an undo target
- a generated `deterministic = false` rule behaves consistently with current engine semantics and does not accidentally regress normal replay-hash behavior on deterministic paths

The first two are much more important than broad coverage of every DSL shape.

## Validation Architecture

This section is intended to feed `06-VALIDATION.md` directly.

### Validation goals

Phase 6 should validate four separate surfaces:

1. Generated operation invariants
2. User-facing diagnostics on failing authored inputs
3. Handwritten-only compatibility
4. Release/documentation boundary clarity

### Validation matrix

| Surface | Requirement | Best test style | Why |
|--------|-------------|-----------------|-----|
| Generated op hash determinism | GEN-04 | unit/integration tests in generated consumer fixture | The property is about op payload bytes, not just emitted source |
| Undo/replay-hash preservation | INT-04 | proptest over generated-rule dispatch and undo | Matches established engine invariant pattern |
| Invalid DSL diagnostics | INT-03 | failing nested consumer builds plus stderr assertions | This is the actual user experience |
| Unsupported semantics diagnostics | INT-03 | unit tests on parse/lower + failing build fixtures | Need both precise compiler-layer coverage and integration-layer coverage |
| Handwritten-only unchanged | INT-05 | compile/run examples and library tests without DSL participation | Protects the existing crate surface |
| Build-time-only boundary | INT-05 + release hardening | docs checks + packaging/dependency review | This is architectural, not just behavioral |

### Recommended automated commands

- `cargo test`
- `cargo test --test phase5_codegen`
- `cargo test --examples`
- `cargo run --example tictactoe`
- `cargo run --example backgammon`
- `cargo run --quiet --manifest-path examples/dsl_consumer/Cargo.toml`
- a new failing-build test command around nested fixture crates, likely executed from a normal integration test via `std::process::Command`

### Recommended test organization

- Keep parser/lowering diagnostics tests near the codegen crate.
- Keep failing build integration tests in `tests/` so they exercise the actual consumer flow.
- Keep generated-rule invariant tests in one focused integration target instead of scattering them across examples.
- Reuse the sibling deterministic/property split already used elsewhere in the repo when practical.

### Coverage threshold for Phase 6 completion

Phase 6 should not be considered complete until all of the following are true:

- at least one property test proves generated-rule dispatch + undo restores both state and replay hash
- at least one targeted test proves generated op `hash_bytes()` stability
- at least one failing build fixture proves invalid DSL produces actionable diagnostics
- `tictactoe` and `backgammon` still compile and run without any DSL setup
- docs explicitly describe build-time-only DSL boundaries and handwritten-only usage

## INT-03 Actionable Diagnostics Strategy

### Current state

The current pipeline reports plain `Diagnostic` values with:

- `Parse`
- `Validation`
- `Io`

and a single freeform message string.

That is enough for Phase 5 unit tests, but not enough for strong INT-03 coverage because:

- authored file path is not preserved in the displayed error
- spans are captured in AST nodes but not surfaced in diagnostics
- messages usually say what failed, but not consistently what to do instead
- `build.rs` currently uses `expect(...)`, so nested consumer failures will be more panic-shaped than compiler-shaped

### Recommended Phase 6 diagnostic improvements

The planner should assume some diagnostics polish is required before INT-03 can be signed off.

Recommended minimum upgrade:

- extend `Diagnostic` to optionally carry:
  - source path
  - span
  - rule id when available
  - short help text
- preserve parse/validation kind
- format errors for `build.rs` in a stable way suitable for stderr assertions

Actionable feedback should answer:

1. which rule or file failed
2. what construct is invalid
3. why it is outside the v1.1 contract
4. what safe alternative to use instead

### Best test harness for INT-03

Use failing nested fixture crates, not just unit tests.

Recommended cases:

- invalid DSL syntax
- duplicate rule id
- unapproved event field or state binding
- unsupported transaction flag
- forbidden `after()` semantics or direct mutation syntax if represented in the fixture corpus
- generated-code integration failure caused by bad backend configuration or unsupported operation shape

The assertions should look for stable substrings, not full stderr snapshots, to keep maintenance low.

If the planner wants the least-friction implementation, use one of these two approaches:

- `trybuild` UI-style tests for small compile-fail fixtures
- existing integration-test style with `Command::new("cargo")` running nested fixture crates and asserting `stderr`

Given the current repo patterns, the second option is lower-risk because the project already runs nested cargo commands in `tests/phase5_codegen.rs`.

## INT-05 Compatibility Without Breaking Handwritten-Only Usage

### Architectural conclusion

The current architecture is already pointed in the right direction:

- root `Cargo.toml` keeps normal `[dependencies]` empty
- `herdingcats_codegen` is not part of the runtime crate API
- the example consumer takes codegen only in `[build-dependencies]`
- existing examples (`tictactoe`, `backgammon`) do not participate in the DSL path

Phase 6 should preserve this exactly. Do not move `pest` or codegen concerns into the main crate.

### What should be explicitly verified

INT-05 is not just "cargo test still passes". The planner should require explicit checks that the handwritten path still works unchanged:

- `cargo run --example tictactoe`
- `cargo run --example backgammon`
- `cargo test --lib --examples`
- no new imports, features, or setup steps in handwritten examples
- no codegen types re-exported from `src/lib.rs`
- root crate docs remain accurate for users who never touch the DSL path

### Practical compatibility guardrails

- keep generated-code APIs confined to `crates/herdingcats_codegen` and consumer `build.rs`
- do not add required Cargo features to `herdingcats`
- do not add runtime dependencies to the root crate for DSL support
- treat the handwritten examples as compatibility fixtures, not just demos

## Docs and Release Hardening Needed

### Missing documentation boundaries

The current DSL docs say the right things in `docs/dsl/*`, but the top-level user-facing docs do not yet make the v1.1 boundary explicit enough.

The release hardening plan should update at least:

- `README.md`
- crate-level rustdoc in `src/lib.rs`
- `docs/dsl/README.md`
- possibly a short build-integration guide or section in the DSL docs

### Documentation points that should become explicit

1. The DSL path is build-time only.
2. There is no runtime parser, runtime rule loading, or runtime scripting.
3. Generated mutation is scoped to `before()` semantics in v1.1.
4. The runtime crate remains usable exactly as before for handwritten rules.
5. The companion compiler belongs in `build.rs` or equivalent build tooling, not in the runtime event loop.
6. Mixed handwritten + generated rule registration is supported; handwritten-only remains first-class.

### Release hardening items beyond docs

- verify packaging/dependency boundary before release:
  - root crate should not gain normal parser/codegen deps
  - codegen crate remains separate and unpublished unless intentionally changed later
- ensure generated-source comments/namespaces are readable enough for debugging
- make failing `build.rs` errors stable and understandable for downstream consumers
- add a short "what this feature is not" section to prevent runtime-scripting expectations

## Recommended Split

### Plan 06-01 — Validation Suite: equivalence tests, property tests, deterministic codegen assertions

This plan should own the machine-checkable invariant work:

- generated-op `hash_bytes()` determinism tests for GEN-04
- proptest-based undo/replay-hash roundtrip tests for generated-rule dispatch
- focused tests for generated `cancel` and `irreversible = false` semantics
- any deterministic generation assertions that belong with invariant coverage rather than docs polish

Primary requirement coverage:

- GEN-04
- INT-04

Secondary support:

- lays groundwork for INT-05 by preserving the existing test matrix

### Plan 06-02 — Release hardening: diagnostics polish, compatibility checks, docs/examples update, API-boundary review

This plan should own the user-facing hardening work:

- richer actionable diagnostics and stable failure formatting
- failing nested consumer-build fixtures for invalid DSL/codegen flows
- explicit handwritten-only compatibility checks
- README/rustdoc/DSL docs updates making the build-time-only boundary explicit
- final review that the root crate surface still does not require the DSL path

Primary requirement coverage:

- INT-03
- INT-05

Secondary support:

- completes release-readiness narrative for GEN-04/INT-04 by documenting boundaries

## Planning Watchouts

### Watchout 1: Over-fuzzing the parser instead of validating engine invariants

Phase 6 is not a parser-fuzzing phase. The highest-value proof is generated behavior under the engine's existing invariants.

### Watchout 2: Treating diagnostics as unit-test-only

INT-03 is about the downstream author experience. Unit tests are necessary but not sufficient; at least one failing consumer build must be exercised.

### Watchout 3: Accidentally making the runtime crate depend on codegen concerns

The current boundary is good. Phase 6 should harden it, not blur it.

### Watchout 4: Forgetting non-default transaction semantics

Generated `cancel` and `irreversible = false` are part of the DSL contract already. They need targeted validation, not just normal emit-op coverage.

## Sources

- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/PROJECT.md`
- `.planning/phases/05-build-time-compiler-and-example-integration/05-RESEARCH.md`
- `.planning/phases/05-build-time-compiler-and-example-integration/05-VALIDATION.md`
- `src/engine.rs`
- `src/lib.rs`
- `examples/backgammon.rs`
- `examples/tictactoe.rs`
- `examples/dsl_consumer/Cargo.toml`
- `examples/dsl_consumer/build.rs`
- `examples/dsl_consumer/src/main.rs`
- `tests/phase5_codegen.rs`
- `crates/herdingcats_codegen/src/lib.rs`
- `crates/herdingcats_codegen/src/codegen.rs`
- `crates/herdingcats_codegen/src/validate.rs`
- `crates/herdingcats_codegen/src/diagnostics.rs`
- `crates/herdingcats_codegen/src/parser.rs`
- `docs/dsl/README.md`
- `docs/dsl/validation.md`
- `docs/dsl/semantics.md`
- `docs/dsl/bindings.md`
- `docs/dsl/ir.md`

---
*Phase researched: 06-validation-and-release-hardening*
*Ready for planning: yes*
