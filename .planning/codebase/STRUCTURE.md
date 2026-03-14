# Codebase Structure

**Analysis Date:** 2026-03-13

## Directory Layout

```
herdingcats/
├── src/
│   └── lib.rs                  # Library root (empty skeleton)
├── examples/
│   ├── tictactoe.rs            # Tic-tac-toe example (empty skeleton)
│   └── backgammon.rs           # Backgammon example (empty skeleton)
├── Cargo.toml                  # Package manifest and dependencies
├── Cargo.lock                  # Locked dependency versions
├── rustfmt.toml                # Rust formatting configuration
├── ARCHITECTURE.md             # Design specification document
├── CHANGELOG.md                # Version history
├── CONTRIBUTING.md             # Contribution guidelines
├── LICENSE-MIT                 # MIT license
├── LICENSE-APACHE              # Apache 2.0 license
├── release-plz.yml             # Release automation config
└── .planning/
    └── codebase/               # Codebase analysis documents (this directory)
```

## Directory Purposes

**src/:**
- Purpose: Core library implementation
- Contains: Rust source files (.rs)
- Status: Skeleton - only `lib.rs` exists but is empty
- Future structure: Will contain modules for engine, behavior traits, diff/trace abstractions

**examples/:**
- Purpose: Demonstrative implementations showing library usage
- Contains: Complete game implementations using the library
- Status: Skeleton - files exist but are empty
- Expected content:
  - `tictactoe.rs`: Simple 2-player tic-tac-toe with minimal rule complexity
  - `backgammon.rs`: Complex 2-player backgammon with rich rule interactions

**.planning/codebase/:**
- Purpose: Architecture and structure analysis documents
- Contains: Markdown documents (ARCHITECTURE.md, STRUCTURE.md, etc.)
- Consumed by: GSD planning and execution commands
- Not committed: These are analysis tools, separate from source code

## Key File Locations

**Entry Points:**

- `src/lib.rs`: Main library entry point
  - Currently empty
  - Will re-export public traits, types, and engine API
  - User-facing API surface (Behavior trait, Input/Output types, engine constructor)

**Configuration:**

- `Cargo.toml`: Package metadata, dependencies, edition (2024), license
  - Minimal dependencies: only `proptest` in dev-dependencies
  - Designed for no external runtime dependencies
  - Targets game development and deterministic systems

- `rustfmt.toml`: Code formatting rules
  - Enforces consistent Rust style across the project

- `release-plz.yml`: Automated versioning and release configuration
  - Supports semantic versioning workflow

**Core Logic:**

- To be implemented in `src/` (specific module structure TBD):
  - Behavior trait definition
  - Engine orchestration (dispatch, undo, redo)
  - Working state management (copy-on-write semantics)
  - History and frame management
  - Outcome type definitions

**Testing:**

- Pattern: Inline `#[cfg(test)]` modules within implementation files (not separate test files)
- Framework: proptest for property-based testing (in dev-dependencies)
- Assertion library: Rust standard assertions
- Locations: To be defined as modules are created

**Examples:**

- `examples/tictactoe.rs`: User-defined types (State, Input, Diff, Trace, Behaviors)
  - Demonstrates minimal rule complexity
  - Shows how to compose behaviors
  - Shows state structure pattern

- `examples/backgammon.rs`: User-defined types with complex rules
  - Demonstrates interaction between multiple behaviors
  - Shows ordering importance in rule resolution
  - Shows CoW benefits in complex state management

## Naming Conventions

**Files:**

- Module files: `lowercase_with_underscores.rs` (Rust convention)
- Example files: `gamename.rs` (descriptive game name)
- Config files: UPPERCASE or `.config.name` format

**Directories:**

- Source modules: `src/`
- Compiled output: `target/` (Cargo default, not committed)
- Examples: `examples/`
- Documentation: Root level or `.planning/`

**Rust Code Elements:**

