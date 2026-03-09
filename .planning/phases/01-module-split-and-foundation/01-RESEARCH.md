# Phase 1: Module Split and Foundation - Research

**Researched:** 2026-03-08
**Domain:** Rust multi-module crate organization, rustdoc, inline unit tests, proptest
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Module files**: `src/hash.rs`, `src/operation.rs`, `src/transaction.rs`, `src/rule.rs`, `src/engine.rs`
- **lib.rs role**: Only `mod` declarations + explicit `pub use` re-exports — zero logic
- **hash module visibility**: `pub(crate)` only — `fnv1a_hash` and FNV constants are never public
- **CommitFrame**: Stays private inside `engine.rs`
- **Build/split order** (DAG): hash → operation → transaction → rule → engine → lib.rs
- **proptest version**: `proptest = "1.10"` in `[dev-dependencies]`
- **Unit test fixture**: Counter game — `State = i32`, `Ops = Inc / Dec / Reset`, `Event = ()`. Defined inline inside `engine.rs #[cfg(test)]` only
- **Other module tests**: `hash.rs` tests hash on raw byte slices. `transaction.rs` tests builder/state. `rule.rs` tests trait object contract. Each file tests its own slice in isolation.
- **Doc audience**: Competent Rust dev encountering this paradigm for the first time. Explain the engine model, not Rust.
- **Doc balance**: Equal weight to design rationale and concrete usage — neither pure reference nor pure tutorial
- **Operation + Rule trait docs**: 3–5 sentence paradigm introductions. A reader who only reads these two trait definitions should understand the whole engine model.
- **Internal items**: `CommitFrame` and `fnv1a_hash` get comments explaining mechanism and why each field/concept exists
- **Doc examples**: Minimal toy game per module — fully runnable `/// # Examples` blocks where possible. `cargo test --doc` must run them.
- **Code style conventions** (carry forward):
  - Section banners (`// ============================================================`) in every file
  - `where` clauses on separate lines
  - `#[derive(Clone)]` on all snapshot-able types
  - No `unwrap()`, `panic!`, or `expect()` in library code

### Claude's Discretion

- Exact wording of doc prose (voice/tone within decided constraints)
- Whether to use `compile_fail` examples for contract violations
- How to handle edge cases in `fnv1a_hash` doc (zero-length input, etc.)
- Test assertion style within `#[cfg(test)]` modules

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MOD-01 | Split `src/lib.rs` into five module files, one concept per file | DAG split order, visibility rules, cross-module `use` pattern |
| MOD-02 | `lib.rs` becomes thin re-export facade; hash module stays `pub(crate)` | `pub use` pattern, `pub(crate)` module visibility, rustdoc re-export rendering |
| MOD-03 | `examples/tictactoe.rs` compiles and runs identically after split | Public API surface map derived from tictactoe source; verified with `cargo test --examples` |
| TEST-01 | Every source file has inline `#[cfg(test)]` module with unit tests | Standard Rust inline test pattern; per-module scope defined |
| TEST-02 | `proptest = "1.10"` added to `[dev-dependencies]` | Confirmed 1.10.0 exists on crates.io; only affects test compilation |
| TEST-03 | `Operation` apply+undo roundtrip verified for every variant | Counter fixture (Inc/Dec/Reset) provides obvious assertions |
| TEST-04 | `hash_bytes()` non-empty and deterministic for every variant | Raw byte slice tests in `hash.rs`; op variant tests in `operation.rs` or `engine.rs` |
| DOC-01 | Every public type, trait, method has `///` rustdoc with paradigm rationale | `cargo doc --no-deps` warning-free is the gate; trait docs explained |
| DOC-02 | Key internal items (`CommitFrame`, `fnv1a_hash`) have `//` comments explaining role | Pattern for private item documentation; not rustdoc-visible but source-readable |
| DOC-03 | Every public method has `/// # Examples` block with runnable example | `cargo test --doc` pattern; `no_run` when needed |
| DOC-04 | `Operation` and `Rule` trait definitions include paradigm-teaching prose | Self-contained paradigm introduction in trait docs |
</phase_requirements>

---

## Summary

