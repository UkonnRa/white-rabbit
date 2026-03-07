# ADR-0002: Event Sourcing with Snapshot Dual-Write Persistence

- Status: Accepted
- Date: 2026-02-27

## Context

The system needs:

- Audit trail and time-travel capability (replay past states from the event stream).
- Efficient read-after-write on the write side (command handler writes, then immediately queries).
- A clear contract for how aggregates produce, apply, and persist domain events.
- A persistence strategy that balances operational simplicity with long-term traceability.

Pure Event Sourcing (store only events) provides perfect time-travel but makes read-after-write expensive and operationally complex.
Pure Snapshot persistence (store only current state) is simple but loses all historical auditability.

## Decision

### 1. Event + Snapshot dual-write in a single local transaction

Both the event stream and the current aggregate snapshot are persisted in the **same database transaction**.

- Events are the authoritative record of what happened.
- Snapshots are a materialized convenience for efficient reads and write-consistency queries.
- Because this is a single-process, single-database system, no distributed transaction is needed.

### 2. `apply` is a strictly pure function

The `apply` function (event replay / state fold) must be:

- **Deterministic**: given the same state and event, it always produces the same next state.
- **Validation-free**: it must not reject events or enforce business rules. All validation happens at command time, before events are produced.
- **Free of non-determinism**: it must not read clocks, generate UUIDs, or access external state. All non-deterministic values are fixed at command time and captured in event fields.
- **Total over historical events**: it must accept any event that was ever persisted, including events produced by older versions of the domain model.

### 3. Command handler returns events directly

`handle()` returns produced events (e.g. `Vec<Event>`) directly to the caller.

- The aggregate does not buffer or publish events internally.
- The application service receives events from `handle()`, calls `apply()` on each to update aggregate state, then passes events to the Unit of Work for buffering.
- The Unit of Work orchestrates: persist events + snapshot, dispatch Phase A handlers, persist outbox records, commit.

### 4. Prefer flat struct with status-field enum for aggregate state

- Full enum-encoded state machines (one enum variant per aggregate lifecycle phase) provide maximum type safety but carry high cognitive cost for teams.
- Prefer a flat struct with a status-field enum (e.g. `status: OrderStatus`) as the default representation.
- Use full enum state machines only when the team has strong type-system fluency and the aggregate lifecycle genuinely benefits from compile-time state transition enforcement.

## Consequences

### Positive

- Time-travel and audit trail from the event stream.
- Efficient read-after-write and queries from snapshots.
- Simple, immediate consistency (single local transaction).
- Clear separation of concerns: `handle()` decides, `apply()` folds, application service orchestrates.

### Negative

- Two writes per command (events + snapshot), increasing write volume.
- Strict `apply` purity requires discipline; violations can silently corrupt replayed state.
- Snapshot schema must evolve carefully to stay compatible with the event stream.

## Alternatives Considered

1. **Pure Event Sourcing (events only)**
   - Rejected: every read requires folding the full event stream (or maintaining separate read projections). Read-after-write in the command flow becomes expensive or forces asynchronous eventual consistency.

2. **Pure Snapshot persistence (state only)**
   - Rejected: loses audit trail and time-travel capability entirely.

3. **Full enum state machine for aggregate state**
   - Not rejected outright, but demoted to opt-in: high cognitive cost for teams, and most business aggregates do not benefit enough from compile-time state transition enforcement to justify the complexity.

## References

- Event Sourcing pattern:
  - https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing
- Snapshotting in Event Sourcing:
  - https://www.eventstore.com/blog/snapshots-in-event-sourcing
- DDD Aggregate design:
  - https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/microservice-domain-model
