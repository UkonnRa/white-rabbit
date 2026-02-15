use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainModel;

use crate::journal::JournalId;

/// The Account is a tree, the root Account is one of the 5 types: Asset, Liability, Equity, Income, Expense
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Account {
    pub id: AccountId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub name: String,
    pub description: String,
    pub journal_id: JournalId,
    pub parent_id: Option<AccountId>,
    // The type of children should be the same as the parent's type
    // For each Journal, we always have 5 roots: Asset, Liability, Equity, Income, Expense
    pub r#type: AccountType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}
