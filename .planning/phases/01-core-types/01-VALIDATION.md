---
phase: 1
slug: core-types
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-13
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — Cargo.toml `[dev-dependencies]` |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green + `cargo clippy -- -D warnings` clean
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | CORE-01 | unit (compile) | `cargo test --lib -- core::spec` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | CORE-02 | unit (compile) | `cargo test --lib -- core::behavior` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | CORE-03 | unit | `cargo test --lib -- core::behavior` | ❌ W0 | ⬜ pending |
| 01-01-04 | 01 | 1 | CORE-04 | unit | `cargo test --lib -- core::outcome` | ❌ W0 | ⬜ pending |
| 01-01-05 | 01 | 1 | CORE-05 | unit | `cargo test --lib -- core::outcome` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `#[cfg(test)]` module in `src/spec.rs` — compile test for CORE-01 (EngineSpec with all 6 associated types)
- [ ] `#[cfg(test)]` module in `src/behavior.rs` — compile + pattern match tests for CORE-02, CORE-03
- [ ] `#[cfg(test)]` module in `src/outcome.rs` — pattern match tests for CORE-04, CORE-05
- [ ] `src/lib.rs` doc-test with `compile_fail` annotation — mutation prevention test for CORE-02

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `#[non_exhaustive]` enforced for downstream users | CORE-05 | Requires a separate crate to observe the effect | Inspect `src/outcome.rs` for `#[non_exhaustive]` attribute on `EngineError` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
