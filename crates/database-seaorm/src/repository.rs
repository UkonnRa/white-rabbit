use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, IntoActiveModel, QuerySelect};
use shared::{Entity, Id, ReadRepository, Result, Specification};
use std::collections::HashMap;

/// Bridge trait between SeaORM entities and the app's `ReadRepository`.
///
/// Implementors define how to convert between SeaORM models and domain entities,
/// and how to translate domain specifications into SeaORM conditions.
#[async_trait::async_trait]
pub trait SeaOrmReadRepository<S: Specification>: ReadRepository<S> {
    type SeaOrmEntity: EntityTrait<Model: IntoActiveModel<Self::SeaOrmActiveModel>>;
    type SeaOrmActiveModel: ActiveModelTrait<Entity = Self::SeaOrmEntity>
        + sea_orm::ActiveModelBehavior
        + Send;

    /// Returns the underlying database connection.
    /// For transaction-aware operations, the concrete repository
    /// implementation handles routing to the active transaction internally.
    fn get_db(&self) -> &DatabaseConnection;

    /// Convert a batch of SeaORM models into domain entities.
    ///
    /// This is a batch operation so that related data (e.g., tags) can be
    /// loaded in a single query rather than N+1 per-entity queries.
    async fn convert_to_entities(
        &self,
        models: Vec<<Self::SeaOrmEntity as EntityTrait>::Model>,
    ) -> Result<Vec<Self::Entity>>;

    /// Convert a domain entity into a SeaORM active model for insert/update.
    fn convert_to_active_model(&self, entity: &Self::Entity) -> Self::SeaOrmActiveModel;

    /// Translate a domain specification into a SeaORM `Condition`.
    fn spec_to_condition(&self, spec: &S) -> Condition;

    /// Convert a domain entity ID to a SeaORM `Value` for queries.
    fn id_to_value(&self, id: &Id<Self::Entity>) -> sea_orm::Value;

    /// Get the primary key column for filtering by ID.
    fn pk_column(&self) -> <Self::SeaOrmEntity as EntityTrait>::Column;

    // ── Default implementations ──────────────────────────────────

    async fn __find_all_by_ids(
        &self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let values: Vec<sea_orm::Value> = ids.iter().map(|id| self.id_to_value(id)).collect();
        let models = Self::SeaOrmEntity::find()
            .filter(self.pk_column().is_in(values))
            .all(self.get_db())
            .await
            .map_err(shared::ErrorKind::internal)?;

        let entities = self.convert_to_entities(models).await?;
        Ok(entities.into_iter().map(|e| (e.id().clone(), e)).collect())
    }

    async fn __find_all(
        &self,
        spec: &S,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let condition = self.spec_to_condition(spec);
        let mut query = Self::SeaOrmEntity::find().filter(condition);
        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        let models = query
            .all(self.get_db())
            .await
            .map_err(shared::ErrorKind::internal)?;

        let entities = self.convert_to_entities(models).await?;
        Ok(entities.into_iter().map(|e| (e.id().clone(), e)).collect())
    }
}

/// Bridge trait for write operations.
#[async_trait::async_trait]
pub trait SeaOrmWriteRepository<S: Specification>: SeaOrmReadRepository<S> {
    /// Called after saving the main entity to handle related data (e.g., tags).
    /// Default: no-op.
    async fn save_related(
        &self,
        _entity: &Self::Entity,
    ) -> std::result::Result<(), sea_orm::DbErr> {
        Ok(())
    }

    /// Called before deleting the main entity to clean up related data.
    /// Default: no-op.
    async fn delete_related(
        &self,
        _id: &Id<Self::Entity>,
    ) -> std::result::Result<(), sea_orm::DbErr> {
        Ok(())
    }

    async fn __save_all(
        &mut self,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        if entities.is_empty() {
            return Ok(HashMap::new());
        }

        let db = self.get_db();

        // Determine which entities already exist
        let ids: Vec<_> = entities.iter().map(|e| e.id().clone()).collect();
        let existing = self.__find_all_by_ids(&ids).await?;

        // Separate into updates and inserts
        let mut to_update = Vec::new();
        let mut to_insert = Vec::new();
        for entity in entities {
            if existing.contains_key(&entity.id().clone()) {
                to_update.push(entity);
            } else {
                to_insert.push(entity);
            }
        }

        // Phase 1: For existing rows, first clear unique-constrained columns
        // to temporary values so that swaps don't violate constraints.
        // Then apply the final values.
        if !to_update.is_empty() {
            // 1a. Set unique columns to temporary values (UUID-based)
            for entity in &to_update {
                let temp_model = self.convert_to_temp_active_model(entity);
                temp_model
                    .update(db)
                    .await
                    .map_err(shared::ErrorKind::internal)?;
            }
            // 1b. Set final values
            for entity in &to_update {
                let active_model = self.convert_to_active_model(entity);
                active_model
                    .update(db)
                    .await
                    .map_err(shared::ErrorKind::internal)?;
                self.save_related(entity)
                    .await
                    .map_err(shared::ErrorKind::internal)?;
            }
        }

        // Phase 2: Insert new rows
        for entity in &to_insert {
            let active_model = self.convert_to_active_model(entity);
            Self::SeaOrmEntity::insert(active_model)
                .exec(db)
                .await
                .map_err(shared::ErrorKind::internal)?;
            self.save_related(entity)
                .await
                .map_err(shared::ErrorKind::internal)?;
        }

        // Re-fetch all saved entities
        self.__find_all_by_ids(&ids).await
    }

    async fn __delete_all_by_ids(
        &mut self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        // First fetch the entities to return them
        let existing = self.__find_all_by_ids(ids).await?;

        // Delete related data first
        for id in ids {
            self.delete_related(id)
                .await
                .map_err(shared::ErrorKind::internal)?;
        }

        // Delete the main entities
        let values: Vec<sea_orm::Value> = ids.iter().map(|id| self.id_to_value(id)).collect();
        Self::SeaOrmEntity::delete_many()
            .filter(self.pk_column().is_in(values))
            .exec(self.get_db())
            .await
            .map_err(shared::ErrorKind::internal)?;

        Ok(existing)
    }

    /// Columns to update on conflict (upsert). Override per entity.
    fn updatable_columns(&self) -> Vec<<Self::SeaOrmEntity as EntityTrait>::Column>;

    /// Create an active model with unique-constrained columns set to temporary
    /// UUID values. Used during batch updates to avoid unique constraint
    /// violations when swapping values (e.g., renaming A→"Beta", B→"Alpha").
    ///
    /// Default: same as `convert_to_active_model` (no unique columns to clear).
    /// Override if your entity has unique columns besides the primary key.
    fn convert_to_temp_active_model(&self, entity: &Self::Entity) -> Self::SeaOrmActiveModel {
        self.convert_to_active_model(entity)
    }
}
