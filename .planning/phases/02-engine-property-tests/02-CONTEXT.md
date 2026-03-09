# Phase 2: Engine Property Tests - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Add proptest-based property tests (PROP-01 through PROP-04) to verify four engine invariants: undo/redo roundtrip correctness, dispatch_preview isolation, rule lifetime off-by-one correctness, and cancelled transaction isolation. No new API, no new modules — only test code in engine.rs.

</domain>

<decisions>
## Implementation Decisions

### Test file location
- All property tests inline in `engine.rs` as a separate `#[cfg(test)] mod props { ... }` block
- The existing `#[cfg(test)] mod tests { ... }` block keeps unit tests; the new `mod props` holds proptest-based tests
- All four property tests in one module — shared CounterOp fixture, shared NoRule, no submodule split

### Op fixture scope
- Property test strategies generate `Inc` and `Dec` only — no `Reset` in generated sequences
  - `Reset { prior }` requires knowing current state at generation time; excluded to keep strategies stateless
  - `Reset` remains in unit tests (mod tests) for targeted roundtrip coverage
- PROP-04 (cancelled transaction): generate arbitrary Vec<CounterOp> (Inc/Dec), push into tx.ops, then set `tx.cancelled = true` — confirms ops are suppressed, not just that an empty tx is a no-op
- PROP-03 (rule lifetimes): generate `n` from range `1..=10` (not arbitrary u32) — dispatch exactly n times, verify disabled at n
- All four property tests use the same fixture type: `Engine<i32, CounterOp, (), u8>`

### Rule presence
- PROP-01 and PROP-02: add `NoRule` (passthrough, no-op `before`/`after`) — tests that rule traversal doesn't affect invariants, slightly more realistic than bare engine
- PROP-03: use a dedicated `CountingRule` with a no-op `before()` hook — lifetime decrement logic is in the engine, not the rule; keep the invariant clean
- PROP-04: NoRule sufficient (cancelled tx suppresses ops regardless of rules)

### Test configuration
- Proptest case count: leave at default (256 cases per test) — override with `PROPTEST_CASES` env var if needed
- Generated op sequence length for PROP-01: `0..=20` ops per sequence
- No `proptest.toml` config file — leave all shrinking settings at proptest defaults

### Claude's Discretion
- Exact `prop_compose!` strategy names and signatures
- How to handle the `tx.irreversible` flag in generated transactions for PROP-01 (must be true for undo stack to record the frame — likely set statically in the strategy)
- `CountingRule` struct placement within `mod props`
- Whether to use `prop::collection::vec` or a custom strategy for op sequence generation

</decisions>

<specifics>
## Specific Ideas

- Every undo property test must assert both `engine.read()` (state) AND `engine.replay_hash()` — not just state equality. This was flagged explicitly in STATE.md decisions.
- The `tx.irreversible` flag must be `true` for dispatch to push a CommitFrame. Property tests for PROP-01 should set this statically so the undo stack is always populated.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CounterOp` (Inc, Dec, Reset{prior}): full fixture in `engine.rs` mod tests — reuse Inc/Dec variants in proptest strategies
- `NoRule`: already in `engine.rs` mod tests — re-declare or share within file for use in mod props
- `Engine<i32, CounterOp, (), u8>`: established test engine type — use throughout

### Established Patterns
- `#[cfg(test)] mod tests` inline in each source file — extend with `mod props` sibling block
- `tx.irreversible = true` needed for undo stack recording — this is critical for PROP-01

### Integration Points
- Property tests touch only `Engine::dispatch`, `Engine::undo`, `Engine::dispatch_preview`, `Engine::add_rule` — all public methods, no need for private access
- `engine.replay_hash()` is the public accessor for hash comparison

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-engine-property-tests*
*Context gathered: 2026-03-08*
