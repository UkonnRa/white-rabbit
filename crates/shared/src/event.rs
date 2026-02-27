use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[cfg(test)]
mod test;

/// Marker trait for domain events produced by aggregate command handlers.
///
/// Analogous to [`Command`](crate::Command) on the input side:
/// every domain event must be serializable, cloneable, and identifiable
/// by a stable `event_type` string (used for routing and persistence).
pub trait DomainEvent:
    Send + Sync + Debug + Clone + PartialEq + Serialize + DeserializeOwned
{
    /// Stable event type identifier, e.g. `"whiterabbit::event::AccountCreated"`.
    ///
    /// Used for topic-based routing (ADR-0003) and event store serialization.
    fn event_type(&self) -> &'static str;
}
