use std::collections::{HashMap, HashSet};

use chrono::Utc;

use crate::error::ErrorKind;
use crate::journal::{JournalId, JournalInput};
use shared::{AggregateRoot, Entity, EntityId};

use super::event::*;
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
    assert_eq!(err.context.resource_type, Some(Account::ENTITY_TYPE));
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
    assert_eq!(err.context.resource_type, Some(Account::ENTITY_TYPE));
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
    assert_eq!(err.context.resource_type, Some(Account::ENTITY_TYPE));
    assert_eq!(err.context.field.as_deref(), Some("parent_id"));
}

// ── AggregateRoot apply tests ────────────────────────────────────

fn default_account() -> Account {
    AccountInput {
        id: AccountId::from_value("seed"),
        journal_id: JournalId::from_value("j"),
        r#type: AccountType::Asset,
        name: "Seed".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap()
}

#[test]
fn test_apply_created_overwrites_all_fields() {
    let mut account = default_account();
    let now = Utc::now();
    account.apply(&AccountEvent::Created(AccountCreated {
        id: AccountId::from("new-id"),
        journal_id: JournalId::from("j-new"),
        parent_id: Some(AccountId::from("parent")),
        r#type: AccountType::Expense,
        name: "Food".to_string(),
        description: "Groceries".to_string(),
        tags: HashSet::from(["grocery".to_string()]),
        created_at: now,
    }));

    assert_eq!(account.id, AccountId::from("new-id"));
    assert_eq!(account.journal_id, JournalId::from("j-new"));
    assert_eq!(account.parent_id, Some(AccountId::from("parent")));
    assert_eq!(account.r#type, AccountType::Expense);
    assert_eq!(account.name.to_string(), "Food");
    assert_eq!(account.description, "Groceries");
    assert_eq!(account.tags.len(), 1);
    assert_eq!(account.created_at, Some(now));
}

#[test]
fn test_apply_updated_patches_changed_fields() {
    let mut account = default_account();
    let now = Utc::now();
    account.apply(&AccountEvent::Updated(AccountUpdated {
        id: account.id.clone(),
        name: Some("Renamed".to_string()),
        description: None,
        tags: None,
        last_modified_at: now,
    }));

    assert_eq!(account.name.to_string(), "Renamed");
    assert_eq!(account.description, "");
    assert_eq!(account.last_modified_at, Some(now));
}

#[test]
fn test_apply_updated_leaves_unset_fields_unchanged() {
    let mut account = default_account();
    let original_name = account.name.to_string();
    let now = Utc::now();
    account.apply(&AccountEvent::Updated(AccountUpdated {
        id: account.id.clone(),
        name: None,
        description: Some("New desc".to_string()),
        tags: None,
        last_modified_at: now,
    }));

    assert_eq!(account.name.to_string(), original_name);
    assert_eq!(account.description, "New desc");
}

#[test]
fn test_apply_archived_sets_archived_at() {
    let mut account = default_account();
    let now = Utc::now();
    account.apply(&AccountEvent::Archived(AccountArchived {
        id: account.id.clone(),
        archived_at: now,
    }));

    assert_eq!(account.archived_at, Some(now));
}

#[test]
fn test_apply_deleted_is_noop() {
    let mut account = default_account();
    let before = account.clone();
    account.apply(&AccountEvent::Deleted(AccountDeleted {
        id: account.id.clone(),
    }));
    assert_eq!(account, before);
}

#[test]
fn test_apply_is_deterministic() {
    let mut a = default_account();
    let mut b = a.clone();
    let event = AccountEvent::Updated(AccountUpdated {
        id: a.id.clone(),
        name: Some("Deterministic".to_string()),
        description: Some("Same".to_string()),
        tags: Some(HashSet::from(["t".to_string()])),
        last_modified_at: Utc::now(),
    });
    a.apply(&event);
    b.apply(&event);
    assert_eq!(a, b);
}

#[test]
fn test_apply_multi_event_fold() {
    let mut account = default_account();
    let t1 = Utc::now();
    let t2 = Utc::now();
    let t3 = Utc::now();

    let events = vec![
        AccountEvent::Created(AccountCreated {
            id: AccountId::from("a1"),
            journal_id: JournalId::from("j1"),
            parent_id: None,
            r#type: AccountType::Income,
            name: "Salary".to_string(),
            description: "Monthly".to_string(),
            tags: HashSet::new(),
            created_at: t1,
        }),
        AccountEvent::Updated(AccountUpdated {
            id: AccountId::from("a1"),
            name: Some("Consulting".to_string()),
            description: None,
            tags: Some(HashSet::from(["work".to_string()])),
            last_modified_at: t2,
        }),
        AccountEvent::Archived(AccountArchived {
            id: AccountId::from("a1"),
            archived_at: t3,
        }),
    ];
    for event in &events {
        account.apply(event);
    }

    assert_eq!(account.id, AccountId::from("a1"));
    assert_eq!(account.name.to_string(), "Consulting");
    assert_eq!(account.description, "Monthly");
    assert!(account.tags.iter().any(|t| t.to_string() == "work"));
    assert_eq!(account.archived_at, Some(t3));
    assert_eq!(account.last_modified_at, Some(t2));
}
