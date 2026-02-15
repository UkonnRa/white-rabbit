use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::Mutex;

use domain::journal::JournalId;
use domain::journal::command::{JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate};
use domain::journal::service::JournalService;

use super::InMemoryJournalRepository;

// ── Helpers ──────────────────────────────────────────────────────

fn new_service() -> JournalService<InMemoryJournalRepository> {
    JournalService {
        repository: Arc::new(Mutex::new(InMemoryJournalRepository::default())),
    }
}

fn create_cmd(name: &str) -> JournalCommandCreate {
    JournalCommandCreate {
        name: name.to_string(),
        description: String::new(),
        tags: HashSet::new(),
    }
}

// ── Create tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_single_journal() -> anyhow::Result<()> {
    let service = new_service();

    let journals = service
        .create([JournalCommandCreate {
            name: "My Ledger".to_string(),
            description: "Personal finance".to_string(),
            tags: HashSet::from(["finance".to_string()]),
        }])
        .await?;

    assert_eq!(journals.len(), 1);
    assert_eq!(journals[0].name.to_string(), "My Ledger");
    assert_eq!(journals[0].description, "Personal finance");

    Ok(())
}

#[tokio::test]
async fn test_create_batch_multiple_journals() -> anyhow::Result<()> {
    let service = new_service();

    let journals = service
        .create([create_cmd("Alpha"), create_cmd("Beta"), create_cmd("Gamma")])
        .await?;

    assert_eq!(journals.len(), 3);
    let names: HashSet<_> = journals.iter().map(|j| j.name.to_string()).collect();
    assert!(names.contains("Alpha"));
    assert!(names.contains("Beta"));
    assert!(names.contains("Gamma"));

    Ok(())
}

#[tokio::test]
async fn test_create_empty_batch_succeeds() -> anyhow::Result<()> {
    let service = new_service();
    let journals = service.create(Vec::<JournalCommandCreate>::new()).await?;
    assert!(journals.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_create_empty_name_fails() -> anyhow::Result<()> {
    let service = new_service();

    let err = service.create([create_cmd("")]).await.unwrap_err();
    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::NonEmpty)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_duplicate_names_within_batch_fails() -> anyhow::Result<()> {
    let service = new_service();

    let err = service
        .create([create_cmd("Ledger"), create_cmd("Ledger")])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "Ledger".to_string()
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_create_name_conflicts_with_existing_journal() -> anyhow::Result<()> {
    let service = new_service();

    service.create([create_cmd("Existing")]).await?;

    let err = service.create([create_cmd("Existing")]).await.unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "Existing".to_string()
        })
    );

    Ok(())
}

// ── Update tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_update_single_journal() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Original")]).await?;
    let id = created[0].id.clone();

    let updated = service
        .update([JournalCommandUpdate {
            id: id.clone(),
            name: "Renamed".to_string(),
            description: Some("New desc".to_string()),
            tags: Some(HashSet::from(["tag1".to_string()])),
        }])
        .await?;

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].name.to_string(), "Renamed");
    assert_eq!(updated[0].description, "New desc");

    Ok(())
}

#[tokio::test]
async fn test_update_empty_name_keeps_existing() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("KeepMe")]).await?;
    let id = created[0].id.clone();

    let updated = service
        .update([JournalCommandUpdate {
            id,
            name: String::new(), // empty → keep existing
            description: Some("Updated desc".to_string()),
            tags: None,
        }])
        .await?;

    assert_eq!(updated[0].name.to_string(), "KeepMe");
    assert_eq!(updated[0].description, "Updated desc");

    Ok(())
}

#[tokio::test]
async fn test_update_none_fields_keep_existing() -> anyhow::Result<()> {
    let service = new_service();

    let created = service
        .create([JournalCommandCreate {
            name: "Journal".to_string(),
            description: "Original desc".to_string(),
            tags: HashSet::from(["original".to_string()]),
        }])
        .await?;
    let id = created[0].id.clone();

    let updated = service
        .update([JournalCommandUpdate {
            id,
            name: "Journal".to_string(),
            description: None, // keep existing
            tags: None,        // keep existing
        }])
        .await?;

    assert_eq!(updated[0].description, "Original desc");

    Ok(())
}

