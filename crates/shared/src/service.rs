use crate::command::Command;
use crate::entity::Entity;
use crate::event::DomainEvent;
use crate::repository::RepositorySession;
use crate::uow::UnitOfWork;

/// Bundles entities and events produced by a command execution.
#[derive(Debug)]
pub struct HandleResult<E: Entity, Ev: DomainEvent> {
    pub entities: Vec<E>,
    pub events: Vec<Ev>,
}

#[async_trait::async_trait]
pub trait WriteService<C: Command>: Send + Sync {
    type Entity: Entity + 'static;
    type Event: DomainEvent + 'static;
    type Session: RepositorySession;
    type Error;

    /// Pure command handler: validates business rules and stages changes
    /// in the UoW. Reads go through the UoW overlay; the session is
    /// immutable. No persistence side effects.
    async fn do_handle(
        &self,
        sess: &Self::Session,
        uow: &mut UnitOfWork,
        command: C,
    ) -> std::result::Result<(), Self::Error>;

    /// Full orchestration: creates a UoW, calls [`do_handle`](Self::do_handle),
    /// flushes changes to the database, and returns entities + events.
    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: C,
    ) -> std::result::Result<HandleResult<Self::Entity, Self::Event>, Self::Error>;
}
