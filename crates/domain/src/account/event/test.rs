use std::collections::HashSet;

use chrono::Utc;
use shared::DomainEvent;

use crate::account::{AccountId, AccountType};
use crate::journal::JournalId;

use super::*;

fn sample_created() -> AccountEvent {
    AccountEvent::Created(AccountCreated {
        id: AccountId::from("acc-1"),
        journal_id: JournalId::from("j-1"),
        parent_id: Some(AccountId::from("root-1")),
        r#type: AccountType::Asset,
        name: "Cash".to_string(),
        description: "Main cash".to_string(),
        tags: HashSet::from(["liquid".to_string()]),
        created_at: Utc::now(),
    })
}

#[test]
fn event_type_returns_expected_strings() {
    assert_eq!(
        sample_created().event_type(),
        "whiterabbit::event::AccountCreated"
    );
    assert_eq!(
        AccountEvent::Updated(AccountUpdated {
            id: AccountId::from("acc-1"),
            name: Some("New".to_string()),
            description: None,
            tags: None,
            last_modified_at: Utc::now(),
        })
        .event_type(),
        "whiterabbit::event::AccountUpdated"
    );
    assert_eq!(
        AccountEvent::Archived(AccountArchived {
            id: AccountId::from("acc-1"),
            archived_at: Utc::now(),
        })
        .event_type(),
        "whiterabbit::event::AccountArchived"
    );
    assert_eq!(
        AccountEvent::Deleted(AccountDeleted {
            id: AccountId::from("acc-1"),
        })
        .event_type(),
        "whiterabbit::event::AccountDeleted"
    );
}

#[test]
fn serde_roundtrip() {
    let event = sample_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AccountEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
