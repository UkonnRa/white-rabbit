use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use shared::Command;

use crate::account::AccountId;
use crate::journal::JournalId;
use crate::record::{RecordId, RecordItemKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordCommand {
    Create(RecordCommandCreate),
    Update(RecordCommandUpdate),
    Delete(HashSet<RecordId>),
    Batch(RecordCommandBatch),
}

impl Command for RecordCommand {
    fn command_type(&self) -> &'static str {
        match self {
            RecordCommand::Create(c) => c.command_type(),
            RecordCommand::Update(c) => c.command_type(),
            RecordCommand::Delete(_) => "whiterabbit::command::RecordCommandDelete",
            RecordCommand::Batch(c) => c.command_type(),
        }
    }
}

/// A single item (posting) within a record command.
///
/// The `account_type` is not included — the service resolves it by looking
/// up the account. This matches Beancount where you specify the account
/// path and the type is implicit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordCommandItem {
    pub account_id: AccountId,
    /// Amount in `"<number> <unit>"` format, e.g. `"100 USD"`.
    pub amount: String,
    pub description: String,
    /// Optional `@` price in `"<number> <unit>"` format, e.g. `"1.09 CAD"`.
    pub price: Option<String>,
    /// Cost specs in `{}` format — each entry is parsed as a date, reference, or price.
    pub cost: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordCommandCreate {
    pub journal_id: JournalId,
    pub date: NaiveDate,
    pub kind: RecordItemKind,
    pub items: Vec<RecordCommandItem>,
    pub description: String,
    pub tags: HashSet<String>,
    pub payee: String,
}

impl Command for RecordCommandCreate {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::RecordCommandCreate"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordCommandUpdate {
    pub id: RecordId,
    /// `None` = keep existing date.
    pub date: Option<NaiveDate>,
    /// `None` = keep existing items; `Some` = replace all items.
    pub items: Option<Vec<RecordCommandItem>>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub payee: Option<String>,
    // journal_id and kind are immutable after creation
}

impl Command for RecordCommandUpdate {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::RecordCommandUpdate"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RecordCommandBatch {
    pub create: Vec<RecordCommandCreate>,
    pub update: Vec<RecordCommandUpdate>,
    pub delete: HashSet<RecordId>,
}

impl Command for RecordCommandBatch {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::RecordCommandBatch"
    }
}
