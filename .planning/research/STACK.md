# Stack Research

**Domain:** Build-time DSL and Rust code generation for `herdingcats` rule extensions
**Researched:** 2026-03-09
**Confidence:** MEDIUM

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `herdingcats` runtime crate | current workspace | Stable engine/runtime traits | Keep the runtime engine unchanged so generated rules target existing `Rule`, `Operation`, and `Transaction` semantics |
| `pest` | current stable | Parse authored DSL files from a `.pest` grammar | Fits the requested PEG approach and keeps grammar definition explicit |
| `pest_derive` | current stable | Generate the parser from `.pest` grammar | Standard pairing with `pest` for a small Rust parser/compiler toolchain |
| `proc-macro2` + `quote` | current stable | Emit Rust source from validated IR | Safer and more maintainable than hand-building Rust strings |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `trybuild` | current stable | Compile-pass and compile-fail tests for generated code | Use to verify build errors and generated API shape |
| `proptest` | existing | Property-test generated operations/rules against engine invariants | Use once generated code reaches `Operation`/`Rule` stage |
| Companion crate such as `herdingcats_codegen` | workspace-local | Hold parser, IR, validator, and codegen | Use instead of pushing parser deps into runtime crate |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `build.rs` | Run DSL compilation in the consumer crate | Write generated Rust to `OUT_DIR` and emit `cargo:rerun-if-changed=` lines |
| `cargo test` | Unified validation entrypoint | Should cover parser tests, codegen tests, fixture builds, and property tests |

## Installation

```bash
# herdingcats runtime stays as the normal dependency
# build-time codegen lives in a companion crate or build-dependency
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `build.rs` source generation | Proc macro | Use proc macros only if authored rules move inline into Rust source instead of external DSL files |
| `quote` token generation | Manual string generation | Only for a trivial prototype; token generation scales better and avoids malformed Rust output |
| Companion build-time crate | Runtime parser in `herdingcats` | Runtime parser only if the project pivots to scripting/modding, which is out of scope for v1.1 |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Runtime parsing/interpreting of rule DSL | Conflicts with design-time compilation goal and adds runtime complexity | Build-time `build.rs` compilation into Rust |
| Adding `pest` to runtime dependencies | Pollutes the shipped crate for a build-time concern | Separate codegen crate or build-dependency |
| `syn` or broader Rust parsing stack | Unneeded unless the DSL starts parsing Rust itself | Keep scope to PEG parser + Rust emission |
| Arbitrary dynamic loading/JIT | Breaks the crate’s deterministic, static compilation direction | Normal Rust compilation of generated code |

## Stack Patterns by Variant

**If the consumer already has handwritten operations:**
- Generate `GeneratedOp` and wrap it in a consumer-defined `GameOp` enum
- Because the engine already expects one concrete `Operation<S>` type

**If the consumer wants minimal integration surface:**
- Generate a `register_generated_rules(...)` helper alongside generated operations
- Because rule registration is the cleanest additive seam in the current engine

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `pest` | `pest_derive` | Keep these aligned within the same release family |
| Generated code | current `herdingcats` traits | Works as long as `Rule`, `Operation`, and `Transaction` signatures stay stable |

## Sources

- `.planning/PROJECT.md` — milestone goals, constraints, and API-stability requirements
- `.planning/codebase/STACK.md` — existing crate stack and dependency posture
- `.planning/codebase/ARCHITECTURE.md` — integration constraints with the runtime engine
- Local research synthesis from parallel explorer agents

---
*Stack research for: build-time pest DSL and Rust codegen*
*Researched: 2026-03-09*