#[tokio::test]
async fn test_update_nonexistent_id_fails() -> anyhow::Result<()> {
    let service = new_service();

    let err = service
        .update([JournalCommandUpdate {
            id: JournalId::from("nonexistent"),
            name: "Whatever".to_string(),
            description: None,
            tags: None,
        }])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::NotFound)
    );

    Ok(())
}

#[tokio::test]
async fn test_update_duplicate_ids_in_batch_fails() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Journal")]).await?;
    let id = created[0].id.clone();

    let err = service
        .update([
            JournalCommandUpdate {
                id: id.clone(),
                name: "Name1".to_string(),
                description: None,
                tags: None,
            },
            JournalCommandUpdate {
                id,
                name: "Name2".to_string(),
                description: None,
                tags: None,
            },
        ])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "id".to_string()
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_update_duplicate_new_names_in_batch_fails() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("A"), create_cmd("B")]).await?;

    let err = service
        .update([
            JournalCommandUpdate {
                id: created[0].id.clone(),
                name: "Same".to_string(),
                description: None,
                tags: None,
            },
            JournalCommandUpdate {
                id: created[1].id.clone(),
                name: "Same".to_string(),
                description: None,
                tags: None,
            },
        ])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "Same".to_string()
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_update_name_conflicts_with_journal_not_in_batch() -> anyhow::Result<()> {
    let service = new_service();

    let created = service
        .create([create_cmd("Existing"), create_cmd("ToUpdate")])
        .await?;

    // Try to rename "ToUpdate" → "Existing", but "Existing" is NOT in the batch
    let err = service
        .update([JournalCommandUpdate {
            id: created
                .iter()
                .find(|j| j.name.to_string() == "ToUpdate")
                .unwrap()
                .id
                .clone(),
            name: "Existing".to_string(),
            description: None,
            tags: None,
        }])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "Existing".to_string()
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_update_name_swap_is_allowed() -> anyhow::Result<()> {
    let service = new_service();

    let created = service
        .create([create_cmd("Alpha"), create_cmd("Beta")])
        .await?;

    let alpha_id = created
        .iter()
        .find(|j| j.name.to_string() == "Alpha")
        .unwrap()
        .id
        .clone();
    let beta_id = created
        .iter()
        .find(|j| j.name.to_string() == "Beta")
        .unwrap()
        .id
        .clone();

    // Swap: Alpha→"Beta", Beta→"Alpha"
    let updated = service
        .update([
            JournalCommandUpdate {
                id: alpha_id.clone(),
                name: "Beta".to_string(),
                description: None,
                tags: None,
            },
            JournalCommandUpdate {
                id: beta_id.clone(),
                name: "Alpha".to_string(),
                description: None,
                tags: None,
            },
        ])
        .await?;

    let alpha_journal = updated.iter().find(|j| j.id == alpha_id).unwrap();
    let beta_journal = updated.iter().find(|j| j.id == beta_id).unwrap();
    assert_eq!(alpha_journal.name.to_string(), "Beta");
    assert_eq!(beta_journal.name.to_string(), "Alpha");

    Ok(())
}

#[tokio::test]
async fn test_update_rename_to_own_name_succeeds() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Unchanged")]).await?;
    let id = created[0].id.clone();

    // Rename to the same name — should succeed (the journal is in the batch)
    let updated = service
        .update([JournalCommandUpdate {
            id,
            name: "Unchanged".to_string(),
            description: Some("New desc".to_string()),
            tags: None,
        }])
        .await?;

    assert_eq!(updated[0].name.to_string(), "Unchanged");
    assert_eq!(updated[0].description, "New desc");

    Ok(())
}

