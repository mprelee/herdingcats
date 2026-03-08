# Contributing

Thank you for your interest in contributing to herdingcats.

This project enforces strict determinism and architectural constraints.
Please review the following before submitting changes:

- docs/ARCHITECTURAL_INVARIANTS.md
- docs/EXTENSION_GUIDELINES.md

These define the engine’s correctness guarantees.

---

## Core Principles

All contributions must preserve:

- Deterministic execution
- Explicit rule ordering
- Transactional state mutation
- Replay hash integrity
- Undo / redo correctness
- Preview isolation
- Static event typing

If a change compromises any of these, it will not be accepted.

---

## Before Opening a PR

1. Ensure the change preserves determinism.
2. Confirm all state mutation occurs via `Operation`.
3. Confirm undo / redo semantics remain correct.
4. Confirm replay hash behavior is unchanged (unless intentionally extended).
5. Ensure preview execution remains isolated.
6. Run tests and ensure reproducibility.

---

## Feature Additions

New mechanics should generally be implemented by:

- Adding new event variants
- Adding new operation variants
- Adding new rule implementations

The engine core should not contain feature-specific logic.

---

## Performance Changes

Optimizations are welcome but must not change:

- Execution order
- Hash semantics
- Undo behavior
- Deterministic guarantees

Parallel rule execution is not permitted.

---

## Versioning

The project is pre‑1.0.

Minor releases may introduce breaking changes if required to preserve architectural clarity.
Once 1.0 is reached, breaking changes will follow semantic versioning.

---

## Discussion

If you are unsure whether a change preserves invariants, open an issue before implementing it.

Determinism and correctness take priority over feature growth.
