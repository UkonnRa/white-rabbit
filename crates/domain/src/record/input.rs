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
use shared::{NonEmpty, NonNegative};
use std::collections::HashSet;

pub enum CostInput {
    Price(AmountInput),
    Date(NaiveDate),
    Reference(String),
}

impl TryFrom<CostInput> for Cost {
    type Error = Error;

    fn try_from(value: CostInput) -> Result<Self> {
        match value {
            CostInput::Price(amount) => Ok(Cost::Price(amount.try_into()?)),
            CostInput::Date(date) => Ok(Cost::Date(date)),
            CostInput::Reference(reference) => Ok(Cost::Reference(
                NonEmpty::try_from(reference).map_err(|_| Error::NonEmpty {
                    typ: "record.cost",
                    field: "reference".to_string(),
                })?,
            )),
        }
    }
}

pub struct AmountInput {
    pub amount: Decimal,
    pub unit: String,
}

impl Default for AmountInput {
    fn default() -> Self {
        Self {
            amount: Decimal::ZERO,
            unit: DEFAULT_UNIT.into(),
        }
    }
}

impl TryFrom<AmountInput> for Amount {
    type Error = Error;

    fn try_from(value: AmountInput) -> Result<Self> {
        Ok(Amount {
            amount: NonNegative::try_from(value.amount).map_err(|_| Error::NonNegativeValue {
                typ: Record::TYPE,
                field: "amount".to_string(),
                current: value.amount.to_string(),
            })?,
            unit: NonEmpty::try_from(value.unit).map_err(|_| Error::NonEmpty {
                typ: Record::TYPE,
                field: "unit".to_string(),
            })?,
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
            None => Err(Error::NonEmpty {
                typ: Record::TYPE,
                field: "items".to_string(),
            }),
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
                    NonEmpty::try_from(transactions).map_err(|_| Error::NonEmpty {
                        typ: Record::TYPE,
                        field: "items".to_string(),
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
                    NonEmpty::try_from(validations).map_err(|_| Error::NonEmpty {
                        typ: Record::TYPE,
                        field: "items".to_string(),
                    })?,
                ))
            }
            _ => Err(Error::CannotExistSameTime {
                typ: Record::TYPE,
                field: "items".to_string(),
                values: vec![
                    RecordItemKind::Transaction.to_string(),
                    RecordItemKind::Validation.to_string(),
                ],
            }),
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
            .map(|tag| {
                NonEmpty::try_from(tag).map_err(|_| Error::NonEmpty {
                    typ: Record::TYPE,
                    field: "tags".to_string(),
                })
            })
            .collect::<Result<HashSet<_>>>()?;

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
