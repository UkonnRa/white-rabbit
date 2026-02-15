//! Journal is a collection of [`crate::account::Account`]

pub mod command;
mod input;
pub mod repository;
pub mod service;
pub mod specification;

#[cfg(test)]
mod test;
use std::collections::HashSet;

pub use input::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{DomainModel, NonEmpty};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Journal {
    pub id: JournalId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub name: NonEmpty<String>,
    pub description: String,
    pub tags: HashSet<NonEmpty<String>>,
}

impl Journal {
    pub const TYPE: &str = "whiterabbit::domain::Journal";
}
