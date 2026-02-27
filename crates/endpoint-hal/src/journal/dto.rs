use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::EntityId;

use crate::hal::HalResource;

// ── Request DTOs ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CreateJournalRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateJournalRequest {
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
}

// ── Response DTO ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct JournalProperties {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl JournalProperties {
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

    pub fn into_hal(self) -> HalResource<Self> {
        let href = format!("/journals/{}", self.id);
        HalResource::new(href, self)
    }
}

impl From<&domain::journal::Journal> for JournalProperties {
    fn from(journal: &domain::journal::Journal) -> Self {
        Self::from_journal(journal)
    }
}
