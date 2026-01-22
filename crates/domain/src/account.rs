use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainModel;

use crate::{journal::JournalId, user::UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
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
