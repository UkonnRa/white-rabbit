use crate::entity::Entity;
use std::fmt::Debug;
use std::hash::Hash;

/// Alias for the ID type of an entity
pub type Id<E> = <E as Entity>::Id;

pub trait EntityId: Clone + Debug + Eq + Hash + Send + Sync + 'static {
    type Entity: Entity;

    /// The type name for this ID (used in GlobalId conversion)
    const TYPE_NAME: &'static str;

    /// Get the underlying string value of this ID
    fn value(&self) -> &str;

    /// Create an ID from a string value
    fn from_value(value: impl Into<String>) -> Self;

    /// Generate a new random UUID-based ID
    fn generate() -> Self
    where
        Self: Sized,
    {
        Self::from_value(uuid::Uuid::now_v7().to_string())
    }
}
