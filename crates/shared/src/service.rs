use crate::entity::Entity;
use crate::repository::ReadRepository;
use crate::specification::Specification;

#[async_trait::async_trait]
pub trait ReadService: Send + Sync {
    type Operator: Send + Sync;
    type Entity: Entity;
    type Specification: Specification;
    type Repository: ReadRepository;

    async fn filter_readable(
        operator: &Self::Operator,
        entities: &[Self::Entity],
    ) -> Vec<Self::Entity>;
}