This phase is a pure reorganization — no new logic is introduced. The single `src/lib.rs` (327 lines) is extracted into five concept-focused modules following the DAG dependency order: `hash.rs` (FNV constants + function), `operation.rs` (Operation trait), `transaction.rs` (Transaction + RuleLifetime), `rule.rs` (Rule trait), and `engine.rs` (CommitFrame + Engine). The new `lib.rs` becomes a thin facade of `mod` declarations plus explicit `pub use` re-exports. The public API surface is fully determined by what `examples/tictactoe.rs` imports — and since tictactoe uses `use herdingcats::*`, every currently-`pub` item must be re-exported.

Three secondary concerns run in parallel with the split: inline unit tests in every module, proptest added as a dev-dependency (but no property tests in Phase 1 — those come in Phase 2), and rustdoc on all public items. The documentation work is the most effort-intensive part of the phase because it requires writing paradigm-teaching prose, not just mechanical doc comments.

**Primary recommendation:** Execute the split strictly in DAG order, one file at a time, running `cargo check` after each file to catch visibility and import errors early. Write tests and docs for each module immediately after extracting it — don't defer to a cleanup wave.

---

## Standard Stack

### Core (this phase)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| proptest | 1.10.0 | Property-based testing framework | Confirmed on crates.io; required by REQUIREMENTS.md; industry standard for Rust property testing |

**No additional runtime dependencies.** The crate currently has zero `[dependencies]`. This phase adds only a `[dev-dependencies]` entry.

**Installation:**
```toml
[dev-dependencies]
proptest = "1.10"
```

### Toolchain Context

| Tool | Version | Notes |
|------|---------|-------|
| rustc | 1.93.1 | Detected on this machine — supports all patterns used |
| cargo | 1.93.1 | `cargo test --examples`, `cargo doc --no-deps`, `cargo test --doc` all work |
| edition | 2024 | Already set in Cargo.toml; no module system changes in 2024 edition that affect this work |

---

## Architecture Patterns

### Recommended Project Structure (post-split)

```
src/
├── lib.rs           # mod declarations + pub use re-exports only; zero logic
├── hash.rs          # FNV-1a constants + fnv1a_hash() — pub(crate) only
├── operation.rs     # Operation<S> trait definition
├── transaction.rs   # Transaction<O> struct + RuleLifetime enum
├── rule.rs          # Rule<S,O,E,P> trait definition
└── engine.rs        # CommitFrame (private) + Engine<S,O,E,P> impl
```

### Pattern 1: Thin Re-Export Facade (lib.rs)

**What:** `lib.rs` declares private modules and selectively re-exports public items. The `hash` module is intentionally excluded from re-exports since it is `pub(crate)` only.

**When to use:** Whenever you want to control public API surface independently of module structure.

**Example:**
```rust
// src/lib.rs — complete file after split
mod hash;
mod operation;
mod transaction;
mod rule;
mod engine;

pub use operation::Operation;
pub use transaction::{RuleLifetime, Transaction};
pub use rule::Rule;
pub use engine::Engine;
```

Note: `hash` module is NOT `pub mod` — it stays private to the crate. The `fnv1a_hash` function is used internally by `engine.rs` via `crate::hash::fnv1a_hash(...)`.

### Pattern 2: Cross-Module Internal Import

**What:** `engine.rs` imports from sibling modules using `crate::` paths. Because `hash` is `pub(crate)`, this works without making it public.

**Example:**
```rust
// src/engine.rs
use std::collections::{HashMap, HashSet};
use crate::hash::{fnv1a_hash, FNV_OFFSET, FNV_PRIME};
use crate::operation::Operation;
use crate::transaction::{RuleLifetime, Transaction};
use crate::rule::Rule;
```

### Pattern 3: Inline Unit Test Module

**What:** `#[cfg(test)]` module at the bottom of each source file, using `use super::*` to access private items.

**Standard form:**
```rust
// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

`use super::*` is the idiomatic exception to the "no wildcard imports" rule — it gives the test module access to private items in the same file, which is the entire point of inline tests.

### Pattern 4: proptest! Macro (Phase 1 adds the dep; Phase 2 writes property tests)

**What:** proptest 1.10 provides `proptest!` macro for property tests. Not used in Phase 1 tests but the dep must be present.

**Standard import for Phase 2 reference:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn my_property(x in 0i32..100) {
        // assertions
    }
}
```

### Pattern 5: Rustdoc with Examples

**What:** `/// # Examples` blocks are compiled and run by `cargo test --doc`. Use `no_run` only when external state is required.

