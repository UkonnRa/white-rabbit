use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::Specification;

use crate::journal::JournalId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalSpecification {
    Id(HashSet<JournalId>),
    Tag(HashSet<String>),
    FullText(String),
}

impl Specification for JournalSpecification {}
