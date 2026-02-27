#[cfg(test)]
mod test;

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};
use database_inmemory::repository::{
    InMemoryReadRepository, InMemorySession, InMemoryWriteRepository,
};
use domain::account::AccountId;
use domain::journal::JournalId;
use domain::record::repository::RecordRepository;
use domain::record::specification::{RecordSpec, RecordSpecification};
use domain::record::{Record, RecordId, RecordItemKind, RecordItems};
use shared::{Id, Persistence, ReadRepository, Result, WriteRepository};

// ── Persistence Object ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecordPo {
    pub id: RecordId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub journal_id: JournalId,
    pub date: NaiveDate,
    pub items: RecordItems,
    pub description: String,
    pub tags: HashSet<String>,
    pub payee: String,
}

impl Persistence for RecordPo {
    type Id = RecordId;
    fn id(&self) -> &RecordId {
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

impl RecordPo {
    fn kind(&self) -> RecordItemKind {
        match &self.items {
            RecordItems::Transactions(_) => RecordItemKind::Transaction,
            RecordItems::Validations(_) => RecordItemKind::Validation,
        }
    }

    fn account_ids(&self) -> HashSet<AccountId> {
        match &self.items {
            RecordItems::Transactions(txns) => txns.iter().map(|t| t.account_id.clone()).collect(),
            RecordItems::Validations(vals) => vals.iter().map(|v| v.account_id.clone()).collect(),
        }
    }
}

// ── Stateless Repository ─────────────────────────────────────────

#[derive(Default)]
pub struct InMemoryRecordRepository;

impl InMemoryRecordRepository {
    fn satisfies_leaf(&self, po: &RecordPo, spec: &RecordSpecification) -> bool {
        match spec {
            RecordSpecification::Id(ids) => ids.contains(&po.id),
            RecordSpecification::JournalId(ids) => ids.contains(&po.journal_id),
            RecordSpecification::AccountId(ids) => {
                let record_account_ids = po.account_ids();
                ids.iter().any(|id| record_account_ids.contains(id))
            }
            RecordSpecification::Date(dates) => dates.contains(&po.date),
            RecordSpecification::DateFrom(date) => po.date >= *date,
            RecordSpecification::DateTo(date) => po.date <= *date,
            RecordSpecification::Payee(payees) => {
                let lower_payee = po.payee.to_lowercase();
                payees.iter().any(|p| p.to_lowercase() == lower_payee)
            }
            RecordSpecification::Tag(tags) => {
                let lower_tags: HashSet<String> = tags.iter().map(|t| t.to_lowercase()).collect();
                po.tags
                    .iter()
                    .any(|t| lower_tags.contains(&t.to_lowercase()))
            }
            RecordSpecification::ItemKind(kind) => po.kind() == *kind,
            RecordSpecification::FullText(query) => {
                let query = query.trim().to_lowercase();
                po.description.to_lowercase().contains(&query)
                    || po.payee.to_lowercase().contains(&query)
                    || po.tags.iter().any(|t| t.to_lowercase().contains(&query))
            }
        }
    }
}

#[async_trait::async_trait]
impl InMemoryReadRepository<RecordSpec> for InMemoryRecordRepository {
    type Persistence = RecordPo;

    fn satisfies(&self, po: &RecordPo, spec: &RecordSpec) -> bool {
        spec.evaluate(&|leaf| self.satisfies_leaf(po, leaf))
    }

    fn convert_to_entity(&self, po: RecordPo) -> Record {
        Record {
            id: po.id,
            version: po.version,
            created_at: po.created_at,
            last_modified_at: po.last_modified_at,
            journal_id: po.journal_id,
            date: po.date,
            items: po.items,
            description: po.description,
            tags: po
                .tags
                .into_iter()
                .filter_map(|t| t.try_into().ok())
                .collect(),
            payee: po.payee,
        }
    }

    fn convert_to_persistence(&self, entity: &Record) -> RecordPo {
        RecordPo {
            id: entity.id.clone(),
            version: entity.version,
            created_at: entity.created_at,
            last_modified_at: entity.last_modified_at,
            journal_id: entity.journal_id.clone(),
            date: entity.date,
            items: entity.items.clone(),
            description: entity.description.clone(),
            tags: entity.tags.iter().map(|t| t.to_string()).collect(),
            payee: entity.payee.clone(),
        }
    }
}

#[async_trait::async_trait]
impl ReadRepository<RecordSpec> for InMemoryRecordRepository {
    type Entity = Record;
    type Session = InMemorySession;

    async fn find_all_by_ids(
        &self,
        sess: &Self::Session,
        ids: &[Id<Record>],
    ) -> Result<HashMap<Id<Record>, Record>> {
        self.__find_all_by_ids(sess, ids).await
    }

    async fn find_all(
        &self,
        sess: &Self::Session,
        spec: &RecordSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Record>, Record>> {
        self.__find_all(sess, spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<RecordSpec> for InMemoryRecordRepository {
    async fn save_all(
        &self,
        sess: &mut Self::Session,
        entities: &[Record],
    ) -> Result<HashMap<Id<Record>, Record>> {
        self.__save_all(sess, entities).await
    }

    async fn delete_all_by_ids(
        &self,
        sess: &mut Self::Session,
        ids: &[Id<Record>],
    ) -> Result<Vec<Id<Record>>> {
        self.__delete_all_by_ids(sess, ids).await
    }
}

impl RecordRepository for InMemoryRecordRepository {}

#[async_trait::async_trait]
impl InMemoryWriteRepository<RecordSpec> for InMemoryRecordRepository {}