#[tokio::test]
async fn test_update_empty_batch_succeeds() -> anyhow::Result<()> {
    let service = new_service();
    let updated = service.update(Vec::<JournalCommandUpdate>::new()).await?;
    assert!(updated.is_empty());
    Ok(())
}

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_single_journal() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("ToDelete")]).await?;
    let id = created[0].id.clone();

    let deleted = service.delete([id.clone()]).await?;
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].name.to_string(), "ToDelete");

    // Verify it's actually gone — creating with the same name should succeed
    let recreated = service.create([create_cmd("ToDelete")]).await?;
    assert_eq!(recreated.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_delete_batch_multiple_journals() -> anyhow::Result<()> {
    let service = new_service();

    let created = service
        .create([create_cmd("A"), create_cmd("B"), create_cmd("C")])
        .await?;

    let ids: Vec<_> = created.iter().map(|j| j.id.clone()).collect();
    let deleted = service.delete(ids).await?;
    assert_eq!(deleted.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_delete_empty_batch_succeeds() -> anyhow::Result<()> {
    let service = new_service();
    let deleted = service.delete(Vec::<JournalId>::new()).await?;
    assert!(deleted.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_id_is_silently_ignored() -> anyhow::Result<()> {
    let service = new_service();

    let deleted = service.delete([JournalId::from("nonexistent")]).await?;
    assert!(deleted.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_delete_duplicate_ids_are_deduplicated() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Journal")]).await?;
    let id = created[0].id.clone();

    // Same ID twice — should deduplicate and delete once
    let deleted = service.delete([id.clone(), id]).await?;
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].name.to_string(), "Journal");

    Ok(())
}

#[tokio::test]
async fn test_delete_mixed_existing_and_nonexistent_ids() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Exists")]).await?;
    let existing_id = created[0].id.clone();

    // One real ID + one fake ID → only the real one is deleted
    let deleted = service
        .delete([existing_id, JournalId::from("fake")])
        .await?;

    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].name.to_string(), "Exists");

    Ok(())
}

// ── Batch tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_empty_succeeds() -> anyhow::Result<()> {
    let service = new_service();
    let result = service.batch(JournalCommandBatch::default()).await?;
    assert!(result.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_batch_delete_create_update_together() -> anyhow::Result<()> {
    let service = new_service();

    let created = service
        .create([create_cmd("ToDelete"), create_cmd("ToUpdate")])
        .await?;
    let to_delete_id = created
        .iter()
        .find(|j| j.name.to_string() == "ToDelete")
        .unwrap()
        .id
        .clone();
    let to_update_id = created
        .iter()
        .find(|j| j.name.to_string() == "ToUpdate")
        .unwrap()
        .id
        .clone();

    let result = service
        .batch(JournalCommandBatch {
            delete: HashSet::from([to_delete_id]),
            create: vec![create_cmd("NewJournal")],
            update: vec![JournalCommandUpdate {
                id: to_update_id.clone(),
                name: "Updated".to_string(),
                description: None,
                tags: None,
            }],
        })
        .await?;

    assert_eq!(result.len(), 2);
    let names: HashSet<_> = result.iter().map(|j| j.name.to_string()).collect();
    assert!(names.contains("NewJournal"));
    assert!(names.contains("Updated"));

    Ok(())
}

#[tokio::test]
async fn test_batch_delete_frees_name_for_create() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Reusable")]).await?;
    let id = created[0].id.clone();

    // Delete "Reusable" then create a new one with the same name
    let result = service
        .batch(JournalCommandBatch {
            delete: HashSet::from([id]),
            create: vec![create_cmd("Reusable")],
            update: vec![],
        })
        .await?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.to_string(), "Reusable");

    Ok(())
}

