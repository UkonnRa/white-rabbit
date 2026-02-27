use std::collections::HashSet;

use chrono::Utc;

use crate::error::ErrorKind;
use shared::{AggregateRoot, Entity, EntityId};

use super::event::*;
use super::{Journal, JournalId, JournalInput};

#[test]
fn test_journal_input_try_into_success() {
    let journal: Journal = JournalInput {
        id: JournalId::from_value("test-journal-id"),
        name: "My Journal".to_string(),
        description: "Personal ledger".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(journal.id.as_ref(), "test-journal-id");
    assert!(format!("{:?}", journal.name).contains("My Journal"));
}

#[test]
fn test_journal_input_with_tags() {
    let journal: Journal = JournalInput {
        id: JournalId::from_value("test-journal-id"),
        name: "Work Journal".to_string(),
        tags: HashSet::from(["business".to_string(), "2024".to_string()]),
        ..Default::default()
    }
    .try_into()
    .unwrap();
    assert_eq!(journal.tags.len(), 2);
}

// ── Error tests ──────────────────────────────────────────────────

#[test]
fn test_error_empty_name_returns_non_empty_with_context() {
    let result: Result<Journal, _> = JournalInput {
        id: JournalId::from_value("test-journal-id"),
        name: "".to_string(),
        ..Default::default()
    }
    .try_into();

    let err = result.unwrap_err();
    assert_eq!(err.error, ErrorKind::Shared(shared::ErrorKind::NonEmpty));
    assert_eq!(err.context.resource_type, Some(Journal::ENTITY_TYPE));
    assert_eq!(err.context.field.as_deref(), Some("name"));
}

// ── AggregateRoot apply tests ────────────────────────────────────

fn default_journal() -> Journal {
    JournalInput {
        id: JournalId::from_value("seed"),
        name: "Seed".to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap()
}

#[test]
fn test_apply_created_overwrites_all_fields() {
    let mut journal = default_journal();
    let now = Utc::now();
    journal.apply(&JournalEvent::Created(JournalCreated {
        id: JournalId::from("j-new"),
        name: "Personal".to_string(),
        description: "My ledger".to_string(),
        tags: HashSet::from(["finance".to_string()]),
        created_at: now,
    }));

    assert_eq!(journal.id, JournalId::from("j-new"));
    assert_eq!(journal.name.to_string(), "Personal");
    assert_eq!(journal.description, "My ledger");
    assert_eq!(journal.tags.len(), 1);
    assert_eq!(journal.created_at, Some(now));
}

#[test]
fn test_apply_updated_patches_changed_fields() {
    let mut journal = default_journal();
    let now = Utc::now();
    journal.apply(&JournalEvent::Updated(JournalUpdated {
        id: journal.id.clone(),
        name: Some("Renamed".to_string()),
        description: None,
        tags: None,
        last_modified_at: now,
    }));

    assert_eq!(journal.name.to_string(), "Renamed");
    assert_eq!(journal.last_modified_at, Some(now));
}

#[test]
fn test_apply_updated_leaves_unset_fields_unchanged() {
    let mut journal = default_journal();
    let original_name = journal.name.to_string();
    let now = Utc::now();
    journal.apply(&JournalEvent::Updated(JournalUpdated {
        id: journal.id.clone(),
        name: None,
        description: Some("New desc".to_string()),
        tags: None,
        last_modified_at: now,
    }));

    assert_eq!(journal.name.to_string(), original_name);
    assert_eq!(journal.description, "New desc");
}

#[test]
fn test_apply_deleted_is_noop() {
    let mut journal = default_journal();
    let before = journal.clone();
    journal.apply(&JournalEvent::Deleted(JournalDeleted {
        id: journal.id.clone(),
    }));
    assert_eq!(journal, before);
}

#[test]
fn test_apply_is_deterministic() {
    let mut a = default_journal();
    let mut b = a.clone();
    let event = JournalEvent::Updated(JournalUpdated {
        id: a.id.clone(),
        name: Some("Same".to_string()),
        description: None,
        tags: None,
        last_modified_at: Utc::now(),
    });
    a.apply(&event);
    b.apply(&event);
    assert_eq!(a, b);
}

#[test]
fn test_apply_multi_event_fold() {
    let mut journal = default_journal();
    let t1 = Utc::now();
    let t2 = Utc::now();

    let events = vec![
        JournalEvent::Created(JournalCreated {
            id: JournalId::from("j1"),
            name: "First".to_string(),
            description: "Original".to_string(),
            tags: HashSet::new(),
            created_at: t1,
        }),
        JournalEvent::Updated(JournalUpdated {
            id: JournalId::from("j1"),
            name: Some("Second".to_string()),
            description: Some("Updated".to_string()),
            tags: Some(HashSet::from(["v2".to_string()])),
            last_modified_at: t2,
        }),
    ];
    for event in &events {
        journal.apply(event);
    }

    assert_eq!(journal.name.to_string(), "Second");
    assert_eq!(journal.description, "Updated");
    assert!(journal.tags.iter().any(|t| t.to_string() == "v2"));
    assert_eq!(journal.created_at, Some(t1));
    assert_eq!(journal.last_modified_at, Some(t2));
}
