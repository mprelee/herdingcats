# Rejected Rule Examples

These examples are intentionally tempting. They show where the v1 DSL contract stops.

## Rejected 1: Direct State Mutation

```text
rule direct_mutation {
  priority 10
  when ScoreEvent.kind == "field_goal"
  set state.rules.field_goal_points = 4
}
```

Why rejected:
- mutates state directly instead of emitting a reversible operation
- breaks the engine's `Operation::apply()` / `undo()` contract
- hides what would need to be stored for exact reversal

Blocked by:
- Phase 4 direct-mutation prohibition
- existing `Operation`-driven undo model

## Rejected 2: Mutating `after()` Behavior

```text
rule score_after_commit {
  priority 50
  when ScoreEvent.kind == "field_goal"
  after emit UpdateSeasonSummary(points = 4)
}
```

Why rejected:
- v1 does not expose authored mutating `after()` semantics
- the current engine contract does not make this a safe promise for generated mutation behavior

Blocked by:
- Phase 4 `before()`-only mutation boundary
- current dispatch ordering in `Engine`

## Rejected 3: Arbitrary Expression Evaluation

```text
rule dynamic_formula {
  priority 30
  when ScoreEvent.kind == "field_goal"
  guard eval(user_formula, state, event) == true
  emit SetFieldGoalPoints(value = compute(user_formula))
}
```

Why rejected:
- sneaks a scripting language into the DSL
- makes determinism, diagnostics, and reversibility much harder to guarantee
- expands the milestone far beyond additive rule authoring

Blocked by:
- no arbitrary expressions
- no runtime scripting

## Rejected 4: Runtime Rule Loading

```text
load rules from "/tmp/season-rules.hc"
```

Why rejected:
- Phase 4 is design-time/build-time only
- runtime loading changes the architecture from generated Rust to interpreted configuration

Blocked by:
- milestone compilation model
- out-of-scope runtime parser behavior

## Rejected 5: Full Game Definition Ambition

```text
game football_variant {
  state { ... }
  events { ... }
  rules { ... }
}
```

Why rejected:
- replaces core handwritten game implementation instead of extending it
- turns the DSL into a full game-definition language
- would force much broader type-system and integration work

Blocked by:
- milestone focus on additional rules only
- no engine/API redesign in v1.1

## What These Rejections Protect

- undo/redo correctness
- deterministic replay behavior
- the additive nature of the feature
- a realistic Phase 5 implementation target
