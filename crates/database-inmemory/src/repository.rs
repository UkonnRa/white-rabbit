use shared::entity::Entity;
use shared::persistence::Persistence;
use shared::repository::ReadRepository;
use std::collections::{HashMap, HashSet};

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

    async fn __find_all_by_ids(&self, ids: &[&str]) -> HashSet<Self::Entity> {
        let storage = self.get_storage();
        ids.iter()
            .filter_map(|id| storage.get(*id).cloned())
            .map(|po| self.convert_to_entity(po))
            .collect()
    }

    async fn __find_all(
        &self,
        spec: &Self::Specification,
        limit: Option<usize>,
    ) -> HashSet<Self::Entity> {
        self.get_storage()
            .values()
            .filter_map(|po| {
                if self.satisfies(po, spec) {
                    Some(self.convert_to_entity(po.clone()))
                } else {
                    None
                }
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect()
    }
}

#[async_trait::async_trait]
pub trait InMemoryWriteRepository: InMemoryReadRepository {
    async fn __save_all(&mut self, entities: &[&Self::Entity]) -> HashSet<Self::Entity> {
        let mut saved_pos = HashSet::new();
        for entity in entities {
            let po = self.convert_to_persistence(entity);
            if let Some(po) = self
                .get_storage_mut()
                .insert(po.id().to_string(), po.clone())
            {
                saved_pos.insert(po);
            }
        }
        saved_pos
            .into_iter()
            .map(|po| self.convert_to_entity(po))
            .collect()
    }

    async fn __delete_all_by_ids(&mut self, ids: &[&str]) -> HashSet<Self::Entity> {
        let mut deleted_pos = HashSet::new();
        for id in ids {
            let storage = self.get_storage_mut();
            if let Some(po) = storage.remove(*id) {
                deleted_pos.insert(po);
            }
        }
        deleted_pos
            .into_iter()
            .map(|po| self.convert_to_entity(po))
            .collect()
    }
}
