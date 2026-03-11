---
phase: quick-4
plan: "01"
subsystem: documentation
tags: [readme, api-docs, terminology]
dependency_graph:
  requires: [quick-2, quick-3]
  provides: [accurate-api-docs]
  affects: [README.md]
tech_stack:
  added: []
  patterns: []
key_files:
  created: []
  modified:
    - README.md
decisions:
  - Replaced 'Transactional state mutation' bullet with 'Atomic state mutation via Action<M>' to remove old Transaction terminology while preserving the concept
metrics:
  duration: "45s"
  completed_date: "2026-03-11"
  tasks_completed: 1
  files_changed: 1
---

# Quick Task 4: Update README.md to Reflect Current API — Summary

**One-liner:** Rewrote README Core Model, feature list, and dispatch API sections to match Engine<S,M,I,P> naming, Action<M> fields, and the dispatch/dispatch_with/dispatch_preview split introduced in quick tasks 2-3.

## What Was Done

The README was written against the pre-v1.1 API and referenced removed concepts:
- Type parameters used `O` (Operation) and `E` (event) instead of `M` (Mutation) and `I` (input)
- Feature list included "Rule lifetimes (per-turn / per-trigger)" — removed in v1.1
- No documentation of the `dispatch` / `dispatch_with` / `dispatch_preview` API surface or their return types
- Terminology used "Operation", "Transactional", "rule" in the technical type-concept sense

## Changes Made

**Core Model section:**
- Updated type parameter table: `O → M` (mutation type, `Mutation<S>`), `E → I` (input / event enum)
- Updated `P` description: `Copy + Ord` rather than `#[repr(i32)]`, sealed
- Changed "State mutation occurs exclusively through `Operation`" to "through `Mutation`"
- Changed "All irreversible transactions" bullet list to "All mutations" (removing the Transaction framing)

**Feature list (Overview):**
- Removed "Rule lifetimes (per-turn / per-trigger)" — feature no longer exists
- Replaced "rule execution" with "behavior execution"
- Replaced "Transactional state mutation" with "Atomic state mutation via `Action<M>`"

**New Dispatch API section (added between Core Model and Determinism Guarantees):**
- Documents `dispatch(event) -> Option<Action<M>>` — simple path via behavior hooks
- Documents `dispatch_with(event, tx) -> Option<Action<M>>` — pre-built action path
- Documents `dispatch_preview(event, tx) -> Action<M>` — dry-run for AI look-ahead and UI preview
- Presented as a concise markdown table

**Determinism Guarantees section:**
- Changed "No unordered rule execution" to "No unordered behavior execution"

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] "Transactional state mutation" used old Transaction framing**
- **Found during:** Task 1 verification
- **Issue:** The feature bullet "Transactional state mutation" would have matched the old terminology scan and carried confusing connotations from the removed Transaction type concept
- **Fix:** Replaced with "Atomic state mutation via `Action<M>`" — preserves the correctness guarantee while using current terminology
- **Files modified:** README.md
- **Commit:** 0f0a7bd

## Verification Results

- `grep -n "Operation\|Rule lifetime\|RuleLifetime\|Transaction" README.md` — no matches
- `grep -n "dispatch_with\|dispatch_preview\|Option<Action" README.md` — 3 matches (table rows for each method)
- `cargo test --doc` — 22 passed, 0 failed

## Self-Check: PASSED

- README.md modified: FOUND
- Commit 0f0a7bd: FOUND
