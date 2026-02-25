# ADR-0001: DDD + Dry-Run + CQRS + Specification

- Status: Accepted
- Date: 2026-02-06

## Context
The system needs to:
- Keep business rules in aggregate boundaries (DDD).
- Execute the same command logic in real-run and dry-run modes.
- Prevent irreversible side effects in dry-run while still computing in-transaction reactions.
- Support CQRS where read models may be delayed (eventual consistency).
- Avoid duplicating query semantics across read and write implementations.
- Support practical command flows that may include "write-then-query" patterns.

## Decision
1. **Domain model boundary**
   - Aggregate roots and value objects are the source of business invariants.
   - Infrastructure and persistence details must not own business rules.

2. **Execution modes**
   - All command handlers support two modes:
     - Real run: persist changes and dispatch side effects after commit.
     - Dry run: do not persist to primary store and do not execute external side effects.
   - Dry run still executes internal deterministic/in-transaction reactions.

3. **Unit of Work as execution boundary**
   - Unit of Work tracks identity, snapshots, and change sets.
   - Unit of Work computes structured diff output for dry-run.
   - Unit of Work separates internal event processing from external side effects.

4. **Two-phase event handling**
   - Phase A: internal in-process events run until stable.
   - Phase B: external side effects run only after commit in real run.
   - In dry run, external side effects are captured as preview output, not executed.

5. **CQRS consistency rule**
   - Command-time decision queries must use write-consistent sources.
   - Read models are optimized for query performance and may be eventually consistent.

6. **Specification strategy**
   - Specification expresses query intent only.
   - Executors/adapters perform storage-specific translation.
   - Read and write sides may use different executors while sharing Specification semantics.

7. **Dry-run data strategy**
   - Use an overlay model (`staging delta + primary data`) for dry-run queries.
   - Query precedence is overlay over primary, including delete/tombstone behavior.
   - Only changed entities are staged.

8. **Error layering**
   - Shared errors remain generic and reusable.
   - Domain/application layers enrich errors with contextual details for diagnostics and API responses.

## Consequences
### Positive
- One command flow supports both execution modes.
- Dry-run previews are accurate and explainable.
- Side effects are safer and easier to audit.
- CQRS consistency pitfalls are reduced in write paths.
- Query intent is reusable even with different storage backends.

### Negative
- Additional infrastructure complexity (unit of work, event routing, overlay query behavior).
- Stricter discipline required around side-effect classification.
- More test surface (mode-specific behavior and event phase behavior).

## Alternatives Considered
1. **Generic CRUD events only**
   - Rejected: weak business semantics and unclear side-effect routing.

2. **Independent dry-run code path**
   - Rejected: high drift risk and duplicated logic.

3. **Read-model queries for command decisions**
   - Rejected: stale data risk under eventual consistency.

4. **Separate Specification definitions per side**
   - Rejected: semantic duplication and long-term maintenance cost.

## References
- Domain events and DDD guidance:
  - https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/domain-events-design-implementation
- CQRS pattern:
  - https://learn.microsoft.com/en-us/azure/architecture/patterns/cqrs
- Unit of Work concept:
  - https://en.wikipedia.org/wiki/Unit_of_work
- Transactional outbox:
  - https://microservices.io/patterns/data/transactional-outbox.html
  - https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-outbox-cosmos
- Error response standards:
  - https://www.rfc-editor.org/rfc/rfc9457.html
  - https://jsonapi.org/format/#errors
