---
phase: 3
slug: history
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` |
| **Config file** | None — Cargo handles test discovery |
| **Quick run command** | `cargo test -p herdingcats` |
| **Full suite command** | `cargo test -p herdingcats` |
| **Estimated runtime** | ~3 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p herdingcats`
- **After every plan wave:** Run `cargo test -p herdingcats`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~3 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | HIST-01, HIST-02, HIST-03, HIST-04 | unit (RED) | `cargo test -p herdingcats` | Wave 0 adds tests to engine.rs | ⬜ pending |
| 03-01-02 | 01 | 1 | HIST-01, HIST-02, HIST-03, HIST-04 | unit (GREEN) | `cargo test -p herdingcats` | ✅ exists after 03-01-01 | ⬜ pending |
| 03-02-01 | 02 | 2 | HIST-03 | unit | `cargo test -p herdingcats` | ✅ exists | ⬜ pending |
| 03-02-02 | 02 | 2 | HIST-01, HIST-02 | unit | `cargo test -p herdingcats` | ✅ exists | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/engine.rs` `#[cfg(test)]` module — add failing tests for `undo()`, `redo()`, `undo_depth()`, `redo_depth()`, irreversibility clearing
- [ ] `src/outcome.rs` `#[cfg(test)]` module — add `HistoryDisallowed` constructability test

*Tests go in existing `#[cfg(test)]` modules per project convention — no new test files.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No `Reversible` trait required on diff types | HIST-04 | Structural — passes if design is correct | `cargo build -p herdingcats` with a diff type that has no Reversible-like impl |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
