---
phase: 1
slug: module-split-and-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-08
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`#[test]`, `#[cfg(test)]`) + proptest 1.10 |
| **Config file** | `Cargo.toml` — `[dev-dependencies]` section (Wave 0 adds proptest) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --examples && cargo doc --no-deps` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --examples && cargo doc --no-deps`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| hash module | 01 | 1 | MOD-01 | compile | `cargo build` | ❌ W0 | ⬜ pending |
| operation module | 01 | 1 | MOD-01 | compile | `cargo build` | ❌ W0 | ⬜ pending |
| transaction module | 01 | 1 | MOD-01 | compile | `cargo build` | ❌ W0 | ⬜ pending |
| rule module | 01 | 1 | MOD-01 | compile | `cargo build` | ❌ W0 | ⬜ pending |
| engine module | 01 | 1 | MOD-01 | compile | `cargo build` | ❌ W0 | ⬜ pending |
| lib.rs facade | 01 | 1 | MOD-02 | compile + examples | `cargo test --examples` | ❌ W0 | ⬜ pending |
| tictactoe unchanged | 01 | 1 | MOD-03 | integration | `cargo run --example tictactoe` | ✅ | ⬜ pending |
| hash tests | 02 | 1 | TEST-01, TEST-04 | unit | `cargo test hash` | ❌ W0 | ⬜ pending |
| operation tests | 02 | 1 | TEST-01, TEST-03 | unit | `cargo test operation` | ❌ W0 | ⬜ pending |
| transaction tests | 02 | 1 | TEST-01 | unit | `cargo test transaction` | ❌ W0 | ⬜ pending |
| rule tests | 02 | 1 | TEST-01 | unit | `cargo test rule` | ❌ W0 | ⬜ pending |
| engine tests | 02 | 1 | TEST-01, TEST-03 | unit | `cargo test engine` | ❌ W0 | ⬜ pending |
| proptest dep | 02 | 0 | TEST-02 | compile | `cargo test` | ❌ W0 | ⬜ pending |
| rustdoc pub items | 03 | 2 | DOC-01, DOC-03 | doc | `cargo doc --no-deps 2>&1 \| grep -c warning` | ❌ W0 | ⬜ pending |
| internal item docs | 03 | 2 | DOC-02 | manual | review source comments | ✅ | ⬜ pending |
| trait paradigm docs | 03 | 2 | DOC-04 | manual + doc | `cargo doc --no-deps` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `proptest = "1.10"` added to `[dev-dependencies]` in `Cargo.toml`
- [ ] `src/hash.rs` stub created (even empty compiles)
- [ ] `src/operation.rs` stub created
- [ ] `src/transaction.rs` stub created
- [ ] `src/rule.rs` stub created
- [ ] `src/engine.rs` stub created
- [ ] `src/lib.rs` updated with `mod` declarations and `pub use` re-exports

*Wave 0 establishes the compilation baseline — all later waves depend on this.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Internal doc quality | DOC-02 | Subjective: explains mechanism + why | Read `CommitFrame` and `fnv1a_hash` source comments; verify they explain mechanism, not just label |
| Paradigm teaching quality | DOC-04 | Subjective: Operation/Rule trait prose teaches the model | Read `Operation` and `Rule` trait docs; verify a Rust dev unfamiliar with this engine would understand the whole model from these two blocks alone |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
