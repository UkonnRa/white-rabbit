use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, Condition, DatabaseConnection, DatabaseTransaction, IntoActiveModel,
    QuerySelect, TransactionTrait,
};
use shared::{Entity, Id, ReadRepository, RepositorySession, Result, Specification};
use std::collections::HashMap;

// ── SeaOrmSession ────────────────────────────────────────────────

/// Session for SeaORM-backed repositories. Holds the database connection
/// and an optional transaction.
pub struct SeaOrmSession {
    db: DatabaseConnection,
    txn: Option<DatabaseTransaction>,
}

impl SeaOrmSession {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db, txn: None }
    }

    // ── Connection dispatch methods ──────────────────────────────
    //
    // These encapsulate the match on txn vs db, eliminating the need
    // for a `with_conn!` macro. SeaORM's `ConnectionTrait` is not
    // dyn-compatible (generic methods), so we duplicate each call.

    /// Execute a SELECT query, returning all matching models.
    pub async fn query_all<E: EntityTrait>(
        &self,
        select: sea_orm::Select<E>,
    ) -> std::result::Result<Vec<E::Model>, sea_orm::DbErr> {
        match &self.txn {
            Some(txn) => select.all(txn).await,
            None => select.all(&self.db).await,
        }
    }

    /// Execute an INSERT statement.
    pub async fn exec_insert<A: ActiveModelTrait>(
        &self,
        insert: sea_orm::Insert<A>,
    ) -> std::result::Result<sea_orm::InsertResult<A>, sea_orm::DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    {
        match &self.txn {
            Some(txn) => insert.exec(txn).await,
            None => insert.exec(&self.db).await,
        }
    }

    /// Execute a DELETE MANY statement.
    pub async fn exec_delete<E: EntityTrait>(
        &self,
        delete: sea_orm::DeleteMany<E>,
    ) -> std::result::Result<sea_orm::DeleteResult, sea_orm::DbErr> {
        match &self.txn {
            Some(txn) => delete.exec(txn).await,
            None => delete.exec(&self.db).await,
        }
    }

    /// Execute an UPDATE on an active model.
    pub async fn exec_update<A: ActiveModelTrait + sea_orm::ActiveModelBehavior + Send>(
        &self,
        model: A,
    ) -> std::result::Result<<A::Entity as EntityTrait>::Model, sea_orm::DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    {
        match &self.txn {
            Some(txn) => model.update(txn).await,
            None => model.update(&self.db).await,
        }
    }
}

#[async_trait::async_trait]
impl RepositorySession for SeaOrmSession {
    async fn begin(&mut self) -> Result<()> {
        self.txn = Some(self.db.begin().await.map_err(shared::ErrorKind::internal)?);
        Ok(())
    }

    async fn commit(&mut self) -> Result<()> {
        if let Some(txn) = self.txn.take() {
            txn.commit().await.map_err(shared::ErrorKind::internal)?;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        if let Some(txn) = self.txn.take() {
            txn.rollback().await.map_err(shared::ErrorKind::internal)?;
        }
        Ok(())
    }
}

// ── SeaOrmReadRepository ─────────────────────────────────────────

#[async_trait::async_trait]
pub trait SeaOrmReadRepository<S: Specification>:
    ReadRepository<S, Session = SeaOrmSession>
{
    type SeaOrmEntity: EntityTrait<Model: IntoActiveModel<Self::SeaOrmActiveModel>>;
    type SeaOrmActiveModel: ActiveModelTrait<Entity = Self::SeaOrmEntity>
        + sea_orm::ActiveModelBehavior
        + Send;

    /// Convert a batch of SeaORM models into domain entities.
    async fn convert_to_entities(
        &self,
        sess: &SeaOrmSession,
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
        sess: &SeaOrmSession,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let values: Vec<sea_orm::Value> = ids.iter().map(|id| self.id_to_value(id)).collect();
        let models = sess
            .query_all(Self::SeaOrmEntity::find().filter(self.pk_column().is_in(values)))
            .await
            .map_err(shared::ErrorKind::internal)?;

        let entities = self.convert_to_entities(sess, models).await?;
        Ok(entities.into_iter().map(|e| (e.id().clone(), e)).collect())
    }

    async fn __find_all(
        &self,
        sess: &SeaOrmSession,
        spec: &S,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        let condition = self.spec_to_condition(spec);
        let mut query = Self::SeaOrmEntity::find().filter(condition);
        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        let models = sess
            .query_all(query)
            .await
            .map_err(shared::ErrorKind::internal)?;

        let entities = self.convert_to_entities(sess, models).await?;
        Ok(entities.into_iter().map(|e| (e.id().clone(), e)).collect())
    }
}

// ── SeaOrmWriteRepository ────────────────────────────────────────

#[async_trait::async_trait]
pub trait SeaOrmWriteRepository<S: Specification>: SeaOrmReadRepository<S> {
    /// Called after saving main entities to handle related data in batch.
    async fn save_all_related(
        &self,
        _sess: &SeaOrmSession,
        _entities: &[Self::Entity],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        Ok(())
    }

