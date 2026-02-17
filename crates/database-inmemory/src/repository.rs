use serde::Serialize;
use serde::de::DeserializeOwned;
use shared::{
    Entity, EntityId, Id, Persistence, ReadRepository, RepositorySession, Result, Specification,
};
use std::collections::HashMap;

/// In-memory session holding a two-level HashMap:
///   entity_type_key -> entity_id -> serialized PO (as JSON Value).
///
/// By using `serde_json::Value` instead of a generic `P`, a single session
/// can hold persistence objects of different types, which is required when
/// a service depends on multiple repositories (e.g. `RecordService` needs
/// both `RecordRepository` and `AccountRepository`).
#[derive(Default)]
pub struct InMemorySession {
    storage: HashMap<String, HashMap<String, serde_json::Value>>,
    snapshot: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

impl InMemorySession {
    /// Get an immutable reference to the storage for a given entity type.
    pub fn get_storage(&self, entity_type: &str) -> Option<&HashMap<String, serde_json::Value>> {
        self.storage.get(entity_type)
    }

    /// Get a mutable reference to the storage for a given entity type,
    /// creating it if it doesn't exist.
    pub fn get_storage_mut(
        &mut self,
        entity_type: &str,
    ) -> &mut HashMap<String, serde_json::Value> {
        self.storage.entry(entity_type.to_string()).or_default()
    }
}

#[async_trait::async_trait]
impl RepositorySession for InMemorySession {
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
    ReadRepository<S, Session = InMemorySession>
{
    type Persistence: Persistence + Serialize + DeserializeOwned;

    fn satisfies(&self, persistence: &Self::Persistence, specification: &S) -> bool;

    fn convert_to_entity(&self, persistence: Self::Persistence) -> Self::Entity;

    fn convert_to_persistence(&self, entity: &Self::Entity) -> Self::Persistence;

    async fn __find_all_by_ids(
        &self,
        sess: &InMemorySession,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let Some(storage) = sess.get_storage(Self::Entity::ENTITY_TYPE) else {
            return Ok(HashMap::new());
        };
        Ok(ids
            .iter()
            .filter_map(|id| {
                storage.get(id.value()).and_then(|v| {
                    let po: Self::Persistence = serde_json::from_value(v.clone()).ok()?;
                    let entity = self.convert_to_entity(po);
                    Some((entity.id().clone(), entity))
                })
            })
            .collect())
    }

    async fn __find_all(
        &self,
        sess: &InMemorySession,
        spec: &S,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let Some(storage) = sess.get_storage(Self::Entity::ENTITY_TYPE) else {
            return Ok(HashMap::new());
        };
        Ok(storage
            .values()
            .filter_map(|v| {
                let po: Self::Persistence = serde_json::from_value(v.clone()).ok()?;
                if self.satisfies(&po, spec) {
                    let entity = self.convert_to_entity(po);
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
        sess: &mut InMemorySession,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let mut saved: HashMap<Id<Self::Entity>, Self::Entity> = HashMap::new();
        let storage = sess.get_storage_mut(Self::Entity::ENTITY_TYPE);
        for entity in entities {
            let po = self.convert_to_persistence(entity);
            let id = po.id().value().to_string();
            let value = serde_json::to_value(&po).expect("PO serialization should not fail");
            storage.insert(id, value);
            let entity = self.convert_to_entity(po);
            saved.insert(entity.id().clone(), entity);
        }
        Ok(saved)
    }

    async fn __delete_all_by_ids(
        &self,
        sess: &mut InMemorySession,
        ids: &[Id<Self::Entity>],
    ) -> Result<Vec<Id<Self::Entity>>> {
        let storage = sess.get_storage_mut(Self::Entity::ENTITY_TYPE);
        let mut deleted = Vec::new();
        for id in ids {
            if storage.remove(id.value()).is_some() {
                deleted.push(id.clone());
            }
        }
        Ok(deleted)
    }
}
