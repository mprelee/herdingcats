---
phase: 04-core-rename
verified: 2026-03-10T00:00:00Z
status: gaps_found
score: 13/14 must-haves verified
gaps:
  - truth: "cargo build succeeds with zero warnings"
    status: failed
    reason: "cargo build emits 1 dead_code warning for RuleLifetime::Turns and RuleLifetime::Triggers variants in engine.rs. These variants are defined in the private internal enum but never constructed (add_behavior always inserts Permanent). The variants ARE matched in dispatch logic but Rust's dead_code lint fires on the constructor side."
    artifacts:
      - path: "src/engine.rs"
        issue: "Private RuleLifetime enum has Turns(u32) and Triggers(u32) variants that are never constructed — only ever pattern-matched. Emits: 'warning: variants Turns and Triggers are never constructed'"
    missing:
      - "Either suppress the dead_code warning on the internal RuleLifetime enum with #[allow(dead_code)] (appropriate since these variants are intentional Phase 5 scaffolding), or remove Turns and Triggers from the enum entirely since add_behavior always inserts Permanent and no code path ever constructs non-Permanent lifetimes after the rename."
---

# Phase 4: Core Rename Verification Report

**Phase Goal:** Rename all public-facing types and methods to use the new domain vocabulary (Mutation, Behavior, Action, add_behavior) with zero breaking changes to existing example behavior.
**Verified:** 2026-03-10T00:00:00Z
**Status:** gaps_found — 1 gap (cargo warning; all functional goals met)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | src/mutation.rs exports `pub trait Mutation<S>` with apply, undo, hash_bytes | VERIFIED | File exists at 135 lines; `pub trait Mutation<S>: Clone` declared at line 15; all three methods present with correct signatures; zero occurrences of "Operation" |
| 2  | src/behavior.rs exports `pub trait Behavior<S,M,I,P>` with before, after, id, priority | VERIFIED | File exists at 154 lines; `pub trait Behavior<S, M, I, P>` declared at line 21; all four methods present; uses `Action<M>` in before/after signatures |
| 3  | src/action.rs exports `pub struct Action<M>` with mutations, deterministic, cancelled (no irreversible, no RuleLifetime) | VERIFIED | File exists at 84 lines; `pub struct Action<M>` declared at line 19; exactly three fields: mutations/deterministic/cancelled; zero occurrences of "irreversible" or "RuleLifetime" |
| 4  | src/lib.rs re-exports Mutation, Behavior, Action, Engine (not Operation, Rule, Transaction, RuleLifetime) | VERIFIED | lib.rs re-exports exactly: `pub use action::Action`, `pub use behavior::Behavior`, `pub use engine::Engine`, `pub use mutation::Mutation`; no old names re-exported |
| 5  | Old module names not declared in lib.rs | VERIFIED | lib.rs declares only: mod action, mod behavior, mod engine, mod hash, mod mutation; no mod operation/rule/transaction |
| 6  | Old source files deleted (operation.rs, rule.rs, transaction.rs) | VERIFIED | `ls src/` shows: action.rs, behavior.rs, engine.rs, hash.rs, lib.rs, mutation.rs only |
| 7  | engine.rs compiles against Mutation, Behavior, Action (new types) | VERIFIED | Imports at lines 8-10: `use crate::mutation::Mutation`, `use crate::behavior::Behavior`, `use crate::action::Action`; no old imports |
| 8  | Engine<S,M,I,P> type with renamed type params (O→M, E→I) | VERIFIED | `pub struct Engine<S, M, I, P>` declared with `behaviors: Vec<Box<dyn Behavior<S, M, I, P>>>` field |
| 9  | add_behavior(b) replaces add_rule(b, lifetime) — no lifetime parameter | VERIFIED | `pub fn add_behavior<B>(&mut self, behavior: B)` at line 205; zero occurrences of "add_rule" in engine.rs |
| 10 | CommitFrame stores Action<M> instead of Transaction<O> | VERIFIED | `struct CommitFrame<S, M>` with `tx: Action<M>` field; all mutations references use `tx.mutations` |
| 11 | examples/tictactoe.rs compiles and runs with new API names | VERIFIED | `impl Mutation<Game> for Op`, `impl Behavior<...> for PlayRule/WinRule`, `tx.mutations.push()`, `engine.add_behavior()`, `Action::new()`; runs correctly printing board with X winning on move 5 |
| 12 | examples/backgammon.rs compiles and runs with new API names | VERIFIED | `impl Mutation<BgState> for BackgammonOp`, `impl Behavior<...> for RollDiceRule/MoveRule`, no irreversible field usage; runs correctly showing dice roll, 2 moves, undo sequence |
| 13 | No reference to Operation, Rule, Transaction, RuleLifetime in examples or public API | VERIFIED | grep for old names in examples/ returns zero matches; lib.rs has no old re-exports; engine.rs has no public occurrences (internal private enum only) |
| 14 | cargo build succeeds with zero warnings | FAILED | Build succeeds (zero errors) but emits 1 dead_code warning: "variants `Turns` and `Triggers` are never constructed" in the private internal `RuleLifetime` enum in engine.rs |

