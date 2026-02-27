//! Journal is a collection of [`crate::account::Account`]

pub mod command;
pub mod event;
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
use shared::{AggregateRoot, DomainModel, NonEmpty};

use crate::journal::event::JournalEvent;

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

impl AggregateRoot for Journal {
    type Event = JournalEvent;

    fn apply(&mut self, event: &JournalEvent) {
        match event {
            JournalEvent::Created(e) => {
                self.id = e.id.clone();
                self.name = e.name.parse().unwrap();
                self.description = e.description.clone();
                self.tags = e.tags.iter().map(|t| t.parse().unwrap()).collect();
                self.created_at = Some(e.created_at);
                self.version = 0;
            }
            JournalEvent::Updated(e) => {
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
            JournalEvent::Deleted(_) => {}
        }
    }
}
