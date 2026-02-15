use crate::entity::Entity;
use crate::repository::ReadRepository;
use crate::specification::Specification;

#[async_trait::async_trait]
pub trait ReadService<S: Specification>: Send + Sync {
    type Entity: Entity;
    type Repository: ReadRepository<S>;
}
