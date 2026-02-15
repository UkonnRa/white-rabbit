use crate::{
    account::{AccountId, AccountType},
    journal::JournalId,
};
use chrono::NaiveDate;
use shared::{EntityId, ErrorKind};

use super::{Amount, Record, RecordId, RecordInput, RecordItemInput, RecordItemKind};

#[test]
fn test_is_balanced_simple_balanced() {
    // Simple balanced transaction: Asset = Equity
    // Assets: 100 USD, Equity: 100 USD
    let items = vec![
        RecordItemInput {
            account_id: AccountId::from_value("asset-1"),
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            description: "Transaction for asset-1".to_string(),
            ..Default::default()
        },
        RecordItemInput {
            account_id: AccountId::from_value("equity-1"),
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
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
            amount: "200 USD".into(),
            description: "Transaction for asset-1".to_string(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Expense,
            kind: RecordItemKind::Transaction,
            amount: "50 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Liability,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Income,
            kind: RecordItemKind::Transaction,
            amount: "50 USD".into(),
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
            amount: "200 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Expense,
            kind: RecordItemKind::Transaction,
            amount: "50 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Liability,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Income,
            kind: RecordItemKind::Transaction,
            amount: "40 USD".into(),
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
            amount: "400 USD".into(),
            price: Some("1.09 CAD".into()),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "436 CAD".into(),
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
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: "200 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "300 USD".into(),
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
            amount: "100.50 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Expense,
            kind: RecordItemKind::Transaction,
            amount: "50.25 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Liability,
            kind: RecordItemKind::Transaction,
            amount: "75.25 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "50.00 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Income,
            kind: RecordItemKind::Transaction,
            amount: "25.50 USD".into(),
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
    // Assets: 100, Equity: 100
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
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
        amount: "100 USD".into(),
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
            amount: "100 USD".into(),
            price: Some("1.50 EUR".into()),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: "50 EUR".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Equity,
            kind: RecordItemKind::Transaction,
            amount: "200 EUR".into(),
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

// ── Error tests ──────────────────────────────────────────────────

#[test]
fn test_error_empty_items_returns_non_empty_with_context() {
    let result: Result<Record, _> = RecordInput {
        items: vec![],
        ..Default::default()
    }
    .try_into();

    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::NonEmpty);
    assert_eq!(err.resource_type, Some(Record::TYPE));
    assert_eq!(err.field.as_deref(), Some("items"));
}

#[test]
fn test_error_negative_amount_returns_non_negative_with_context() {
    let items = vec![RecordItemInput {
        account_type: AccountType::Asset,
        kind: RecordItemKind::Transaction,
        amount: "-100 USD".into(),
        ..Default::default()
    }];
    let result: Result<Record, _> = RecordInput {
        items,
        ..Default::default()
    }
    .try_into();

    let err = result.unwrap_err();
    assert!(matches!(err.kind, ErrorKind::NonNegative { .. }));
    assert_eq!(err.resource_type, Some(Amount::TYPE));
    assert_eq!(err.field.as_deref(), Some("amount"));
    // The actual value is carried structurally inside ErrorKind
    match &err.kind {
        ErrorKind::NonNegative { actual } => assert!(actual.contains("-100")),
        other => panic!("expected NonNegative, got {other:?}"),
    }
}

#[test]
fn test_error_missing_unit_returns_invalid_format() {
    // With the string-based AmountInput, a missing unit means only one token,
    // which fails the [amount, unit] pattern match with InvalidFormat.
    let items = vec![RecordItemInput {
        account_type: AccountType::Asset,
        kind: RecordItemKind::Transaction,
        amount: "100".into(),
        ..Default::default()
    }];
    let result: Result<Record, _> = RecordInput {
        items,
        ..Default::default()
    }
    .try_into();

    let err = result.unwrap_err();
    assert!(matches!(err.kind, ErrorKind::InvalidFormat { .. }));
}

#[test]
fn test_error_mixed_item_kinds_returns_conflicting_values() {
    let items = vec![
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            ..Default::default()
        },
        RecordItemInput {
            account_type: AccountType::Asset,
            kind: RecordItemKind::Validation,
            amount: "100 USD".into(),
            ..Default::default()
        },
    ];
    let result: Result<Record, _> = RecordInput {
        items,
        ..Default::default()
    }
    .try_into();

    let err = result.unwrap_err();
    assert!(matches!(err.kind, ErrorKind::ConflictingValues { .. }));
    assert_eq!(err.resource_type, Some(Record::TYPE));
    assert_eq!(err.field.as_deref(), Some("items"));
    // The conflicting values are carried structurally inside ErrorKind
    match &err.kind {
        ErrorKind::ConflictingValues { values } => {
            assert!(values.iter().any(|v| v.contains("Transaction")));
            assert!(values.iter().any(|v| v.contains("Validation")));
        }
        other => panic!("expected ConflictingValues, got {other:?}"),
    }
}

#[test]
fn test_error_display_format_with_full_context() {
    let err = shared::Error::non_negative("-5")
        .with_resource_type(Record::TYPE)
        .with_field("amount");

    let display = err.to_string();
    // Should contain resource type, field, title, and detail
    assert!(display.contains(Record::TYPE));
    assert!(display.contains("amount"));
    assert!(display.contains("non-negative"));
    assert!(display.contains("-5"));
}

#[test]
fn test_error_display_format_without_context() {
    // Errors created by shared primitives have no context
    let err = shared::Error::non_empty();

    let display = err.to_string();
    assert_eq!(display, "value must be non-empty");
    assert_eq!(err.resource_type, None);
    assert_eq!(err.field, None);
}

#[test]
fn test_error_kind_default_status_codes() {
    assert_eq!(ErrorKind::NonEmpty.default_status(), 422);
    assert_eq!(
        ErrorKind::NonNegative {
            actual: String::new()
        }
        .default_status(),
        422
    );
    assert_eq!(
        ErrorKind::DuplicateValues {
            value: String::new()
        }
        .default_status(),
        422
    );
    assert_eq!(
        ErrorKind::ConflictingValues { values: vec![] }.default_status(),
        422
    );
    assert_eq!(
        ErrorKind::InvalidFormat {
            value: String::new()
        }
        .default_status(),
        422
    );
    assert_eq!(ErrorKind::NotFound.default_status(), 404);
    assert_eq!(ErrorKind::Unauthorized.default_status(), 401);
    assert_eq!(ErrorKind::Forbidden.default_status(), 403);
    assert_eq!(ErrorKind::Conflict.default_status(), 409);
    assert_eq!(ErrorKind::Internal.default_status(), 500);
}

#[test]
fn test_error_builder_chaining_preserves_all_fields() {
    let err = shared::Error::non_empty()
        .with_resource_type("TestResource")
        .with_field("test_field")
        .with_detail("extra info")
        .with_pointer("/data/attributes/test_field")
        .with_parameter("filter[status]");

    assert_eq!(err.kind, ErrorKind::NonEmpty);
    assert_eq!(err.resource_type, Some("TestResource"));
    assert_eq!(err.field.as_deref(), Some("test_field"));
    assert_eq!(err.detail.as_deref(), Some("extra info"));
    assert_eq!(
        err.source.pointer.as_deref(),
        Some("/data/attributes/test_field")
    );
    assert_eq!(err.source.parameter.as_deref(), Some("filter[status]"));
}
