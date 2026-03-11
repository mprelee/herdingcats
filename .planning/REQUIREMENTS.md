# Requirements: herdingcats

**Defined:** 2026-03-10
**Core Value:** The engine's determinism and undo/redo correctness must be provably sound — property-based tests using proptest make this machine-verifiable, not just manually checked.

## v1.1 Requirements

Requirements for the Rename & Reversibility milestone. Continues numbering from v1.0 (all v1.0 requirements are now Validated in PROJECT.md).

### Rename

- [x] **REN-01**: `Operation<S>` trait renamed to `Mutation<S>` — `apply`, `undo`, `hash_bytes` method signatures preserved
- [x] **REN-02**: `Rule<S,O,E,P>` trait renamed to `Behavior<S,M,I,P>` — `before`, `after`, `id`, `priority` method signatures preserved
- [x] **REN-03**: `Transaction<O>` renamed to `Action<M>` — `mutations` vec (was `ops`), `deterministic`, `cancelled` fields preserved; `irreversible` field removed
- [ ] **REN-04**: All public re-exports in `lib.rs`, all doctests, and both examples compile and pass under new names with no behavioral changes

### Reversibility

- [ ] **REV-01**: `Mutation` trait gains `fn is_reversible(&self) -> bool { true }` as a default method — opt-out model, reversible by default
- [ ] **REV-02**: `Action<M>` derives reversibility from its mutations at commit time — reversible iff `mutations.iter().all(|m| m.is_reversible())`; no explicit reversibility field on `Action`
- [ ] **REV-03**: Engine clears the undo stack when committing an irreversible `Action` — enforces undo barrier (cannot undo past last irreversible commit)
- [ ] **REV-04**: Reversible `Action` commits push `CommitFrame` to undo stack as before; undo/redo semantics unchanged for reversible commits

### Behavior Lifecycle

- [ ] **LIFE-01**: `Behavior` trait gains `fn is_active(&self) -> bool { true }` default method — engine checks this per dispatch to determine if behavior participates
- [ ] **LIFE-02**: `Behavior` trait gains `fn on_dispatch(&mut self) {}` default method — called after each committed action (including redo) so behaviors can update internal state
- [ ] **LIFE-03**: `Behavior` trait gains `fn on_undo(&mut self) {}` default method — called when a reversible action is undone, enabling behaviors to reverse their own state changes
- [ ] **LIFE-04**: `engine.add_behavior(behavior)` replaces `engine.add_rule(behavior, lifetime)` — no lifetime parameter; behaviors self-manage their active status
- [ ] **LIFE-05**: Engine replaces `lifetimes: HashMap<&'static str, RuleLifetime>` + `enabled: HashSet<&'static str>` with per-dispatch `behavior.is_active()` checks
- [ ] **LIFE-06**: Engine calls `on_dispatch()` / `on_undo()` on all behaviors in a separate pass after state mutations are applied, avoiding borrow conflicts with state access in `before`/`after` hooks

### Tests

- [ ] **TEST-01**: All existing unit tests and proptest property tests updated for new names and passing
- [ ] **TEST-02**: New proptest: any `Action` containing a mutation where `is_reversible() = false` results in an empty undo stack after commit
- [ ] **TEST-03**: New proptest: reversible `Action`s committed after an irreversible one are individually undoable; `engine.undo()` halts at the barrier (undo stack empty)
- [ ] **TEST-04**: New unit test: stateful `Behavior` using `on_dispatch` counter deactivates after N dispatches — replaces `RuleLifetime::Turns` proptest coverage
- [ ] **TEST-05**: `examples/backgammon.rs` updated — dice roll mutation returns `is_reversible() = false`; `RollDiceRule` uses `on_dispatch`/`is_active` instead of `RuleLifetime`; compiles and passes
- [ ] **TEST-06**: `examples/tictactoe.rs` updated to new names; compiles and runs unchanged

### Documentation

- [ ] **DOC-01**: All public types — `Mutation<S>`, `Behavior<S,M,I,P>`, `Action<M>`, `Engine<S,M,I,P>` — have comprehensive rustdoc explaining their role in the state machine model, with usage guidance
- [ ] **DOC-02**: All new trait methods (`is_reversible`, `is_active`, `on_dispatch`, `on_undo`) have doc comments with runnable doctests demonstrating correct usage
- [ ] **DOC-03**: `cargo doc --no-deps` generates zero warnings; module-level prose updated to reflect Mealy/Moore state machine framing

### Extended Tests

- [ ] **TEST-07**: Unit tests for reversibility edge cases — empty `Action` (no mutations) is reversible; Action where all mutations are irreversible clears undo stack; Action with mixed mutations (some `is_reversible() = true`, some `false`) is treated as irreversible
- [ ] **TEST-08**: Unit tests for behavior lifecycle edge cases — `is_active() = false` skips `before`/`after` hooks; `on_undo()` called correctly when undoing; behavior deactivating during a dispatch sequence doesn't affect already-queued hooks

## v2 Requirements

### Output / Event Emission

- Engine emits `Output<O>` events (Effects + Requests) for UI consumption — separation of concerns between engine and presentation layer
- Mealy machine output: `(State, Input) → (State, [Output])` formally modeled
- `pest` grammar integration for `Input<I>` parsing

## Out of Scope

| Feature | Reason |
|---------|--------|
| Output/event emission | Architectural addition deferred to v1.2 or v2.0 |
| `pest` integration | Future milestone; `Input<I>` naming lays groundwork |
| Doubling cube, Crawford/Jacoby rules | Not needed for engine testing |
| Async / networked gameplay | Not part of this library's purpose |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REN-01 | Phase 4 | Complete |
| REN-02 | Phase 4 | Complete |
| REN-03 | Phase 4 | Complete |
| REN-04 | Phase 4 | Pending |
| REV-01 | Phase 5 | Pending |
| REV-02 | Phase 5 | Pending |
| REV-03 | Phase 5 | Pending |
| REV-04 | Phase 5 | Pending |
| LIFE-01 | Phase 5 | Pending |
| LIFE-02 | Phase 5 | Pending |
| LIFE-03 | Phase 5 | Pending |
| LIFE-04 | Phase 5 | Pending |
| LIFE-05 | Phase 5 | Pending |
| LIFE-06 | Phase 5 | Pending |
| TEST-01 | Phase 6 | Pending |
| TEST-02 | Phase 6 | Pending |
| TEST-03 | Phase 6 | Pending |
| TEST-04 | Phase 6 | Pending |
| TEST-05 | Phase 6 | Pending |
| TEST-06 | Phase 6 | Pending |
| DOC-01 | Phase 7 | Pending |
| DOC-02 | Phase 7 | Pending |
| DOC-03 | Phase 7 | Pending |
| TEST-07 | Phase 7 | Pending |
| TEST-08 | Phase 7 | Pending |

**Coverage:**
- v1.1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-10*
*Last updated: 2026-03-10 after Phase 7 (docs + extended tests) added*
