# ADR-0005: Unit of Work Implementation and WriteService Split

- Status: Accepted
- Date: 2026-02-28

## Context

The system's domain services (`JournalService`, `AccountService`, `RecordService`)
mix business-rule validation with persistence side effects inside the same
`do_create` / `do_update` / `do_delete` methods. This creates several problems:

1. **Find-back anti-pattern.** Endpoint handlers call a service method that
   returns only events, then must re-query the repository to obtain the
   entity the service just created or updated.

2. **Batch ordering dependency on persistence.** Batch operations
   (delete → create → update) require each step to flush writes before the
   next step's validation queries see the changes (e.g., a deleted name
   must be invisible to the create name-uniqueness check).

3. **Dry-run is impossible.** Because every command handler writes to the
   database, there is no way to execute the same logic without side effects
   to produce a preview diff.

ADR-0001 already commits to Unit of Work as the execution boundary and to
supporting dry-run. This ADR records the concrete implementation decisions.

## Decision

### 1. Generic, type-erased Unit of Work in the shared crate

A `UnitOfWork` struct in `shared` tracks pending changes for any entity type:

- **`EntityChangeSet<E>`** per entity type: `new`, `dirty` (both
  `HashMap<Id<E>, E>`), and `deleted` (`HashSet<Id<E>>`).
- Type erasure via `Any` + `TypeId` allows a single `UnitOfWork` instance
  to hold change sets for multiple entity types without generics on the struct.
- Events are buffered as `Box<dyn Any + Send + Sync>` and extracted by type.

### 2. Overlay-aware query methods on UnitOfWork

UoW provides query helpers that compose in-memory changes with database results:

- `find_all(repo, sess, spec, limit)` — queries the DB, removes tombstoned
  IDs, merges in new/dirty entities that match the spec.
- `find_all_by_ids(repo, sess, ids)` — same overlay logic, ID-based.
- `find_one(repo, sess, spec)` — convenience wrapper returning first match.

These methods require `SpecificationEvaluator<E>` (see below) to evaluate
specifications against in-memory entities.

### 3. `SpecificationEvaluator` trait for in-memory matching

A new trait in `shared`:

```
trait SpecificationEvaluator<E: Entity> {
    fn matches(&self, entity: &E) -> bool;
}
```

Each domain specification enum (`JournalSpecification`, `AccountSpecification`,
`RecordSpecification`) implements this trait for its entity type. The matching
logic is extracted from the existing `satisfies_leaf` methods in
`domain-database-inmemory`, which are then refactored to delegate to the
evaluator.

### 4. WriteService split: `do_handle` (pure) + `handle` (orchestrating)

The `WriteService` trait gains two methods:

- **`do_handle(&self, sess, uow, command) -> Result<()>`** — pure decision
  logic. Reads go through UoW overlay (session is immutable). Writes are
  staged in UoW. No persistence side effects.
- **`handle(&self, sess, command) -> Result<HandleResult>`** — creates a UoW,
  calls `do_handle`, flushes changes to the database, returns entities + events.

`HandleResult<E, Ev>` bundles `Vec<E>` and `Vec<Ev>` so callers receive
entities directly — eliminating the find-back anti-pattern.

### 5. Write-path queries are single-aggregate-type

All UoW overlay queries on the write path operate on a single entity type.
Cross-aggregate resolution (e.g., Record → Account lookup) is decomposed
by the domain service into separate UoW-mediated queries per entity type.
No database-level joins (`$lookup`, `$graphLookup`, SQL JOIN) are used
for write-path queries.

### 6. Timestamps fixed at command time

Per ADR-0002, non-deterministic values must be captured in events at
command time. `created_at` and `last_modified_at` are set in the domain
input before registering entities in the UoW, not assigned by the database
after save.

## Consequences

### Positive

- Endpoints receive entities directly from `handle` — no re-query.
- Batch operations work correctly without interleaved persistence; the
  UoW overlay provides consistent visibility across steps.
- `do_handle` can be called without `handle` for dry-run previews.
- Specification matching logic is defined once in the domain crate and
  reused by both UoW overlay queries and in-memory test repositories.

### Negative

- Type erasure in UoW adds runtime downcasting; incorrect type parameters
  cause panics rather than compile errors.
- In-memory specification evaluation may not exactly match database-level
  behaviour for edge cases (e.g., collation, full-text ranking).
- Additional trait bound (`SpecificationEvaluator`) on UoW query methods
  increases generic complexity at call sites.

## Alternatives Considered

1. **Per-aggregate UoW (no type erasure)**
   - Simpler per aggregate but cannot support commands that read multiple
     entity types (e.g., `RecordService` reads accounts).

2. **Interleaved persistence for batch (Option A from discussion)**
   - Simpler but breaks `do_handle` purity and prevents dry-run.

3. **Hibernate-style auto-flush before cross-entity queries**
   - Viable escape hatch for future cross-aggregate write-path queries,
     but not needed today. Documented as fallback strategy.

## References

- ADR-0001: DDD + Dry-Run + CQRS + Specification
- ADR-0002: Event Sourcing with Snapshot Dual-Write Persistence
- Architecture Overview, sections 3.1, 5, 7
- Unit of Work concept: https://en.wikipedia.org/wiki/Unit_of_work
