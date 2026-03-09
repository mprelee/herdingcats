---
phase: 2
slug: engine-property-tests
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-08
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner + proptest 1.10 |
| **Config file** | none — proptest already in Cargo.toml dev-dependencies |
| **Quick run command** | `cargo test -p herdingcats -- props` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p herdingcats -- props`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | PROP-01 | proptest | `cargo test -- props::prop_undo_roundtrip` | ✅ | ⬜ pending |
| 02-01-02 | 01 | 1 | PROP-02, PROP-03, PROP-04 | proptest | `cargo test -- props` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

- proptest 1.10 already in `[dev-dependencies]`
- `src/engine.rs` exists with `#[cfg(test)] mod tests` — adding sibling `mod props` requires no new files

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Shrinking produces a minimal failing case | PROP-01 | Requires intentionally breaking the engine to verify shrinking works | Temporarily comment out `self.replay_hash = frame.state_hash_before` in `undo()`, run `cargo test -- props`, verify proptest output shows minimal counterexample |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
