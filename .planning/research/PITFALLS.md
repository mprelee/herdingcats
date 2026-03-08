# Pitfalls Research

**Domain:** Rust library module refactoring + proptest property-based testing for stateful game engine
**Researched:** 2026-03-08
**Confidence:** HIGH (module splitting pitfalls from official docs + community; proptest pitfalls from official docs + 2024 real-world case studies)

---

## Critical Pitfalls

### Pitfall 1: Accidental Public API Break Via Incomplete Re-exports

**What goes wrong:**
When splitting `src/lib.rs` into modules, items that were previously public at the crate root become public inside a private submodule. If `lib.rs` does not re-export every previously-public item with `pub use`, callers get `use herdingcats::Transaction` -> "not found" errors. Because Rust's name resolution requires the full path to be public — not just the leaf item — a missed re-export is a semver-breaking change on a published crate.

**Why it happens:**
The split feels mechanical ("move code into files") but visibility in Rust is path-dependent. Developers check that the item has `pub` on it and assume that's enough; they don't verify that the path `herdingcats::X` still resolves.

**How to avoid:**
- Keep `lib.rs` as a thin re-export façade: `pub use engine::Engine; pub use transaction::Transaction;` etc. for every type/trait currently in the public API.
- After each module is extracted, run `cargo test --examples` immediately — the `examples/tictactoe.rs` file acts as an integration test for the full public surface.
- Run `cargo doc --no-deps` and confirm every previously-documented item appears.
- Optionally run `cargo semver-checks` (dev tool, not a project dependency) to catch missed re-exports before publishing.

**Warning signs:**
- `tictactoe.rs` fails to compile after a module is extracted — this is the canary.
- `cargo doc` output is missing types that were present before.
- Clippy `unreachable_pub` lint fires on an item inside a module that should be public externally — means the re-export chain is broken.

**Phase to address:** Module splitting phase (extracting `hash.rs`, `operation.rs`, `transaction.rs`, `rule.rs`, `engine.rs`). Each module extraction must end with a green `cargo test --examples` before proceeding to the next.

---

### Pitfall 2: `pub use` Glob Shadow Breaking Re-exports

**What goes wrong:**
If `lib.rs` uses a glob re-export (`pub use engine::*`) alongside an explicit item, Rust's name resolution can silently shadow or drop the glob-imported item. This was identified as the cause of hundreds of accidental semver breaks among the top 1000 Rust crates. The item disappears from the public API without a compile error in the library itself — downstream code breaks.

**Why it happens:**
Glob re-exports are convenient but name resolution is greedy: adding any same-named item in the same scope shadows the glob import, with no warning by default.

**How to avoid:**
Prefer explicit `pub use module::SpecificItem` re-exports in `lib.rs` rather than `pub use module::*`. Each re-exported item is named explicitly, making shadowing impossible and making the public API surface auditable by inspection.

**Warning signs:**
- `lib.rs` uses `pub use somemodule::*` patterns.
- A new name added to any module or imported in `lib.rs` matches an existing item name.
- `cargo semver-checks` reports a removed item.

**Phase to address:** Module splitting phase. Establish the explicit re-export convention from the first module extracted and never introduce glob re-exports.

---

### Pitfall 3: `cfg(test)` Visibility Cannot See `pub(crate)` Across Module Boundaries Without Proper Structure

**What goes wrong:**
When tests move from a single `lib.rs` into `#[cfg(test)] mod tests` blocks inside individual module files, internal helpers that were previously co-located become invisible. For example, `CommitFrame` is private in `lib.rs` now; tests in `engine.rs` can see it directly, but if a test helper lives in `hash.rs` and needs to invoke an engine internal, the paths break. Conversely, marking things `pub(crate)` to fix this can accidentally widen visibility beyond what is safe.

