# Extension Guidelines

This document defines how herdingcats may be extended
without violating architectural invariants.

---

## 1. Add New Mechanics

Allowed:

- Add new event enum variants
- Add new operation variants
- Add new rule implementations

All state mutation must remain inside `Operation`.

Engine core must not be modified for feature-specific logic.

---

## 2. Add New Events

- Extend the event enum
- Update rules via pattern matching

Do not introduce dynamic event registries.
Do not use runtime type inspection.

---

## 3. Add New Operations

New operations must implement:

- `apply()`
- `undo()`
- `hash_bytes()`

Operations must be fully deterministic.

---

## 4. Add Temporary Effects

Use `RuleLifetime`.

Do not store duration inside rule structs
unless the lifetime is tracked in commit frames.

---

## 5. Add Deterministic RNG

If added:

- RNG state must live inside `S`
- RNG changes must occur through `Operation`
- RNG state must be hashable and undoable

---

## 6. Add Cancellation or Propagation Flags

May extend `Transaction` with additional flags.

Dispatch must respect flags deterministically.

---

## 7. Performance Changes

Optimizations must not change:

- Rule ordering
- Hash semantics
- Undo behavior
- Determinism

Parallel rule execution is not permitted.

---

## 8. Engine Modifications

Engine may be modified only if:

- Determinism is preserved
- Replay integrity is preserved
- Undo safety is preserved
- Preview isolation is preserved

Feature-specific logic must not be added to the engine core.

---

When uncertain, preserve:

- Explicitness
- Determinism
- Logged state
- Static typing
