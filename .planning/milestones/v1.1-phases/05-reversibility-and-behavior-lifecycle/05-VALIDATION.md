---
phase: 05
slug: reversibility-and-behavior-lifecycle
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + proptest |
| **Config file** | `Cargo.toml` — proptest already in `[dev-dependencies]` |
| **Quick run command** | `cargo test --lib 2>&1 \| tail -5` |
| **Full suite command** | `cargo test 2>&1 \| tail -10` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib 2>&1 | tail -5`
- **After every plan wave:** Run `cargo test 2>&1 | tail -10`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 05-01-01 | 01 | 1 | REV-01 | unit | `cargo test --lib mutation 2>&1 \| tail -5` | ⬜ pending |
| 05-01-02 | 01 | 1 | REV-02 | unit | `cargo test --lib action 2>&1 \| tail -5` | ⬜ pending |
| 05-01-03 | 01 | 1 | LIFE-01,LIFE-02,LIFE-03 | unit | `cargo test --lib behavior 2>&1 \| tail -5` | ⬜ pending |
| 05-02-01 | 02 | 2 | REV-03,REV-04,LIFE-04,LIFE-05,LIFE-06 | unit+prop | `cargo test --lib 2>&1 \| tail -10` | ⬜ pending |
| 05-02-02 | 02 | 2 | REV-03,REV-04 | prop | `cargo test --lib props 2>&1 \| tail -5` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. `cargo test --lib` already runs all inline `#[cfg(test)]` modules. No new test infrastructure needed — new tests are added inline in modified files.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
