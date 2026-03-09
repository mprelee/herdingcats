# Codebase Concerns

**Analysis Date:** 2026-03-08

## Tech Debt

**`Transaction<O>` missing `Default` implementation:**
- Issue: `Transaction::new()` exists but `Default` is not implemented, which is a clippy-flagged deviation from idiomatic Rust. Callers must use `Transaction::new()` rather than the `Default` trait.
- Files: `src/lib.rs` (line 48)
- Impact: Prevents use of `..Default::default()` struct update syntax; incompatible with generic code that bounds on `Default`; Clippy warns on every build.
- Fix approach: Add `impl<O> Default for Transaction<O>` delegating to `Self::new()`, as Clippy suggests.

**Unused `deterministic` field on `Transaction<O>`:**
- Issue: `Transaction.deterministic` is a public field initialized to `true` in `Transaction::new()` but is never read or acted on anywhere in `src/lib.rs`. It has no effect on engine behavior.
- Files: `src/lib.rs` (lines 43, 52)
- Impact: Dead API surface. Users may set this field expecting it to change behavior, and it will not. Creates confusion about what the field does and whether it is safe to rely on.
- Fix approach: Either implement the semantics it implies (e.g., gate replay hash updates) or remove it before 1.0 to avoid a breaking API change post-stabilization.

**Nested `if` blocks in `dispatch()` not collapsed:**
- Issue: Two nested `if` / `if let` chains in `dispatch()` can be collapsed into single guarded patterns. Clippy reports both as `collapsible_if` warnings.
- Files: `src/lib.rs` (lines 230–239, 256–263)
- Impact: Minor readability issue; Clippy warnings on every build create noise that can mask real warnings.
- Fix approach: Apply `cargo clippy --fix` suggestions or manually collapse guards with `&&`.

**`redo()` restores the wrong lifetime/enabled snapshots:**
- Issue: `redo()` restores `lifetime_snapshot` and `enabled_snapshot` from the commit frame, which captures state *before* the transaction was applied (they were snapshotted at the start of `dispatch()`). On redo this puts lifetimes back to their pre-commit values rather than their post-commit values, potentially double-decrementing `RuleLifetime::Turns` or re-enabling rules that were expired by the commit being redone.
- Files: `src/lib.rs` (lines 303–315)
- Impact: Rule lifetime correctness after undo/redo sequences involving `RuleLifetime::Turns` or `RuleLifetime::Triggers` may be incorrect. This is a semantic bug in the engine's core guarantee.
- Fix approach: Store a separate `post_commit_lifetime_snapshot` and `post_commit_enabled_snapshot` in `CommitFrame` so `redo()` can restore the correct post-commit state.

**`add_rule()` sorts on every insertion:**
- Issue: `self.rules.sort_by_key(|r| r.priority())` is called inside `add_rule()`, which re-sorts the entire `rules` Vec on each call. This is O(n log n) per insertion.
- Files: `src/lib.rs` (line 173)
- Impact: Negligible for games with few rules. For games with many dynamic rules (add_rule called frequently), this becomes a performance concern. More importantly, it calls `priority()` on every existing rule on every insertion, requiring a trait dispatch per comparison.
- Fix approach: Use an insertion-sorted approach (binary search + insert), or accept the current behavior with a documented note that `add_rule` is intended only for setup, not hot paths.

**`Engine.state` is fully public:**
- Issue: `pub state: S` on `Engine` allows callers to mutate state directly, bypassing the `Operation` layer entirely. This undermines the core architectural invariant that "State must not be mutated outside `Operation`."
- Files: `src/lib.rs` (line 129), `docs/ARCHITECTURAL_INVARIANTS.md` (Invariant 2)
- Impact: A caller can silently corrupt undo history, desync the replay hash, and violate determinism — all without any compiler warning. The `write()` method exists for authorized snapshot replacement, but the bare field access is unrestricted.
- Fix approach: Make `state` private and expose read access via `read()` (already exists). Use `write()` for snapshot replacement. The example `examples/tictactoe.rs` accesses `engine.state` directly (line 271) and would need updating.

**`Engine` has no `disable_rule` or `enable_rule` API:**
- Issue: Rules can be added and lifetimes managed, but there is no public method to manually enable or disable a rule by id outside of lifetime expiry. The `enabled` set is private with no mutation surface exposed to callers.
- Files: `src/lib.rs` (lines 135, 163–177)
- Impact: Users who need to toggle rules based on game logic must encode this as an `Operation` affecting rule-adjacent state, or work around the limitation. The `EXTENSION_GUIDELINES.md` mentions no workaround.
- Fix approach: Expose `enable_rule(id)` / `disable_rule(id)` methods, ensuring they participate in undo snapshots if called within a transaction context, or document the intended pattern for manual rule toggling.

## Known Bugs

**`redo()` post-commit snapshot mismatch (detailed above):**
- Symptoms: After undo then redo, `RuleLifetime::Turns` rules may have incorrect remaining counts; rules that should remain disabled may become re-enabled.
- Files: `src/lib.rs` (lines 303–315)
- Trigger: Any sequence involving `undo()` followed by `redo()` where at least one rule uses `RuleLifetime::Turns` or `RuleLifetime::Triggers`.
- Workaround: None currently. Avoid undo/redo with `Turns`/`Triggers` lifetimes until fixed.