**Why it happens:**
In a single-file crate, everything is in the same module scope. After splitting, Rust's privacy rules apply: sibling modules cannot see each other's private items. Developers either over-use `pub` (widening the API) or under-use it (leaving tests that can't compile).

**How to avoid:**
- For items that only tests need to cross-access: `pub(crate)` is the correct scope — it does not affect the external API.
- Keep tests in the same file as the code they test: `#[cfg(test)] mod tests { use super::*; }` at the bottom of each `.rs` file. `super::*` imports all items from the enclosing module, including private ones, which is exactly what unit tests need.
- Integration-style tests that need to compose multiple modules go in `tests/` (separate integration test files) using only the public API.

**Warning signs:**
- A test file uses `crate::engine::CommitFrame` (tries to reach into private internal across crate path).
- Adding `pub` to internal struct/enum to make a test compile — if it's not in the re-export list in `lib.rs`, that `pub` is useless externally, but it's a warning that the test structure is wrong.
- Clippy `redundant_pub_crate` fires unexpectedly.

**Phase to address:** Module splitting phase. Establish the `#[cfg(test)] mod tests { use super::*; }` pattern in the first module and apply it consistently.

---

### Pitfall 4: proptest Filtering (`prop_assume` / `.prop_filter`) Breaks Shrinking

**What goes wrong:**
To generate a "valid game state" for backgammon (board positions that are physically possible), developers reach for `prop_assume!(is_valid_state(&state))` or `.prop_filter(...)`. This appears to work — it generates valid inputs — but it silently degrades shrinking. When proptest tries to simplify a failing test case, rejected values during shrinking cause it to "back away from simplification" rather than continue, so the minimized failing case is much larger and harder to debug than it should be.

**Why it happens:**
Rejection sampling is conceptually simple. The connection between filtering and shrinking quality is non-obvious: the proptest book documents it but developers don't read that section until they encounter terrible shrinking output.

**How to avoid:**
- **Do not use filtering to constrain generated game states.** Instead, build strategies that directly produce valid states by construction. For backgammon: generate a die roll (1..=6), then generate only moves that are legal given that roll, using `prop_flat_map` to make the second strategy depend on the first.
- For the engine's undo/redo tests specifically: generate a sequence of `N` operations using `prop::collection::vec(valid_operation_strategy(), 0..10)` where `valid_operation_strategy()` only produces operations that are inherently valid, not a broad space filtered down.
- Reserve `prop_assume!` for conditions that are cheap, rarely rejected (<10% of generated values), and not part of core invariants.

**Warning signs:**
- Test output shows minimal failing cases like "after 47 operations" when logically a 2-3 step case should exist.
- Test suite is slow — `cargo test` takes minutes for a few proptest tests.
- `PROPTEST_MAX_SHRINK_ITERS` has been increased as a "fix" — this treats the symptom, not the cause.

**Phase to address:** Property testing phase (when writing undo/redo roundtrip tests and hash determinism tests). The strategy design must produce valid states by construction, not filtering.

---

### Pitfall 5: Incomplete Invariant Checking in State Machine Tests Hides Real Bugs

**What goes wrong:**
A proptest state machine test that only checks "the final state equals expected" misses bugs that occur mid-sequence and self-repair. For the `Engine`: a rule lifetime that decrements incorrectly during a sequence of dispatches might cancel out over N operations, making the final state correct while intermediate states are wrong. The real bug (double-decrement of `Triggers(n)`) goes undetected.

**Why it happens:**
Writing a postcondition for every step is more work than writing one final assertion. Developers start with a single final check and never go back.

**How to avoid:**
- Check invariants **after each step** in stateful proptest tests, not only at the end. For the engine, after every `dispatch`, verify: (a) `replay_hash` matches a reference computation, (b) enabled rule set is consistent with lifetimes, (c) `undo_stack.len()` matches the number of irreversible non-cancelled dispatches.
- The `proptest-stateful` crate provides `check_invariants()` called after each step — use this structure rather than hand-rolling a test loop.
- For undo/redo roundtrip tests: assert state equality after each undo, not only after undoing all moves.

**Warning signs:**
- A proptest test only has one `prop_assert_eq!` at the very end.
- The test state struct is a single "expected output" rather than a model tracking intermediate states.
- A bug is found manually in the tic-tac-toe example that the proptest tests did not catch.

**Phase to address:** Property testing phase. The test structure (step-wise invariant checking) must be established before writing individual test cases, not retrofitted.

---

### Pitfall 6: Testing `Box<dyn Rule>` Without `Clone` Support Blocks Strategy Composition

**What goes wrong:**
`Engine<S, O, E, P>` holds `rules: Vec<Box<dyn Rule<S, O, E, P>>>`. For proptest to generate `Engine` instances with rules pre-installed, you'd need to clone the engine or the rules. But `Box<dyn Rule>` is not `Clone` — `Rule` does not require `Clone` and adding it would break dyn-compatibility. This silently blocks the strategy `any::<Engine<...>>()` from being derivable, and attempts to derive `Arbitrary` for `Engine` will fail to compile.

**Why it happens:**
The proptest `Arbitrary` derive works well for simple structs with all-clonable fields. The interaction between `Box<dyn Trait>` and proptest's `Clone`-requiring infrastructure is a compile-time surprise, not a runtime failure.

**How to avoid:**
- Do not attempt to implement `Arbitrary` for `Engine` directly. Instead, generate test engines by composing smaller strategies: generate the state `S`, generate a sequence of `Event + Transaction` pairs, construct the engine from scratch in the test body using `Engine::new(state)` and `engine.dispatch(...)`, and use `proptest`'s `prop_flat_map` to thread state through steps.
- For tests that need pre-populated engines with rules: create concrete test rule structs (not trait objects) inside the test module — concrete types are `Clone`-able. Install them via `add_rule()` in test setup, not via strategy.

**Warning signs:**
- Attempt to `#[derive(proptest_derive::Arbitrary)]` on `Engine` — this will fail at compile time with a confusing error about `Clone`.
- Any strategy that generates a `Box<dyn Rule>` — there is no sound way to do this without a dedicated trait extension.

**Phase to address:** Property testing phase, specifically at the start when designing test helpers. Establish the "build engine from generated moves, not generated engines" pattern first.

---

### Pitfall 7: Undo/Redo Tests That Only Check State, Not Hash

**What goes wrong:**
`Engine` maintains `replay_hash` as an independent invariant. A test that dispatches N operations, undoes them all, and asserts `engine.state == initial_state` can pass while `engine.replay_hash` is wrong. The undo path correctly restores `state_hash_before` from `CommitFrame`, but any test that doesn't assert `replay_hash` after undo never validates this. A bug in the hash restore path is invisible until replay-based features break.

**Why it happens:**
`engine.state` is the visible, intuitive thing to check. `replay_hash` is internal (`pub` but not in the "obvious" assertion list) and easy to overlook.

**How to avoid:**
- In every undo/redo roundtrip property test, assert both `engine.read() == original_state` AND `engine.replay_hash() == original_hash` after undoing all operations.
- Define a helper `engine_snapshot(e: &Engine<...>) -> (S, u64)` that captures both, and use it as the baseline for comparison.

**Warning signs:**
- Undo tests only assert on `engine.read()`.
- `replay_hash()` is not referenced anywhere in the test module.

**Phase to address:** Property testing phase. Codify the dual-assertion pattern in the first undo roundtrip test.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `pub use module::*` glob re-exports in `lib.rs` | Less typing during split | Shadowing bugs, hard to audit public surface | Never — use explicit re-exports |
| `prop_assume!` to filter invalid game states | Simple to write | Broken shrinking, slow CI, hard-to-debug failures | Only when rejection rate is <5% and invariant is peripheral |
| Single final assertion in stateful tests | Fast to write | Misses mid-sequence bugs that cancel out | Never for engine invariants |
| Testing only `engine.state`, not `replay_hash` | Less to type | Hash bugs invisible until replay features | Never — always assert both |
| Marking internal structs `pub` to satisfy test visibility | Test compiles | Widens API unintentionally, confuses future readers | Never — use `pub(crate)` or `use super::*` in test submodule |

---

## Integration Gotchas

This project has no external service integrations. The relevant "integration" is between proptest and the engine's generic type parameters.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| proptest + `Engine<S,O,E,P>` | Trying to generate `Engine` instances directly via `Arbitrary` | Generate `(S, Vec<(E, Transaction<O>)>)`, build engine in test body |
| proptest + `Box<dyn Rule>` | Trying to make rules part of a generated strategy | Use concrete test rule types inside `#[cfg(test)]`, not trait objects |
| `cargo test` + `examples/` | Extracting modules without running `--examples` after each step | Make `cargo test --examples` part of the definition of "done" for each module extraction |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| `prop_assume!` with >10% rejection rate | Tests take 5-10x longer, CI timeouts | Build valid-state strategies by construction via `prop_flat_map` | From the first test run; gets worse with more complex state |
| Default proptest case count (256) for heavy state machine tests | Tests time out in CI | Tune `ProptestConfig::with_cases(50)` for heavy multi-step tests; keep high for cheap unit tests | When each test case involves 10+ engine dispatches |
| Cloning full `Engine` state for every undo snapshot | Acceptable now; problematic with large game states | Not a current concern for backgammon/tictactoe scale | Only relevant at 10K+ element state, not applicable here |

---

## "Looks Done But Isn't" Checklist

- [ ] **Module splitting:** All modules extracted, but `pub use` re-exports in `lib.rs` verified by compiling `examples/tictactoe.rs` — do not rely on `cargo check` of the library alone.
- [ ] **proptest undo tests:** Tests assert both `engine.read()` and `engine.replay_hash()` after undo, not only state equality.
- [ ] **Rule lifetime correctness:** Tests exercise `RuleLifetime::Triggers(n)` with n=1 (expires after one use) — this is the edge case most likely to have an off-by-one.
- [ ] **Cancelled transaction invariant:** A dispatched transaction with `tx.cancelled = true` must not push a `CommitFrame` — proptest test explicitly generates cancelled transactions and asserts undo stack length is unchanged.
- [ ] **`dispatch_preview` has no side effects:** Property test verifies that `replay_hash` and `lifetimes` are identical before and after `dispatch_preview` — the snapshot/restore in that path is easy to break.
- [ ] **`write()` resets all stacks:** Test verifies `undo_stack` and `redo_stack` are empty and `replay_hash == FNV_OFFSET` after `engine.write(snapshot)`.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Missed re-export breaks public API | LOW (before publish) / HIGH (after publish, requires semver major bump) | Add missing `pub use` in `lib.rs`, compile examples, re-publish patch if not yet released |
| Glob shadow breaks re-export | MEDIUM | Replace `pub use module::*` with explicit re-exports, verify with `cargo semver-checks` |
| `prop_assume` degrading shrinking | MEDIUM | Rewrite strategy using `prop_flat_map` to generate valid states by construction; existing tests remain valid |
| Missing hash assertion in undo tests | LOW | Add `engine.replay_hash()` assertion to existing tests; mechanically straightforward |
| Wrong test visibility structure | LOW | Move test helpers into `#[cfg(test)] mod tests { use super::*; }` in the relevant file |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Incomplete re-exports breaking public API | Module splitting — each extraction | `cargo test --examples` green after every module move |
| Glob shadow via `pub use *` | Module splitting — establish convention at start | Inspect `lib.rs`: every re-export is explicit and named |
| `cfg(test)` visibility across modules | Module splitting — first module sets pattern | All test modules use `use super::*`; no `pub` added solely for tests |
| `prop_assume` filter breaking shrinking | Property testing — strategy design before first test | No `prop_assume!` in engine property tests; `prop_flat_map` used for dependent generation |
| Incomplete per-step invariant checking | Property testing — test structure design | Every stateful test checks invariants inside the operation loop, not only after |
| `Box<dyn Rule>` blocking `Arbitrary` | Property testing — first proptest helper written | No `Arbitrary` impl on `Engine`; engine is built from generated operations in test body |
| Undo tests missing hash assertion | Property testing — first undo roundtrip test | `replay_hash()` appears in every undo/redo assertion |
| `dispatch_preview` side-effect regression | Property testing — when preview test is written | Snapshot-equality test for `replay_hash` and `lifetimes` before/after `dispatch_preview` |

---

## Sources

- [Proptest Filtering Documentation — official proptest book](https://altsysrq.github.io/proptest-book/proptest/tutorial/filtering.html) — HIGH confidence
- [Proptest State Machine Testing — official proptest book](https://proptest-rs.github.io/proptest/proptest/state-machine.html) — HIGH confidence
- [Stateful Property Testing in Rust — Readyset Engineering Blog, 2024](https://readyset.io/blog/stateful-property-testing-in-rust) — HIGH confidence (verified against proptest docs)
- [Property Testing Stateful Code in Rust — rtpg.co, 2024](https://rtpg.co/2024/02/02/property-testing-with-imperative-rust/) — MEDIUM confidence (practitioner case study)
- [Breaking semver in Rust by adding a private type or import — predr.ag](https://predr.ag/blog/breaking-semver-in-rust-by-adding-private-type-or-import/) — HIGH confidence (cargo-semver-checks maintainer)
- [SemVer in Rust: Tooling, Breakage, and Edge Cases — FOSDEM 2024](https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/) — HIGH confidence
- [Two ways of interpreting visibility in Rust — Kobzol's blog, 2025](https://kobzol.github.io/rust/2025/04/23/two-ways-of-interpreting-visibility-in-rust.html) — MEDIUM confidence (community blog, verified against Rust reference)
- [Visibility and Privacy — The Rust Reference](https://doc.rust-lang.org/reference/visibility-and-privacy.html) — HIGH confidence (official)
- [SemVer Compatibility — The Cargo Book](https://doc.rust-lang.org/cargo/reference/semver.html) — HIGH confidence (official)
- Code analysis of `src/lib.rs` — direct inspection of `CommitFrame`, `dispatch`, `dispatch_preview`, `undo`, `redo` implementations

---
*Pitfalls research for: herdingcats — Rust library module refactoring and proptest property-based testing*
*Researched: 2026-03-08*
