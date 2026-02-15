use crate::DEFAULT_UNIT;
use crate::account::{AccountId, AccountType};
use crate::error::{Error, Result};
use crate::journal::JournalId;
use crate::record::{
    Amount, Cost, Record, RecordId, RecordItemKind, RecordItemTransaction, RecordItemValidation,
    RecordItems,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use shared::NonEmpty;
use std::collections::HashSet;

pub struct CostInput(String);

impl TryFrom<CostInput> for Cost {
    type Error = Error;
    fn try_from(value: CostInput) -> Result<Self> {
        if let Ok(date) = value.0.parse() {
            Ok(Cost::Date(date))
        } else if let Ok(reference) = value.0.parse() {
            Ok(Cost::Reference(reference))
        } else {
            Ok(Cost::Price(AmountInput(value.0).try_into()?))
        }
    }
}

pub struct AmountInput(String);

impl<S> From<S> for AmountInput
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

impl Default for AmountInput {
    fn default() -> Self {
        Self(format!("0 {}", DEFAULT_UNIT))
    }
}

impl TryFrom<AmountInput> for Amount {
    type Error = Error;
    fn try_from(value: AmountInput) -> Result<Self> {
        let split = value.0.split_whitespace().collect::<Vec<_>>();
        let [amount, unit] = split.as_slice() else {
            return Err(shared::ErrorKind::invalid_format(value.0).convert());
        };

        let amount = amount
            .parse::<Decimal>()
            .map_err(|_| shared::ErrorKind::invalid_format(amount).convert())?;
        let unit = unit.parse::<NonEmpty<String>>().map_err(|e| {
            e.with_resource_type(Amount::TYPE)
                .with_field("unit")
                .convert()
        })?;

        Ok(Amount {
            amount: amount.try_into().map_err(|e: shared::Error| {
                e.with_resource_type(Amount::TYPE)
                    .with_field("amount")
                    .convert()
            })?,
            unit,
        })
    }
}

pub struct RecordItemInput {
    pub account_id: AccountId,
    pub account_type: AccountType,
    pub kind: RecordItemKind,
    pub amount: AmountInput,
    pub description: String,
    pub price: Option<AmountInput>,
    pub cost: HashSet<CostInput>,
}

impl Default for RecordItemInput {
    fn default() -> Self {
        Self {
            account_id: AccountId::default(),
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput::default(),
            description: String::default(),
            price: None,
            cost: HashSet::new(),
        }
    }
}

impl TryFrom<Vec<RecordItemInput>> for RecordItems {
    type Error = Error;
    fn try_from(value: Vec<RecordItemInput>) -> Result<Self> {
        match value.first().map(|item| item.kind) {
            None => Err(shared::ErrorKind::non_empty()
                .with_resource_type(Record::TYPE)
                .with_field("items")
                .convert()),
            Some(RecordItemKind::Transaction)
                if value
                    .iter()
                    .all(|item| item.kind == RecordItemKind::Transaction) =>
            {
                let transactions: Vec<RecordItemTransaction> = value
                    .into_iter()
                    .map(|item| {
                        Ok(RecordItemTransaction {
                            account_id: item.account_id,
                            account_type: item.account_type,
                            amount: item.amount.try_into()?,
                            description: item.description.trim().to_string(),
                            price: item.price.map(|p| p.try_into()).transpose()?,
                            cost: item
                                .cost
                                .into_iter()
                                .map(|c| c.try_into())
                                .collect::<Result<HashSet<_>>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(RecordItems::Transactions(
                    NonEmpty::try_from(transactions).map_err(|e| {
                        e.with_resource_type(Record::TYPE)
                            .with_field("items")
                            .convert()
                    })?,
                ))
            }
            Some(RecordItemKind::Validation)
                if value
                    .iter()
                    .all(|item| item.kind == RecordItemKind::Validation) =>
            {
                let validations: Vec<RecordItemValidation> = value
                    .into_iter()
                    .map(|item| {
                        Ok(RecordItemValidation {
                            account_id: item.account_id,
                            amount: item.amount.try_into()?,
                            description: item.description.trim().to_string(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(RecordItems::Validations(
                    NonEmpty::try_from(validations).map_err(|e| {
                        e.with_resource_type(Record::TYPE)
                            .with_field("items")
                            .convert()
                    })?,
                ))
            }
            _ => Err(shared::ErrorKind::conflicting_values(&[
                RecordItemKind::Transaction.to_string(),
                RecordItemKind::Validation.to_string(),
            ])
            .with_resource_type(Record::TYPE)
            .with_field("items")
            .convert()),
        }
    }
}

#[derive(Default)]
pub struct RecordInput {
    pub id: RecordId,
    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub journal_id: JournalId,
    pub date: NaiveDate,
    pub items: Vec<RecordItemInput>,
    pub description: String,
    pub tags: Vec<String>,
    pub payee: String,
}

impl TryFrom<RecordInput> for Record {
    type Error = Error;
    fn try_from(value: RecordInput) -> Result<Self> {
        let items = RecordItems::try_from(value.items)?;
        let tags = value
            .tags
            .into_iter()
            .filter_map(|tag| NonEmpty::try_from(tag).ok())
            .collect::<HashSet<_>>();

        Ok(Record {
            id: value.id,
            version: value.version,
            created_at: value.created_at,
            last_modified_at: value.last_modified_at,
            journal_id: value.journal_id,
            date: value.date,
            items,
            description: value.description,
            tags,
            payee: value.payee,
        })
    }
}
