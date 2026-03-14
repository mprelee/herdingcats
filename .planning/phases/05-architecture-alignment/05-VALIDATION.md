---
phase: 5
slug: architecture-alignment
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-13
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) + proptest |
| **Config file** | Cargo.toml (proptest as dev-dependency) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo run --example tictactoe && cargo run --example backgammon` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo run --example tictactoe && cargo run --example backgammon`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | SC-1 | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 05-01-02 | 01 | 1 | SC-1 | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 05-02-01 | 02 | 2 | SC-2,SC-4 | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 05-03-01 | 03 | 3 | SC-3,SC-5,SC-6,SC-7 | compile+examples | `cargo test && cargo run --example tictactoe && cargo run --example backgammon` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
