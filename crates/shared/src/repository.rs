use crate::entity::Entity;
use crate::specification::Specification;
use std::collections::HashSet;

#[async_trait::async_trait]
pub trait ReadRepository: Send + Sync {
    type Entity: Entity;
    type Specification: Specification;

    async fn find_one_by_id(&self, id: &str) -> Option<Self::Entity> {
        self.find_all_by_ids(&[id]).await.into_iter().next()
    }

    async fn find_all_by_ids(&self, ids: &[&str]) -> HashSet<Self::Entity>;

    async fn find_one(&self, spec: &Self::Specification) -> Option<Self::Entity> {
        self.find_all(spec, Some(1)).await.into_iter().next()
    }

    async fn find_all(
        &self,
        spec: &Self::Specification,
        limit: Option<usize>,
    ) -> HashSet<Self::Entity>;
}

#[async_trait::async_trait]
pub trait WriteRepository: ReadRepository {
    async fn save_all(&mut self, entities: &[&Self::Entity]) -> HashSet<Self::Entity>;

    async fn delete_all_by_ids(&mut self, ids: &[&str]) -> HashSet<Self::Entity>;

    async fn delete_all(&mut self, entities: &[Self::Entity]) -> HashSet<Self::Entity> {
        let ids: Vec<_> = entities.iter().map(|e| e.id()).collect();
        self.delete_all_by_ids(&ids).await
    }
}