**`WinRule` can emit multiple `SetWinner` ops in a single turn:**
- Symptoms: If a board state satisfies more than one winning line simultaneously (possible when an `undo` followed by `redo` replays into an already-won state), the `after()` handler in `WinRule` loops all eight lines and pushes a `SetWinner` op for each matching line, potentially pushing two ops for two overlapping winning lines.
- Files: `examples/tictactoe.rs` (lines 195–223)
- Trigger: Constructing a board where two winning lines are completed in a single `apply()` call (unlikely in normal play but possible if operations are composed manually).
- Workaround: This is in example code only and does not affect the engine itself.

## Security Considerations

**No applicable security surface:**
- This is a pure game logic library with no network I/O, file I/O, authentication, or external data parsing.
- Risk: None identified.
- Recommendations: Not applicable.

## Performance Bottlenecks

**State clone on every `dispatch_preview()` call:**
- Problem: `dispatch_preview()` clones the full game state (`self.state.clone()`) before applying ops, then restores it. For large game states this is an allocation-heavy operation called potentially many times per turn (e.g., AI lookahead).
- Files: `src/lib.rs` (line 186)
- Cause: No structural sharing or copy-on-write; full clone is the only rollback mechanism.
- Improvement path: Document that `S` should be kept small and cheaply cloneable. Alternatively, expose a diff/patch layer so preview can roll back via `undo` rather than snapshot-and-restore.

**Rules re-sorted on every `add_rule()` call:**
- Problem: See tech debt item above. For setup-time use this is acceptable; if `add_rule` is called in hot game loops it becomes O(n log n) per call.
- Files: `src/lib.rs` (line 173)
- Cause: Full sort rather than insertion sort.
- Improvement path: Binary search insert, or document that `add_rule` must not be called after engine initialization.

## Fragile Areas

**`CommitFrame` lifetime snapshot semantics:**
- Files: `src/lib.rs` (lines 107–115, 276–286, 289–315)
- Why fragile: The snapshot captures `lifetimes` and `enabled` before the commit executes. `undo()` correctly restores pre-commit state. `redo()` incorrectly restores pre-commit state instead of post-commit state (see Known Bugs). Any modification to snapshot/restore logic must account for both directions independently.
- Safe modification: Add a second post-commit snapshot field to `CommitFrame` and update `redo()` to use it. Do not reuse the existing snapshot field for both directions.
- Test coverage: Zero — there are no tests in the library. Correctness of undo/redo after lifetime interactions is entirely untested.

**FNV-1a replay hash collisions:**
- Files: `src/lib.rs` (lines 9–18, 268–272)
- Why fragile: The hash is 64-bit FNV-1a XOR-folded per op. Two different operation sequences that happen to produce the same hash value would be considered identical replays. FNV-1a has known weaknesses against crafted inputs, and XOR-then-multiply chaining over a sequence does not guarantee unique hashes for unique sequences.
- Safe modification: The hash is used for replay integrity, not security. Collision probability is acceptable for honest game clients. If used in adversarial contexts (e.g., anti-cheat), replace with a cryptographic MAC.
- Test coverage: None.

## Scaling Limits

**Undo stack grows unbounded:**
- Current capacity: No limit enforced.
- Limit: For long-running sessions with many irreversible commits, `undo_stack` will grow without bound, accumulating `CommitFrame` entries each holding a clone of `lifetimes`, `enabled`, and the full `Transaction<O>`.
- Scaling path: Expose a max-depth parameter on `Engine::new()`, or provide a `trim_undo_history(n)` method that drops frames beyond depth `n`.

## Dependencies at Risk

**No external dependencies:**
- `Cargo.toml` lists no `[dependencies]`. No third-party dependency risk.

## Missing Critical Features

**No test suite:**
- Problem: `src/lib.rs` contains zero `#[test]` functions. `cargo test` runs 0 tests. The engine's correctness guarantees (determinism, undo/redo, preview isolation, replay hash integrity) are entirely undocumented by automated tests.
- Blocks: Safe refactoring, contributor confidence, CI/CD gating.

**No `#[cfg(feature)]` or feature flags:**
- Problem: There is no way to opt into or out of parts of the API (e.g., disabling undo stack for memory-constrained targets). All functionality is always compiled in.
- Blocks: Embedding in environments where undo history is unnecessary overhead.

**No documentation on public API items:**
- Problem: No `///` doc comments exist on any public type, trait, or method in `src/lib.rs`. `cargo doc` generates an empty documentation surface. The library is pre-1.0 but is published to crates.io (`documentation = "https://docs.rs/herdingcats"`).
- Blocks: External adoption; docs.rs page is effectively empty.

## Test Coverage Gaps

**Engine core — zero coverage:**
- What's not tested: `dispatch()`, `dispatch_preview()`, `undo()`, `redo()`, `add_rule()`, `write()`, lifetime expiry for all three `RuleLifetime` variants, hash accumulation, preview isolation.
- Files: `src/lib.rs` (entire file)
- Risk: Any refactor can silently break determinism, undo correctness, or preview isolation without detection.
- Priority: High

**`RuleLifetime::Turns` / `Triggers` expiry — zero coverage:**
- What's not tested: Countdown logic, boundary behavior at `n = 1` and `n = 0`, interaction with undo.
- Files: `src/lib.rs` (lines 230–263)
- Risk: Off-by-one errors in decrement/disable logic would not be caught.
- Priority: High

**Undo / redo stack interaction — zero coverage:**
- What's not tested: Multi-step undo, redo after undo, redo stack cleared on new commit, hash restoration.
- Files: `src/lib.rs` (lines 289–315)
- Risk: The known redo snapshot bug (above) has no regression test to prevent reintroduction after any fix.
- Priority: High

---

*Concerns audit: 2026-03-08*
