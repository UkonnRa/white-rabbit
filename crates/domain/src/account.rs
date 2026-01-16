use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{Entity, define_id};

use crate::{journal::JournalId, user::UserId};

define_id!(AccountId, Account);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by_id: Option<UserId>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub last_modified_by_id: Option<UserId>,

    pub name: String,
    pub description: String,
    pub journal_id: JournalId,
    pub parent_id: Option<AccountId>,
}

impl Entity for Account {
    type Id = AccountId;
    type OperatorId = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> usize {
        self.version
    }

    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn created_by_id(&self) -> Option<&Self::OperatorId> {
        self.created_by_id.as_ref()
    }

    fn last_modified_at(&self) -> Option<DateTime<Utc>> {
        self.last_modified_at
    }

    fn last_modified_by_id(&self) -> Option<&Self::OperatorId> {
        self.last_modified_by_id.as_ref()
    }
}
