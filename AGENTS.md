# AGENTS.md

## Scope

- This file defines project-level constraints for all AI/code agents in this repository.
- Rules here are normative. Explanations and long-form rationale belong in `docs/`.

## Source Of Truth

- Domain and implementation constraints: this file (`AGENTS.md`).
- Architecture overview: `docs/architecture/README.md`.
- Architecture decisions and trade-offs: `docs/adr/`.

## Standard Working Rules

- Keep changes small, reviewable, and reversible.
- Prefer explicit domain behavior over framework magic.
- Do not introduce language- or framework-specific assumptions
  into architecture docs.
- Do not encode product/business policy in shared utility packages.
- Add or update docs when behavior or constraints change.

## Architecture Constraints (Project Rules)

- Use DDD boundaries: domain rules live in aggregates/value objects,
  not in infrastructure layers.
- Commands must support both real execution and dry-run simulation paths.
- Dry-run must not execute irreversible side effects (email, HTTP, message brokers).
- Internal in-process domain reactions may run in dry-run
  when they are pure/in-transaction effects.
- Command-time decision queries must read from the
  write-consistent model, not eventually-consistent read models.
- CQRS is allowed with different read/write executors, but shared
  query semantics should be expressed via Specification.
- Specification defines query intent; storage-specific translation belongs to executors/adapters.
- `apply` (event replay) must be a pure function: no business
  validation, no non-determinism, no event rejection.
- Command handlers return events directly; application service
  orchestrates persistence and dispatch.
- Events and snapshots are persisted in the same database
  transaction (dual-write).
- Phase A (in-process) event handlers share the command's
  transaction, execute synchronously, and must not perform
  external I/O.
- Long-held database transactions are prohibited; external I/O
  must occur outside transaction boundaries.

## Error Design Constraints

- Keep `shared` errors generic and reusable across projects.
- Keep project/domain errors contextual and rich
  (entity type, field, offending value, etc.).
- Map generic errors to contextual domain/application errors at boundaries.
- API-facing error payloads should be machine-readable and
  standards-aligned (Problem Details / JSON:API style).

## Documentation Rules

- Put stable architectural principles in `docs/architecture/README.md`.
- Put significant decisions in ADRs under `docs/adr/` (one decision per file).
- ADR files use numbered names: `NNNN-short-title.md`.
- New architectural constraints require: (1) ADR,
  (2) architecture doc update, (3) this file update if enforceable.

## Testing And Verification

- Validate behavior changes with focused tests nearest to changed logic.
- For dry-run features, verify both: side effects blocked and diff/report generated.
- For CQRS-related changes, verify write-side correctness
  before read-model performance concerns.
