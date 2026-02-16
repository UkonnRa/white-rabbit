use crate::command::Command;
use crate::entity::Entity;
use crate::repository::RepositorySession;

#[async_trait::async_trait]
pub trait WriteService<C: Command>: Send + Sync {
    type Entity: Entity;
    type Session: RepositorySession;
    type Error;

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: C,
    ) -> std::result::Result<Vec<Self::Entity>, Self::Error>;
}
