# External Integrations

**Analysis Date:** 2026-03-13

## APIs & External Services

**None detected** - This is a library with no external API dependencies or service integrations.

## Data Storage

**Databases:**
- Not applicable - Library has no persistent data storage

**File Storage:**
- Local filesystem only - Used by test infrastructure (tempfile) for temporary test files

**Caching:**
- Not applicable

## Authentication & Identity

**Auth Provider:**
- Not applicable - This is a library with no authentication system

## Monitoring & Observability

**Error Tracking:**
- None detected

**Logs:**
- Standard Rust logging available via log crate 0.4.29 (transitive dependency)
- No specific logging framework configured

## CI/CD & Deployment

**Hosting:**
- crates.io (Rust package registry)
- GitHub repository hosting

**CI Pipeline:**
- GitHub Actions via `.github/workflows/`

**CI Jobs:**
1. **Rust CI** (`rust.yml`):
   - Runs on: Ubuntu-latest
   - Steps:
     - Checkout code (actions/checkout@v4)
     - Install Rust toolchain (dtolnay/rust-toolchain@stable)
     - Cache dependencies (Swatinem/rust-cache@v2)
     - Check code formatting (cargo fmt --check)
     - Run Clippy linting (cargo clippy --all-targets --all-features)
     - Execute tests (cargo test --all-features)
   - Triggered on: Push to main, Pull requests

2. **Release** (`release-plz.yml`):
   - Runs on: Ubuntu-latest
   - Automated release workflow using release-plz
   - Steps:
     - Checkout code (actions/checkout@v4)
     - Install Rust toolchain (dtolnay/rust-toolchain@stable)
     - Release plz action (release-plz/action@v0.5)
   - Environment variables:
     - GITHUB_TOKEN - For GitHub API access (secret)
     - CARGO_REGISTRY_TOKEN - For crates.io publishing (secret)
   - Triggered on: Push to main

**Release Configuration:**
- `release-plz.yml` - Changelog generation enabled

## Environment Configuration

**Required Env Vars:**
- None for normal usage
- CI/CD requires secrets:
  - `RELEASE_PLZ_TOKEN` - GitHub token for release PRs
  - `CARGO_REGISTRY_TOKEN` - Crates.io token for publishing

**Secrets Location:**
- GitHub repository secrets (configured in repository settings)

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- GitHub Actions publishes to crates.io registry on release
- release-plz may create pull requests for version updates

---

*Integration audit: 2026-03-13*
