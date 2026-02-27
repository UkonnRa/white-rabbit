use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainEvent;

use crate::journal::JournalId;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalEvent {
    Created(JournalCreated),
    Updated(JournalUpdated),
    Deleted(JournalDeleted),
}

impl DomainEvent for JournalEvent {
    fn event_type(&self) -> &'static str {
        match self {
            JournalEvent::Created(_) => "whiterabbit::event::JournalCreated",
            JournalEvent::Updated(_) => "whiterabbit::event::JournalUpdated",
            JournalEvent::Deleted(_) => "whiterabbit::event::JournalDeleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCreated {
    pub id: JournalId,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalUpdated {
    pub id: JournalId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub last_modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalDeleted {
    pub id: JournalId,
}
