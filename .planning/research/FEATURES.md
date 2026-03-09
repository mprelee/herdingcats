# Feature Research

**Domain:** Build-time authored rule DSL for `herdingcats`
**Researched:** 2026-03-09
**Confidence:** MEDIUM

## Table Stakes

- A `.pest` grammar that can describe extra rules on top of handwritten engine/game logic
- Build-time parsing and code generation, not runtime interpretation
- Generated Rust that compiles into `herdingcats`-compatible `Rule` and `Operation` implementations
- Access to state and event context through a constrained binding surface
- Reversible emitted operations so undo/redo and replay hashing remain valid
- Existing handwritten usage continues to work unchanged

## Differentiators

- Declarative iteration on ruleset variants without turning the project into a scripting engine
- Generated Rust remains readable and debuggable, so authors can inspect what the DSL means
- Behavioral equivalence tests between handwritten and generated rules
- A small, opinionated workflow that makes rule experimentation fast without broadening the engine API

## Anti-Features

- Runtime rule loading, hot reload, or modding
- Full game-definition language replacing handwritten game logic
- Arbitrary general-purpose programming in the DSL
- Engine trait redesign just to accommodate code generation
- New unrelated game systems or runtime dependency expansion

## Expected User Workflow

1. Define core state, events, and baseline operations in Rust.
2. Author extra rules in DSL files parsed by a `.pest` grammar.
3. Run normal Cargo build; `build.rs` compiles the DSL into Rust in `OUT_DIR`.
4. Include generated code in the consumer crate and register generated rules with the engine.
5. Dispatch events normally; generated rules participate through `before()` and transaction op injection.

## Scope Recommendation

- Keep v1.1 narrow and additive.
- Start with `before()`-phase rules, event matching, guards, priorities, lifetimes, and reversible op emission.
- Require a constrained host binding layer instead of arbitrary state introspection.
- Prefer clear build errors and generated-code debuggability over DSL expressiveness.
- Defer runtime scripting, `after()` mutation semantics, and broad language features to later milestones.

## Suggested Feature Categories

### Authoring Surface

Table stakes:
- Rule identifiers
- Priorities
- Lifetimes
- Event matches
- Guard conditions

Differentiators:
- Human-readable authored syntax
- Good source-to-error mapping

### Semantics and Generation

Table stakes:
- Lowering to validated IR before code generation
- Generated reversible operations
- Generated rule registration helper

Differentiators:
- Readable generated Rust
- Behavioral equivalence harness against handwritten reference rules

### Integration and Validation

Table stakes:
- Consumer `build.rs` integration
- Fixture example proving end-to-end flow
- Property tests for generated undo/hash behavior

Differentiators:
- Compile-fail tests with actionable diagnostics
- Deterministic codegen ordering guarantees

## Sources

- `.planning/PROJECT.md`
- `src/rule.rs`
- `src/operation.rs`
- `src/transaction.rs`
- Local research synthesis from parallel explorer agents

---
*Feature research for: herdingcats authored-rule DSL*
*Researched: 2026-03-09*
