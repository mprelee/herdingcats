# Phase 1: Module Split and Foundation - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Split `src/lib.rs` into five concept-focused modules (`hash.rs`, `operation.rs`, `transaction.rs`, `rule.rs`, `engine.rs`), with `lib.rs` becoming a thin re-export facade. Add `proptest = "1.10"` as a dev-dependency. Add inline `#[cfg(test)]` unit tests to every module file with a counter-based test fixture. Add `///` rustdoc to every public type, trait, and method — plus key internal items — with self-contained runnable code examples. Tictactoe example must continue to compile and run unchanged.

</domain>

<decisions>
## Implementation Decisions

### Documentation Voice

- **Audience**: A competent Rust developer encountering this paradigm for the first time. Assume they know Rust idioms; don't explain Rust. Do explain the engine model.
- **Balance**: Equal weight to design rationale ("why this abstraction exists") and concrete usage ("here's how you use it in a game"). Neither pure reference nor pure tutorial — both.
- **Trait docs depth**: `Operation` and `Rule` trait definitions get 3–5 sentence paradigm introductions. A reader who only reads these two trait definitions should come away understanding the whole engine model.
- **Internal items**: `CommitFrame` and `fnv1a_hash` get `//` comments that explain the mechanism and why it exists — e.g., what fields CommitFrame stores and why each is needed for undo correctness.

### Doc Code Examples

- **Game choice**: Minimal toy game per module — the simplest possible concrete types that illustrate the concept. No need to use tictactoe types everywhere; keep examples tight and scannable.
- **Runability**: Fully runnable `/// # Examples` blocks where possible. `cargo test --doc` should run them and catch regressions. Use `no_run` only when external state (filesystem, network, randomness) would be required.
- **Self-contained snippets**: Each example includes enough setup to run standalone — define minimal types inline, build what's needed, call the method. Long enough to copy-paste and run, not just fragments.

### Unit Test Fixtures

- **Fixture choice**: A dedicated minimal counter game — `State = i32`, `Ops = Inc / Dec / Reset`, `Event = ()` (unit). Trivially simple; test assertions are obvious (`apply(Inc)` → state goes `0 → 1`).
- **Location**: Defined inline inside `engine.rs`'s `#[cfg(test)]` module — that's the only file that needs a full game fixture. Other modules (`hash.rs`, `transaction.rs`, etc.) test their own slice in isolation without a game fixture.
- **Fixture scope**: `hash.rs` tests hash correctness on raw byte slices. `transaction.rs` tests `Transaction` builder/state. `rule.rs` tests the trait object contract. `engine.rs` tests the full engine with the counter fixture.

### Module Structure (locked from project setup)

- Files: `src/hash.rs`, `src/operation.rs`, `src/transaction.rs`, `src/rule.rs`, `src/engine.rs`
- `src/lib.rs`: only `mod` declarations + explicit `pub use` — no logic
- `hash` module: `pub(crate)` only — `fnv1a_hash` and FNV constants are implementation details, never public
- `CommitFrame`: stays private inside `engine.rs`
- Build order follows DAG: hash → operation → transaction → rule → engine → lib.rs

### Code Style (carries forward from existing conventions)

- Section banners (`// ============================================================`) used within each file to separate logical sections
- `where` clauses on separate lines for generic bounds
- `#[derive(Clone)]` on all snapshot-able types
- No `unwrap()`, `panic!`, or `expect()` in library code

### Claude's Discretion

- Exact wording of doc prose (voice/tone within the decided constraints)
- Whether to use `compile_fail` examples for any contract violations
- How to handle edge cases in `fnv1a_hash` doc (zero-length input, etc.)
- Test assertion style within `#[cfg(test)]` modules

</decisions>

<specifics>
## Specific Ideas

- The `Operation` and `Rule` trait docs should feel like they could stand alone as a one-page explanation of the paradigm — if someone copy-pasted just those two doc blocks into a README, a reader would understand what this library does and why.
- "Anyone reading the source should be taught the paradigm from reading it" — this is the guiding principle for all documentation decisions.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets

- `src/lib.rs` (full source): All five module files are direct extractions — no new logic, just reorganization with `pub use` wiring
- `examples/tictactoe.rs`: Reference implementation showing how a consumer implements `Operation`, `Rule`, `Engine`. Can be referenced in doc examples as a more complex case.

### Established Patterns

- Section banner comments (`// ============================================================`): Must be used in every new file to separate logical sections — strict existing convention
- `where` clause placement: On separate lines from `impl`/`fn` signatures — strict existing convention
- `pub` on all consumer-facing items, private on internals — `CommitFrame`, `fnv1a_hash` stay private
- No `Result`/`?` in library core — all operations are infallible; invalid state → `tx.cancelled = true`
- `#[derive(Clone)]` on all types that need snapshotting

### Integration Points

- `lib.rs` re-exports: Every currently-`pub` item in `src/lib.rs` must appear in a `pub use` in the new `lib.rs` — verified by `cargo test --examples` passing with `tictactoe.rs` unchanged
- `proptest` in `[dev-dependencies]`: Zero impact on release build; only affects `#[cfg(test)]` compilation

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-module-split-and-foundation*
*Context gathered: 2026-03-08*
