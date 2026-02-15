use chrono::{DateTime, Utc};
use shared::{Error, NonEmpty, Result};
use std::collections::HashSet;

use crate::account::{Account, AccountId, AccountType};
use crate::journal::JournalId;

#[derive(Default)]
pub struct AccountInput {
    pub id: AccountId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub journal_id: JournalId,
    pub parent_id: Option<AccountId>,
    // The type of children should be the same as the parent's type
    // For each Journal, we always have 5 roots: Asset, Liability, Equity, Income, Expense
    pub r#type: AccountType,

    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl TryFrom<AccountInput> for Account {
    type Error = Error;

    fn try_from(value: AccountInput) -> Result<Self> {
        Ok(Account {
            id: value.id,
            version: value.version,
            created_at: value.created_at,
            last_modified_at: value.last_modified_at,
            archived_at: value.archived_at,

            name: NonEmpty::try_from(value.name)
                .map_err(|e| e.with_resource_type(Account::TYPE).with_field("name"))?,
            description: value.description,
            tags: value
                .tags
                .into_iter()
                .filter_map(|tag| NonEmpty::try_from(tag).ok())
                .collect::<HashSet<_>>(),
            journal_id: value.journal_id,
            parent_id: value.parent_id,
            r#type: value.r#type,
        })
    }
}
