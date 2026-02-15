use std::collections::HashMap;

use crate::entity::AuthEntity;
use crate::permission::Permission;
use shared::{Entity, Id, ReadService, Result, Specification};

#[async_trait::async_trait]
pub trait AuthReadService<S: Specification>: ReadService<S> {
    type Operator: AuthEntity;

    async fn calculate_permission(
        &self,
        operator: &Self::Operator,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Permission>>;

    async fn filter_by_permission(
        &self,
        operator: &Self::Operator,
        entities: impl IntoIterator<Item = Self::Entity> + Send + Sync,
        permission: Permission,
    ) -> Result<Vec<Self::Entity>> {
        let entities = entities.into_iter().collect::<Vec<Self::Entity>>();
        let permissions = self.calculate_permission(operator, &entities).await?;
        Ok(entities
            .into_iter()
            .filter(|entity| {
                if let Some(p) = permissions.get(entity.id()) {
                    p.contains(&permission)
                } else {
                    false
                }
            })
            .collect())
    }
}
