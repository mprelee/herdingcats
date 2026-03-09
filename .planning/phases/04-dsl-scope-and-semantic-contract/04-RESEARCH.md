# Phase 04: DSL Scope and Semantic Contract - Research

**Researched:** 2026-03-09
**Domain:** `pest`-backed build-time DSL design for generated `herdingcats` rules
**Confidence:** HIGH

<user_constraints>
## User Constraints (from milestone requirements and discussion)

### Locked Decisions

**Compilation model:**
- This feature is build-time Rust generation, not runtime compilation or runtime interpretation
- The default integration path is external DSL files plus generated Rust included by the consumer crate
- Proc-macro ergonomics can be revisited later, but Phase 4 should assume the build-time source-generation model

**Problem framing:**
- The DSL is for defining additional rules and state mutators outside the original handwritten engine/game logic
- The feature is generic, not specific to card games
- A representative use case is overriding or extending game rules such as changing football scoring behavior

**Engine-model fit:**
- Generated code must fit the existing `Rule`, `Operation`, and `Transaction` model
- Generated mutating rules are scoped to `before()` semantics for v1.1
- Direct state mutation from generated rules is out of scope; all mutation must flow through reversible operations

**Milestone boundaries:**
- Phase 4 is about language scope and semantic contract, not end-to-end parser/codegen implementation
- Runtime scripting, full game-definition language ambitions, and broad `after()` mutation remain out of scope

### Claude's Discretion
- Exact DSL syntax choices as long as they stay small, explicit, and engine-compatible
- Internal AST/IR shape and naming
- Binding/config representation between consumer types and generated code
- How much authored example material is needed to make the semantic contract concrete

### Deferred Ideas (OUT OF SCOPE)

- Runtime rule loading or modding
- Arbitrary general-purpose expressions in the DSL
- Safe `after()` mutation support before engine semantics are clarified or changed
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DSL-01 | Consumer can define additional rules in external DSL files parsed from a `.pest` grammar | Phase 4 must lock file-based authoring and the initial grammar surface |
| DSL-02 | Each authored rule can declare stable id, priority, and lifetime | Rule ids are engine keys; priorities and lifetimes must map directly to existing engine semantics |
| DSL-03 | Each authored rule can match on constrained event surface and approved state/event bindings | Binding model is the main unresolved design problem and must be specified before implementation |
| DSL-04 | DSL scope is limited to semantics that compile into engine-compatible `before()` behavior | Current engine dispatch order makes `before()` the safe mutating target |
| GEN-01 | Authored DSL lowers into validated IR before Rust generation | Parser output should not feed codegen directly; validation layer is mandatory |
</phase_requirements>

---

## Summary

Phase 4 exists to prevent a parser-first mistake. The milestone research already established that this feature succeeds or fails on semantic fit with the engine, not on grammar novelty. `herdingcats` rules inject reversible operations through `Transaction<O>` and rely on stable rule ids, priority ordering, and replay-safe deterministic operations. The DSL therefore has to describe those concepts directly rather than pretending to be a general rule language.

The clean split for planning is:
- Plan 04-01 defines the authored language boundary: what a rule can say, what it cannot say, and concrete examples proving the language is expressive enough for the initial milestone.
- Plan 04-02 defines the semantic lowering contract: AST to IR shape, reversibility checks, id/binding rules, and validation failures that must happen before any Rust emission.

The most important design constraint is the current `Engine::dispatch()` flow. `before()` hooks run before operations are applied; `after()` hooks run later while the final transaction is still what gets committed. That means Phase 4 should explicitly prohibit generated mutating behavior in `after()`. Observational `after()` semantics can be a future milestone once the engine contract is revisited.

## Standard Stack

### Core
| Library / Concept | Version | Purpose | Why Standard |
|-------------------|---------|---------|--------------|
| `.pest` grammar | Phase 4 design target | External DSL syntax definition | Matches requested PEG approach and keeps syntax explicit |
| AST + validated IR | Phase 4 design target | Separate syntax from semantics | Prevents codegen from depending on raw parser tree structure |
| Existing `Rule` / `Operation` / `Transaction` traits | current crate | Semantic target for generated code | Avoids engine API churn and keeps feature additive |

