# External Integrations

**Analysis Date:** 2026-03-08

## APIs & External Services

None. This codebase has zero external dependencies and makes no network calls. It is a pure Rust library crate with no FFI, HTTP clients, or SDK integrations.

## Data Storage

**Databases:** Not applicable - no database integration.

**File Storage:** Not applicable - no file I/O.

**Caching:** Not applicable.

## Authentication & Identity

Not applicable - this is a library crate with no user-facing authentication surface.

## Monitoring & Observability

**Error Tracking:** None.

**Logs:** None - the library uses no logging framework. The example binary (`examples/tictactoe.rs`) uses `println!` for stdout output only.

## CI/CD & Deployment

**Hosting:** Published to crates.io (`herdingcats`). Docs auto-published to docs.rs.

**CI Pipeline:** No CI configuration files detected (no `.github/workflows/`, no `Makefile`, no `justfile`).

## Environment Configuration

**Required env vars:** None.

**Secrets location:** Not applicable.

## Webhooks & Callbacks

**Incoming:** None.

**Outgoing:** None.

## Integration Summary

`herdingcats` is a zero-dependency pure Rust library. All integration surface exists at the consumer's application layer. Consumers implement the `Operation<S>` and `Rule<S, O, E, P>` traits and wire their own state types. The library itself has no opinion on networking, persistence, or external services.

---

*Integration audit: 2026-03-08*
