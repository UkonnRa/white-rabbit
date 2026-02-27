#[cfg(test)]
mod test;

use chrono::{DateTime, Utc};
use database_inmemory::repository::{
    InMemoryReadRepository, InMemorySession, InMemoryWriteRepository,
};
use domain::account::repository::AccountRepository;
use domain::account::specification::AccountSpec;
use domain::account::{Account, AccountId, AccountType};
use domain::journal::JournalId;
use shared::{Id, Persistence, ReadRepository, Result, WriteRepository};
use std::collections::{HashMap, HashSet};

// ── Persistence Object ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountPo {
    pub id: AccountId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub journal_id: JournalId,
    pub parent_id: Option<AccountId>,
    pub r#type: AccountType,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
}

impl Persistence for AccountPo {
    type Id = AccountId;
    fn id(&self) -> &AccountId {
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

// ── Stateless Repository ─────────────────────────────────────────

#[derive(Default)]
pub struct InMemoryAccountRepository;

impl InMemoryAccountRepository {}

#[async_trait::async_trait]
impl InMemoryReadRepository<AccountSpec> for InMemoryAccountRepository {
    type Persistence = AccountPo;

    fn satisfies(&self, po: &AccountPo, spec: &AccountSpec) -> bool {
        let entity = self.convert_to_entity(po.clone());
        spec.matches(&entity)
    }

    fn convert_to_entity(&self, po: AccountPo) -> Account {
        Account {
            id: po.id,
            version: po.version,
            created_at: po.created_at,
            last_modified_at: po.last_modified_at,
            archived_at: po.archived_at,
            journal_id: po.journal_id,
            parent_id: po.parent_id,
            r#type: po.r#type,
            name: po.name.try_into().unwrap(),
            description: po.description,
            tags: po
                .tags
                .into_iter()
                .filter_map(|t| t.try_into().ok())
                .collect(),
        }
    }

    fn convert_to_persistence(&self, entity: &Account) -> AccountPo {
        AccountPo {
            id: entity.id.clone(),
            version: entity.version,
            created_at: entity.created_at,
            last_modified_at: entity.last_modified_at,
            archived_at: entity.archived_at,
            journal_id: entity.journal_id.clone(),
            parent_id: entity.parent_id.clone(),
            r#type: entity.r#type,
            name: entity.name.to_string(),
            description: entity.description.clone(),
            tags: entity.tags.iter().map(|t| t.to_string()).collect(),
        }
    }
}

#[async_trait::async_trait]
impl ReadRepository<AccountSpec> for InMemoryAccountRepository {
    type Entity = Account;
    type Session = InMemorySession;

    async fn find_all_by_ids(
        &self,
        sess: &Self::Session,
        ids: &[Id<Account>],
    ) -> Result<HashMap<Id<Account>, Account>> {
        self.__find_all_by_ids(sess, ids).await
    }

    async fn find_all(
        &self,
        sess: &Self::Session,
        spec: &AccountSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Account>, Account>> {
        self.__find_all(sess, spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<AccountSpec> for InMemoryAccountRepository {
    async fn save_all(
        &self,
        sess: &mut Self::Session,
        entities: &[Account],
    ) -> Result<HashMap<Id<Account>, Account>> {
        self.__save_all(sess, entities).await
    }

    async fn delete_all_by_ids(
        &self,
        sess: &mut Self::Session,
        ids: &[Id<Account>],
    ) -> Result<Vec<Id<Account>>> {
        self.__delete_all_by_ids(sess, ids).await
    }
}

impl AccountRepository for InMemoryAccountRepository {}

#[async_trait::async_trait]
impl InMemoryWriteRepository<AccountSpec> for InMemoryAccountRepository {}
