use shared::entity::Entity;
use shared::persistence::Persistence;
use shared::repository::ReadRepository;
use shared::{EntityId, Id, Result};
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait InMemoryReadRepository: ReadRepository {
    type Persistence: Persistence;

    fn satisfies(
        &self,
        persistence: &Self::Persistence,
        specification: &Self::Specification,
    ) -> bool;

    fn get_storage(&self) -> &HashMap<String, Self::Persistence>;

    fn get_storage_mut(&mut self) -> &mut HashMap<String, Self::Persistence>;

    fn convert_to_entity(&self, persistence: Self::Persistence) -> Self::Entity;

    fn convert_to_persistence(&self, entity: &Self::Entity) -> Self::Persistence;

    async fn __find_all_by_ids(
        &self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let storage = self.get_storage();
        Ok(ids
            .iter()
            .filter_map(|id| {
                storage.get(id.value()).cloned().map(|po| {
                    let entity = self.convert_to_entity(po);
                    (entity.id().clone(), entity)
                })
            })
            .collect())
    }

    async fn __find_all(
        &self,
        spec: &Self::Specification,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        Ok(self
            .get_storage()
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
pub trait InMemoryWriteRepository: InMemoryReadRepository {
    async fn __save_all(
        &mut self,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let mut saved_pos: HashMap<Id<Self::Entity>, Self::Entity> = HashMap::new();
        for entity in entities {
            let po = self.convert_to_persistence(entity);
            if let Some(po) = self
                .get_storage_mut()
                .insert(po.id().value().to_string(), po.clone())
            {
                let entity = self.convert_to_entity(po);
                saved_pos.insert(entity.id().clone(), entity);
            }
        }
        Ok(saved_pos)
    }

    async fn __delete_all_by_ids(
        &mut self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let mut deleted_pos: HashMap<Id<Self::Entity>, Self::Entity> = HashMap::new();
        for id in ids {
            let storage = self.get_storage_mut();
            if let Some(po) = storage.remove(id.value()) {
                let entity = self.convert_to_entity(po);
                deleted_pos.insert(entity.id().clone(), entity);
            }
        }
        Ok(deleted_pos)
    }
}
