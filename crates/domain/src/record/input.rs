use crate::account::AccountId;
use crate::error::Result;
use crate::record::{
    Amount, Cost, Record, RecordItemKind, RecordItemTransaction, RecordItemValidation,
    RecordItems,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashSet;

pub enum CostInput {
    Price(AmountInput),
    Date(NaiveDate),
    Reference(String),
}

impl TryFrom<CostInput> for Cost {
    type Error = crate::error::Error;

    fn try_from(value: CostInput) -> Result<Self> {
        match value {
            CostInput::Price(amount) => Ok(Cost::Price(amount.try_into()?)),
            CostInput::Date(date) => Ok(Cost::Date(date)),
            CostInput::Reference(reference) => match reference.trim() {
                "" => Err(crate::error::Error::NonEmpty {
                    typ: "record.cost",
                    field: "reference".to_string(),
                }),
                _ => Ok(Cost::Reference(reference.trim().to_string())),
            },
        }
    }
}

pub struct AmountInput {
    pub amount: Decimal,
    pub unit: String,
}

impl TryFrom<AmountInput> for Amount {
    type Error = crate::error::Error;

    fn try_from(value: AmountInput) -> Result<Self> {
        if value.amount < Decimal::ZERO {
            Err(crate::error::Error::NonNegativeValue {
                typ: Record::TYPE,
                field: "amount".to_string(),
                current: value.amount.to_string(),
            })
        } else {
            match value.unit.trim() {
                "" => Err(crate::error::Error::NonEmpty {
                    typ: Record::TYPE,
                    field: "unit".to_string(),
                }),
                _ => Ok(Amount {
                    amount: value.amount,
                    unit: value.unit.trim().to_string(),
                }),
            }
        }
    }
}

pub struct RecordItemInput {
    pub account_id: AccountId,
    pub kind: RecordItemKind,
    pub amount: AmountInput,
    pub description: String,
    pub price: Option<AmountInput>,
    pub cost: HashSet<Cost>,
}

impl TryFrom<Vec<RecordItemInput>> for RecordItems {
    type Error = crate::error::Error;

    fn try_from(value: Vec<RecordItemInput>) -> Result<Self> {
        match value.first().map(|item| item.kind) {
            None => Err(crate::error::Error::NonEmpty {
                typ: Record::TYPE,
                field: "items".to_string(),
            }),
            Some(RecordItemKind::Transaction)
                if value
                    .iter()
                    .all(|item| item.kind == RecordItemKind::Transaction) =>
            {
                Ok(RecordItems::Transactions(
                    value
                        .into_iter()
                        .map(|item| {
                            Ok(RecordItemTransaction {
                                account_id: item.account_id,
                                amount: item.amount.try_into()?,
                                description: item.description.trim().to_string(),
                                price: if let Some(price) = item.price {
                                    Some(price.try_into()?)
                                } else {
                                    None
                                },
                                cost: item.cost,
                            })
                        })
                        .collect::<Result<Vec<_>>>()?,
                ))
            }
            Some(RecordItemKind::Validation)
                if value
                    .iter()
                    .all(|item| item.kind == RecordItemKind::Validation) =>
            {
                Ok(RecordItems::Validations(
                    value
                        .into_iter()
                        .map(|item| {
                            Ok(RecordItemValidation {
                                account_id: item.account_id,
                                amount: item.amount.try_into()?,
                                description: item.description.trim().to_string(),
                            })
                        })
                        .collect::<Result<Vec<_>>>()?,
                ))
            }
            _ => Err(crate::error::Error::CannotExistSameTime {
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
