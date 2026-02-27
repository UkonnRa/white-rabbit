use std::collections::HashSet;

use crate::{
    account::{AccountId, AccountType},
    error::ErrorKind,
    journal::JournalId,
};
use chrono::{NaiveDate, Utc};
use shared::{AggregateRoot, Entity, EntityId};

use super::event::*;
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
    assert_eq!(err.context.resource_type, Some(Record::ENTITY_TYPE));
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
    assert_eq!(err.context.resource_type, Some(Amount::ENTITY_TYPE));
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
    assert_eq!(err.context.resource_type, Some(Record::ENTITY_TYPE));
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
        .with_resource_type(Record::ENTITY_TYPE)
        .with_field("amount");
    // Display delegates to the inner error only; context is accessed programmatically
    let display = err.to_string();
    assert!(display.contains("non-negative"));
    assert!(display.contains("-5"));
    // Context is separate, not in Display
    assert_eq!(err.context.resource_type, Some(Record::ENTITY_TYPE));
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

// ── AggregateRoot apply tests ────────────────────────────────────

fn default_record() -> Record {
    RecordInput {
        id: RecordId::from_value("seed"),
        journal_id: JournalId::from_value("j"),
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        items: vec![RecordItemInput {
            account_id: AccountId::from_value("acc"),
            account_type: AccountType::Asset,
            kind: RecordItemKind::Transaction,
            amount: "100 USD".into(),
            description: String::new(),
            ..Default::default()
        }],
        description: "Seed".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap()
}

fn sample_created_event() -> RecordCreated {
    let record = default_record();
    RecordCreated {
        id: RecordId::from("r-new"),
        journal_id: JournalId::from("j-new"),
        date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        items: record.items.clone(),
        description: "Created".to_string(),
        tags: HashSet::from(["tag1".to_string()]),
        payee: "Cafe".to_string(),
        created_at: Utc::now(),
    }
}

#[test]
fn test_apply_created_overwrites_all_fields() {
    let mut record = default_record();
    let event = sample_created_event();
    record.apply(&RecordEvent::Created(event.clone()));

    assert_eq!(record.id, RecordId::from("r-new"));
    assert_eq!(record.journal_id, JournalId::from("j-new"));
    assert_eq!(record.date, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    assert_eq!(record.description, "Created");
    assert_eq!(record.payee, "Cafe");
    assert_eq!(record.tags.len(), 1);
    assert_eq!(record.created_at, Some(event.created_at));
}

#[test]
fn test_apply_updated_patches_changed_fields() {
    let mut record = default_record();
    let new_date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
    let now = Utc::now();
    record.apply(&RecordEvent::Updated(RecordUpdated {
        id: record.id.clone(),
        date: Some(new_date),
        items: None,
        description: Some("Updated".to_string()),
        tags: None,
        payee: Some("New Payee".to_string()),
        last_modified_at: now,
    }));

    assert_eq!(record.date, new_date);
    assert_eq!(record.description, "Updated");
    assert_eq!(record.payee, "New Payee");
    assert_eq!(record.last_modified_at, Some(now));
}

#[test]
fn test_apply_updated_leaves_unset_fields_unchanged() {
    let mut record = default_record();
    let original_date = record.date;
    let original_payee = record.payee.clone();
    let now = Utc::now();
    record.apply(&RecordEvent::Updated(RecordUpdated {
        id: record.id.clone(),
        date: None,
        items: None,
        description: Some("Only desc changed".to_string()),
        tags: None,
        payee: None,
        last_modified_at: now,
    }));

    assert_eq!(record.date, original_date);
    assert_eq!(record.payee, original_payee);
    assert_eq!(record.description, "Only desc changed");
}

#[test]
fn test_apply_deleted_is_noop() {
    let mut record = default_record();
    let before = record.clone();
    record.apply(&RecordEvent::Deleted(RecordDeleted {
        id: record.id.clone(),
    }));
    assert_eq!(record, before);
}

#[test]
fn test_apply_is_deterministic() {
    let mut a = default_record();
    let mut b = a.clone();
    let event = RecordEvent::Updated(RecordUpdated {
        id: a.id.clone(),
        date: Some(NaiveDate::from_ymd_opt(2024, 12, 25).unwrap()),
        items: None,
        description: Some("Same".to_string()),
        tags: None,
        payee: None,
        last_modified_at: Utc::now(),
    });
    a.apply(&event);
    b.apply(&event);
    assert_eq!(a, b);
}

#[test]
fn test_apply_multi_event_fold() {
    let mut record = default_record();
    let t1 = Utc::now();
    let t2 = Utc::now();

    let created = sample_created_event();
    let events = vec![
        RecordEvent::Created(RecordCreated {
            created_at: t1,
            ..created
        }),
        RecordEvent::Updated(RecordUpdated {
            id: RecordId::from("r-new"),
            date: Some(NaiveDate::from_ymd_opt(2024, 8, 1).unwrap()),
            items: None,
            description: Some("Final".to_string()),
            tags: Some(HashSet::from(["final".to_string()])),
            payee: None,
            last_modified_at: t2,
        }),
    ];
    for event in &events {
        record.apply(event);
    }

    assert_eq!(record.id, RecordId::from("r-new"));
    assert_eq!(record.date, NaiveDate::from_ymd_opt(2024, 8, 1).unwrap());
    assert_eq!(record.description, "Final");
    assert!(record.tags.iter().any(|t| t.to_string() == "final"));
    assert_eq!(record.created_at, Some(t1));
    assert_eq!(record.last_modified_at, Some(t2));
}