**Standard form for a method:**
```rust
/// Applies this operation to the given state.
///
/// # Examples
///
/// ```
/// use herdingcats::{Operation, Transaction};
///
/// // ... minimal setup ...
/// ```
pub fn apply(&self, state: &mut S) { ... }
```

**Doctest nuance:** Doctests in `pub use` re-export facades run against the re-exported path. If examples in module files use `use herdingcats::Operation`, that works because `lib.rs` re-exports it. If examples use `use herdingcats::hash::fnv1a_hash`, that will fail because `hash` is not public. Keep examples within the public API surface.

### Anti-Patterns to Avoid

- **`pub mod hash` in lib.rs**: The hash module must be private (`mod hash;` not `pub mod hash;`). Making it public would expose `fnv1a_hash` as a public API item, violating MOD-02.
- **Glob re-export (`pub use engine::*`)**: Explicit `pub use` for each item is required — glob re-exports can accidentally expose private items or internal types if the module contents change.
- **Logic in lib.rs**: Any function or struct body in the new `lib.rs` is a mistake. Only `mod` and `pub use` statements belong there.
- **Deferring tests and docs to a cleanup pass**: The success criteria require `cargo doc --no-deps` warning-free and `cargo test --examples` passing as a gate — do tests+docs per module during the split.

---

## Public API Surface Map

This is what tictactoe.rs imports (via `use herdingcats::*`) — every item listed must appear in `lib.rs` as an explicit `pub use`:

| Item | Source Module | Current Visibility | Re-export Path |
|------|--------------|--------------------|----------------|
| `Operation` (trait) | operation.rs | `pub trait` | `pub use operation::Operation` |
| `Transaction` (struct) | transaction.rs | `pub struct` | `pub use transaction::Transaction` |
| `RuleLifetime` (enum) | transaction.rs | `pub enum` | `pub use transaction::RuleLifetime` |
| `Rule` (trait) | rule.rs | `pub trait` | `pub use rule::Rule` |
| `Engine` (struct) | engine.rs | `pub struct` | `pub use engine::Engine` |

The `state` field on `Engine` is `pub` — it is a field, not re-exported via `pub use`, but it must remain `pub` in engine.rs for tictactoe to access `engine.state` directly.

The FNV constants and `fnv1a_hash` are internal — NOT re-exported.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Property-based test generation | Custom random generators | proptest 1.10 | Handles shrinking, reproducibility, coverage — non-trivial to replicate |
| Module visibility scoping | Manual re-export management | Explicit `pub use` per item | Rust's module system handles this correctly; hand-rolling leads to accidental exposure |
| Doc example compilation | Manual verification | `cargo test --doc` | Compiler catches stale examples automatically |

**Key insight:** The "don't hand-roll" risk in this phase is minimal — the main trap is attempting to write pub use globs instead of explicit per-item re-exports. Explicit is correct here.

---

## Common Pitfalls

### Pitfall 1: FNV Constants Visibility

**What goes wrong:** `engine.rs` needs `FNV_OFFSET` and `FNV_PRIME` from `hash.rs`. If `hash.rs` declares them `pub(crate)` (or simply `pub`) and `hash` module is private to `lib.rs`, the `crate::hash::FNV_OFFSET` path works fine from `engine.rs`. But if the constants are accidentally declared `pub` and `hash` module is accidentally made `pub mod`, they become visible to downstream consumers.

**Why it happens:** Confusion between "accessible within the crate" and "publicly exported."

**How to avoid:** Constants in `hash.rs` should be `pub(crate) const`. The `mod hash;` in `lib.rs` stays private (no `pub`). `engine.rs` accesses via `crate::hash::fnv1a_hash(...)`.

**Warning signs:** `cargo doc --no-deps` showing `fnv1a_hash` or `FNV_OFFSET` in the generated documentation.

### Pitfall 2: Missing Re-Exports Breaking tictactoe

**What goes wrong:** `cargo test --examples` fails to compile tictactoe after the split because a required item is missing from `lib.rs` re-exports.

**Why it happens:** `tictactoe.rs` uses `use herdingcats::*` which resolves against whatever `lib.rs` re-exports. Any missed item causes a "cannot find X in scope" error.

**How to avoid:** Cross-reference the Public API Surface Map above. After writing `lib.rs`, run `cargo test --examples` before declaring the task complete.

**Warning signs:** Compilation errors mentioning `Operation`, `Rule`, `Engine`, `Transaction`, or `RuleLifetime` not found in scope.

### Pitfall 3: Doctests Using Internal Paths

**What goes wrong:** `/// # Examples` blocks in module files reference `crate::hash::fnv1a_hash` or other non-public items. `cargo test --doc` fails because doctests compile as external crates.

