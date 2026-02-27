use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainEvent;

use crate::account::{AccountId, AccountType};
use crate::journal::JournalId;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountEvent {
    Created(AccountCreated),
    Updated(AccountUpdated),
    Archived(AccountArchived),
    Deleted(AccountDeleted),
}

impl DomainEvent for AccountEvent {
    fn event_type(&self) -> &'static str {
        match self {
            AccountEvent::Created(_) => "whiterabbit::event::AccountCreated",
            AccountEvent::Updated(_) => "whiterabbit::event::AccountUpdated",
            AccountEvent::Archived(_) => "whiterabbit::event::AccountArchived",
            AccountEvent::Deleted(_) => "whiterabbit::event::AccountDeleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCreated {
    pub id: AccountId,
    pub journal_id: JournalId,
    pub parent_id: Option<AccountId>,
    pub r#type: AccountType,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountUpdated {
    pub id: AccountId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub last_modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountArchived {
    pub id: AccountId,
    pub archived_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountDeleted {
    pub id: AccountId,
}