### Supporting
| Item | Source | Purpose | When to Use |
|------|--------|---------|-------------|
| Binding/config layer | milestone research | Define what generated code may reference in host state/event types | Needed before parser/codegen implementation begins |
| Authored examples | this phase | Stress-test language boundary | Needed to verify syntax is neither too weak nor too broad |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `before()`-only mutation in v1.1 | Broad `after()` support | Current engine semantics make this unsafe without deeper redesign |
| File-based DSL + build-time generation | Proc-macro-first authoring | Proc macro may be ergonomic later, but external files better fit the current milestone framing |
| Constrained bindings | Arbitrary Rust expressions in DSL | Arbitrary expressions create type-safety, reversibility, and API-stability problems too early |

## Architecture Patterns

### Pattern 1: Rule-Centric Authored Model

Each authored rule should map cleanly onto:
- stable rule id
- priority
- lifetime
- event match
- zero or more guard predicates
- zero or more transaction effects or emitted reversible ops

This mirrors the current engine model closely enough that later codegen stays mechanical.

### Pattern 2: Two-Stage Semantic Lowering

Parser output should first become an AST that preserves syntax structure. A second validation/lowering pass should produce a more semantic IR with:
- unique stable rule ids
- resolved lifetime model
- resolved event/binding references
- validated reversible-op requirements
- explicit rejection reasons for unsupported constructs

### Pattern 3: Constrained Host Bindings

The DSL should not inspect arbitrary Rust state. Instead, the consumer exposes a narrow set of names or helper surfaces that Phase 5 codegen can target. Phase 4 needs to decide:
- how state fields or helper methods are named in authored rules
- whether event matching is by variant name only or supports field extraction
- how emitted ops refer to generated vs handwritten operation families

## Validation Architecture

Phase 4 is planning/spec work, but it still needs executable verification:

1. Markdown/spec artifacts must exist:
   - `04-RESEARCH.md`
   - `04-VALIDATION.md`
   - `04-01-PLAN.md`
   - `04-02-PLAN.md`
2. Both plans must collectively cover all Phase 4 requirement IDs: `DSL-01`, `DSL-02`, `DSL-03`, `DSL-04`, `GEN-01`
3. Plan 04-01 must end with concrete authored-rule examples and a locked `before()`-only semantic boundary
4. Plan 04-02 must produce a validation-first IR/binding contract and explicit rejection rules for non-reversible or out-of-scope constructs
5. The plan set should be checker-verifiable without relying on unstated assumptions about runtime scripting or engine trait changes

Suggested automated checks during execution:
- `test -f` on each required artifact
- `rg` for each requirement ID across Phase 4 plan files
- `rg "before\\(\\)|before\\(\\)-only|before\\(\\) semantics"` against research and plans
- `cargo test` as a regression guard if any exploratory code or examples are added during execution

## Risks and Watchouts

### Risk 1: Designing Syntax Before Semantics

If Phase 4 optimizes for “nice-looking grammar” first, the resulting language may not map cleanly onto reversible operations or stable engine behavior.

### Risk 2: Binding Model Vagueness

If the consumer binding surface is left vague, Phase 5 codegen will be forced to guess how to access state/events and the implementation work will sprawl.

### Risk 3: Hidden Scope Creep

The most likely scope creep vectors are:
- arbitrary expressions
- `after()` mutation
- full game-definition ambitions
- implicit mutation that avoids explicit reversible ops

These should all be rejected or deferred in Phase 4 artifacts.

## Recommended Split

### Plan 04-01 — Language Design and Examples
- Decide authored file shape and top-level grammar structure
- Lock ids, priority, lifetime, event match, guards, and allowed effect categories
- Produce representative positive and negative authored examples
- Document the v1.1 language boundary explicitly

### Plan 04-02 — Semantic Validation and Binding Contract
- Define AST-to-IR lowering stages
- Specify rule-id uniqueness and stability policy
- Define reversible-op admissibility rules
- Define binding/config model for state/event access
- Document semantic validation failures required before codegen

## Sources

- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/research/SUMMARY.md`
- `.planning/research/ARCHITECTURE.md`
- `src/rule.rs`
- `src/engine.rs`
- `src/operation.rs`
- `src/transaction.rs`

---
*Phase researched: 04-dsl-scope-and-semantic-contract*
*Ready for planning: yes*
