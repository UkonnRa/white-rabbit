pub mod command;
pub mod event;
mod input;
pub mod repository;
pub mod service;
pub mod specification;
#[cfg(test)]
mod test;
mod value;

pub use input::*;
pub use value::*;

use crate::record::event::RecordEvent;
use crate::{account::AccountType, journal::JournalId};
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::{AggregateRoot, DomainModel, NonEmpty};
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

impl AggregateRoot for Record {
    type Event = RecordEvent;

    fn apply(&mut self, event: &RecordEvent) {
        match event {
            RecordEvent::Created(e) => {
                self.id = e.id.clone();
                self.journal_id = e.journal_id.clone();
                self.date = e.date;
                self.items = e.items.clone();
                self.description = e.description.clone();
                self.tags = e.tags.iter().map(|t| t.parse().unwrap()).collect();
                self.payee = e.payee.clone();
                self.created_at = Some(e.created_at);
                self.version = 0;
            }
            RecordEvent::Updated(e) => {
                if let Some(date) = e.date {
                    self.date = date;
                }
                if let Some(items) = &e.items {
                    self.items = items.clone();
                }
                if let Some(desc) = &e.description {
                    self.description = desc.clone();
                }
                if let Some(tags) = &e.tags {
                    self.tags = tags.iter().map(|t| t.parse().unwrap()).collect();
                }
                if let Some(payee) = &e.payee {
                    self.payee = payee.clone();
                }
                self.last_modified_at = Some(e.last_modified_at);
            }
            RecordEvent::Deleted(_) => {}
        }
    }
}

impl Record {
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
