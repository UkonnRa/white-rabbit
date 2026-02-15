#[cfg(test)]
mod test;

pub mod entity;
pub mod tag_entity;

use std::collections::{HashMap, HashSet};

use database_seaorm::repository::{SeaOrmReadRepository, SeaOrmWriteRepository};
use domain::journal::repository::JournalRepository;
use domain::journal::specification::{JournalSpec, JournalSpecification};
use domain::journal::{Journal, JournalId};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, QueryTrait,
    Set,
};
use shared::{EntityId, Id, ReadRepository, Result, SpecificationExpression, WriteRepository};

// ── Repository ───────────────────────────────────────────────────

pub struct SeaOrmJournalRepository {
    db: DatabaseConnection,
}

impl SeaOrmJournalRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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

    /// Load tags for multiple journals in a single query.
    async fn load_tags_batch(
        &self,
        journal_ids: &[String],
    ) -> std::result::Result<HashMap<String, HashSet<String>>, sea_orm::DbErr> {
        if journal_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let tags = tag_entity::Entity::find()
            .filter(tag_entity::Column::JournalId.is_in(journal_ids.to_vec()))
            .all(&self.db)
            .await?;

        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for tag in tags {
            result.entry(tag.journal_id).or_default().insert(tag.tag);
        }
        Ok(result)
    }
}

#[async_trait::async_trait]
impl SeaOrmReadRepository<JournalSpec> for SeaOrmJournalRepository {
    type SeaOrmEntity = entity::Entity;
    type SeaOrmActiveModel = entity::ActiveModel;

    fn get_db(&self) -> &DatabaseConnection {
        &self.db
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

    fn convert_to_active_model(&self, entity: &Journal) -> entity::ActiveModel {
        entity::ActiveModel {
            id: Set(entity.id.value().to_string()),
            version: Set(entity.version as i32),
            created_at: Set(entity.created_at),
            last_modified_at: Set(entity.last_modified_at),
            archived_at: Set(entity.archived_at),
            name: Set(entity.name.to_string()),
            description: Set(entity.description.clone()),
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

    fn pk_column(&self) -> entity::Column {
        entity::Column::Id
    }
}

#[async_trait::async_trait]
impl SeaOrmWriteRepository<JournalSpec> for SeaOrmJournalRepository {
    async fn save_related(&self, entity: &Journal) -> std::result::Result<(), sea_orm::DbErr> {
        let id = entity.id.value().to_string();

        // Delete existing tags
        tag_entity::Entity::delete_many()
            .filter(tag_entity::Column::JournalId.eq(&id))
            .exec(&self.db)
            .await?;

        // Insert new tags
        for tag in &entity.tags {
            let tag_model = tag_entity::ActiveModel {
                journal_id: Set(id.clone()),
                tag: Set(tag.to_string()),
            };
            tag_entity::Entity::insert(tag_model).exec(&self.db).await?;
        }

        Ok(())
    }

    async fn delete_related(&self, id: &Id<Journal>) -> std::result::Result<(), sea_orm::DbErr> {
        tag_entity::Entity::delete_many()
            .filter(tag_entity::Column::JournalId.eq(id.value()))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    fn convert_to_temp_active_model(&self, entity: &Journal) -> entity::ActiveModel {
        // Set name to a temporary UUID to avoid unique constraint violations
        // during batch updates (e.g., name swaps).
        entity::ActiveModel {
            id: Set(entity.id.value().to_string()),
            version: Set(entity.version as i32),
            created_at: Set(entity.created_at),
            last_modified_at: Set(entity.last_modified_at),
            archived_at: Set(entity.archived_at),
            name: Set(format!("__temp__{}", uuid::Uuid::now_v7())),
            description: Set(entity.description.clone()),
        }
    }

    fn updatable_columns(&self) -> Vec<entity::Column> {
        vec![
            entity::Column::Version,
            entity::Column::CreatedAt,
            entity::Column::LastModifiedAt,
            entity::Column::ArchivedAt,
            entity::Column::Name,
            entity::Column::Description,
        ]
    }
}

// ── ReadRepository / WriteRepository impls ───────────────────────

#[async_trait::async_trait]
impl ReadRepository<JournalSpec> for SeaOrmJournalRepository {
    type Entity = Journal;

    async fn find_all_by_ids(&self, ids: &[Id<Journal>]) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__find_all_by_ids(ids).await
    }

    async fn find_all(
        &self,
        spec: &JournalSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__find_all(spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<JournalSpec> for SeaOrmJournalRepository {
    async fn save_all(&mut self, entities: &[Journal]) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__save_all(entities).await
    }

    async fn delete_all_by_ids(
        &mut self,
        ids: &[Id<Journal>],
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__delete_all_by_ids(ids).await
    }
}

impl JournalRepository for SeaOrmJournalRepository {}
