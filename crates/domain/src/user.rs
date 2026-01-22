use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::DomainModel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct User {
    pub id: UserId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by_id: Option<UserId>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub last_modified_by_id: Option<UserId>,

    pub name: String,
    pub login_name: String,
    pub global_admin: bool,
}
