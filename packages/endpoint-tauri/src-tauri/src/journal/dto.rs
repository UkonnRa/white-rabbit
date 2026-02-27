use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::EntityId;

#[derive(Debug, Deserialize)]
pub struct CreateJournalRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: HashSet<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalRequest {
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct JournalFilter {
    pub id: Option<String>,
    pub name: Option<String>,
    pub tag: Option<String>,
    #[serde(rename = "fullText")]
    pub full_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JournalResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl JournalResponse {
    pub fn from_journal(journal: &domain::journal::Journal) -> Self {
        Self {
            id: journal.id.value().to_string(),
            name: journal.name.to_string(),
            description: journal.description.clone(),
            tags: journal.tags.iter().map(|t| t.to_string()).collect(),
            created_at: journal.created_at,
            last_modified_at: journal.last_modified_at,
            archived_at: journal.archived_at,
        }
    }
}
