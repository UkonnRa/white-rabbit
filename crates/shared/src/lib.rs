pub mod entity;
pub mod error;
pub mod id;
pub mod persistence;
pub mod repository;
pub mod service;
pub mod specification;

pub use entity::Entity;
pub use error::{Error, Result};
pub use id::EntityId;
pub use persistence::Persistence;
pub use repository::{ReadRepository, WriteRepository};
pub use service::ReadService;
pub use specification::Specification;

/// Alias for the ID type of an entity
pub type Id<E> = <E as Entity>::Id;
