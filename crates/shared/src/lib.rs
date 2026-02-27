mod aggregate;
mod command;
mod entity;
mod error;
mod event;
mod id;
mod persistence;
mod repository;
mod service;
mod specification;
mod r#type;

pub use aggregate::AggregateRoot;
pub use command::Command;
pub use entity::Entity;
pub use error::{
    ContextualError, Error, ErrorContext, ErrorKind, ErrorKindInfo, ErrorSource, Result,
};
pub use event::DomainEvent;
pub use id::{EntityId, Id};
pub use persistence::Persistence;
pub use repository::{ReadRepository, RepositorySession, WriteRepository};
pub use service::WriteService;
pub use shared_derive::DomainModel;
pub use specification::{Specification, SpecificationExpression};
pub use r#type::{NonEmpty, NonNegative};
