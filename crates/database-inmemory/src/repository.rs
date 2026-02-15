use shared::{
    Entity, EntityId, Id, Persistence, ReadRepository, RepositorySession, Result, Specification,
};
use std::collections::HashMap;

/// In-memory session holding the storage HashMap and optional snapshot for
/// transaction support.
pub struct InMemorySession<P: Persistence> {
    pub storage: HashMap<String, P>,
    snapshot: Option<HashMap<String, P>>,
}

impl<P: Persistence> Default for InMemorySession<P> {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
            snapshot: None,
        }
    }
}

#[async_trait::async_trait]
impl<P: Persistence> RepositorySession for InMemorySession<P> {
    async fn begin(&mut self) -> Result<()> {
        self.snapshot = Some(self.storage.clone());
        Ok(())
    }

    async fn commit(&mut self) -> Result<()> {
        self.snapshot = None;
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        if let Some(snapshot) = self.snapshot.take() {
            self.storage = snapshot;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait InMemoryReadRepository<S: Specification>:
    ReadRepository<S, Session = InMemorySession<Self::Persistence>>
{
    type Persistence: Persistence;

    fn satisfies(&self, persistence: &Self::Persistence, specification: &S) -> bool;

    fn convert_to_entity(&self, persistence: Self::Persistence) -> Self::Entity;

    fn convert_to_persistence(&self, entity: &Self::Entity) -> Self::Persistence;

    async fn __find_all_by_ids(
        &self,
        sess: &InMemorySession<Self::Persistence>,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        Ok(ids
            .iter()
            .filter_map(|id| {
                sess.storage.get(id.value()).cloned().map(|po| {
                    let entity = self.convert_to_entity(po);
                    (entity.id().clone(), entity)
                })
            })
            .collect())
    }

    async fn __find_all(
        &self,
        sess: &InMemorySession<Self::Persistence>,
        spec: &S,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        Ok(sess
            .storage
            .values()
            .filter_map(|po| {
                if self.satisfies(po, spec) {
                    let entity = self.convert_to_entity(po.clone());
                    Some((entity.id().clone(), entity))
                } else {
                    None
                }
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect::<HashMap<Id<Self::Entity>, Self::Entity>>())
    }
}

#[async_trait::async_trait]
pub trait InMemoryWriteRepository<S: Specification>: InMemoryReadRepository<S> {
    async fn __save_all(
        &self,
        sess: &mut InMemorySession<Self::Persistence>,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let mut saved: HashMap<Id<Self::Entity>, Self::Entity> = HashMap::new();
        for entity in entities {
            let po = self.convert_to_persistence(entity);
            sess.storage.insert(po.id().value().to_string(), po.clone());
            let entity = self.convert_to_entity(po);
            saved.insert(entity.id().clone(), entity);
        }
        Ok(saved)
    }

    async fn __delete_all_by_ids(
        &self,
        sess: &mut InMemorySession<Self::Persistence>,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let mut deleted: HashMap<Id<Self::Entity>, Self::Entity> = HashMap::new();
        for id in ids {
            if let Some(po) = sess.storage.remove(id.value()) {
                let entity = self.convert_to_entity(po);
                deleted.insert(entity.id().clone(), entity);
            }
        }
        Ok(deleted)
    }
}