**Why it happens:** Module-file authors use `crate::` paths that work in regular code but not in doctest context.

**How to avoid:** All doctest examples must use only the public API (`use herdingcats::Operation`, etc.). For internal functions like `fnv1a_hash`, either use `no_run` + `ignore` on the example, or test the observable behavior through the public API.

**Warning signs:** `cargo test --doc` failing with "unresolved import" or "module not found."

### Pitfall 4: `cargo doc` Warnings From Undocumented Items

**What goes wrong:** `cargo doc --no-deps` emits warnings for missing docs on public items. DOC-01 requires zero warnings.

**Why it happens:** The `missing_docs` lint or rustdoc's built-in warnings fire for any public item without `///` docs.

**How to avoid:** Enable `#![warn(missing_docs)]` in `lib.rs` as a compile-time guard during the documentation pass. Every `pub` item in every module needs at least one `///` line.

**Warning signs:** `cargo doc --no-deps` output containing `warning: missing documentation for`.

### Pitfall 5: Operation Undo Tests Missing All Variants

**What goes wrong:** TEST-03 says "for every op variant" — a test that only covers `Inc` but not `Dec` and `Reset` is incomplete.

**Why it happens:** Developers write one happy-path test and miss the requirement for complete variant coverage.

**How to avoid:** Write one test per variant. With the counter fixture (`Inc`, `Dec`, `Reset`), that's three apply+undo tests. Assertions should check `state == original_state` before and after the roundtrip.

---

## Code Examples

Verified patterns from official sources and current codebase analysis:

### Counter Fixture (engine.rs test module)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // --------------------------------------------------------
    // Counter Fixture
    // --------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    enum CounterOp {
        Inc,
        Dec,
        Reset,
    }

    impl Operation<i32> for CounterOp {
        fn apply(&self, state: &mut i32) {
            match self {
                CounterOp::Inc => *state += 1,
                CounterOp::Dec => *state -= 1,
                CounterOp::Reset => *state = 0,
            }
        }

        fn undo(&self, state: &mut i32) {
            match self {
                CounterOp::Inc => *state -= 1,
                CounterOp::Dec => *state += 1,
                CounterOp::Reset => { /* Reset undo needs original value — see note */ }
            }
        }

        fn hash_bytes(&self) -> Vec<u8> {
            match self {
                CounterOp::Inc   => vec![0],
                CounterOp::Dec   => vec![1],
                CounterOp::Reset => vec![2],
            }
        }
    }

    struct NoRule;
    impl Rule<i32, CounterOp, (), u8> for NoRule {
        fn id(&self) -> &'static str { "no_rule" }
        fn priority(&self) -> u8 { 0 }
    }
}
```

**Note on Reset undo:** `Reset` is lossy — it discards the previous value. The Operation trait as written requires `undo` to restore state exactly, which `Reset` cannot do if the previous value is not stored in the op. The planner must decide: either exclude `Reset` from the undo roundtrip test (TEST-03 says "for every op variant"), or store the prior value in `CounterOp::Reset { prior: i32 }`. This is a **design decision to surface to the planner.** The simplest resolution: make the Reset variant store `prior: i32` so undo can restore it. Alternatively, mark `Reset` as `irreversible: true` in the transaction and exclude it from roundtrip tests with a comment explaining why.

### Inline Test: hash_bytes Determinism (operation.rs or engine.rs)
```rust
#[test]
fn hash_bytes_determinism() {
    let op = CounterOp::Inc;
    assert_eq!(op.hash_bytes(), op.hash_bytes());
    assert!(!op.hash_bytes().is_empty());
}

