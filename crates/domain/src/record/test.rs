use crate::{
    account::{AccountId, AccountType},
    error::ErrorKind,
    journal::JournalId,
};
use chrono::NaiveDate;
use shared::EntityId;

use super::{Amount, Record, RecordId, RecordInput, RecordItemInput, RecordItemKind};

#[test]
fn test_is_balanced_simple_balanced() {
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
    let items = vec![RecordItemInput {
        account_type: AccountType::Asset,
        kind: RecordItemKind::Validation,
        amount: "100 USD".into(),
        ..Default::default()
    }];
    let record: Record = RecordInput {
        items: items,
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(record.is_balanced(), None);
}

#[test]
fn test_is_balanced_with_mixed_price_conversions() {
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
    assert_eq!(err.error, ErrorKind::Shared(shared::ErrorKind::NonEmpty));
    assert_eq!(err.context.resource_type, Some(Record::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("items"));
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
    assert!(matches!(
        err.error,
        ErrorKind::Shared(shared::ErrorKind::NonNegative { .. })
    ));
    assert_eq!(err.context.resource_type, Some(Amount::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("amount"));
    match &err.error {
        ErrorKind::Shared(shared::ErrorKind::NonNegative { actual }) => {
            assert!(actual.contains("-100"))
        }
        other => panic!("expected NonNegative, got {other:?}"),
    }
}

#[test]
fn test_error_missing_unit_returns_invalid_format() {
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
    assert!(matches!(
        err.error,
        ErrorKind::Shared(shared::ErrorKind::InvalidFormat { .. })
    ));
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
    assert!(matches!(
        err.error,
        ErrorKind::Shared(shared::ErrorKind::ConflictingValues { .. })
    ));
    assert_eq!(err.context.resource_type, Some(Record::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("items"));
    match &err.error {
        ErrorKind::Shared(shared::ErrorKind::ConflictingValues { values }) => {
            assert!(values.iter().any(|v| v.contains("Transaction")));
            assert!(values.iter().any(|v| v.contains("Validation")));
        }
        other => panic!("expected ConflictingValues, got {other:?}"),
    }
}

#[test]
fn test_error_display_delegates_to_inner_error() {
    let err = shared::ErrorKind::non_negative("-5")
        .with_resource_type(Record::TYPE)
        .with_field("amount");
    // Display delegates to the inner error only; context is accessed programmatically
    let display = err.to_string();
    assert!(display.contains("non-negative"));
    assert!(display.contains("-5"));
    // Context is separate, not in Display
    assert_eq!(err.context.resource_type, Some(Record::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("amount"));
}

#[test]
fn test_error_display_format_without_context() {
    let err = shared::ErrorKind::non_empty();
    let display = err.to_string();
    assert_eq!(display, "value must be non-empty");
    assert_eq!(err.context.resource_type, None);
    assert_eq!(err.context.field, None);
}

#[test]
fn test_shared_error_kind_default_status_codes() {
    use shared::ErrorKindInfo;
    assert_eq!(shared::ErrorKind::NonEmpty.default_status(), 422);
    assert_eq!(
        shared::ErrorKind::NonNegative {
            actual: String::new()
        }
        .default_status(),
        422
    );
    assert_eq!(shared::ErrorKind::NotFound.default_status(), 404);
    assert_eq!(shared::ErrorKind::Unauthorized.default_status(), 401);
    assert_eq!(shared::ErrorKind::Forbidden.default_status(), 403);
    assert_eq!(shared::ErrorKind::Conflict.default_status(), 409);
    assert_eq!(shared::ErrorKind::Internal.default_status(), 500);
}

#[test]
fn test_error_builder_chaining_preserves_all_fields() {
    let err = shared::ErrorKind::non_empty()
        .with_resource_type("TestResource")
        .with_field("test_field")
        .with_detail("extra info")
        .with_pointer("/data/attributes/test_field")
        .with_parameter("filter[status]");

    assert_eq!(err.error, shared::ErrorKind::NonEmpty);
    assert_eq!(err.context.resource_type, Some("TestResource"));
    assert_eq!(err.context.field.as_deref(), Some("test_field"));
    assert_eq!(err.context.detail.as_deref(), Some("extra info"));
    assert_eq!(
        err.context.source.pointer.as_deref(),
        Some("/data/attributes/test_field")
    );
    assert_eq!(
        err.context.source.parameter.as_deref(),
        Some("filter[status]")
    );
}

#[test]
fn test_shared_error_converts_to_domain_via_map_error() {
    fn shared_op() -> shared::Result<()> {
        Err(shared::ErrorKind::non_empty().with_resource_type("Test"))
    }

    fn domain_op() -> crate::error::Result<()> {
        shared_op().map_err(|e| e.convert())?;
        Ok(())
    }

    let err = domain_op().unwrap_err();
    assert_eq!(err.error, ErrorKind::Shared(shared::ErrorKind::NonEmpty));
    assert_eq!(err.context.resource_type, Some("Test"));
}
