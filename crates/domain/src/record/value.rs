use std::collections::HashSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::account::AccountId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordItems {
    Transactions(Vec<RecordItemTransaction>),
    Validations(Vec<RecordItemValidation>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordItemTransaction {
    pub account_id: AccountId,
    pub amount: Amount,
    // The [@ part](https://beancount.github.io/docs/beancount_language_syntax.html#costs-and-prices) in beancount,  e.g.:
    //   2012-11-03 * "Transfer to account in Canada"
    //     Assets:MyBank:Checking            -400.00 USD @ 1.09 CAD
    //     Assets:FR:SocGen:Checking          436.01 CAD
    pub price: Option<Amount>,
    // The [{} part](https://beancount.github.io/docs/beancount_language_syntax.html#costs-and-prices) in beancount,  e.g.:
    //   2014-02-11 * "Bought shares of S&P 500"
    //     Assets:ETrade:IVV                10 IVV {183.07 USD}
    //     Assets:ETrade:Cash         -1830.70 USD
    pub cost: HashSet<Cost>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct RecordItemValidation {
    pub account_id: AccountId,
    // the expected amount of the validation on this account
    pub amount: Amount,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, strum::Display)]
pub enum RecordItemKind {
    Transaction,
    Validation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Amount {
    pub amount: Decimal,
    // currency, stock symbol, commodity, etc.
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Cost {
    Price(Amount),
    Date(NaiveDate),
    Reference(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, strum::Display)]
pub enum CostKind {
    Price,
    Date,
    Reference,
}
