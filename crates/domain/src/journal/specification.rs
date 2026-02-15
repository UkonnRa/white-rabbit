use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::{Specification, SpecificationExpression};

use crate::journal::JournalId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalSpecification {
    Id(HashSet<JournalId>),
    Name(HashSet<String>),
    Tag(HashSet<String>),
    FullText(String),
}

impl Specification for JournalSpecification {}

/// Composable journal specification expression.
pub type JournalSpec = SpecificationExpression<JournalSpecification>;

impl JournalSpecification {
    pub fn id(id: impl Into<JournalId>) -> JournalSpec {
        JournalSpec::Leaf(Self::Id(HashSet::from([id.into()])))
    }

    pub fn ids(ids: impl IntoIterator<Item = impl Into<JournalId>>) -> JournalSpec {
        JournalSpec::Leaf(Self::Id(ids.into_iter().map(Into::into).collect()))
    }

    pub fn name(name: impl Into<String>) -> JournalSpec {
        JournalSpec::Leaf(Self::Name(HashSet::from([name.into()])))
    }

    pub fn names(names: impl IntoIterator<Item = impl Into<String>>) -> JournalSpec {
        JournalSpec::Leaf(Self::Name(names.into_iter().map(Into::into).collect()))
    }

    pub fn tag(tag: impl Into<String>) -> JournalSpec {
        JournalSpec::Leaf(Self::Tag(HashSet::from([tag.into()])))
    }

    pub fn tags(tags: impl IntoIterator<Item = impl Into<String>>) -> JournalSpec {
        JournalSpec::Leaf(Self::Tag(tags.into_iter().map(Into::into).collect()))
    }

    pub fn full_text(query: impl Into<String>) -> JournalSpec {
        JournalSpec::Leaf(Self::FullText(query.into()))
    }
}
