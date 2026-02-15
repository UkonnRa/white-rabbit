#[cfg(test)]
mod test;

use chrono::{DateTime, Utc};
use database_inmemory::repository::{InMemoryReadRepository, InMemoryWriteRepository};
use domain::journal::repository::JournalRepository;
use domain::journal::specification::JournalSpecification;
use domain::journal::{Journal, JournalId};
use shared::{Id, Persistence, ReadRepository, Result, WriteRepository};
use std::collections::{HashMap, HashSet};

// ── Persistence Object ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalPo {
    pub id: JournalId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
}

impl Persistence for JournalPo {
    type Id = JournalId;
    fn id(&self) -> &JournalId {
        &self.id
    }
    fn version(&self) -> usize {
        self.version
    }
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }
    fn last_modified_at(&self) -> Option<DateTime<Utc>> {
        self.last_modified_at
    }
}

// ── In-Memory Repository ─────────────────────────────────────────

#[derive(Default)]
pub struct InMemoryJournalRepository {
    storage: HashMap<String, JournalPo>,
}

#[async_trait::async_trait]
impl InMemoryReadRepository<JournalSpecification> for InMemoryJournalRepository {
    type Persistence = JournalPo;

    fn satisfies(&self, po: &JournalPo, spec: &JournalSpecification) -> bool {
        match spec {
            JournalSpecification::Id(ids) => ids.contains(&po.id),
            JournalSpecification::Tag(tags) => po.tags.iter().any(|t| tags.contains(t)),
            JournalSpecification::FullText(query) => {
                let query = query.trim().to_lowercase();
                po.name.to_lowercase().contains(&query)
                    || po.description.to_lowercase().contains(&query)
                    || po.tags.iter().any(|t| t.to_lowercase().contains(&query))
            }
        }
    }

    fn get_storage(&self) -> &HashMap<String, JournalPo> {
        &self.storage
    }

    fn get_storage_mut(&mut self) -> &mut HashMap<String, JournalPo> {
        &mut self.storage
    }

    fn convert_to_entity(&self, po: JournalPo) -> Journal {
        Journal {
            id: po.id,
            version: po.version,
            created_at: po.created_at,
            last_modified_at: po.last_modified_at,
            archived_at: po.archived_at,
            name: po.name.try_into().unwrap(),
            description: po.description,
            tags: po
                .tags
                .into_iter()
                .filter_map(|t| t.try_into().ok())
                .collect(),
        }
    }

    fn convert_to_persistence(&self, entity: &Journal) -> JournalPo {
        JournalPo {
            id: entity.id.clone(),
            version: entity.version,
            created_at: entity.created_at,
            last_modified_at: entity.last_modified_at,
            archived_at: entity.archived_at,
            name: entity.name.to_string(),
            description: entity.description.clone(),
            tags: entity.tags.iter().map(|t| t.to_string()).collect(),
        }
    }
}

#[async_trait::async_trait]
impl ReadRepository<JournalSpecification> for InMemoryJournalRepository {
    type Entity = Journal;

    async fn find_all_by_ids(&self, ids: &[Id<Journal>]) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__find_all_by_ids(ids).await
    }

    async fn find_all(
        &self,
        spec: &JournalSpecification,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__find_all(spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<JournalSpecification> for InMemoryJournalRepository {
    async fn save_all(&mut self, entities: &[Journal]) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__save_all(entities).await
    }

    async fn save(&mut self, entity: &Journal) -> Result<Journal> {
        self.__save_all(std::slice::from_ref(entity)).await?;
        Ok(entity.clone())
    }

    async fn delete_all_by_ids(
        &mut self,
        ids: &[Id<Journal>],
    ) -> Result<HashMap<Id<Journal>, Journal>> {
        self.__delete_all_by_ids(ids).await
    }
}

impl JournalRepository for InMemoryJournalRepository {}

#[async_trait::async_trait]
impl InMemoryWriteRepository<JournalSpecification> for InMemoryJournalRepository {}
