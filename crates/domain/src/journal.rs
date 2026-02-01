use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainModel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Journal {
    pub id: JournalId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub name: String,
    pub description: String,
}
