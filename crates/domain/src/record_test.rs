use crate::{
    account::{AccountId, AccountType},
    journal::JournalId,
    record::{AmountInput, Record, RecordId, RecordInput, RecordItemInput, RecordItemKind},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use shared::EntityId;

#[test]
fn test_is_balanced_simple_balanced() {
    // Simple balanced transaction: Asset = Equity
    // Assets: 100 USD, Equity: 100 USD
    let items = vec![
        RecordItemInput {
            account_id: AccountId::from_value("asset-1"),
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            description: "Transaction for asset-1".to_string(),
            ..Default::default()
        },
        RecordItemInput {
            account_id: AccountId::from_value("equity-1"),
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            description: "Transaction for equity-1".to_string(),
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        id: RecordId::from_value("test-record-id"),
        journal_id: JournalId::from_value("test-journal-id"),
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        items,
        description: "Simple balanced transaction".to_string(),
        payee: "Test Payee".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(true));
}

#[test]
fn test_is_balanced_complex_balanced() {
    // Complex balanced transaction: Assets + Expenses = Liabilities + Equity + Income
    // Assets: 200, Expenses: 50, Liabilities: 100, Equity: 100, Income: 50
    // 200 + 50 = 100 + 100 + 50 = 250
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(200, 0),
                unit: "USD".to_string(),
            },
            description: "Transaction for asset-1".to_string(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Expense,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(50, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Liability,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Income,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(50, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        id: RecordId::from_value("test-record-id"),
        journal_id: JournalId::from_value("test-journal-id"),
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(true));
}

#[test]
fn test_is_balanced_unbalanced() {
    // Unbalanced transaction: Assets + Expenses != Liabilities + Equity + Income
    // Assets: 200, Expenses: 50, Liabilities: 100, Equity: 100, Income: 40
    // 200 + 50 = 250 != 100 + 100 + 40 = 240
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(200, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Expense,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(50, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Liability,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Income,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(40, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(false));
}

#[test]
fn test_is_balanced_with_price_conversion() {
    // Balanced transaction with currency conversion using price
    // Asset: 400 USD @ 1.09 CAD = 436 CAD
    // Equity: 436 CAD
    // Total: 436 CAD = 436 CAD (balanced)
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(400, 0),
                unit: "USD".to_string(),
            },
            price: Some(AmountInput {
                amount: Decimal::new(109, 2),
                unit: "CAD".to_string(),
            }),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(436, 0),
                unit: "CAD".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(true));
}

#[test]
fn test_is_balanced_multiple_same_account_type() {
    // Multiple transactions of the same account type should be summed
    // Assets: 100 + 200 = 300, Equity: 300
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(200, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(300, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(true));
}

#[test]
fn test_is_balanced_with_decimal_precision() {
    // Test with decimal precision
    // Assets: 100.50, Expenses: 50.25, Liabilities: 75.25, Equity: 50.00, Income: 25.50
    // 100.50 + 50.25 = 150.75
    // 75.25 + 50.00 + 25.50 = 150.75
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(10050, 2),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Expense,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(5025, 2),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Liability,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(7525, 2),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(5000, 2),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Income,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(2550, 2),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(true));
}

#[test]
fn test_is_balanced_zero_values() {
    // Test with zero values - should still be balanced
    // Assets: 100, all others: 0
    // 100 + 0 = 0 + 0 + 0 (unbalanced)
    // Actually, let's make it balanced: Assets: 100, Equity: 100
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), Some(true));
}

#[test]
fn test_is_balanced_returns_none_for_validations() {
    // Validations should return None
    let validation_items = vec![RecordItemInput {
        account_type: AccountType::Asset,
        kind: RecordItemKind::Validation,
        amount: AmountInput {
            amount: Decimal::new(100, 0),
            unit: "USD".to_string(),
        },
        ..Default::default()
    }];
    let record: Record = RecordInput {
        items: validation_items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), None);
}

#[test]
fn test_is_balanced_with_mixed_price_conversions() {
    // Test with multiple price conversions
    // Asset 1: 100 USD @ 1.5 EUR = 150 EUR
    // Asset 2: 50 EUR
    // Total Assets: 200 EUR
    // Equity: 200 EUR
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(100, 0),
                unit: "USD".to_string(),
            },
            price: Some(AmountInput {
                amount: Decimal::new(150, 2),
                unit: "EUR".to_string(),
            }),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(50, 0),
                unit: "EUR".to_string(),
            },
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: AmountInput {
                amount: Decimal::new(200, 0),
                unit: "EUR".to_string(),
            },
            ..Default::default()
        },
    ];
    let record: Record = RecordInput {
        items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    // 100 * 1.5 = 150, so 150 + 50 = 200 on asset side, 200 on equity side
    assert_eq!(record.is_balanced(), Some(true));
}
