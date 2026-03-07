# ADR-0004: Transaction Boundaries and Long-Running Processes

- Status: Accepted
- Date: 2026-02-27

## Context

The system must handle several classes of transactional consistency challenge:

- In-process event handlers that react to domain events within the same service.
- Handlers that need to trigger external I/O (HTTP calls, message broker sends).
- Cross-service multi-step business flows where partial failure must be recoverable.
- Long-running business flows that include external waits (e.g. a 30-second third-party API call).

A naive approach — holding a database transaction open across all of these — leads to lock contention, connection pool exhaustion, and MVCC bloat. Long-held transactions are one of the most dangerous architectural anti-patterns in high-concurrency systems.

## Decision

### 1. In-process handlers (Phase A): Transactional Event Bus

All Phase A (internal, in-process) event handlers share the command's database transaction.

- Handlers execute **synchronously**, one at a time (no concurrency within Phase A).
- Any handler returning an error rolls back the **entire** transaction, including the originating command's writes.
- Handlers must not perform external I/O or irreversible side effects.
- The event-based dispatch is used for **local service decoupling**: one service does not need to take a compile-time dependency on another. Publishing an event is effectively a synchronous in-process method call within the transaction boundary.

### 2. External-I/O handlers (Phase B): Transactional Outbox

Handlers that trigger external I/O use the transactional outbox pattern.

- Outbox records (describing the intended external action) are written inside the command's database transaction alongside events and snapshots.
- After commit, a background worker reads outbox records and delivers them to external systems.
- This guarantees **at-least-once delivery** without coupling transaction commit success to external system availability.
- External systems must be designed to handle duplicate deliveries (idempotent receivers).

### 3. Cross-service multi-step flows: Saga with Compensation

When a business flow spans multiple services or databases:

- Each step is an independent short transaction.
- On failure, the orchestrator publishes compensating (reverse) events to undo previously committed steps.
- Compensation is logical (e.g. issue a refund), not a true database rollback.

### 4. Long-running flows: State Machine Splitting with Sweeper

When a business flow includes a long external wait (e.g. a 30-second API call):

- Split the flow into independent short transactions separated by a persisted intermediate state (e.g. `Processing`, `AwaitingExternalResponse`).
- The first transaction commits the work done so far and records the intermediate state.
- After the external call completes, a second transaction picks up from the intermediate state and completes the flow.
- A background **sweeper** periodically scans for zombie intermediate states (e.g. stuck in `Processing` beyond a timeout) and triggers compensation or retry.
- The domain model must explicitly model these intermediate states as first-class lifecycle phases.

### Key constraint

Long-held database transactions are prohibited. External I/O must never occur inside a database transaction boundary. The engineering discipline is to identify which operations belong inside a short transaction (DB-intensive) and which must be outside (I/O-intensive), then design appropriate failure recovery for each boundary.

## Consequences

### Positive

- Database transactions remain short, reducing lock contention and connection pressure.
- Failure boundaries are explicit and recoverable.
- Phase A handlers get full transactional atomicity with the originating command.
- Phase B handlers get at-least-once delivery guarantees via outbox.
- Long-running flows are crash-recoverable via persisted intermediate states and sweeper.

### Negative

- Intermediate states must be explicitly modeled in the domain, increasing domain model surface area.
- Compensation logic adds code complexity and must be tested independently.
- Outbox delivery introduces a latency window (eventual consistency for external systems).
- Saga compensation is logical, not physical — some business operations cannot be perfectly reversed.

## Alternatives Considered

1. **Hold transaction open across external calls**

- Rejected: causes lock contention, connection pool exhaustion, and MVCC bloat under concurrency. Databases do not support "suspending" a transaction while waiting for external I/O without holding resources.

2. **Fire-and-forget event publishing after commit**

- Rejected: if the process crashes between commit and publish, events are lost permanently. No delivery guarantee.

3. **Two-phase commit (2PC) / distributed transactions**

- Rejected: not applicable. The system is a single-process, single-database application. 2PC adds complexity without benefit in this context, and is fragile in practice.

## References

- Transactional outbox pattern:
  - [https://microservices.io/patterns/data/transactional-outbox.html](https://microservices.io/patterns/data/transactional-outbox.html)
  - [https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-outbox-cosmos](https://learn.microsoft.com/en-us/azure/architecture/databases/guide/transactional-outbox-cosmos)
- Saga pattern:
  - [https://microservices.io/patterns/data/saga.html](https://microservices.io/patterns/data/saga.html)
  - [https://learn.microsoft.com/en-us/azure/architecture/reference-architectures/saga/saga](https://learn.microsoft.com/en-us/azure/architecture/reference-architectures/saga/saga)
- Long-running processes and process managers:
  - [https://www.enterpriseintegrationpatterns.com/patterns/messaging/ProcessManager.html](https://www.enterpriseintegrationpatterns.com/patterns/messaging/ProcessManager.html)
