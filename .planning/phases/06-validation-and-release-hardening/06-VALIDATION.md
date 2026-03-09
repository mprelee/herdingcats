---
phase: 6
slug: validation-and-release-hardening
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-09
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` + example run smoke checks via Cargo |
| **Config file** | none |
| **Quick run command** | `cargo test --test phase6_validation` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test phase6_validation`
- **After every plan wave:** Run `cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 6-01-01 | 01 | 1 | GEN-04 | integration | `cargo test --test phase6_validation generated_op` | ✅ | ⬜ pending |
| 6-01-02 | 01 | 1 | INT-04 | property | `cargo test --test phase6_validation generated_rule` | ✅ | ⬜ pending |
| 6-01-03 | 01 | 1 | INT-04 | roundtrip | `cargo test --test phase6_validation generated_roundtrip` | ✅ | ⬜ pending |
| 6-01-04 | 01 | 1 | INT-03 | integration | `cargo test diagnostics -- --nocapture` | ✅ | ⬜ pending |
| 6-02-01 | 02 | 2 | INT-05 | compatibility | `cargo test --lib --examples` | ✅ | ⬜ pending |
| 6-02-02 | 02 | 2 | INT-05 | runtime smoke | `cargo run --example tictactoe && cargo run --example backgammon` | ✅ | ⬜ pending |
| 6-02-03 | 02 | 2 | INT-05 | docs + boundary | `rg "build-time|runtime scripting|after\\(\\)|handwritten" README.md src/lib.rs docs/dsl/README.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] None — Phase 6 can reuse the existing Cargo test/example infrastructure and the Phase 5 consumer fixture

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Diagnostic text feels actionable to downstream users, not just technically correct | INT-03 | Usefulness and clarity require human judgment | Run one failing consumer fixture and confirm stderr identifies the file/rule, the invalid construct, and a safe alternative |
| Release docs make the v1.1 boundary obvious to handwritten-only users | INT-05 | Architectural messaging quality is not fully machine-checkable | Review README and crate docs after execution and confirm they explicitly say build-time only, no runtime parser, and handwritten usage remains unchanged |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-09
