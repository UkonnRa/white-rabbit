use std::collections::HashSet;

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use shared::DomainEvent;

use crate::account::{AccountId, AccountType};
use crate::journal::JournalId;
use crate::record::{Amount, RecordId, RecordItemTransaction, RecordItems};

use super::*;

fn sample_created() -> RecordEvent {
    RecordEvent::Created(RecordCreated {
        id: RecordId::from("r-1"),
        journal_id: JournalId::from("j-1"),
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        items: RecordItems::Transactions(
            vec![RecordItemTransaction {
                account_id: AccountId::from("acc-1"),
                account_type: AccountType::Asset,
                amount: Amount {
                    amount: Decimal::from(100).try_into().unwrap(),
                    unit: "USD".to_string().try_into().unwrap(),
                },
                price: None,
                cost: HashSet::new(),
                description: String::new(),
            }]
            .try_into()
            .unwrap(),
        ),
        description: "Test".to_string(),
        tags: HashSet::from(["food".to_string()]),
        payee: "Cafe".to_string(),
        created_at: Utc::now(),
    })
}

#[test]
fn event_type_returns_expected_strings() {
    assert_eq!(
        sample_created().event_type(),
        "whiterabbit::event::RecordCreated"
    );
    assert_eq!(
        RecordEvent::Updated(RecordUpdated {
            id: RecordId::from("r-1"),
            date: None,
            items: None,
            description: Some("Updated".to_string()),
            tags: None,
            payee: None,
            last_modified_at: Utc::now(),
        })
        .event_type(),
        "whiterabbit::event::RecordUpdated"
    );
    assert_eq!(
        RecordEvent::Deleted(RecordDeleted {
            id: RecordId::from("r-1"),
        })
        .event_type(),
        "whiterabbit::event::RecordDeleted"
    );
}

#[test]
fn serde_roundtrip() {
    let event = sample_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RecordEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
