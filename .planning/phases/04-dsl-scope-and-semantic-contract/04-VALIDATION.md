---
phase: 4
slug: dsl-scope-and-semantic-contract
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-09
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | shell checks + `cargo test` regression guard |
| **Config file** | none |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-01-01 | 01 | 1 | DSL-01 | artifact | `test -f .planning/phases/04-dsl-scope-and-semantic-contract/04-01-PLAN.md` | ✅ | ⬜ pending |
| 4-01-02 | 01 | 1 | DSL-02 | content | `rg "DSL-02|priority|lifetime|rule id" .planning/phases/04-dsl-scope-and-semantic-contract/04-01-PLAN.md` | ✅ | ⬜ pending |
| 4-01-03 | 01 | 1 | DSL-03 | content | `rg "event match|guard|binding" .planning/phases/04-dsl-scope-and-semantic-contract/04-01-PLAN.md` | ✅ | ⬜ pending |
| 4-01-04 | 01 | 1 | DSL-04 | content | `rg "before\\(\\)|before\\(\\)-only|before\\(\\) semantics" .planning/phases/04-dsl-scope-and-semantic-contract/04-01-PLAN.md` | ✅ | ⬜ pending |
| 4-02-01 | 02 | 2 | GEN-01 | content | `rg "IR|intermediate representation|validation|lowering" .planning/phases/04-dsl-scope-and-semantic-contract/04-02-PLAN.md` | ✅ | ⬜ pending |
| 4-02-02 | 02 | 2 | DSL-03 | content | `rg "binding|state/event access|config" .planning/phases/04-dsl-scope-and-semantic-contract/04-02-PLAN.md` | ✅ | ⬜ pending |
| 4-02-03 | 02 | 2 | DSL-04 | content | `rg "reject|non-reversible|after\\(\\)" .planning/phases/04-dsl-scope-and-semantic-contract/04-02-PLAN.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] None — Phase 4 uses existing Cargo test infrastructure and planning artifacts only

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Authored examples feel narrow enough for v1.1 but expressive enough for intended rule overrides | DSL-01, DSL-03, DSL-04 | Scope quality is judgment-heavy | Read both plan files and confirm the examples stay additive, reversible, and `before()`-only |
| Binding contract is concrete enough to implement without guessing in Phase 5 | DSL-03, GEN-01 | Quality of abstraction is architectural, not fully machine-checkable | Review Plan 04-02 and verify it names allowed binding surfaces and rejection cases explicitly |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-09
