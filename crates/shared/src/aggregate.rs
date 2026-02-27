use crate::entity::Entity;
use crate::event::DomainEvent;

#[cfg(test)]
mod test;

/// An entity that supports event-sourced state transitions.
///
/// `AggregateRoot` connects an [`Entity`] to its [`DomainEvent`] type
/// and provides the pure `apply` function for replaying events into state.
///
/// # Contract (per ADR-0002)
///
/// The `apply` function **must** be:
/// - **Deterministic**: same state + event = same next state.
/// - **Validation-free**: must not reject any event.
/// - **Free of non-determinism**: no clocks, no random IDs, no I/O.
/// - **Total over history**: must accept any event that was ever persisted,
///   including events from older domain model versions.
pub trait AggregateRoot: Entity {
    type Event: DomainEvent;

    fn apply(&mut self, event: &Self::Event);
}
