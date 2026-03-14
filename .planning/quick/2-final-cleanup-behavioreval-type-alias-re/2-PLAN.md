---
phase: quick-2
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/behavior.rs
  - src/lib.rs
  - README.md
  - src/apply.rs
autonomous: true
requirements: [CLEANUP-01]

must_haves:
  truths:
    - "cargo clippy produces zero warnings"
    - "cargo test --doc passes all doctests"
    - "BehaviorEval type alias is publicly exported and used in BehaviorDef"
    - "README uses consistent architecture terminology"
    - "Apply contract is clearly documented as a trusted invariant"
  artifacts:
    - path: "src/behavior.rs"
      provides: "BehaviorEval type alias, BehaviorDef using it"
      contains: "pub type BehaviorEval"
    - path: "src/lib.rs"
      provides: "Re-export of BehaviorEval"
      contains: "BehaviorEval"
    - path: "README.md"
      provides: "Properly formatted README with architecture status"
    - path: "src/apply.rs"
      provides: "Apply trait with clarified contract documentation"
  key_links:
    - from: "src/behavior.rs"
      to: "src/lib.rs"
      via: "pub use re-export"
      pattern: "pub use.*BehaviorEval"
---

<objective>
Final cleanup: add BehaviorEval type alias to fix clippy warning, polish README formatting, clarify Apply contract docs.

Purpose: Eliminate the last clippy warning and ensure all documentation is clean and consistent with architecture terminology.
Output: Zero-warning build with polished docs.
</objective>

<execution_context>
@/Users/mprelee/.claude/get-shit-done/workflows/execute-plan.md
@/Users/mprelee/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/behavior.rs
@src/lib.rs
@src/apply.rs
@README.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add BehaviorEval type alias and fix clippy warning</name>
  <files>src/behavior.rs, src/lib.rs</files>
  <action>
In src/behavior.rs, add a public type alias ABOVE the BehaviorDef struct definition:

```rust
/// Type alias for the evaluate fn pointer signature used by [`BehaviorDef`].
///
/// Receives immutable borrows of the input and state, returning a
/// [`BehaviorResult`] with diffs or a stop signal.
pub type BehaviorEval<E> = fn(
    &<E as EngineSpec>::Input,
    &<E as EngineSpec>::State,
) -> BehaviorResult<
    <E as EngineSpec>::Diff,
    <E as EngineSpec>::NonCommittedInfo,
>;
```

Update BehaviorDef's evaluate field (line 161) from the inline fn type to:
```rust
pub evaluate: BehaviorEval<E>,
```

In src/lib.rs, update the behavior re-export line to include BehaviorEval:
```rust
pub use crate::behavior::{BehaviorDef, BehaviorEval, BehaviorResult};
```

Do NOT change any test code, examples, or other files. The type alias is structural — all existing code that constructs BehaviorDef with an evaluate fn pointer continues to work unchanged.
  </action>
  <verify>
    <automated>cargo clippy 2>&1 | grep -c "warning" | grep "^0$" && cargo test 2>&1 | tail -5</automated>
  </verify>
  <done>cargo clippy produces zero warnings. All tests pass. BehaviorEval is publicly exported.</done>
</task>

<task type="auto">
  <name>Task 2: Polish README and clarify Apply contract docs</name>
  <files>README.md, src/apply.rs</files>
  <action>
README.md changes (minimal — the README is already well-structured):
- In the "Architecture Status" section, add a note that this is the MVP architecture. Change the section to read:

```markdown
## Architecture Status (MVP)

v0.5.0 implements the full MVP architecture described in `ARCHITECTURE.md`:

- Static behavior set via `BehaviorDef` structs (fn pointers, no trait objects)
- Copy-on-write working state (zero-clone until first diff)
- Snapshot-based undo/redo (no `Reversible` trait burden)
- Deterministic `(order_key, name)` behavior ordering
- `Apply` trace contract enforced by `debug_assert!` in dispatch
```

- In the Quick Start code block, add `use herdingcats::Outcome;` to the imports and add a match on the outcome after the dispatch call to show what `Committed` looks like:

```rust
match outcome {
    Outcome::Committed(frame) => println!("committed {} diffs", frame.diffs.len()),
    other => println!("outcome: {:?}", other),
}
```

- Also fix the Quick Start dispatch call: `engine` must be `mut` (it already needs `&mut self`). Ensure `let mut engine = ...` and `let outcome = engine.dispatch(...)`. Remove the `.unwrap()` — use a `match` instead to be more idiomatic in a README example.

src/apply.rs doc changes:
- Expand the Apply trait doc comment (above `pub trait Apply<E: EngineSpec>`) to clarify the trusted invariant. Add a paragraph after the existing doc:

```rust
/// # Trace contract (trusted invariant)
///
/// Each state-mutating `apply` call **must** return at least one trace entry.
/// A no-op diff (one that does not mutate state) may return an empty `Vec`.
///
/// This contract is **not** enforced by the trait signature — `Vec<E::Trace>`
/// cannot express "non-empty if state changed" at the type level. Instead, the
/// engine enforces it with a `debug_assert!` in dispatch: violations panic in
/// debug and test builds. Production builds skip the check for performance.
///
/// Existing contract tests in this module verify both sides of the rule.
```

Do NOT modify the Apply trait signature, existing tests, or existing contract tests. Do NOT add any new traits, macros, or abstractions. Preserve architecture exactly.
  </action>
  <verify>
    <automated>cargo test --doc 2>&1 | tail -5 && cargo doc 2>&1 | grep -c "warning" | grep "^0$"</automated>
  </verify>
  <done>README shows MVP status note and improved Quick Start. Apply contract is clearly documented as a trusted invariant. All doctests pass. Zero doc warnings.</done>
</task>

</tasks>

<verification>
cargo clippy 2>&1 — zero warnings
cargo test 2>&1 — all tests pass
cargo test --doc 2>&1 — all doctests pass
cargo doc 2>&1 — zero warnings
grep "BehaviorEval" src/lib.rs — confirms re-export
</verification>

<success_criteria>
- cargo clippy produces zero warnings (type_complexity resolved)
- BehaviorEval<E> type alias is public and used in BehaviorDef
- README has MVP architecture status note and improved Quick Start
- Apply contract documented as trusted invariant (not enforced by signature)
- All existing tests continue to pass unchanged
- No new traits, macros, trait objects, or terminology introduced
</success_criteria>

<output>
After completion, create `.planning/quick/2-final-cleanup-behavioreval-type-alias-re/2-SUMMARY.md`
</output>
