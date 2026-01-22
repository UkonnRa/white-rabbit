mod entity;
mod error;
mod id;
mod persistence;
mod repository;
mod service;

pub use entity::Entity;
pub use error::{Error, Result};
pub use id::EntityId;
pub use persistence::Persistence;
pub use repository::{ReadRepository, WriteRepository};
pub use service::ReadService;
pub use shared_derive::DomainModel;

use std::fmt::Debug;

/// Alias for the ID type of an entity
pub type Id<E> = <E as Entity>::Id;

pub trait Specification: Send + Sync + Debug {}

pub trait Command: Send + Sync + Debug {
    fn command_type() -> &'static str;
}
