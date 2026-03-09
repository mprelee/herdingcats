# Accepted Rule Examples

These examples are intentionally narrow. They demonstrate what Phase 4 considers in scope for the v1 DSL contract.

## Example 1: Scoring Override Toggle

```text
rule field_goal_bonus {
  priority 10
  lifetime permanent
  when ScoreEvent.kind == "field_goal"
  guard state.rules.field_goal_bonus_enabled == true
  emit SetFieldGoalPoints(value = 4, prior = state.rules.field_goal_points)
}
```

Why accepted:
- matches a constrained event surface
- uses an approved state binding in the guard
- emits a reversible operation rather than mutating state directly
- stays inside `before()` transaction semantics

Engine mapping:
- `field_goal_bonus` -> `Rule::id()`
- `priority 10` -> `Rule::priority()`
- `lifetime permanent` -> `RuleLifetime::Permanent`
- `emit` -> push operation into `Transaction::ops`

## Example 2: Lifetime-Limited Modifier

```text
rule overtime_bonus_window {
  priority 20
  lifetime turns(2)
  when OvertimeStarted
  emit EnableBonusRule(name = "four_point_field_goal")
}
```

Why accepted:
- exercises `RuleLifetime::Turns`
- event match is simple and explicit
- emitted effect is additive and reversible

Engine mapping:
- lifetime compiles directly into `RuleLifetime::Turns(2)`
- the effect remains an operation or generated wrapper op, not direct state mutation

## Example 3: Guarded Cancellation Rule

```text
rule disallow_bonus_without_toggle {
  priority 5
  lifetime permanent
  when ScoreEvent.kind == "field_goal"
  guard state.rules.field_goal_bonus_enabled == false
  cancel
}
```

Why accepted:
- uses an approved guard surface
- cancellation is already part of `Transaction`
- still fits `before()` semantics cleanly

Engine mapping:
- `cancel` lowers to `tx.cancelled = true`
- no hidden side effects are required

## Example 4: Deterministic Flag Override

```text
rule cosmetic_banner_rule {
  priority 200
  lifetime triggers(1)
  when SeasonEvent.kind == "launch_banner"
  set deterministic = false
  emit ShowBannerEffect(message = "Playoff rules active")
}
```

Why accepted:
- demonstrates an explicitly approved transaction flag
- keeps the flag change explicit instead of implying magical runtime behavior
- still lowers through `before()`

Constraint note:
- this remains valid only if the semantic contract explicitly approves the flag and the emitted effect still maps to supported operation behavior

## What These Examples Prove

- useful rule overrides fit the authored model without runtime scripting
- `RuleLifetime` and priorities belong directly in the authored surface
- guards must be narrow and consumer-approved
- emitted effects remain reversible operations or explicit transaction edits
