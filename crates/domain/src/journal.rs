use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainModel;

use crate::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Journal {
    pub id: JournalId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by_id: Option<UserId>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub last_modified_by_id: Option<UserId>,

    pub name: String,
    pub description: String,

    pub admin_ids: HashSet<UserId>,
}
