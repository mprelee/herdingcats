# AI Modification Policy

This repository is governed by strict architectural invariants.

Before proposing or applying changes, you must read:

- docs/ARCHITECTURAL_INVARIANTS.md
- docs/EXTENSION_GUIDELINES.md

These documents define:

- Determinism requirements
- Replay integrity requirements
- Undo / redo guarantees
- Preview isolation rules
- Rule ordering semantics
- Priority enforcement constraints

All changes must preserve these invariants.

---

## Hard Requirements

AI-generated modifications must not:

- Introduce nondeterministic behavior
- Mutate state outside `Operation`
- Modify replay hash semantics
- Bypass undo logging
- Use dynamic event dispatch
- Introduce `Any` or runtime type inspection
- Replace priority enums with raw integers
- Modify rule execution order
- Mutate state during preview
- Introduce hidden mutable state

If a proposed change risks violating determinism or replay integrity, it must be rejected.

---

## Extension Policy

New functionality must follow the patterns described in:

- docs/EXTENSION_GUIDELINES.md

Feature-specific logic must not be embedded in the engine core.

All state mutation must remain explicit and transactional.

---

## Default Principle

When uncertain, preserve:

- Determinism
- Explicit ordering
- Logged state mutation
- Static typing
- Replay correctness

The engine prioritizes correctness over flexibility.
