# Architecture Overview

This document defines the project architecture in a language-neutral way.
It explains principles, runtime behavior, and boundaries without tying decisions to any specific programming language or framework.

## 1. Goals
- Use DDD aggregate boundaries as the source of business invariants.
- Reuse the same command logic for both real execution and dry-run simulation.
- Support CQRS in practical systems where write-side decisions and complex queries can coexist.
- Use Specification as a shared query intent model across read and write sides.

## 2. Non-Goals
- Rebuilding a full in-memory SQL join engine or datastore query planner.
- Forcing an immediate redesign of all existing command flows to avoid complex command-time queries.

## 3. Core Principles

### 3.1 Unit of Work Is About More Than Transaction Rollback
Unit of Work is used to:
- Track change sets and snapshots.
- Build accurate dry-run diffs.
- Stage side effects for controlled dispatch.
- Provide a unified boundary for internal event processing.

### 3.2 Domain Events Should Be Business-Semantic
Use business-meaningful events, not generic CRUD events.
- Good: `ShippingInfoRefreshed`
- Weak: `EntityUpdated`

Business-semantic events improve readability, handler design, and dry-run side-effect control.

### 3.3 Command-Time Queries Must Be Write-Consistent
Command decisions must query write-consistent data sources.
Read models can be eventually consistent and are not authoritative for write-time decisions.

## 4. Runtime Modes

### 4.1 Real Run
- Persist writes to the primary write store.
- Commit transaction.
- Dispatch external side effects after commit (or via outbox).

### 4.2 Dry Run
- Do not mutate the primary write store.
- Execute internal deterministic/in-transaction reactions.
- Capture external side effects without executing them.
- Return structured diff and side-effect preview.

## 5. Components And Responsibilities
- **Command Bus**: command execution entrypoint and execution context setup.
- **Execution Context**: request/transaction scoped runtime mode and unit-of-work reference.
- **Unit of Work**: identity map, snapshots, event buffers, and diff builder.
- **Repository**: stateless adapter that attaches loaded entities to current unit-of-work.
- **Event Router**:
  - Internal/in-process events (allowed in dry-run).
  - External side-effect events (blocked in dry-run, captured only).
- **Query Engine**: executes Specifications against write store and dry-run overlays.
- **Dry-Run Result Builder**: entity/relation diffs, executed internal events, intercepted side effects.

## 5.1 Aggregate And Command Handler Contract
- Command handler (`handle`) validates business rules and returns produced events directly.
- The aggregate does not buffer or publish events itself.
- Application service receives events from `handle`, calls `apply` on each to update aggregate state, then passes events to Unit of Work for buffering.
- `apply` is a pure function: deterministic, no validation, no non-determinism (clocks, random IDs), no rejection of historical events. All non-deterministic values are fixed at command time and captured in event fields.
- Unit of Work holds the event buffer and orchestrates persistence and dispatch.
- The full flow is: `handle()` produces events -> application service calls `apply()` per event -> Unit of Work buffers events and snapshot -> Unit of Work persists and dispatches.

## 6. Event Processing Model

### 6.1 Phase A: Internal In-Transaction Events
- Executed until stable (no new internal events).
- Must be deterministic and free of irreversible external IO.
- All Phase A handlers share the command's database transaction.
- Handlers execute synchronously (no concurrency within Phase A).
- The event pattern is used for local service decoupling; publishing is effectively a synchronous in-process method call within the transaction boundary.
- Any handler error rolls back the entire transaction, including the originating command's writes.

### 6.2 Phase B: External Side Effects
- Real run: dispatch via transactional outbox. Outbox records are written inside the command transaction; a background worker delivers them asynchronously after commit. This guarantees at-least-once delivery without coupling commit success to external system availability.
- Dry run: do not dispatch; record planned side effects.

### 6.3 Persistence Model
- The system uses Event + Snapshot dual-write.
- Both the event stream and the current aggregate snapshot are persisted in the same database transaction.
- Events provide audit trail and time-travel capability (replay to any past state).
- Snapshots provide efficient read-after-write and query access on the write side.
- See `docs/adr/0002-event-sourcing-dual-write-persistence.md`.

## 7. Dry-Run Data Isolation
Use a virtual overlay model:
- **Primary**: canonical write store.
- **Staging Overlay**: transaction-scoped delta records for dry-run.
- Query resolution precedence: `overlay > primary`, with delete tombstone handling.

Only changed entities are staged. Unchanged data is read from primary.

## 8. Specification Model
- Specification defines query intent only.
- Executors/adapters translate intent into storage-specific queries.
- Read and write sides can share Specification semantics while using different executors.

## 9. Error Design Boundary
- Shared package errors should remain generic and reusable.
- Domain/application errors should carry context required for diagnostics and API responses.
- Context enrichment happens at domain/application boundaries.

## 10. Transaction Boundaries And Long-Running Processes
- Database transactions must be short: contain only DB-intensive operations.
- External I/O (HTTP calls, message broker sends, third-party APIs) must not occur inside a database transaction.
- Long-running business flows that span external waits must be modeled as explicit state machines with persisted intermediate states.
- Recovery from crashes or partial failures uses background sweepers that detect and compensate zombie intermediate states.
- Cross-service multi-step flows use Saga with compensation.
- See `docs/adr/0004-transaction-boundaries-long-running-processes.md`.

## 11. Event Bus Routing
- In-process event bus uses topic-based routing with guaranteed delivery (no data loss on slow consumers).
- Subscription granularity is per domain-event category (one topic per aggregate or bounded context), not per individual event type.
- Subscribers filter for specific event variants locally after receiving category-level messages.
- See `docs/adr/0003-in-memory-event-bus-routing.md` for implementation-specific choices.

## 12. Decision Records
Architecture decisions and trade-offs are recorded in ADRs:
- See `docs/adr/README.md`
- See `docs/adr/0001-ddd-dry-run-cqrs-specification.md`
- See `docs/adr/0002-event-sourcing-dual-write-persistence.md`
- See `docs/adr/0003-in-memory-event-bus-routing.md`
- See `docs/adr/0004-transaction-boundaries-long-running-processes.md`
