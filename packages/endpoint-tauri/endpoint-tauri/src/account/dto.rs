use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::EntityId;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub journal_id: String,
    pub parent_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: HashSet<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountFilter {
    pub id: Option<String>,
    pub journal_id: Option<String>,
    pub parent_id: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub tag: Option<String>,
    pub full_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub id: String,
    pub journal_id: String,
    pub parent_id: Option<String>,
    pub r#type: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AccountResponse {
    pub fn from_account(account: &domain::account::Account) -> Self {
        Self {
            id: account.id.value().to_string(),
            journal_id: account.journal_id.value().to_string(),
            parent_id: account.parent_id.as_ref().map(|id| id.value().to_string()),
            r#type: account.r#type.to_string(),
            name: account.name.to_string(),
            description: account.description.clone(),
            tags: account.tags.iter().map(|t| t.to_string()).collect(),
            created_at: account.created_at,
            last_modified_at: account.last_modified_at,
            archived_at: account.archived_at,
        }
    }
}
