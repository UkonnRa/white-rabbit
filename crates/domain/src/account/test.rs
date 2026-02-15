use std::collections::HashMap;

use crate::error::ErrorKind;
use crate::journal::{JournalId, JournalInput};
use shared::EntityId;

use super::{Account, AccountContext, AccountId, AccountInput, AccountType};

// ── Input tests ──────────────────────────────────────────────────

#[test]
fn test_account_input_try_into_success() {
    let account: Account = AccountInput {
        id: AccountId::from_value("test-account-id"),
        journal_id: JournalId::from_value("test-journal-id"),
        r#type: AccountType::Asset,
        name: "Cash".to_string(),
        description: "Cash account".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(account.id.as_ref(), "test-account-id");
    assert_eq!(account.r#type, AccountType::Asset);
    assert!(format!("{:?}", account.name).contains("Cash"));
}

#[test]
fn test_account_input_with_tags() {
    let account: Account = AccountInput {
        id: AccountId::from_value("test-account-id"),
        journal_id: JournalId::from_value("test-journal-id"),
        r#type: AccountType::Expense,
        name: "Office Supplies".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(account.tags.len(), 2);
}

#[test]
fn test_account_input_with_parent_id() {
    let account: Account = AccountInput {
        id: AccountId::from_value("child-account-id"),
        journal_id: JournalId::from_value("test-journal-id"),
        parent_id: Some(AccountId::from_value("parent-account-id")),
        r#type: AccountType::Asset,
        name: "Checking".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(
        account.parent_id.as_ref().map(|id| id.as_ref()),
        Some("parent-account-id")
    );
}

// ── Input error tests ────────────────────────────────────────────

#[test]
fn test_error_empty_name_returns_non_empty_with_context() {
    let result: Result<Account, _> = AccountInput {
        id: AccountId::from_value("test-account-id"),
        journal_id: JournalId::from_value("test-journal-id"),
        r#type: AccountType::Asset,
        name: "".to_string(),
        ..Default::default()
    }
    .try_into();

    let err = result.unwrap_err();
    assert_eq!(err.error, ErrorKind::Shared(shared::ErrorKind::NonEmpty));
    assert_eq!(err.context.resource_type, Some(Account::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("name"));
}

#[test]
fn test_account_type_default_is_asset() {
    assert_eq!(AccountType::default(), AccountType::Asset);
}

// ── Context validation tests ─────────────────────────────────────

#[test]
fn test_context_valid_three_level_chain() {
    let root: Account = AccountInput {
        id: AccountId::from_value("root"),
        r#type: AccountType::Asset,
        name: "root".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    let mid: Account = AccountInput {
        id: AccountId::from_value("mid"),
        parent_id: Some(AccountId::from_value("root")),
        r#type: AccountType::Asset,
        name: "mid".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    let leaf: Account = AccountInput {
        id: AccountId::from_value("leaf"),
        parent_id: Some(AccountId::from_value("mid")),
        r#type: AccountType::Asset,
        name: "leaf".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();

    let accounts = HashMap::from([(root.id.clone(), root), (mid.id.clone(), mid)]);
    let ctx = AccountContext {
        entity: leaf,
        journal: JournalInput {
            name: "j".into(),
            ..Default::default()
        }
        .try_into()
        .unwrap(),
        accounts,
    };
    ctx.validate().unwrap();
}

#[test]
fn test_context_root_account_no_parent_is_valid() {
    let root: Account = AccountInput {
        id: AccountId::from_value("root"),
        r#type: AccountType::Income,
        name: "root".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();

    let ctx = AccountContext {
        entity: root,
        journal: JournalInput {
            name: "j".into(),
            ..Default::default()
        }
        .try_into()
        .unwrap(),
        accounts: HashMap::new(),
    };
    ctx.validate().unwrap();
}

#[test]
fn test_context_mismatch_immediate_parent() {
    let root: Account = AccountInput {
        id: AccountId::from_value("root"),
        r#type: AccountType::Asset,
        name: "root".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    let child: Account = AccountInput {
        id: AccountId::from_value("child"),
        parent_id: Some(AccountId::from_value("root")),
        r#type: AccountType::Expense,
        name: "child".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();

    let accounts = HashMap::from([(root.id.clone(), root)]);
    let ctx = AccountContext {
        entity: child,
        journal: JournalInput {
            name: "j".into(),
            ..Default::default()
        }
        .try_into()
        .unwrap(),
        accounts,
    };

    let err = ctx.validate().unwrap_err();
    assert!(matches!(err.error, ErrorKind::Mismatch { .. }));
    assert_eq!(err.context.resource_type, Some(Account::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("type"));
    match &err.error {
        ErrorKind::Mismatch { expected, actual } => {
            assert_eq!(expected, "Asset");
            assert_eq!(actual, "Expense");
        }
        other => panic!("expected Mismatch, got {other:?}"),
    }
}

#[test]
fn test_context_mismatch_deep_in_chain() {
    let root: Account = AccountInput {
        id: AccountId::from_value("root"),
        r#type: AccountType::Asset,
        name: "root".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    let mid: Account = AccountInput {
        id: AccountId::from_value("mid"),
        parent_id: Some(AccountId::from_value("root")),
        r#type: AccountType::Asset,
        name: "mid".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    let leaf: Account = AccountInput {
        id: AccountId::from_value("leaf"),
        parent_id: Some(AccountId::from_value("mid")),
        r#type: AccountType::Liability,
        name: "leaf".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();

    let accounts = HashMap::from([(root.id.clone(), root), (mid.id.clone(), mid)]);
    let ctx = AccountContext {
        entity: leaf,
        journal: JournalInput {
            name: "j".into(),
            ..Default::default()
        }
        .try_into()
        .unwrap(),
        accounts,
    };

    let err = ctx.validate().unwrap_err();
    assert!(matches!(err.error, ErrorKind::Mismatch { .. }));
}

#[test]
fn test_context_parent_not_found() {
    let child: Account = AccountInput {
        id: AccountId::from_value("child"),
        parent_id: Some(AccountId::from_value("nonexistent")),
        r#type: AccountType::Asset,
        name: "child".into(),
        ..Default::default()
    }
    .try_into()
    .unwrap();

    let ctx = AccountContext {
        entity: child,
        journal: JournalInput {
            name: "j".into(),
            ..Default::default()
        }
        .try_into()
        .unwrap(),
        accounts: HashMap::new(),
    };

    let err = ctx.validate().unwrap_err();
    assert_eq!(err.error, ErrorKind::Shared(shared::ErrorKind::NotFound));
    assert_eq!(err.context.resource_type, Some(Account::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("parent_id"));
}
