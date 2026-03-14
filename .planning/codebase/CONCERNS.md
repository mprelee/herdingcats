# Codebase Concerns

**Analysis Date:** 2026-03-13

## Severe: Current Branch Emptied

**State of Current Branch:**
- Current branch `maddie-edits` is empty: `src/lib.rs` contains 0 lines, `examples/` are empty files
- Most recent commit `25d6c96` ("Delete most things") removed all implementation code
- All working code exists only on `main` branch (v0.3.1)
- This blocks any development or testing on current branch

**Impact:**
- Cannot build or test current branch
- Any new work on `maddie-edits` starts from scratch without access to existing implementation

**Recovery Path:**
- Merge `main` into `maddie-edits` to restore working codebase
- Or reset branch to a previous commit with complete implementation (e.g., commit `30bf745`)

---

## Design: Behavior Contract Complexity

**Issue:** Behavior lifecycle management is non-obvious
- Files: `src/behavior.rs`, `src/engine.rs` (main branch)
- Behaviors implement `before()` and `after()` hooks with asymmetric priority ordering
- `before()` runs in ascending priority order; `after()` runs in descending priority order
- Behaviors can enable/disable themselves via internal state
- `dispatch_preview()` mutates behavior lifetimes (`enabled`, `lifetimes`) without committing state

**Contract Risk:**
- `dispatch_preview()` for AI look-ahead has side effects on behavior state, even though it's labeled a "dry run"
- Property test PROP-03 (`prop_03_preview_dispatch_isolation`) validates this is intentional but fragile
- Library users may misunderstand that preview dispatch affects behavior lifetimes/enabled sets

**Recommended Approach:**
- Document clearly that `dispatch_preview()` has behavior-internal side effects
- Consider separating "true preview" (no side effects) from "look-ahead" (with side effects)
- Add assertion tests that preview dispatch state mutations are intentional

---

## Data Flow: Irreversibility Boundary

**Issue:** Irreversible mutations create one-way barriers
- Files: `src/engine.rs`, `src/mutation.rs` (main branch)
- `Mutation` trait has optional `is_reversible()` method (default: true)
- Irreversible mutations clear both undo and redo stacks immediately upon commit
- Library user responsible for designating which mutations are irreversible

**Risk Areas:**
- No centralized list of irreversible mutation types; determined by `is_reversible()` implementation
- If designer forgets to mark a mutation irreversible (e.g., dice roll reveal), undo may become invalid
- Once undo stack is cleared, user loses all history — recovery is impossible

**Recommended Approach:**
- Consider audit mechanism to verify irreversible designations match domain semantics
- Add documentation of what mutations must be marked irreversible (randomness reveal, public information leak)
- Consider stricter type system to enforce irreversibility at compile time

---

## Architecture: Behavior Ordering Ties

**Issue:** Equal priority behaviors have deterministic but implicit ordering
- Files: `src/engine.rs` lines 131-136 (main branch)
- Multiple behaviors with same priority are sorted by memory address (`Ord` on boxed trait objects)
- ARCHITECTURE.md notes behaviors are sorted by `(order_key, behavior_name)` but implementation uses `priority()` only
- Design doc specifies name-based tiebreaker, but actual code does not implement this

**Gap Between Spec and Implementation:**
- Architecture specifies: "Designers may assign unique order keys to every behavior for full explicit ordering"
- Implementation: `Behavior` trait has `priority()` returning a single value, no separate `order_key` field
- No tiebreaker between behaviors with equal priority — ordering is deterministic but opaque

**Recommended Approach:**
- Clarify whether equal-priority behaviors need stable naming tiebreaker
- If ordering matters, add explicit `order_key()` method to `Behavior` trait alongside `priority()`
- Document the current sorting implementation clearly in behavior.rs

---

## State Management: Behavior Lifetimes Are Mutable

**Issue:** Behavior enable/disable state lives outside main state tree
- Files: `src/engine.rs` lines 36-39 (main branch) (`enabled` and `lifetimes` fields are private)
- `enabled` set and `lifetimes` map are engine internals, not part of user's `State`
- Cannot serialize/deserialize behavior state as part of game state snapshot
- Undo/redo does not affect behavior lifetimes — they persist across undo

**Problems:**
- If behavior has limited lifetime (e.g., "active for 3 turns"), undo does not revert the countdown
- Behavior state diverges from committed game state
- Cannot perfectly save/load game with behavior lifecycle state preserved

