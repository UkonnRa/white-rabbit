use crate::entity::Entity;
use crate::error::Result;
use crate::repository::{ReadRepository, WriteRepository};
use crate::{Command, Specification};

#[async_trait::async_trait]
pub trait ReadService<S: Specification>: Send + Sync {
    type Operator: Send + Sync;
    type Entity: Entity;
    type Repository: ReadRepository<S>;

    async fn filter_readable(
        operator: &Self::Operator,
        entities: &[Self::Entity],
    ) -> Result<Vec<Self::Entity>>;
}

#[async_trait::async_trait]
pub trait WriteService<S: Specification, C: Command>: Send + Sync + ReadService<S> {
    type Repository: WriteRepository<S>;

    async fn execute(operator: &Self::Operator, command: C) -> Result<Vec<Self::Entity>>;
}
