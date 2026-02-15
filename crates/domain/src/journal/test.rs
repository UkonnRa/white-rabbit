use crate::error::ErrorKind;
use shared::EntityId;

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
        tags: vec!["business".to_string(), "2024".to_string()],
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
    assert_eq!(err.context.resource_type, Some(Journal::TYPE));
    assert_eq!(err.context.field.as_deref(), Some("name"));
}