**Recommended Approach:**
- Move behavior lifetimes into user-defined `State` struct as part of domain architecture
- Provide helper types (e.g., `LifetimeCounter`) to manage common patterns
- Document that engine-internal behavior state is not undoable

---

## Encapsulation: undo_stack and redo_stack Are Public

**Issue:** Undo/redo stacks are public fields but not part of the stable API
- Files: `src/engine.rs` line 35 (main branch)
- Tests access `engine.undo_stack` and `engine.redo_stack` directly to check invariants
- No stable getter methods; tests and users can directly inspect/manipulate these stacks
- If internal representation changes, all consumer tests break

**Risk:**
- Tests that inspect `undo_stack.len()` will fail if stack implementation changes (e.g., to a deque or rolling buffer)
- Users may write code depending on stack visibility even though these are internal details

**Recommended Approach:**
- Add getter methods: `pub fn undo_depth(&self) -> usize` and `pub fn redo_depth(&self) -> usize`
- Make stacks private and provide only high-level query/manipulation APIs
- Update all tests to use getters instead of direct field access

---

## Testing: Property Tests Assume Specific Behavior

**Issue:** proptest suite is tightly coupled to implementation
- Files: `src/engine.rs` lines 1400+ (main branch)
- Property tests verify undo/redo behavior by checking internal stack states
- Tests assume specific behavior: e.g., PROP-03 expects `dispatch_preview()` to mutate `lifetimes`
- If engine refactoring changes how preview works, tests will fail even if semantics are preserved

**Example:** PROP-04 tests that cancelled actions don't affect replay_hash
- Test directly compares `engine.replay_hash()` value before/after
- If hash algorithm changes (e.g., from FNV-1a to SipHash), test fails even if properties hold

**Recommended Approach:**
- Separate tests into "implementation-detail tests" and "behavior invariant tests"
- For implementation-detail tests (stack structure), add comment explaining why they're checking internals
- Consider property-based testing of semantic properties (idempotence, atomicity) separate from hash algorithm

---

## Robustness: Example Complexity

**Issue:** tic-tac-toe example is large and may discourage adoption
- Files: `examples/tictactoe.rs` (main branch)
- 264 lines just for a simple game
- Example implements validation, UI state, and multiple behaviors — complex for a hello-world
- No minimal "3-behavior counter" example showing core concepts

**Impact:**
- New users may think herdingcats is more complex than it is
- Example doesn't clearly separate core engine from game logic

**Recommended Approach:**
- Keep tic-tac-toe as advanced example
- Add minimal 30-line counter example in docs showing `Mutation`, `Behavior`, and dispatch flow
- Document where domain logic ends and engine responsibility begins

---

## Documentation: API Surface Incomplete in ARCHITECTURE.md

**Issue:** ARCHITECTURE.md describes design but doesn't match implementation API
- The document specifies `Outcome` enum with `Committed`, `Undone`, `Redone`, `NoChange` variants
- Main branch implementation (v0.3.1+) changed to dispatch returning `Option<Action<M>>` instead
- Recent commit `54265e6` ("Dispatch return") updated API but docs may be stale
- CHANGELOG notes "Dispatch return (#21)" but doesn't describe what changed

**Impact:**
- Users reading ARCHITECTURE.md will see different API than what actually exists
- Code examples in architecture may not compile

**Recommended Approach:**
- Update ARCHITECTURE.md to reflect current `Option<Action<M>>` return type
- Add changelog entries explaining why `Outcome` enum was replaced
- Document the change in API design rationale

---

## Technical Debt: FNV-1a Hash May Collision on Complex State

**Issue:** FNV-1a is used for replay_hash fingerprinting
- Files: `src/hash.rs`, `src/engine.rs` (main branch)
- FNV-1a has known weaknesses with small input sizes and structured data patterns
- Used to verify determinism across engine instances — collision could silently hide divergence

**Risk:**
- Two different mutation sequences could theoretically produce same `replay_hash`
- If hidden information is revealed differently, hash may match despite different actual game states
- No collision detection or warning if hash matches but mutation sequence differs

**Recommended Approach:**
- Consider switching to SipHash or BLAKE3 for stronger collision resistance
- Or add secondary hash (e.g., hash of mutation count) for quick divergence detection
- Document FNV-1a limitations in code comments

---

## Performance: Copy-on-Write Not Implemented

