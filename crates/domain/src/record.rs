mod input;
#[cfg(test)]
mod test;
mod value;

pub use input::*;
pub use value::*;

use crate::{account::AccountType, journal::JournalId};
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::{DomainModel, NonEmpty};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DomainModel)]
pub struct Record {
    pub id: RecordId,

    pub version: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,

    pub journal_id: JournalId,
    // For [Validations], the validation always happens at the end of the date, after all the transactions are calculated
    pub date: NaiveDate,
    pub items: RecordItems,

    pub description: String,
    pub tags: HashSet<NonEmpty<String>>,
    pub payee: String,
}

impl Record {
    pub const TYPE: &str = "whiterabbit::domain::Record";

    // is the transaction record balanced? Balanced means Assets + Expenses = Liabilities + Equity + Income
    // if the record is Validations, return None
    // TODO: this is not correct, the cost, especially the currency exchange rate, should be recorded in another data structure.
    //   Not all data is integrated into one record.
    pub fn is_balanced(&self) -> Option<bool> {
        match &self.items {
            RecordItems::Transactions(transactions) => {
                let group_by_type = transactions
                    .iter()
                    .chunk_by(|transaction| transaction.account_type)
                    .into_iter()
                    .map(|(account_type, transactions)| {
                        let total_amount = transactions
                            .into_iter()
                            .map(|transaction| transaction.amount_number())
                            .sum();
                        (account_type, total_amount)
                    })
                    .collect::<HashMap<AccountType, Decimal>>();
                let total_assets = group_by_type
                    .get(&AccountType::Asset)
                    .unwrap_or(&Decimal::ZERO);
                let total_expenses = group_by_type
                    .get(&AccountType::Expense)
                    .unwrap_or(&Decimal::ZERO);
                let total_liabilities = group_by_type
                    .get(&AccountType::Liability)
                    .unwrap_or(&Decimal::ZERO);
                let total_equity = group_by_type
                    .get(&AccountType::Equity)
                    .unwrap_or(&Decimal::ZERO);
                let total_income = group_by_type
                    .get(&AccountType::Income)
                    .unwrap_or(&Decimal::ZERO);
                Some(
                    total_assets + total_expenses
                        == total_liabilities + total_equity + total_income,
                )
            }
            RecordItems::Validations(_) => None,
        }
    }
}
