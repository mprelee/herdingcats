# Technology Stack

**Analysis Date:** 2026-03-08

## Languages

**Primary:**
- Rust (Edition 2024) - All library and example code

## Runtime

**Environment:**
- Rust native binary / library crate
- No managed runtime

**Package Manager:**
- Cargo 1.93.1
- Lockfile: `Cargo.lock` present and committed

## Frameworks

**Core:**
- None - pure Rust standard library only (`std::collections::{HashMap, HashSet}`)

**Testing:**
- Rust built-in test harness (`#[test]`, `#[cfg(test)]`) - no external test framework detected

**Build/Dev:**
- Cargo (standard Rust build tool)
- No build scripts (`build.rs`) detected

## Key Dependencies

**Critical:**
- None - `[dependencies]` section in `Cargo.toml` is empty; zero external crates

**Standard Library Usage:**
- `std::collections::HashMap` - rule lifetime and enabled-state tracking
- `std::collections::HashSet` - enabled rule ID tracking
- `std::marker::PhantomData` - compile-time state type marker in `CommitFrame`

## Configuration

**Build:**
- `Cargo.toml` at project root
- Crate type: library (`src/lib.rs`) with examples (`examples/tictactoe.rs`)
- Edition 2024 (requires Rust 1.85+)
- Dual-licensed: MIT OR Apache-2.0

**Publish:**
- Published to crates.io as `herdingcats`
- Docs published to docs.rs: `https://docs.rs/herdingcats`
- Repository: `https://github.com/mprelee/herdingcats`

**Environment:**
- No environment variables required
- No `.env` files present

## Platform Requirements

**Development:**
- Rust toolchain 1.85+ (Edition 2024 requirement)
- Detected toolchain: rustc 1.93.1, cargo 1.93.1
- No `rust-toolchain.toml` pinning file present

**Production:**
- Compiled as a Rust library crate; consumers embed it as a dependency
- No server process, no deployment target

---

*Stack analysis: 2026-03-08*