    /// Called before deleting main entities to clean up related data in batch.
    async fn delete_all_related(
        &self,
        _sess: &SeaOrmSession,
        _ids: &[Id<Self::Entity>],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        Ok(())
    }

    async fn __save_all(
        &self,
        sess: &mut SeaOrmSession,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        if entities.is_empty() {
            return Ok(HashMap::new());
        }

        let ids: Vec<_> = entities.iter().map(|e| e.id().clone()).collect();
        let existing = self.__find_all_by_ids(sess, &ids).await?;

        let mut to_update = Vec::new();
        let mut to_insert = Vec::new();
        for entity in entities {
            if existing.contains_key(&entity.id().clone()) {
                to_update.push(entity);
            } else {
                to_insert.push(entity);
            }
        }

        if !to_update.is_empty() {
            // Phase 1a: Temp values for unique columns (swap support)
            for entity in &to_update {
                let temp_model = self.convert_to_temp_active_model(entity);
                sess.exec_update(temp_model)
                    .await
                    .map_err(shared::ErrorKind::internal)?;
            }
            // Phase 1b: Final values
            for entity in &to_update {
                let active_model = self.convert_to_active_model(entity);
                sess.exec_update(active_model)
                    .await
                    .map_err(shared::ErrorKind::internal)?;
            }
            let updated: Vec<_> = to_update.iter().map(|&e| e.clone()).collect();
            self.save_all_related(sess, &updated)
                .await
                .map_err(shared::ErrorKind::internal)?;
        }

        // Phase 2: Insert new rows
        for entity in &to_insert {
            let active_model = self.convert_to_active_model(entity);
            sess.exec_insert(Self::SeaOrmEntity::insert(active_model))
                .await
                .map_err(shared::ErrorKind::internal)?;
        }
        if !to_insert.is_empty() {
            let inserted: Vec<_> = to_insert.iter().map(|&e| e.clone()).collect();
            self.save_all_related(sess, &inserted)
                .await
                .map_err(shared::ErrorKind::internal)?;
        }

        self.__find_all_by_ids(sess, &ids).await
    }

    async fn __delete_all_by_ids(
        &self,
        sess: &mut SeaOrmSession,
        ids: &[Id<Self::Entity>],
    ) -> Result<Vec<Id<Self::Entity>>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Check which IDs actually exist
        let existing = self.__find_all_by_ids(sess, ids).await?;
        let existing_ids: Vec<_> = existing.keys().cloned().collect();

        if existing_ids.is_empty() {
            return Ok(vec![]);
        }

        self.delete_all_related(sess, &existing_ids)
            .await
            .map_err(shared::ErrorKind::internal)?;

        let values: Vec<sea_orm::Value> =
            existing_ids.iter().map(|id| self.id_to_value(id)).collect();
        sess.exec_delete(Self::SeaOrmEntity::delete_many().filter(self.pk_column().is_in(values)))
            .await
            .map_err(shared::ErrorKind::internal)?;

        Ok(existing_ids)
    }

    fn updatable_columns(&self) -> Vec<<Self::SeaOrmEntity as EntityTrait>::Column>;

    fn convert_to_temp_active_model(&self, entity: &Self::Entity) -> Self::SeaOrmActiveModel {
        self.convert_to_active_model(entity)
    }
}
