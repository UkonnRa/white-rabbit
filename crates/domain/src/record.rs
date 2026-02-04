mod input;
mod value;

pub use input::*;
pub use value::*;

use crate::journal::JournalId;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainModel;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Record {
    pub id: RecordId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,

    pub journal_id: JournalId,
    // For [Validations], the validation always happens at the end of the date, after all the transactions are calculated
    pub date: NaiveDate,
    pub items: RecordItems,

    pub description: String,
    pub tags: HashSet<String>,
    pub payee: String,
}

impl Record {
    pub const TYPE: &str = "whiterabbit::domain::Record";
}
