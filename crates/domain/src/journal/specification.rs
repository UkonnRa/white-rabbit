use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::{Specification, SpecificationEvaluator, SpecificationExpression};

use crate::journal::{Journal, JournalId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalSpecification {
    Id(HashSet<JournalId>),
    Name(HashSet<String>),
    Tag(HashSet<String>),
    FullText(String),
}

impl Specification for JournalSpecification {}

impl SpecificationEvaluator<Journal> for JournalSpecification {
    fn matches(&self, j: &Journal) -> bool {
        match self {
            Self::Id(ids) => ids.contains(&j.id),
            Self::Name(names) => {
                let name = j.name.to_string().to_lowercase();
                names
                    .iter()
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .any(|s| s == name)
            }
            Self::Tag(tags) => {
                let lower_tags: HashSet<String> = tags.iter().map(|t| t.to_lowercase()).collect();
                j.tags
                    .iter()
                    .any(|t| lower_tags.contains(&t.to_string().to_lowercase()))
            }
            Self::FullText(query) => {
                let q = query.trim().to_lowercase();
                j.name.to_string().to_lowercase().contains(&q)
                    || j.description.to_lowercase().contains(&q)
                    || j.tags
                        .iter()
                        .any(|t| t.to_string().to_lowercase().contains(&q))
            }
        }
    }
}

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
