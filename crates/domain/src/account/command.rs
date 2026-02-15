use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::Command;

use crate::account::AccountId;
use crate::journal::JournalId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountCommand {
    Create(AccountCommandCreate),
    Update(AccountCommandUpdate),
    Delete(HashSet<AccountId>),
    Archive(AccountCommandArchive),
    Batch(AccountCommandBatch),
}

impl Command for AccountCommand {
    fn command_type(&self) -> &'static str {
        match self {
            AccountCommand::Create(c) => c.command_type(),
            AccountCommand::Update(c) => c.command_type(),
            AccountCommand::Delete(_) => "whiterabbit::command::AccountCommandDelete",
            AccountCommand::Archive(c) => c.command_type(),
            AccountCommand::Batch(c) => c.command_type(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCommandCreate {
    pub journal_id: JournalId,
    /// Required — user accounts always have a parent.
    /// Root accounts are created by the system, not by user command.
    pub parent_id: AccountId,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
}

impl Command for AccountCommandCreate {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::AccountCommandCreate"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCommandUpdate {
    pub id: AccountId,
    /// Empty string means keep existing name.
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    // parent_id and type are NOT updatable (tree structure is immutable)
}

impl Command for AccountCommandUpdate {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::AccountCommandUpdate"
    }
}

/// Archive accounts and cascade to all descendants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCommandArchive {
    pub ids: HashSet<AccountId>,
    pub archived_at: DateTime<Utc>,
}

impl Command for AccountCommandArchive {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::AccountCommandArchive"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccountCommandBatch {
    pub create: Vec<AccountCommandCreate>,
    pub update: Vec<AccountCommandUpdate>,
    pub delete: HashSet<AccountId>,
    pub archive: Vec<AccountCommandArchive>,
}

impl Command for AccountCommandBatch {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::AccountCommandBatch"
    }
}
