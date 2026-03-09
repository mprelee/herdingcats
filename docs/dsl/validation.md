# Validation Contract

This document defines the checks that must happen before Rust code generation.

## Validation Stages

1. parse validation
2. AST shape validation
3. semantic validation during AST -> IR lowering
4. IR readiness check before codegen

Only semantically valid IR may reach code generation.

## Mandatory Semantic Failures

Validation must reject authored rules that contain any of the following.

### Rule Identity Failures

- duplicate rule ids in the authored input set
- empty or unstable rule ids
- ids that cannot become stable generated `'static` strings

### Lifetime Failures

- unsupported lifetime forms
- invalid numeric lifetime values
- lifetime syntax that cannot map directly onto `RuleLifetime`

### Event Surface Failures

- event matches outside the approved event set
- field extraction on event data the consumer did not expose
- matching semantics that require runtime reflection or ad hoc code

### Guard Failures

- references to bindings not present in the approved binding surface
- arbitrary expressions instead of approved comparisons
- mutation or side effects inside guards

### Effect Failures

- direct state mutation
- effects that imply hidden snapshots
- lossy updates that cannot prove reversibility
- effects that require mutating `after()` behavior
- unsupported transaction flags

## Reversibility Rule

If an authored effect cannot be represented as a reversible operation, validation must fail before code generation.

Examples of constructs that should fail:
- `set state.score = 4`
- `remove matching item` without explicit prior-state capture
- `append to collection` when undo information is not representable

## `after()` Rule

Any authored construct that implies mutating `after()` behavior is invalid in v1.1.

This should fail as a semantic validation error, not as an accidental parser omission.

## Determinism Rule

Validation must ensure the IR is safe for later deterministic code generation:
- canonical rule ordering
- canonical effect ordering
- canonical binding resolution
- no constructs that imply unstable name or payload generation

## Error Quality Expectations

Validation errors should identify:
- which authored rule failed
- which semantic rule was violated
- why the construct is unsupported
- what the author can do instead, when a safe alternative exists
