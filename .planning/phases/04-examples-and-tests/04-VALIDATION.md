---
phase: 4
slug: examples-and-tests
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-13
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + proptest 1.10 |
| **Config file** | Cargo.toml (`[dev-dependencies]`) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test && cargo run --example tictactoe && cargo run --example backgammon` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test && cargo run --example tictactoe && cargo run --example backgammon`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-01-01 | 01 | 1 | EXAM-01 | integration | `cargo run --example tictactoe` | ❌ W0 | ⬜ pending |
| 4-01-02 | 01 | 1 | EXAM-01 | integration | `cargo run --example tictactoe` | ❌ W0 | ⬜ pending |
| 4-02-01 | 02 | 1 | EXAM-02 | integration | `cargo run --example backgammon` | ❌ W0 | ⬜ pending |
| 4-02-02 | 02 | 1 | EXAM-02 | integration | `cargo run --example backgammon` | ❌ W0 | ⬜ pending |
| 4-03-01 | 03 | 2 | TEST-01 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| 4-03-02 | 03 | 2 | TEST-01 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| 4-04-01 | 04 | 2 | TEST-02 | property | `cargo test proptest` | ❌ W0 | ⬜ pending |
| 4-04-02 | 04 | 2 | TEST-02 | property | `cargo test proptest` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `examples/tictactoe.rs` — stub with `fn main()` compiling cleanly (EXAM-01)
- [ ] `examples/backgammon.rs` — stub with `fn main()` compiling cleanly (EXAM-02)
- [ ] `tests/integration.rs` or inline `#[cfg(test)]` module — unit test stubs for 15 invariants (TEST-01)
- [ ] proptest strategy stubs for determinism/atomicity/undo-redo properties (TEST-02)

*Existing `proptest` dev-dependency in Cargo.toml — no new packages needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| tictactoe demonstrates all Outcome variants in sequence | EXAM-01 | Visual output check | Run `cargo run --example tictactoe`, confirm each Outcome variant appears in output |
| backgammon undo history clears after Irreversible dispatch | EXAM-02 | Visual output check | Run `cargo run --example backgammon`, confirm output shows undo history cleared after dice-roll |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