#[test]
fn hash_bytes_variant_uniqueness() {
    // Different ops should produce different byte sequences
    assert_ne!(CounterOp::Inc.hash_bytes(), CounterOp::Dec.hash_bytes());
    assert_ne!(CounterOp::Inc.hash_bytes(), CounterOp::Reset.hash_bytes());
}
```

### Inline Test: Transaction Builder State (transaction.rs)
```rust
#[test]
fn transaction_new_defaults() {
    let tx: Transaction<i32> = Transaction::new();
    assert!(tx.ops.is_empty());
    assert!(tx.irreversible);
    assert!(tx.deterministic);
    assert!(!tx.cancelled);
}
```

### pub use Facade Pattern (lib.rs)
```rust
// Source: Rust Reference — Use Declarations
mod hash;          // private — hash module stays internal
mod operation;
mod transaction;
mod rule;
mod engine;

pub use operation::Operation;
pub use transaction::{RuleLifetime, Transaction};
pub use rule::Rule;
pub use engine::Engine;
```

### Rustdoc for a Trait (example voice/structure for Operation)
```rust
/// The atomic unit of state change in the engine.
///
/// An `Operation` represents a single, reversible mutation to game state `S`.
/// Operations are never applied directly by user code — they are collected into
/// a [`Transaction`] by [`Rule`] implementations during event dispatch, then
/// applied (or undone) by the engine. This indirection allows rules to inspect
/// and modify what operations will occur before any state changes take effect.
///
/// Implementing `Operation` requires three things: an `apply` that mutates state
/// forward, an `undo` that reverses the mutation exactly, and `hash_bytes` that
/// returns a deterministic byte sequence identifying this specific operation. The
/// engine uses `hash_bytes` to build a `replay_hash` — a running fingerprint of
/// every committed operation — that can verify game sequences are identical
/// across replays.
///
/// # Examples
///
/// ```
/// use herdingcats::Operation;
///
/// #[derive(Clone)]
/// enum CounterOp { Inc }
///
/// impl Operation<i32> for CounterOp {
///     fn apply(&self, state: &mut i32) { *state += 1; }
///     fn undo(&self, state: &mut i32)  { *state -= 1; }
///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
/// }
///
/// let op = CounterOp::Inc;
/// let mut state = 0i32;
/// op.apply(&mut state);
/// assert_eq!(state, 1);
/// op.undo(&mut state);
/// assert_eq!(state, 0);
/// ```
pub trait Operation<S>: Clone {
    // ...
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single-file `lib.rs` with all code | Multi-module crate with re-export facade | This phase | No external behavior change; internal organization only |
| No dev-dependencies | `proptest = "1.10"` in `[dev-dependencies]` | This phase | Zero impact on release build |
| No unit tests | Inline `#[cfg(test)]` in every module | This phase | `cargo test` now runs |
| No rustdoc | `///` on all public items + `/// # Examples` | This phase | `cargo test --doc` now runs; `cargo doc --no-deps` must be warning-free |

**Confirmed current:** proptest 1.10.0 is the latest stable on crates.io (verified 2026-03-08). The update from 1.9.0 to 1.10.0 included: MSRV bump to 1.82.0, `rand` dep updated from 0.8 to 0.9, added `ProptestResultExt` trait, replaced `lazy_static` with `std::sync::LazyLock`, fixed arithmetic overflow on 32-bit processors.

---

## Open Questions

1. **Reset op undo correctness**
   - What we know: `CounterOp::Reset` is lossy unless the prior value is captured in the variant
   - What's unclear: Should the fixture use `Reset { prior: i32 }` (correct but slightly more complex), or omit `Reset` from the roundtrip test with an explicit `irreversible` flag, or use a different set of counter ops entirely?
   - Recommendation: Use `Reset { prior: i32 }` to keep the fixture honest and maximally illustrative of the Operation contract. The planner should make this a concrete task decision.

2. **`#![warn(missing_docs)]` crate attribute**
   - What we know: Adding this to `lib.rs` would cause `cargo doc --no-deps` to error (not just warn) on any undocumented public item, providing a compile-time gate for DOC-01
   - What's unclear: Whether the user wants this enforced at compile time vs. verified manually
   - Recommendation: Add `#![warn(missing_docs)]` to `lib.rs` — it matches the "no doc warnings" success criterion and is trivial to add.

3. **`engine.state` field is `pub` — needs a doc comment**
   - What we know: `engine.state` is directly accessed by tictactoe.rs as `engine.state`. It is a `pub` field.
   - What's unclear: Should it have a `///` doc comment? DOC-01 requires every public item to be documented.
   - Recommendation: Yes — `pub` struct fields are public items. The `state` field needs a `///` comment explaining that it exposes the current committed game state and noting that direct mutation bypasses the engine's rule system.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (rustc 1.93.1) + proptest 1.10.0 |
| Config file | None — standard `cargo test` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --examples && cargo test --doc` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOD-01 | Five module files exist | structural | `ls src/*.rs \| wc -l` (expect 6) | ❌ Wave 0 |
| MOD-02 | lib.rs has only mod + pub use | compilation | `cargo build` (implicit) | ❌ Wave 0 |
| MOD-03 | tictactoe compiles and runs identically | integration | `cargo test --examples` | ✅ (exists, 0 tests) |
| TEST-01 | Every module has #[cfg(test)] block | unit | `cargo test` | ❌ Wave 0 |
| TEST-02 | proptest in dev-dependencies | compilation | `cargo build --tests` | ❌ Wave 0 |
| TEST-03 | apply+undo roundtrip for all op variants | unit | `cargo test apply_undo` | ❌ Wave 0 |
| TEST-04 | hash_bytes non-empty + deterministic | unit | `cargo test hash_bytes` | ❌ Wave 0 |
| DOC-01 | Every public item has /// doc | doc check | `cargo doc --no-deps 2>&1 \| grep warn` | ❌ Wave 0 |
| DOC-02 | Internal items have // comments | manual review | `cargo doc --no-deps` (won't catch private) | ❌ Wave 0 |
| DOC-03 | Every public method has # Examples | doctest | `cargo test --doc` | ❌ Wave 0 |
| DOC-04 | Operation + Rule have paradigm prose | manual review | `cargo doc --no-deps` (visual inspection) | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo check` (fast, catches structural errors)
- **Per wave merge:** `cargo test && cargo test --examples && cargo test --doc`
- **Phase gate:** `cargo test && cargo test --examples && cargo test --doc && cargo doc --no-deps 2>&1 | grep -c warn | xargs -I{} test {} -eq 0`

### Wave 0 Gaps

- [ ] `src/hash.rs` — module file does not exist yet
- [ ] `src/operation.rs` — module file does not exist yet
- [ ] `src/transaction.rs` — module file does not exist yet
- [ ] `src/rule.rs` — module file does not exist yet
- [ ] `src/engine.rs` — module file does not exist yet
- [ ] `Cargo.toml` — `[dev-dependencies]` section missing (Cargo.toml has zero dependencies currently)
- [ ] All test functions — no tests exist in the codebase yet (confirmed: `running 0 tests`)

---

## Sources

### Primary (HIGH confidence)
- Crates.io API (`/api/v1/crates/proptest`) — confirmed proptest 1.10.0 is current stable, verified 2026-03-08
- `src/lib.rs` (this repo) — source of truth for public API surface, module structure, all code to be split
- `examples/tictactoe.rs` (this repo) — defines the complete public API that must remain intact
- `Cargo.toml` (this repo) — confirmed zero dependencies, edition 2024, no dev-dependencies section yet
- `cargo test --examples` output — confirmed baseline: 0 tests, tictactoe compiles and passes
- `cargo doc --no-deps` output — confirmed baseline: generates without warnings (0 documented items currently)
- Rust Reference, Use Declarations — `pub use` re-export pattern is stable and unchanged in 2024 edition
- docs.rs/proptest/latest — proptest 1.10.0 API confirmed: `proptest!`, `prop_compose!`, `prop_assert!` macros

### Secondary (MEDIUM confidence)
- WebSearch: proptest 1.10.0 includes rand 0.9 update, MSRV 1.82.0 (from Rust 2025 ecosystem reports)
- Rust 2024 edition announcement: No changes to module system or `pub use` patterns (edition focuses on async, lifetime capture, unsafe extern)

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — proptest 1.10.0 existence confirmed via crates.io API; no ambiguity
- Architecture: HIGH — module split is mechanical extraction from existing code; pub use pattern is stable Rust
- Pitfalls: HIGH — all pitfalls derived from direct code analysis of the actual source files
- Open questions: MEDIUM — Reset undo issue is a real design gap that requires a planner decision

**Research date:** 2026-03-08
**Valid until:** 2026-09-08 (stable Rust module system; proptest is passive maintenance)
