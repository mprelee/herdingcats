# Technology Stack

**Analysis Date:** 2026-03-13

## Languages

**Primary:**
- Rust (Edition 2024) - Core library implementation

## Runtime

**Environment:**
- Rust Compiler (1.93.1+)

**Package Manager:**
- Cargo (Rust's built-in package manager)
- Lockfile: `Cargo.lock` (present)

## Frameworks

**Core:**
- None - This is a lightweight, zero-dependency library

**Testing:**
- proptest 1.10 - Property-based testing framework for dev dependencies

**Build/Dev:**
- Cargo (build system)
- rustfmt - Code formatting (configured in `rustfmt.toml`)
- Clippy - Linting (via CI/CD)

## Key Dependencies

**Testing Dependencies:**
- proptest 1.10.0 - Property-based testing generating random test cases

**Transitive Dependencies (from proptest):**
- rand 0.9.2 - Random number generation
- regex-syntax 0.8.10 - Regex parsing
- semver 1.0.27 - Version parsing
- serde 1.0.228 - Serialization framework
- serde_json 1.0.149 - JSON serialization
- tempfile 3.26.0 - Temporary file handling
- rusty-fork 0.3.1 - Process forking for test isolation

**Note:** The library itself has zero runtime dependencies. The dependency tree exists only for testing via proptest's property testing infrastructure.

## Configuration

**Environment:**
- No environment variables required
- No .env or configuration files

**Build:**
- `rustfmt.toml` - Code formatter configuration (edition = "2024")
- `Cargo.toml` - Package manifest with metadata
- `Cargo.lock` - Locked dependency versions

## Platform Requirements

**Development:**
- Rust toolchain (1.93.1 or later recommended)
- Cargo package manager
- Unix/Linux, macOS, or Windows compatible

**Production:**
- None - This is a library, not a standalone application
- Can be compiled to library target (`*.rlib`) for use in other Rust projects
- Published to crates.io as downloadable package

## Project Metadata

**License:**
- MIT OR Apache-2.0 (dual licensing)

**Distribution:**
- crates.io registry
- GitHub repository: https://github.com/mprelee/herdingcats
- Documentation: https://docs.rs/herdingcats

**Keywords:**
- game, deterministic, rules, engine, turn-based

---

*Stack analysis: 2026-03-13*
