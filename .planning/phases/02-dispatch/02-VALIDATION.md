---
phase: 2
slug: dispatch
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-13
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — `#[cfg(test)]` modules per source file (consistent with Phase 1) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --doc` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --doc`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-xx-01 | TBD | 1 | DISP-01 | unit | `cargo test cow_no_clone_on_no_op_dispatch` | ❌ W0 | ⬜ pending |
| 02-xx-02 | TBD | 1 | DISP-01 | unit | `cargo test cow_clones_on_first_diff` | ❌ W0 | ⬜ pending |
| 02-xx-03 | TBD | 1 | DISP-02 | unit | `cargo test dispatch_evaluates_in_deterministic_order` | ❌ W0 | ⬜ pending |
| 02-xx-04 | TBD | 1 | DISP-02 | unit | `cargo test later_behavior_sees_earlier_diffs` | ❌ W0 | ⬜ pending |
| 02-xx-05 | TBD | 1 | DISP-02 | unit | `cargo test trace_appended_at_diff_application` | ❌ W0 | ⬜ pending |
| 02-xx-06 | TBD | 1 | DISP-02 | unit | `cargo test stop_halts_dispatch` | ❌ W0 | ⬜ pending |
| 02-xx-07 | TBD | 1 | DISP-03 | unit | `cargo test no_frame_on_no_diffs` | ❌ W0 | ⬜ pending |
| 02-xx-08 | TBD | 1 | DISP-03 | unit | `cargo test frame_contains_input_diffs_trace` | ❌ W0 | ⬜ pending |
| 02-xx-09 | TBD | 1 | DISP-04 | compile | `cargo test` (compile error if Reversibility omitted) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/apply.rs` — `apply_trait_compiles_and_returns_traces` test
- [ ] `src/reversibility.rs` — `reversibility_is_copy_and_eq` test
- [ ] `src/engine.rs` — all DISP-01 through DISP-04 tests listed above
- [ ] Updated `src/spec.rs` — verify `Apply<Self>` bound compiles (TestSpec Diff must impl Apply)
- [ ] Updated `src/outcome.rs` — if `Frame.diff`/`trace` changed to `Vec<_>`, update existing tests accordingly

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Compiler rejects `dispatch()` call missing `Reversibility` arg | DISP-04 | Verified structurally by the type system — no runtime test needed | Inspect `dispatch()` signature has `reversibility: Reversibility` param |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