**Score:** 13/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/mutation.rs` | Mutation<S> trait definition | VERIFIED | 135 lines; pub trait Mutation<S>: Clone with apply/undo/hash_bytes; zero old names |
| `src/behavior.rs` | Behavior<S,M,I,P> trait definition | VERIFIED | 154 lines; imports Action and Mutation from new modules; before/after use Action<M> |
| `src/action.rs` | Action<M> struct definition | VERIFIED | 84 lines; 3 fields only; Default impl present; tests pass |
| `src/lib.rs` | Public API re-exports | VERIFIED | 14 lines; re-exports Mutation/Behavior/Action/Engine only |
| `src/engine.rs` | Engine runtime with new types | VERIFIED (with warning) | Imports new types; add_behavior method; CommitFrame<S,M>; but has dead_code warning |
| `examples/tictactoe.rs` | Updated tictactoe example | VERIFIED | impl Mutation<Game> for Op; cargo run succeeds with correct output |
| `examples/backgammon.rs` | Updated backgammon example | VERIFIED | impl Mutation<BgState> for BackgammonOp; cargo run succeeds with correct output |
| `Cargo.toml` | Version bump to 0.3.0 | VERIFIED | version = "0.3.0" confirmed |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| src/lib.rs | src/mutation.rs | mod mutation | WIRED | `mod mutation` + `pub use mutation::Mutation` in lib.rs |
| src/lib.rs | src/behavior.rs | mod behavior | WIRED | `mod behavior` + `pub use behavior::Behavior` in lib.rs |
| src/lib.rs | src/action.rs | mod action | WIRED | `mod action` + `pub use action::Action` in lib.rs |
| src/behavior.rs | src/mutation.rs | use crate::mutation::Mutation | WIRED | Line 6: `use crate::mutation::Mutation` |
| src/behavior.rs | src/action.rs | use crate::action::Action | WIRED | Line 5: `use crate::action::Action` |
| src/engine.rs | src/mutation.rs | use crate::mutation::Mutation | WIRED | Line 8: `use crate::mutation::Mutation` |
| src/engine.rs | src/behavior.rs | use crate::behavior::Behavior | WIRED | Line 9: `use crate::behavior::Behavior` |
| src/engine.rs | src/action.rs | use crate::action::Action | WIRED | Line 10: `use crate::action::Action` |
| examples/tictactoe.rs | src/lib.rs | use herdingcats::* | WIRED | Line 1: `use herdingcats::*` |
| examples/backgammon.rs | src/lib.rs | use herdingcats::* | WIRED | Line 2: `use herdingcats::*` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| REN-01 | 04-01, 04-02 | Operation<S> renamed to Mutation<S> — apply/undo/hash_bytes preserved | SATISFIED | src/mutation.rs: `pub trait Mutation<S>: Clone` with all 3 methods; examples use `impl Mutation<...>` |
| REN-02 | 04-01, 04-02 | Rule<S,O,E,P> renamed to Behavior<S,M,I,P> — before/after/id/priority preserved | SATISFIED | src/behavior.rs: `pub trait Behavior<S, M, I, P>` with all 4 methods; examples use `impl Behavior<...>` |
| REN-03 | 04-01, 04-02 | Transaction<O> renamed to Action<M> — mutations vec, deterministic, cancelled preserved; irreversible removed | SATISFIED | src/action.rs: `pub struct Action<M>` with exactly mutations/deterministic/cancelled; no irreversible field anywhere |
| REN-04 | 04-03 | All public re-exports, doctests, and both examples compile and pass under new names with no behavioral changes | SATISFIED (with warning gap) | lib.rs re-exports correct names only; `cargo test` 17 tests pass; both examples run with correct output; 1 build warning exists |

**Orphaned requirements check:** REQUIREMENTS.md maps REN-01 through REN-04 to Phase 4. All four are claimed by plans 04-01, 04-02, and 04-03. No orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| src/engine.rs | 17-18 | `Turns(u32)` and `Triggers(u32)` defined but never constructed — dead_code warning | Warning | Causes `cargo build` to emit 1 warning, violating Plan 03 truth #4 |

**Dead_code context:** The internal private `RuleLifetime` enum was intentionally kept in engine.rs per Plan 02 locked decisions ("Internal RuleLifetime kept as private enum for Phase 4 compatibility; Phase 5 will remove it"). However, since `add_behavior` always inserts `RuleLifetime::Permanent`, the `Turns` and `Triggers` variants are never constructed (only pattern-matched against — which Rust flags). The Plan 02 design note acknowledges this enum is transitional scaffolding, but the SUMMARY's claim that `cargo build` "succeeds with zero errors" is accurate — the warning was present at checkpoint time and noted in the SUMMARY.

### Human Verification Required

None required. All functional behaviors are verifiable through `cargo build`, `cargo test`, `cargo run --example`, and file inspection.

### Gaps Summary

The phase achieves its primary goal: all public-facing types and methods use the new domain vocabulary (Mutation, Behavior, Action, add_behavior) and both examples run with identical behavioral output. The API rename is complete and correct.

One gap exists against the stated success criteria:

**Gap 1: cargo build warning (Plan 03 truth #4).** The build emits a `dead_code` warning for `RuleLifetime::Turns` and `RuleLifetime::Triggers` in engine.rs. This is a consequence of Plan 02's design decision to keep the internal `RuleLifetime` private enum for Phase 4/5 compatibility while removing all public construction of `Turns`/`Triggers` variants. The fix is either:
- Add `#[allow(dead_code)]` to the `RuleLifetime` enum in engine.rs (appropriate given the documented intent to remove in Phase 5), OR
- Remove `Turns` and `Triggers` from the enum entirely, since no code path constructs them after the rename (add_behavior always inserts Permanent; the pattern-match branches at lines 341 and 365 become unreachable dead code)

The gap is low severity — it does not affect correctness, test results, or example behavior. However, it explicitly violates the Plan 03 success criterion of zero warnings.

---

_Verified: 2026-03-10_
_Verifier: Claude (gsd-verifier)_
