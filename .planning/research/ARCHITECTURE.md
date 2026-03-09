# Architecture Research

**Domain:** Build-time DSL compiler for `herdingcats`
**Researched:** 2026-03-09
**Confidence:** MEDIUM

## Recommended Architecture

### New Components

1. Build-time companion crate such as `herdingcats_codegen`
   - Owns `.pest` grammar, parser, AST, semantic validator, IR, and Rust code generator
2. Consumer `build.rs`
   - Invokes codegen and writes generated Rust into `OUT_DIR`
3. Generated integration module
   - Emits generated operations, generated rules, and a rule-registration helper
4. Consumer binding/config layer
   - Exposes host state, event, and helper APIs the DSL is allowed to target

### Modified Components

- Consumer crates add `build.rs` and an `include!` of generated code
- Consumers with existing operations wrap generated ops inside a shared operation enum
- `herdingcats` runtime crate ideally does not change for v1.1

## Data Flow

### Build Time

1. Author writes DSL files plus grammar-driven rule declarations.
2. `build.rs` invokes the codegen crate.
3. Parser builds AST from `pest`.
4. Validator lowers AST to an IR with unique rule ids, resolved bindings, and reversible-op checks.
5. Codegen emits Rust source into `OUT_DIR`.
6. Consumer crate compiles generated code as normal Rust.

### Runtime

1. Consumer constructs `Engine<S, GameOp, E, P>`.
2. Consumer registers handwritten rules and generated rules.
3. Generated `before()` methods inspect state/event through allowed bindings.
4. Generated rules push reversible ops into `Transaction<GameOp>` or cancel/flag the transaction.
5. Engine applies ops and maintains undo/hash/lifetimes unchanged.

## Key Constraints

- All mutation must still flow through `Operation`; generated rules must not mutate state directly.
- Generated operations must be fully undoable and produce stable `hash_bytes()`.
- Rule ids must remain stable `'static` strings because engine lifetime/enabled tracking keys off them.
- Priority and lifetime semantics belong to the existing engine and should be compiled into, not redefined.
- Generated mutating rules should target `before()`, not `after()`, under the current engine dispatch semantics.

## Suggested Build Order

1. Narrow DSL language design around engine-compatible concepts.
2. Parser and AST.
3. Semantic validator and IR.
4. Rust code generation.
5. `build.rs` integration and generated-module contract.
6. End-to-end example fixture.
7. Property and compile-fail validation.

## Sources

- `.planning/PROJECT.md`
- `.planning/codebase/ARCHITECTURE.md`
- `.planning/codebase/STRUCTURE.md`
- `src/rule.rs`
- `src/engine.rs`
- `src/operation.rs`
- `src/transaction.rs`
- Local research synthesis from parallel explorer agents

---
*Architecture research for: herdingcats build-time DSL compiler*
*Researched: 2026-03-09*
