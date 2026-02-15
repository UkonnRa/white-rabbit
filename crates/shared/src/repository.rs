use crate::entity::Entity;
use crate::error::{ErrorKind, Result};
use crate::id::Id;
use crate::specification::Specification;
use std::array;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait ReadRepository<S: Specification>: Send + Sync {
    type Entity: Entity;

    /// Find an entity by its ID
    async fn find_one_by_id(&self, id: &Id<Self::Entity>) -> Result<Option<Self::Entity>> {
        Ok(self
            .find_all_by_ids(array::from_ref(id))
            .await?
            .get(id)
            .cloned())
    }

    /// Find all entities matching the given IDs
    async fn find_all_by_ids(
        &self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>>;

    /// Find the first entity matching the specification
    async fn find_one(&self, spec: &S) -> Result<Option<Self::Entity>> {
        Ok(self.find_all(spec, Some(1)).await?.values().next().cloned())
    }

    /// Find all entities matching the specification with optional limit
    async fn find_all(
        &self,
        spec: &S,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>>;
}

#[async_trait::async_trait]
pub trait WriteRepository<S: Specification>: ReadRepository<S> {
    /// Begin a transaction. Subsequent reads and writes operate within it.
    /// Default: no-op (for backends without transaction support).
    async fn begin(&mut self) -> Result<()> {
        Ok(())
    }

    /// Commit the current transaction.
    /// Default: no-op.
    async fn commit(&mut self) -> Result<()> {
        Ok(())
    }

    /// Rollback the current transaction, undoing all mutations since `begin`.
    /// Default: no-op.
    async fn rollback(&mut self) -> Result<()> {
        Ok(())
    }

    /// Save all entities and return the saved entities
    async fn save_all(
        &mut self,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>>;

    /// Save a single entity and return the saved entity
    async fn save(&mut self, entity: &Self::Entity) -> Result<Self::Entity> {
        self.save_all(array::from_ref(entity))
            .await?
            .into_values()
            .next()
            .ok_or_else(ErrorKind::not_found)
    }

    /// Delete entities by their IDs and return the deleted entities
    async fn delete_all_by_ids(
        &mut self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>>;

    /// Delete a single entity by ID and return it
    async fn delete(&mut self, id: &Id<Self::Entity>) -> Result<Option<Self::Entity>> {
        Ok(self
            .delete_all_by_ids(array::from_ref(id))
            .await?
            .into_values()
            .next())
    }

    /// Delete all given entities and return the deleted entities
    async fn delete_all(
        &mut self,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let ids: Vec<_> = entities.iter().map(|e| e.id().clone()).collect();
        self.delete_all_by_ids(&ids).await
    }
}
