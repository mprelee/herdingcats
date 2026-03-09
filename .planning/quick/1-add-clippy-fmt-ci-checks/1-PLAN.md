---
phase: quick-1-add-clippy-fmt-ci-checks
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - rustfmt.toml
  - .github/workflows/rust.yml
autonomous: true
requirements:
  - CI-FMT-01
  - CI-CLIPPY-01

must_haves:
  truths:
    - "cargo fmt --check passes on CI without failures"
    - "cargo clippy passes on CI with -D warnings (no warnings treated as errors)"
    - "rustfmt edition is pinned to match Cargo.toml edition so fmt behavior is reproducible"
  artifacts:
    - path: "rustfmt.toml"
      provides: "Pinned rustfmt configuration (edition = 2024)"
    - path: ".github/workflows/rust.yml"
      provides: "CI workflow with fmt + clippy steps"
  key_links:
    - from: "rustfmt.toml"
      to: "cargo fmt"
      via: "rustfmt picks up config from project root automatically"
    - from: ".github/workflows/rust.yml"
      to: "cargo clippy --all-targets --all-features -- -D warnings"
      via: "CI step runs on every push/PR"
---

<objective>
Ensure cargo fmt and cargo clippy are enforced in CI on every push and pull request.

Purpose: Prevent style drift and catch lint issues before they accumulate. The CI workflow already has both checks in place; this plan pins the rustfmt edition to match the project's Cargo.toml edition (2024) and verifies the full workflow runs cleanly.
Output: rustfmt.toml with edition pinned, CI workflow confirmed working.
</objective>

<execution_context>
@/Users/mprelee/.claude/get-shit-done/workflows/execute-plan.md
@/Users/mprelee/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

Current CI file: .github/workflows/rust.yml
Already contains: cargo fmt -- --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test
Cargo.toml edition: 2024
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add rustfmt.toml to pin edition</name>
  <files>rustfmt.toml</files>
  <action>
    Create rustfmt.toml in the project root with the following content:

    ```toml
    edition = "2024"
    ```

    This pins the rustfmt edition to match `Cargo.toml`'s `edition = "2024"`, ensuring consistent formatting behavior regardless of the installed rustfmt version's default. No other settings needed — keep it minimal.
  </action>
  <verify>
    <automated>cargo fmt -- --check</automated>
  </verify>
  <done>rustfmt.toml exists with `edition = "2024"`, and `cargo fmt -- --check` exits 0 (no formatting changes needed).</done>
</task>

<task type="auto">
  <name>Task 2: Verify CI workflow is complete and passing locally</name>
  <files>.github/workflows/rust.yml</files>
  <action>
    The existing rust.yml already contains fmt, clippy, and test steps. Verify the current workflow content is correct and all three checks pass locally:

    1. Run `cargo fmt -- --check` — must exit 0.
    2. Run `cargo clippy --all-targets --all-features -- -D warnings` — must exit 0 with no warnings.
    3. Run `cargo test --all-features` — must exit 0 with all tests passing.

    If rust.yml is missing `--all-features` on the fmt check, add it for consistency. No other changes needed — the workflow is already correctly structured with `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2`.

    The workflow triggers on `push` to `main` and all `pull_request` events — this is correct, no change needed.
  </action>
  <verify>
    <automated>cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features</automated>
  </verify>
  <done>All three CI commands exit 0 locally. rust.yml is confirmed correct and unchanged (or has minor consistency fix if needed).</done>
</task>

</tasks>

<verification>
Run the full CI command sequence locally to confirm everything passes:

```
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features
```

All three must exit 0.
</verification>

<success_criteria>
- rustfmt.toml exists at project root with `edition = "2024"`
- `cargo fmt -- --check` exits 0
- `cargo clippy --all-targets --all-features -- -D warnings` exits 0
- `cargo test --all-features` exits 0
- .github/workflows/rust.yml enforces all three checks on every push and PR
</success_criteria>

<output>
After completion, create `.planning/quick/1-add-clippy-fmt-ci-checks/1-SUMMARY.md`
</output>
