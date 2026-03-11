---
phase: quick-4
plan: 01
type: execute
wave: 1
depends_on: []
files_modified: [README.md]
autonomous: true
requirements: [QUICK-4]
must_haves:
  truths:
    - "README type parameters match Engine<S,M,I,P> as used in src/lib.rs"
    - "README dispatch API reflects dispatch(event) -> Option<Action<M>> and dispatch_with(event, tx) -> Option<Action<M>>"
    - "README Action<M> description reflects only mutations and cancelled fields (no deterministic)"
    - "README dispatch_preview() description reflects correct signature and return type Action<M>"
    - "README feature list does not claim Rule lifetimes (removed in v1.1)"
    - "README terminology uses Mutation/Behavior/Action (not Operation/Rule/Transaction)"
  artifacts:
    - path: "README.md"
      provides: "Accurate API documentation"
  key_links:
    - from: "README.md Core Model"
      to: "src/lib.rs Engine<S,M,I,P>"
      via: "Type parameter table"
    - from: "README.md dispatch section"
      to: "src/engine.rs dispatch/dispatch_with"
      via: "Function signatures and return types"
---

<objective>
Bring README.md into sync with the current API after quick tasks 2 and 3.

Purpose: The README still references old terminology (Operation, Rule, Transaction), removed features (Rule lifetimes), and is missing the new dispatch API surface (dispatch/dispatch_with split, Option<Action<M>> return types, dispatch_preview signature).

Output: Updated README.md with accurate type parameters, terminology, feature list, and dispatch API description.
</objective>

<execution_context>
@/Users/mprelee/.claude/get-shit-done/workflows/execute-plan.md
@/Users/mprelee/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/Users/mprelee/herdingcats/.planning/STATE.md

<interfaces>
<!-- Current public API extracted from source -->

From src/lib.rs — Engine type parameters:
  Engine<S, M, I, P>
    S — game state (Clone)
    M — mutation type (implements Mutation<S>)
    I — input/event enum
    P — priority type (Copy + Ord, typically u8 or enum repr(i32))

From src/action.rs — Action<M> fields:
  pub mutations: Vec<M>
  pub cancelled: bool
  (no deterministic field — removed in quick-3)

From src/engine.rs — dispatch methods:
  pub fn dispatch(&mut self, event: I) -> Option<Action<M>>
    // simple path: fresh empty action, returns Some if mutations committed, None if cancelled or empty

  pub fn dispatch_with(&mut self, event: I, tx: Action<M>) -> Option<Action<M>>
    // pre-built action path; dispatch() delegates to this

  pub fn dispatch_preview(&mut self, event: I, tx: Action<M>) -> Action<M>
    // dry-run, rolls back all state; returns the Action after behaviors ran

From src/lib.rs — quick-start example shows:
  let mut tx = Action::new();
  tx.mutations.push(CounterOp::Inc);
  let _ = engine.dispatch_with((), tx);
  // OR
  let _ = engine.dispatch(());  // for behavior-driven mutations
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update README.md to reflect current API</name>
  <files>/Users/mprelee/herdingcats/README.md</files>
  <action>
Rewrite the following sections of README.md. Do NOT change the Overview intro paragraph, Determinism Guarantees, Intended Use, Status, License, or AI Usage Policy sections — those are correct.

**Core Model section** — Replace the type parameter table:
- Change `O — operation type (Operation<S>)` to `M — mutation type (Mutation<S>)`
- Change `E — event enum` to `I — input / event enum`
- Keep `S — game state` and `P — priority enum` but update P description: remove `#[repr(i32)], sealed` and replace with `Copy + Ord (e.g. u8 or #[repr(i32)] enum)`
- Change `State mutation occurs exclusively through Operation.` to `State mutation occurs exclusively through Mutation.`

**Feature list in Overview** — Remove:
- `Rule lifetimes (per-turn / per-trigger)` — this feature was removed in v1.1 (RuleLifetime enum deleted)

**Add a Dispatch API section** after Core Model (before Determinism Guarantees). This section should explain:
- `dispatch(event)` — simple path; behaviors inject mutations via `before` hooks; returns `Option<Action<M>>` (Some if mutations committed, None if cancelled or no mutations produced)
- `dispatch_with(event, tx)` — pre-built action path; pass an `Action<M>` with mutations already populated; returns `Option<Action<M>>`
- `dispatch_preview(event, tx)` — dry-run; same pipeline but all state changes are rolled back; returns `Action<M>` (not Option); useful for AI look-ahead and UI preview

Keep the section concise — 3 bullet points or a small table is fine. No code example needed here.

**Terminology** — Scan the entire README for any remaining mentions of "Operation", "Rule" (as a type/concept), or "Transaction" used in the old sense and replace with "Mutation", "Behavior", "Action" respectively. Exception: the AI Usage Policy section is prose and should not be changed.
  </action>
  <verify>
    <automated>cd /Users/mprelee/herdingcats && grep -c "Operation\|Rule lifetime\|Transaction" README.md; echo "exit:$?"</automated>
  </verify>
  <done>
README.md contains no references to removed terminology (Operation type, Rule lifetimes, Transaction as a type).
Core Model type parameter table uses S/M/I/P with correct descriptions.
A Dispatch API section exists describing dispatch, dispatch_with, and dispatch_preview with their return types.
  </done>
</task>

</tasks>

<verification>
After updating README.md:
1. Run `grep -n "Operation\|Rule lifetime\|RuleLifetime" README.md` — should return no matches (or only matches in prose that are contextually correct).
2. Run `grep -n "dispatch_with\|dispatch_preview\|Option<Action" README.md` — should return matches confirming new API is documented.
3. Run `cargo test --doc 2>&1 | tail -5` from /Users/mprelee/herdingcats — doc tests in README are not expected (README has no doc test blocks), but ensure the crate still compiles.
</verification>

<success_criteria>
README.md accurately reflects the v1.1+ API:
- Type params: Engine&lt;S, M, I, P&gt; with correct descriptions
- Action&lt;M&gt; has mutations + cancelled only
- dispatch() / dispatch_with() / dispatch_preview() documented with correct return types
- No mention of Operation, Rule lifetimes, or Transaction as removed concepts
</success_criteria>

<output>
After completion, create `/Users/mprelee/herdingcats/.planning/quick/4-update-readme-md-to-reflect-current-api/4-SUMMARY.md` with what was changed and why.
</output>
