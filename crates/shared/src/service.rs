use crate::command::Command;
use crate::entity::Entity;
use crate::error::Result;
use crate::repository::{ReadRepository, WriteRepository};
use crate::specification::Specification;

#[async_trait::async_trait]
pub trait ReadService<S: Specification>: Send + Sync {
    type Entity: Entity;
    type Repository: ReadRepository<S>;
}

#[async_trait::async_trait]
pub trait WriteService<S: Specification, C: Command>: Send + Sync + ReadService<S> {
    type Repository: WriteRepository<S>;

    async fn execute(&self, command: C) -> Result<Vec<Self::Entity>>;
}
