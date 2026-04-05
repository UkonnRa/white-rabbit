# Auth Isolation

## Problem

White Rabbit runs in two modes:

- **Local** (Tauri): single-user, no authentication, no permission checks.
- **Remote** (web): multi-user, full AuthN + AuthZ at the Journal level.

The domain layer (`crates/domain`) must be identical in both modes.
It cannot contain any auth checks, user references, or role logic.
At the same time, the remote endpoint must enforce permissions before
executing domain commands.

## Constraint

`crates/domain` must never import `crates/shared-auth`.
This is a hard architectural rule — enforced by keeping `shared-auth` out of
every `domain` `Cargo.toml` dependency list.

## Current state

`crates/shared-auth` exists and defines three primitives:

- `AuthEntity` — a trait for entities that record `created_by_id` and
  `last_modified_by_id`. Not yet used by `crates/domain`.
- `Permission` — a two-level enum (`ReadOnly`, `ReadWrite`).
- `AuthReadService` — a trait for resolving a caller's permission for a
  given entity.

These are not yet wired to the domain or any endpoint.

## Options

The core design challenge: the remote endpoint needs to check whether a
caller has the right to execute a command before passing it to the domain
service.

### Option A — Endpoint-level guard (current leaning)

The remote endpoint resolves auth **before** calling the domain service.
The domain service is called only if the guard passes.
Domain commands carry no identity fields.

```text
HTTP request
  → auth middleware resolves caller identity
  → permission guard checks Journal role
  → if allowed: call domain service with plain command
  → if denied: return 403 before domain is touched
```

Pros: domain stays completely clean; easy to test domain in isolation.
Cons: the guard needs to know which Journal a command targets, which may
require a pre-flight read.

### Option B — Operator context in the session

The session/transaction object passed to the domain service carries an
optional `OperatorId`. The domain ignores it; the endpoint or a middleware
layer logs or audits it after the fact.

Pros: audit trail attached to the transaction naturally.
Cons: bleeds identity awareness into the session abstraction.

### Option C — Decorator / proxy service

Wrap the domain service with an auth-aware proxy that checks permission
then delegates to the real service.

Pros: clean separation; domain untouched.
Cons: one extra layer per aggregate service.

## Decision

Not yet made. Option A is the current leaning because it keeps the domain
and session types completely free of identity concepts.

When this is decided, record it as `docs/adr/0007-auth-isolation-strategy.md`
and update `crates/shared-auth` accordingly.

## What must NOT happen

- `crates/domain` structs or commands must never contain `user_id`,
  `operator_id`, `role`, or any auth-related field.
- `crates/domain` services must never call `AuthReadService` or any
  permission check.
- `crates/shared-auth` must not be listed in any `crates/domain/Cargo.toml`
  dependency.
