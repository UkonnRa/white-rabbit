use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::{Specification, SpecificationExpression};

use crate::account::{AccountId, AccountType};
use crate::journal::JournalId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountSpecification {
    Id(HashSet<AccountId>),
    JournalId(HashSet<JournalId>),
    ParentId(HashSet<AccountId>),
    Name(HashSet<String>),
    Type(HashSet<AccountType>),
    Tag(HashSet<String>),
    FullText(String),
    /// `true` = only archived accounts, `false` = only active (non-archived).
    Archived(bool),
}

impl Specification for AccountSpecification {}

/// Composable account specification expression.
pub type AccountSpec = SpecificationExpression<AccountSpecification>;

impl AccountSpecification {
    pub fn id(id: impl Into<AccountId>) -> AccountSpec {
        AccountSpec::Leaf(Self::Id(HashSet::from([id.into()])))
    }

    pub fn ids(ids: impl IntoIterator<Item = impl Into<AccountId>>) -> AccountSpec {
        AccountSpec::Leaf(Self::Id(ids.into_iter().map(Into::into).collect()))
    }

    pub fn journal_id(id: impl Into<JournalId>) -> AccountSpec {
        AccountSpec::Leaf(Self::JournalId(HashSet::from([id.into()])))
    }

    pub fn parent_id(id: impl Into<AccountId>) -> AccountSpec {
        AccountSpec::Leaf(Self::ParentId(HashSet::from([id.into()])))
    }

    pub fn parent_ids(ids: impl IntoIterator<Item = impl Into<AccountId>>) -> AccountSpec {
        AccountSpec::Leaf(Self::ParentId(ids.into_iter().map(Into::into).collect()))
    }

    pub fn name(name: impl Into<String>) -> AccountSpec {
        AccountSpec::Leaf(Self::Name(HashSet::from([name.into()])))
    }

    pub fn names(names: impl IntoIterator<Item = impl Into<String>>) -> AccountSpec {
        AccountSpec::Leaf(Self::Name(names.into_iter().map(Into::into).collect()))
    }

    pub fn account_type(t: AccountType) -> AccountSpec {
        AccountSpec::Leaf(Self::Type(HashSet::from([t])))
    }

    pub fn tag(tag: impl Into<String>) -> AccountSpec {
        AccountSpec::Leaf(Self::Tag(HashSet::from([tag.into()])))
    }

    pub fn tags(tags: impl IntoIterator<Item = impl Into<String>>) -> AccountSpec {
        AccountSpec::Leaf(Self::Tag(tags.into_iter().map(Into::into).collect()))
    }

    pub fn full_text(query: impl Into<String>) -> AccountSpec {
        AccountSpec::Leaf(Self::FullText(query.into()))
    }

    pub fn archived() -> AccountSpec {
        AccountSpec::Leaf(Self::Archived(true))
    }

    pub fn active() -> AccountSpec {
        AccountSpec::Leaf(Self::Archived(false))
    }
}
