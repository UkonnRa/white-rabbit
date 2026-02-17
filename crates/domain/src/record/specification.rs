use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use shared::{Specification, SpecificationExpression};

use crate::account::AccountId;
use crate::journal::JournalId;
use crate::record::{RecordId, RecordItemKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordSpecification {
    Id(HashSet<RecordId>),
    JournalId(HashSet<JournalId>),
    /// Records whose items reference any of the given accounts.
    AccountId(HashSet<AccountId>),
    /// Exact date match.
    Date(HashSet<NaiveDate>),
    /// Records on or after this date.
    DateFrom(NaiveDate),
    /// Records on or before this date.
    DateTo(NaiveDate),
    Payee(HashSet<String>),
    Tag(HashSet<String>),
    ItemKind(RecordItemKind),
    FullText(String),
}

impl Specification for RecordSpecification {}

/// Composable record specification expression.
pub type RecordSpec = SpecificationExpression<RecordSpecification>;

impl RecordSpecification {
    pub fn id(id: impl Into<RecordId>) -> RecordSpec {
        RecordSpec::Leaf(Self::Id(HashSet::from([id.into()])))
    }

    pub fn ids(ids: impl IntoIterator<Item = impl Into<RecordId>>) -> RecordSpec {
        RecordSpec::Leaf(Self::Id(ids.into_iter().map(Into::into).collect()))
    }

    pub fn journal_id(id: impl Into<JournalId>) -> RecordSpec {
        RecordSpec::Leaf(Self::JournalId(HashSet::from([id.into()])))
    }

    pub fn account_id(id: impl Into<AccountId>) -> RecordSpec {
        RecordSpec::Leaf(Self::AccountId(HashSet::from([id.into()])))
    }

    pub fn account_ids(ids: impl IntoIterator<Item = impl Into<AccountId>>) -> RecordSpec {
        RecordSpec::Leaf(Self::AccountId(ids.into_iter().map(Into::into).collect()))
    }

    pub fn date(date: NaiveDate) -> RecordSpec {
        RecordSpec::Leaf(Self::Date(HashSet::from([date])))
    }

    pub fn dates(dates: impl IntoIterator<Item = NaiveDate>) -> RecordSpec {
        RecordSpec::Leaf(Self::Date(dates.into_iter().collect()))
    }

    pub fn date_from(date: NaiveDate) -> RecordSpec {
        RecordSpec::Leaf(Self::DateFrom(date))
    }

    pub fn date_to(date: NaiveDate) -> RecordSpec {
        RecordSpec::Leaf(Self::DateTo(date))
    }

    pub fn payee(payee: impl Into<String>) -> RecordSpec {
        RecordSpec::Leaf(Self::Payee(HashSet::from([payee.into()])))
    }

    pub fn payees(payees: impl IntoIterator<Item = impl Into<String>>) -> RecordSpec {
        RecordSpec::Leaf(Self::Payee(payees.into_iter().map(Into::into).collect()))
    }

    pub fn tag(tag: impl Into<String>) -> RecordSpec {
        RecordSpec::Leaf(Self::Tag(HashSet::from([tag.into()])))
    }

    pub fn tags(tags: impl IntoIterator<Item = impl Into<String>>) -> RecordSpec {
        RecordSpec::Leaf(Self::Tag(tags.into_iter().map(Into::into).collect()))
    }

    pub fn item_kind(kind: RecordItemKind) -> RecordSpec {
        RecordSpec::Leaf(Self::ItemKind(kind))
    }

    pub fn full_text(query: impl Into<String>) -> RecordSpec {
        RecordSpec::Leaf(Self::FullText(query.into()))
    }
}
