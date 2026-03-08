# Architectural Invariants

These constraints define the core semantics of herdingcats.
They must not be weakened.

---

## 1. Determinism

All irreversible state mutation must:

- Occur exclusively through `Operation`
- Be undoable
- Be hashable
- Be replay-safe

No hidden side effects.
No global mutable state.
No nondeterministic ordering.

---

## 2. Transaction Model

All state changes occur via:

    Transaction<O>

Where:

- `O: Operation<S>`
- `apply()`
- `undo()`
- `hash_bytes()`

State must not be mutated outside `Operation`.

---

## 3. Replay Hash

- FNV‑1a 64-bit
- Updated only on irreversible commit
- Restored on undo / redo
- Not modified during preview

Replay integrity must be preserved.

---

## 4. Preview Isolation

`dispatch_preview()` must not mutate:

- State history
- Rule lifetimes
- Enabled rules
- Replay hash
- Undo / redo stacks

Preview must be fully reversible.

---

## 5. Rule Ordering

Rules must execute:

- BEFORE: low → high priority
- AFTER: high → low priority

Ordering must be deterministic and explicit.

---

## 6. Priority Enforcement

Priority type `P` must:

- Be an enum
- Use `#[repr(i32)]`
- Implement sealed `PriorityValue`
- Provide explicit integer representation

Raw integers must not replace priority enums.

---

## 7. Rule Lifetimes

Lifetimes must:

- Be deterministic
- Be stored in commit frames
- Be restored on undo / redo
- Not decrement during preview

---

## 8. Event System

Events must:

- Be represented as a closed enum
- Be statically typed
- Not use dynamic dispatch
- Not use `Any`

---

## 9. Undo / Redo

Undo must restore:

- State
- Lifetimes
- Enabled rules
- Replay hash

Redo must restore the same.

---

These invariants define the engine’s correctness.
If a change violates them, it is invalid.
