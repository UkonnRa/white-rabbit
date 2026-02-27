pub mod command;
mod context;
pub mod event;
mod input;
pub mod repository;
pub mod service;
pub mod specification;

#[cfg(test)]
mod test;

pub use context::*;
pub use input::*;

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{AggregateRoot, DomainModel, NonEmpty};

use crate::account::event::*;
use crate::journal::JournalId;

/// The Account is a tree, the root Account is one of the 5 types: Asset, Liability, Equity, Income, Expense
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Account {
    pub id: AccountId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub journal_id: JournalId,
    pub parent_id: Option<AccountId>,
    // The type of children should be the same as the parent's type
    // For each Journal, we always have 5 roots: Asset, Liability, Equity, Income, Expense
    pub r#type: AccountType,

    pub name: NonEmpty<String>,
    pub description: String,
    pub tags: HashSet<NonEmpty<String>>,
}

impl Account {
    /// The 5 reserved root account names (one per `AccountType`).
    pub const RESERVED_ROOT_NAMES: [&str; 5] =
        ["Asset", "Liability", "Equity", "Income", "Expense"];

    /// Returns true if the given name is one of the reserved root names
    /// (case-insensitive).
    pub fn is_reserved_name(name: &str) -> bool {
        Self::RESERVED_ROOT_NAMES
            .iter()
            .any(|r| r.eq_ignore_ascii_case(name))
    }

    /// Returns true if this account is archived.
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    /// Returns true if this is a root account (no parent).
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl AggregateRoot for Account {
    type Event = AccountEvent;

    fn apply(&mut self, event: &AccountEvent) {
        match event {
            AccountEvent::Created(e) => {
                self.id = e.id.clone();
                self.journal_id = e.journal_id.clone();
                self.parent_id = e.parent_id.clone();
                self.r#type = e.r#type;
                // apply is validation-free: unwrap is safe because events
                // are only produced from already-validated command state.
                self.name = e.name.parse().unwrap();
                self.description = e.description.clone();
                self.tags = e.tags.iter().map(|t| t.parse().unwrap()).collect();
                self.created_at = Some(e.created_at);
                self.version = 0;
            }
            AccountEvent::Updated(e) => {
                if let Some(name) = &e.name {
                    self.name = name.parse().unwrap();
                }
                if let Some(desc) = &e.description {
                    self.description = desc.clone();
                }
                if let Some(tags) = &e.tags {
                    self.tags = tags.iter().map(|t| t.parse().unwrap()).collect();
                }
                self.last_modified_at = Some(e.last_modified_at);
            }
            AccountEvent::Archived(e) => {
                self.archived_at = Some(e.archived_at);
            }
            AccountEvent::Deleted(_) => {}
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default, strum::Display,
)]
pub enum AccountType {
    #[default]
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}
