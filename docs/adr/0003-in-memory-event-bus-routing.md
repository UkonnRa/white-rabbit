# ADR-0003: In-Memory Event Bus Routing (Rust/Tokio)

- Status: Accepted
- Date: 2026-02-27

## Context
The system runs as a single-process Rust application on Tokio's async runtime.
It needs an in-memory event bus that:
- Guarantees all subscribers receive every published event (no data loss on slow consumers).
- Routes events by domain category with compile-time type safety.
- Avoids a monolithic global enum that couples all bounded contexts.
- Supports backpressure to prevent unbounded memory growth.

`tokio::sync::broadcast` was initially considered but drops older messages when a subscriber falls behind, making it unsuitable for business-critical event delivery.

## Decision

### 1. Topic identification via `&'static str` with Trait associated constants
Each domain event category defines a static string topic key (e.g. via a trait associated constant `const TOPIC: &'static str`).
- This provides compile-time type safety within the codebase.
- It is portable: topic strings can be serialized for future cross-process or cross-network use.
- `TypeId` was considered but rejected because it is not stable across compiler versions, dynamic libraries, or network boundaries.

### 2. Bounded `mpsc` channel per subscriber
Each subscriber gets its own bounded `mpsc` channel.
- **Guaranteed delivery**: no message is dropped; a slow consumer blocks the publisher (backpressure) rather than losing data.
- **Backpressure**: bounded capacity prevents unbounded memory growth. The slowest consumer determines the maximum publish rate.
- **Isolation**: one slow subscriber does not interfere with other subscribers' channel buffers.

### 3. Subscription at domain-event-category granularity
Subscribers register interest in a domain event category (e.g. all `OrderEvent` variants), not in individual event variants.
- Rust enum variants are not independent types and cannot be distinguished by `TypeId`.
- One enum per aggregate or bounded context is the natural subscription unit.
- Subscribers use pattern matching to handle specific variants of interest.

## Consequences

### Positive
- No data loss: every subscriber receives every event it subscribed to.
- Backpressure protects against OOM under load spikes.
- Topic strings are future-proof for cross-process scenarios.
- Category-level subscription aligns naturally with DDD bounded contexts.

### Negative
- Bounded channels mean the slowest subscriber can become a bottleneck for the entire publish path.
- `&'static str` topics require manual uniqueness discipline (no compiler enforcement of topic key uniqueness across crates).
- Category-level subscription means subscribers receive events they may not care about and must filter locally.

## Alternatives Considered

1. **`tokio::sync::broadcast`**
   - Rejected: drops messages when subscriber lag exceeds buffer capacity. Unacceptable for business-critical domain events.

2. **Unbounded `mpsc` per subscriber**
   - Rejected: no backpressure. Under sustained load imbalance, memory grows without bound until OOM.

3. **`TypeId`-keyed dictionary bus**
   - Rejected: `TypeId` is a 128-bit hash that is unique only within a single compilation unit. It is not stable across compiler versions, dynamic libraries, or network serialization. Not suitable as a long-term routing key.

4. **Monolithic global event enum**
   - Rejected: couples all bounded contexts into a single type. Violates DDD boundary isolation and forces recompilation of the entire enum for any event addition.

## References
- Tokio mpsc channel:
  - https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html
- Tokio broadcast channel:
  - https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html
- `TypeId` stability discussion:
  - https://doc.rust-lang.org/std/any/struct.TypeId.html