- Traits: PascalCase (e.g., `Behavior`)
- Enums: PascalCase (e.g., `Outcome`, `BehaviorResult`)
- Structs: PascalCase (e.g., `Frame`)
- Functions: snake_case (e.g., `dispatch`, `apply_diff`)
- Constants: UPPERCASE_SNAKE_CASE (e.g., `MAX_BEHAVIORS`)
- Type parameters: Single uppercase letters or PascalCase (e.g., `S`, `I`, `D`, `State`)
- Module names: lowercase_snake_case or descriptive nouns

## Where to Add New Code

**New Engine Feature:**
- Implementation: `src/engine.rs` (or appropriate module)
- Tests: `#[cfg(test)]` module within the feature file or `src/tests/` if tests grow large
- Public API exposure: Re-export from `src/lib.rs`

**New Behavior Trait or Extension:**
- Implementation: `src/behavior.rs` (or appropriate module)
- Examples: Demonstrate in `examples/tictactoe.rs` or `examples/backgammon.rs`

**New Example Implementation:**
- Main file: `examples/gamename.rs`
- Supporting module (if needed): `examples/gamename/` directory
- Must define: `State`, `Input`, `Diff`, `Trace`, behavior implementations
- Must demonstrate: Usage of dispatch, undo/redo, behavior composition

**Test Fixtures or Shared Test Utilities:**
- Location: `src/tests/fixtures.rs` or within `#[cfg(test)]` modules
- Pattern: Factory functions or builder patterns for constructing test states/inputs

**Documentation:**
- API documentation: Doc comments in source code (triple-slash `///`)
- Architecture guides: `.planning/codebase/*.md` files
- Examples/tutorials: In `examples/` with inline comments

## Special Directories

**target/:**
- Purpose: Cargo build output (compiled binaries, artifacts)
- Generated: Yes (by `cargo build`)
- Committed: No (.gitignore)

**.git/:**
- Purpose: Git version control metadata
- Generated: Yes (by `git init` and operations)
- Committed: No (directory itself)

**.github/:**
- Purpose: GitHub-specific configuration (CI/CD workflows)
- Generated: No (manually maintained)
- Committed: Yes

**.planning/:**
- Purpose: GSD system files and analysis documents
- Generated: Partially (analysis documents created by mapping commands)
- Committed: Yes (for team visibility)

**Ignored by .gitignore:**
- `target/`: Build artifacts
- `*.swp`, `*.swo`: Editor temp files
- `.DS_Store`: macOS metadata
- Anything not explicitly tracked

## Current Status and Next Steps

**Current State:**
- Skeleton structure in place: `src/lib.rs` is empty
- Examples exist but are empty: `tictactoe.rs`, `backgammon.rs`
- No implementation yet

**Implementation Order (Recommended):**
1. Define core types in `src/lib.rs`:
   - `Behavior` trait with generic type parameters
   - `BehaviorResult<D, O>` enum
   - `Outcome<F, N>` enum
   - `Frame<I, D, T>` struct

2. Implement engine core in `src/engine.rs`:
   - `Engine<S, I, D, T, O, B>` struct
   - `dispatch()` method
   - `undo()` and `redo()` methods
   - Copy-on-write working state construction

3. Create first example (`examples/tictactoe.rs`):
   - Define simple `GameState`, `Move`, `MoveDiff`, `GameTrace`
   - Implement 2-3 simple behaviors
   - Test dispatch, undo, redo

4. Create second example (`examples/backgammon.rs`):
   - Define complex `GameState` with substates
   - Implement multiple behaviors with ordering importance
   - Demonstrate CoW benefits

**Testing Strategy:**
- Property tests: Using proptest in `#[cfg(test)]` modules
- Unit tests: Inline within implementation modules
- Roundtrip tests: Verify dispatch + undo/redo returns to original state
- Ordering tests: Verify behavior execution order matters and is deterministic

---

*Structure analysis: 2026-03-13*
