---
phase: 5
slug: build-time-compiler-and-example-integration
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-09
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust unit/integration tests + fixture build checks |
| **Config file** | none |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~20 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 20 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 5-01-01 | 01 | 1 | GEN-02 | compile | `cargo test` | ✅ | ⬜ pending |
| 5-01-02 | 01 | 1 | GEN-03 | semantic | `cargo test` | ✅ | ⬜ pending |
| 5-01-03 | 01 | 1 | INT-01 | fixture-smoke | `cargo test` | ✅ | ⬜ pending |
| 5-02-01 | 02 | 2 | GEN-02 | compile | `cargo test` | ✅ | ⬜ pending |
| 5-02-02 | 02 | 2 | GEN-05 | compile | `cargo test` | ✅ | ⬜ pending |
| 5-02-03 | 02 | 2 | GEN-03 | semantic | `cargo test` | ✅ | ⬜ pending |
| 5-03-01 | 03 | 3 | INT-01 | build | `cargo test` | ✅ | ⬜ pending |
| 5-03-02 | 03 | 3 | INT-02 | end-to-end | `cargo test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] None — existing Cargo test infrastructure is sufficient for this phase

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Generated code shape is readable enough for a consumer to inspect during debugging | GEN-05 | Readability is subjective | Inspect generated module output and confirm identifiers, helper names, and registration path are understandable |
| Consumer integration feels real rather than test-double driven | INT-01, INT-02 | Architectural realism is not fully machine-checkable | Review the fixture/example crate and verify it uses `build.rs`, `OUT_DIR`, generated inclusion, and a real engine dispatch |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 20s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-09
