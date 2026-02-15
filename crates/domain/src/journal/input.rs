use std::collections::HashSet;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use shared::{Error, NonEmpty, Result};

use crate::journal::{Journal, JournalId};

#[derive(Default)]
pub struct JournalInput {
    pub id: JournalId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl TryFrom<JournalInput> for Journal {
    type Error = Error;

    fn try_from(value: JournalInput) -> Result<Self> {
        Ok(Journal {
            id: value.id,
            version: value.version,
            created_at: value.created_at,
            last_modified_at: value.last_modified_at,
            archived_at: value.archived_at,

            name: NonEmpty::try_from(value.name)
                .map_err(|e| e.with_resource_type(Journal::TYPE).with_field("name"))?,
            description: value.description,
            tags: value
                .tags
                .into_iter()
                .filter_map_ok(|tag| {
                    NonEmpty::try_from(tag).ok()
                })
                .collect::<HashSet<_>>(),
        })
    }
}