**Issue:** ARCHITECTURE.md specifies copy-on-write semantics, but implementation uses full clone
- Files: `src/engine.rs` lines 365-380 (main branch) — `dispatch_preview` clones full state
- `dispatch_preview()` does `let mut working = self.state.clone()` on every call
- For large game states, this is expensive and defeats the purpose of speculative dispatch
- No attempt to use shared references or Rc/Arc for working state

**Impact:**
- AI look-ahead or large-state games will have performance degradation
- Each preview dispatch clones the entire state tree

**Recommended Approach:**
- Document that current implementation uses eager clone, not lazy CoW
- For performance-critical applications, provide guidance on state layout to minimize clone cost
- Consider future refactor using Rc<RefCell<>> or arena allocators for working state

---

## Missing Validation: No Cycle Detection in Behavior Dependencies

**Issue:** Behaviors can implicitly depend on each other through ordering
- Files: `src/engine.rs` (main branch)
- If behavior A's `after()` generates mutations that behavior B's `before()` checks, order matters
- No mechanism to detect if behavior ordering creates a dependency cycle or deadlock
- Library user must manually ensure behavior order is acyclic

**Risk:**
- Designer adds two behaviors with circular dependencies (A before B, B before A)
- Engine compiles and runs, but behavior interactions are unpredictable
- No error message or warning

**Recommended Approach:**
- Document behavior dependency contract clearly
- Consider compile-time checklist or lint for common patterns
- Could add debug mode that detects cycles by running behaviors twice and checking idempotence

---

## Correctness: Missing Tests for Behavior Mutation Capture

**Issue:** Behaviors can modify action mutations, but this is not extensively tested
- Files: `src/behavior.rs`, `src/engine.rs` (main branch)
- Behaviors can push/pop mutations from `action.mutations` during `before()` hook
- Only basic happy-path tests exist; edge cases like "behavior A removes mutation that behavior B added" are not covered
- No test for behavior adding mutation that another behavior then modifies

**Impact:**
- If behavior interaction rules are subtle, mutation capture could fail silently
- Complex multi-behavior systems may have hard-to-reproduce bugs

**Recommended Approach:**
- Add tests for mutation chaining: behavior A adds mutation M1, behavior B (higher priority) sees M1 and adds M2
- Test mutation cancellation and re-addition across behavior priority levels
- Document the expected semantics clearly in behavior.rs

---

## Documentation: No Migration Guide from v0.2 to v0.3+

**Issue:** API changed significantly but no migration documentation
- v0.2 vs v0.3 had refactoring in behavior naming and mutation handling
- Recent v0.4 changed dispatch return type from `Outcome` to `Option<Action<M>>`
- CHANGELOG is sparse; users upgrading will struggle to understand what broke

**Impact:**
- Users stuck on older version may not upgrade
- No clear guide on how to adapt existing code

**Recommended Approach:**
- Add `MIGRATION.md` documenting v0.2→v0.3 and v0.3→v0.4 changes
- Provide before/after code examples
- Highlight which changes are breaking vs additive

---

## Scalability: Behavior Registry Is Linear

**Issue:** `add_behavior()` re-sorts entire behavior list on every call
- Files: `src/engine.rs` line 143 (main branch)
- Engine maintains `behaviors` vec and sorts by priority on each `add_behavior()`
- With 100+ behaviors, adding a new one becomes O(n log n)
- No batch registration API

**Impact:**
- Not a problem for typical use (games have 10-50 behaviors), but could matter for complex rule systems
- Dynamic behavior registration (if allowed) would be slow

**Recommended Approach:**
- Document that behavior registration should happen during setup, not during gameplay
- Consider providing `add_behaviors_batch()` for registering multiple behaviors at once
- For now, the current O(n log n) per call is acceptable given typical behavior counts

---

## Summary of Priorities

| Concern | Severity | Effort | Priority |
|---------|----------|--------|----------|
| Current branch emptied | Severe | Low | Must fix first |
| Behavior lifetimes not undoable | High | Medium | Block serialization feature |
| API documentation stale vs implementation | High | Low | Quick win |
| Preview dispatch has side effects | Medium | Low | Clarify in docs |
| Irreversibility contract implicit | Medium | Medium | Add type safety |
| Property tests too implementation-specific | Medium | Medium | Refactor test suite |
| Stack fields public instead of private | Medium | Low | Quick cleanup |

---

*Concerns audit: 2026-03-13*