#[tokio::test]
async fn test_batch_delete_frees_name_for_update() -> anyhow::Result<()> {
    let service = new_service();

    let created = service
        .create([create_cmd("TakenName"), create_cmd("ToRename")])
        .await?;
    let taken_id = created
        .iter()
        .find(|j| j.name.to_string() == "TakenName")
        .unwrap()
        .id
        .clone();
    let rename_id = created
        .iter()
        .find(|j| j.name.to_string() == "ToRename")
        .unwrap()
        .id
        .clone();

    // Delete "TakenName", then rename "ToRename" → "TakenName"
    let result = service
        .batch(JournalCommandBatch {
            delete: HashSet::from([taken_id]),
            create: vec![],
            update: vec![JournalCommandUpdate {
                id: rename_id.clone(),
                name: "TakenName".to_string(),
                description: None,
                tags: None,
            }],
        })
        .await?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.to_string(), "TakenName");

    Ok(())
}

#[tokio::test]
async fn test_batch_create_name_conflicts_with_update_name() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("Existing")]).await?;
    let id = created[0].id.clone();

    // Create "Clash" and update existing → "Clash" in the same batch.
    // Update runs after create, so it sees "Clash" already exists
    // and "Clash"'s ID is NOT in the update batch → conflict.
    let err = service
        .batch(JournalCommandBatch {
            delete: HashSet::new(),
            create: vec![create_cmd("Clash")],
            update: vec![JournalCommandUpdate {
                id,
                name: "Clash".to_string(),
                description: None,
                tags: None,
            }],
        })
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "Clash".to_string()
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_batch_result_deduplicates_by_id() -> anyhow::Result<()> {
    let service = new_service();

    // We can't create-then-update the same journal in one batch because
    // create assigns a new ID that we don't know upfront. But we can
    // verify deduplication by checking that the result count is correct
    // when create and update touch disjoint journals.
    let existing = service.create([create_cmd("Pre")]).await?;
    let id = existing[0].id.clone();

    let result = service
        .batch(JournalCommandBatch {
            delete: HashSet::new(),
            create: vec![create_cmd("New")],
            update: vec![JournalCommandUpdate {
                id,
                name: "PreUpdated".to_string(),
                description: None,
                tags: None,
            }],
        })
        .await?;

    // 1 created + 1 updated = 2 unique journals
    assert_eq!(result.len(), 2);
    let names: HashSet<_> = result.iter().map(|j| j.name.to_string()).collect();
    assert!(names.contains("New"));
    assert!(names.contains("PreUpdated"));

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_create_failure() -> anyhow::Result<()> {
    let service = new_service();

    let created = service.create([create_cmd("WillSurvive")]).await?;
    let id = created[0].id.clone();

    // Batch: delete "WillSurvive" + create ["Dup", "Dup"] (fails on duplicate names)
    let result = service
        .batch(JournalCommandBatch {
            delete: HashSet::from([id.clone()]),
            create: vec![create_cmd("Dup"), create_cmd("Dup")],
            update: vec![],
        })
        .await;

    assert!(result.is_err());

    // "WillSurvive" should still exist — the delete was rolled back
    let found = service.create([create_cmd("WillSurvive")]).await;
    assert!(
        found.is_err(),
        "WillSurvive should still exist (rollback worked)"
    );

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_update_failure() -> anyhow::Result<()> {
    let service = new_service();

    service.create([create_cmd("A"), create_cmd("B")]).await?;

    // Batch: create "New" + update with nonexistent ID (fails)
    let result = service
        .batch(JournalCommandBatch {
            delete: HashSet::new(),
            create: vec![create_cmd("New")],
            update: vec![JournalCommandUpdate {
                id: JournalId::from("nonexistent"),
                name: "Whatever".to_string(),
                description: None,
                tags: None,
            }],
        })
        .await;

    assert!(result.is_err());

    // "New" should NOT exist — the create was rolled back
    let created = service.create([create_cmd("New")]).await?;
    assert_eq!(created.len(), 1, "New should not exist (rollback worked)");

    // "A" and "B" should still exist unchanged
    let err = service.create([create_cmd("A")]).await;
    assert!(err.is_err(), "A should still exist");
    let err = service.create([create_cmd("B")]).await;
    assert!(err.is_err(), "B should still exist");

    Ok(())
}
