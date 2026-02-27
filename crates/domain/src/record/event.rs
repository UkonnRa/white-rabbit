use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainEvent;

use crate::journal::JournalId;
use crate::record::{RecordId, RecordItems};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordEvent {
    Created(RecordCreated),
    Updated(RecordUpdated),
    Deleted(RecordDeleted),
}

impl DomainEvent for RecordEvent {
    fn event_type(&self) -> &'static str {
        match self {
            RecordEvent::Created(_) => "whiterabbit::event::RecordCreated",
            RecordEvent::Updated(_) => "whiterabbit::event::RecordUpdated",
            RecordEvent::Deleted(_) => "whiterabbit::event::RecordDeleted",
        }
    }
}

/// Carries the fully resolved record state at creation time.
/// Items already contain the looked-up account types and parsed amounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordCreated {
    pub id: RecordId,
    pub journal_id: JournalId,
    pub date: NaiveDate,
    pub items: RecordItems,
    pub description: String,
    pub tags: HashSet<String>,
    pub payee: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordUpdated {
    pub id: RecordId,
    pub date: Option<NaiveDate>,
    pub items: Option<RecordItems>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub payee: Option<String>,
    pub last_modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordDeleted {
    pub id: RecordId,
}
