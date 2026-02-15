use crate::journal::JournalId;
use shared::{EntityId, ErrorKind};

use super::{Account, AccountId, AccountInput, AccountType};

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

// ── Error tests ──────────────────────────────────────────────────

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
    assert_eq!(err.kind, ErrorKind::NonEmpty);
    assert_eq!(err.resource_type, Some(Account::TYPE));
    assert_eq!(err.field.as_deref(), Some("name"));
}

#[test]
fn test_account_type_default_is_asset() {
    assert_eq!(AccountType::default(), AccountType::Asset);
}
