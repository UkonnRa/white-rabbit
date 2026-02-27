use std::collections::HashSet;

use chrono::Utc;
use shared::DomainEvent;

use crate::journal::JournalId;

use super::*;

fn sample_created() -> JournalEvent {
    JournalEvent::Created(JournalCreated {
        id: JournalId::from("j-1"),
        name: "My Ledger".to_string(),
        description: "Personal".to_string(),
        tags: HashSet::from(["finance".to_string()]),
        created_at: Utc::now(),
    })
}

#[test]
fn event_type_returns_expected_strings() {
    assert_eq!(
        sample_created().event_type(),
        "whiterabbit::event::JournalCreated"
    );
    assert_eq!(
        JournalEvent::Updated(JournalUpdated {
            id: JournalId::from("j-1"),
            name: Some("Renamed".to_string()),
            description: None,
            tags: None,
            last_modified_at: Utc::now(),
        })
        .event_type(),
        "whiterabbit::event::JournalUpdated"
    );
    assert_eq!(
        JournalEvent::Deleted(JournalDeleted {
            id: JournalId::from("j-1"),
        })
        .event_type(),
        "whiterabbit::event::JournalDeleted"
    );
}

#[test]
fn serde_roundtrip() {
    let event = sample_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: JournalEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
