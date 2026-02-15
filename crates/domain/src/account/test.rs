use std::collections::HashMap;

use crate::error::ErrorKind;
use crate::journal::{Journal, JournalId};
use shared::{EntityId, NonEmpty};

use super::{Account, AccountContext, AccountId, AccountInput, AccountType};

fn make_account(id: &str, parent_id: Option<&str>, account_type: AccountType) -> Account {
    Account {
        id: AccountId::from_value(id),
        version: 0,
        created_at: None,
        last_modified_at: None,
        archived_at: None,
        journal_id: JournalId::from_value("test-journal"),
        parent_id: parent_id.map(AccountId::from_value),
        r#type: account_type,
        name: NonEmpty::try_from("test".to_string()).unwrap(),
        description: String::new(),
        tags: Default::default(),
    }
}

fn make_journal() -> Journal {
    Journal {
        id: JournalId::from_value("test-journal"),
        version: 0,
        created_at: None,
        last_modified_at: None,
        archived_at: None,
        name: NonEmpty::try_from("test".to_string()).unwrap(),
        description: String::new(),
        tags: Default::default(),
    }
}

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
    let root = make_account("root", None, AccountType::Asset);
    let mid = make_account("mid", Some("root"), AccountType::Asset);
    let leaf = make_account("leaf", Some("mid"), AccountType::Asset);

    let accounts = HashMap::from([(root.id.clone(), root), (mid.id.clone(), mid)]);
    let ctx = AccountContext {
        entity: leaf,
        journal: make_journal(),
        accounts,
    };
    ctx.validate().unwrap();
}

#[test]
fn test_context_root_account_no_parent_is_valid() {
    let root = make_account("root", None, AccountType::Income);
    let ctx = AccountContext {
        entity: root,
        journal: make_journal(),
        accounts: HashMap::new(),
    };
    ctx.validate().unwrap();
}

#[test]
fn test_context_mismatch_immediate_parent() {
    let root = make_account("root", None, AccountType::Asset);
    let child = make_account("child", Some("root"), AccountType::Expense);

    let accounts = HashMap::from([(root.id.clone(), root)]);
    let ctx = AccountContext {
        entity: child,
        journal: make_journal(),
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
    let root = make_account("root", None, AccountType::Asset);
    let mid = make_account("mid", Some("root"), AccountType::Asset);
    let leaf = make_account("leaf", Some("mid"), AccountType::Liability);

    let accounts = HashMap::from([(root.id.clone(), root), (mid.id.clone(), mid)]);
    let ctx = AccountContext {
        entity: leaf,
        journal: make_journal(),
        accounts,
    };

    let err = ctx.validate().unwrap_err();
    assert!(matches!(err.error, ErrorKind::Mismatch { .. }));
}

#[test]
fn test_context_parent_not_found() {
    let child = make_account("child", Some("nonexistent"), AccountType::Asset);
    let ctx = AccountContext {
        entity: child,
        journal: make_journal(),
        accounts: HashMap::new(),
    };

    let err = ctx.validate().unwrap_err();
    assert_eq!(err.error, ErrorKind::Shared(shared::ErrorKind::NotFound));
    assert_eq!(err.context.resource_type, Some(Account::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("parent_id"));
}
