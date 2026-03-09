---
phase: 3
slug: backgammon-example-and-integration-properties
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-08
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | proptest 1.10 (already in Cargo.toml dev-dependencies) |
| **Config file** | none — default proptest settings |
| **Quick run command** | `cargo test --example backgammon` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --example backgammon`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 0 | BACK-01 | smoke | `cargo run --example backgammon` | ❌ W0 | ⬜ pending |
| 3-01-02 | 01 | 0 | BACK-02 | unit | `cargo test --example backgammon` | ❌ W0 | ⬜ pending |
| 3-01-03 | 01 | 1 | BACK-03 | unit | `cargo test --example backgammon` | ❌ W0 | ⬜ pending |
| 3-01-04 | 01 | 1 | BACK-04 | unit | `cargo test --example backgammon` | ❌ W0 | ⬜ pending |
| 3-01-05 | 01 | 1 | BACK-05 | property | `cargo test --example backgammon prop_board_conservation` | ❌ W0 | ⬜ pending |
| 3-01-06 | 01 | 1 | BACK-06 | property | `cargo test --example backgammon prop_per_die_undo` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `examples/backgammon.rs` — entire phase lives here; file does not exist yet
- [ ] No additional test infrastructure needed — proptest already in dev-dependencies, no new fixtures required

*Existing infrastructure covers all phase requirements once the example file is created.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Output readability — labeled println! annotations are self-explanatory | BACK-01 | Subjective output quality | Run `cargo run --example backgammon`, verify each step is labeled (dice roll, move, undo) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
