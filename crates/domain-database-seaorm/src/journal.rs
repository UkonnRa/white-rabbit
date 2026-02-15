#[cfg(test)]
mod test;

pub mod entity;
pub mod tag_entity;

use std::collections::{HashMap, HashSet};

use domain::journal::repository::JournalRepository;
use domain::journal::specification::{JournalSpec, JournalSpecification};
use domain::journal::{Journal, JournalId};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, QuerySelect, QueryTrait, Set, TransactionTrait,
};
use shared::{
    Entity, EntityId, Id, ReadRepository, Result, SpecificationExpression, WriteRepository,
};

/// Runs an async expression against either the transaction or the database connection.
/// This avoids the need for `&dyn ConnectionTrait` (which is not dyn-compatible in SeaORM).
macro_rules! with_conn {
    ($self:expr, |$conn:ident| $body:expr) => {
        match &$self.txn {
            Some($conn) => $body,
            None => {
                let $conn = &$self.db;
                $body
            }
        }
    };
}

// ── Repository ───────────────────────────────────────────────────

pub struct SeaOrmJournalRepository {
    db: DatabaseConnection,
    txn: Option<DatabaseTransaction>,
}

impl SeaOrmJournalRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db, txn: None }
    }

    fn spec_leaf_to_condition(&self, spec: &JournalSpecification) -> Condition {
        match spec {
            JournalSpecification::Id(ids) => {
                let values: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                Condition::any().add(entity::Column::Id.is_in(values))
            }
            JournalSpecification::Name(names) => {
                let values: Vec<String> = names.iter().cloned().collect();
                Condition::any().add(entity::Column::Name.is_in(values))
            }
            JournalSpecification::Tag(tags) => {
                let values: Vec<String> = tags.iter().cloned().collect();
                let subquery = tag_entity::Entity::find()
                    .filter(tag_entity::Column::Tag.is_in(values))
                    .select_only()
                    .column(tag_entity::Column::JournalId)
                    .into_query();
                Condition::any().add(entity::Column::Id.in_subquery(subquery))
            }
            JournalSpecification::FullText(query) => {
                let pattern = format!("%{query}%");
                Condition::any()
                    .add(entity::Column::Name.contains(&pattern))
                    .add(entity::Column::Description.contains(&pattern))
            }
        }
    }

    fn spec_to_condition(&self, spec: &JournalSpec) -> Condition {
        match spec {
            SpecificationExpression::Leaf(leaf) => self.spec_leaf_to_condition(leaf),
            SpecificationExpression::All(specs) => {
                let mut cond = Condition::all();
                for s in specs {
                    cond = cond.add(self.spec_to_condition(s));
                }
                cond
            }
            SpecificationExpression::Any(specs) => {
                let mut cond = Condition::any();
                for s in specs {
                    cond = cond.add(self.spec_to_condition(s));
                }
                cond
            }
            SpecificationExpression::Not(inner) => {
                Condition::all().add(self.spec_to_condition(inner).not())
            }
        }
    }

    fn id_to_value(&self, id: &Id<Journal>) -> sea_orm::Value {
        sea_orm::Value::from(id.value().to_string())
    }

    fn convert_to_active_model(&self, journal: &Journal) -> entity::ActiveModel {
        entity::ActiveModel {
            id: Set(journal.id.value().to_string()),
            version: Set(journal.version as i32),
            created_at: Set(journal.created_at),
            last_modified_at: Set(journal.last_modified_at),
            archived_at: Set(journal.archived_at),
            name: Set(journal.name.to_string()),
            description: Set(journal.description.clone()),
        }
    }

    fn convert_to_temp_active_model(&self, journal: &Journal) -> entity::ActiveModel {
        entity::ActiveModel {
            id: Set(journal.id.value().to_string()),
            version: Set(journal.version as i32),
            created_at: Set(journal.created_at),
            last_modified_at: Set(journal.last_modified_at),
            archived_at: Set(journal.archived_at),
            name: Set(format!("__temp__{}", uuid::Uuid::now_v7())),
            description: Set(journal.description.clone()),
        }
    }

    async fn load_tags_batch(
        &self,
        journal_ids: &[String],
    ) -> std::result::Result<HashMap<String, HashSet<String>>, sea_orm::DbErr> {
        if journal_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let ids = journal_ids.to_vec();
        let tags = with_conn!(self, |conn| {
            tag_entity::Entity::find()
                .filter(tag_entity::Column::JournalId.is_in(ids))
                .all(conn)
                .await?
        });

        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for tag in tags {
            result.entry(tag.journal_id).or_default().insert(tag.tag);
        }
        Ok(result)
    }

    async fn convert_to_entities(&self, models: Vec<entity::Model>) -> Result<Vec<Journal>> {
        let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        let tags_by_id = self
            .load_tags_batch(&ids)
            .await
            .map_err(shared::ErrorKind::internal)?;

        models
            .into_iter()
            .map(|model| {
                let tags = tags_by_id.get(&model.id).cloned().unwrap_or_default();
                Ok(Journal {
                    id: JournalId::from(model.id),
                    version: model.version as usize,
                    created_at: model.created_at,
                    last_modified_at: model.last_modified_at,
                    archived_at: model.archived_at,
                    name: model
                        .name
                        .try_into()
                        .map_err(|e: shared::Error| e.convert())?,
                    description: model.description,
                    tags: tags.into_iter().filter_map(|t| t.try_into().ok()).collect(),
                })
            })
            .collect()
    }

    async fn do_find_all_by_ids(
        &self,
        ids: &[Id<Journal>],
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let values: Vec<sea_orm::Value> = ids.iter().map(|id| self.id_to_value(id)).collect();
        let models = with_conn!(self, |conn| {
            entity::Entity::find()
                .filter(entity::Column::Id.is_in(values))
                .all(conn)
                .await
                .map_err(shared::ErrorKind::internal)?
        });
        let entities = self.convert_to_entities(models).await?;
        Ok(entities.into_iter().map(|e| (e.id().clone(), e)).collect())
    }

    async fn do_find_all(
        &self,
        spec: &JournalSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        let condition = self.spec_to_condition(spec);
        let mut query = entity::Entity::find().filter(condition);
        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }
        let models = with_conn!(self, |conn| {
            query.all(conn).await.map_err(shared::ErrorKind::internal)?
        });
        let entities = self.convert_to_entities(models).await?;
        Ok(entities.into_iter().map(|e| (e.id().clone(), e)).collect())
    }

    async fn save_tags(&self, journal: &Journal) -> std::result::Result<(), sea_orm::DbErr> {
        let id = journal.id.value().to_string();
        with_conn!(self, |conn| {
            tag_entity::Entity::delete_many()
                .filter(tag_entity::Column::JournalId.eq(&id))
                .exec(conn)
                .await?;

            for tag in &journal.tags {
                let tag_model = tag_entity::ActiveModel {
                    journal_id: Set(id.clone()),
                    tag: Set(tag.to_string()),
                };
                tag_entity::Entity::insert(tag_model).exec(conn).await?;
            }
            Ok(())
        })
    }

    async fn do_save_all(&mut self, entities: &[Journal]) -> Result<HashMap<Id<Journal>, Journal>> {
        if entities.is_empty() {
            return Ok(HashMap::new());
        }

        let ids: Vec<_> = entities.iter().map(|e| e.id().clone()).collect();
        let existing = self.do_find_all_by_ids(&ids).await?;

        let mut to_update = Vec::new();
        let mut to_insert = Vec::new();
        for entity in entities {
            if existing.contains_key(&entity.id().clone()) {
                to_update.push(entity);
            } else {
                to_insert.push(entity);
            }
        }

        // Phase 1: Update existing (temp names first for swap support)
        if !to_update.is_empty() {
            for e in &to_update {
                let temp = self.convert_to_temp_active_model(e);
                with_conn!(self, |conn| {
                    temp.update(conn)
                        .await
                        .map_err(shared::ErrorKind::internal)?
                });
            }
            for e in &to_update {
                let am = self.convert_to_active_model(e);
                with_conn!(self, |conn| {
                    am.update(conn).await.map_err(shared::ErrorKind::internal)?
                });
                self.save_tags(e)
                    .await
                    .map_err(shared::ErrorKind::internal)?;
            }
        }

        // Phase 2: Insert new rows
        for e in &to_insert {
            let am = self.convert_to_active_model(e);
            with_conn!(self, |conn| {
                entity::Entity::insert(am)
                    .exec(conn)
                    .await
                    .map_err(shared::ErrorKind::internal)?
            });
            self.save_tags(e)
                .await
                .map_err(shared::ErrorKind::internal)?;
        }

        self.do_find_all_by_ids(&ids).await
    }

    async fn do_delete_all_by_ids(
        &mut self,
        ids: &[Id<Journal>],
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let existing = self.do_find_all_by_ids(ids).await?;

        for id in ids {
            with_conn!(self, |conn| {
                tag_entity::Entity::delete_many()
                    .filter(tag_entity::Column::JournalId.eq(id.value()))
                    .exec(conn)
                    .await
                    .map_err(shared::ErrorKind::internal)?
            });
        }

        let values: Vec<sea_orm::Value> = ids.iter().map(|id| self.id_to_value(id)).collect();
        with_conn!(self, |conn| {
            entity::Entity::delete_many()
                .filter(entity::Column::Id.is_in(values))
                .exec(conn)
                .await
                .map_err(shared::ErrorKind::internal)?
        });

        Ok(existing)
    }
}

// ── ReadRepository / WriteRepository impls ───────────────────────

#[async_trait::async_trait]
impl ReadRepository<JournalSpec> for SeaOrmJournalRepository {
    type Entity = Journal;

    async fn find_all_by_ids(&self, ids: &[Id<Journal>]) -> Result<HashMap<Id<Journal>, Journal>> {
        self.do_find_all_by_ids(ids).await
    }

    async fn find_all(
        &self,
        spec: &JournalSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        self.do_find_all(spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<JournalSpec> for SeaOrmJournalRepository {
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

    async fn save_all(&mut self, entities: &[Journal]) -> Result<HashMap<Id<Journal>, Journal>> {
        self.do_save_all(entities).await
    }

    async fn delete_all_by_ids(
        &mut self,
        ids: &[Id<Journal>],
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        self.do_delete_all_by_ids(ids).await
    }
}

impl JournalRepository for SeaOrmJournalRepository {}
