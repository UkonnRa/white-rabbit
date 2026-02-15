use std::collections::HashSet;
use std::sync::Arc;

use database_seaorm::repository::SeaOrmSession;
use sea_orm::Database;

use database_seaorm_migration::{Migrator, MigratorTrait};
use domain::journal::JournalId;
use domain::journal::command::{JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate};
use domain::journal::service::JournalService;

use super::SeaOrmJournalRepository;

// ── Helpers ──────────────────────────────────────────────────────

async fn new_service() -> (JournalService<SeaOrmJournalRepository>, SeaOrmSession) {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    let service = JournalService {
        repository: Arc::new(SeaOrmJournalRepository),
    };
    let sess = SeaOrmSession::new(db);
    (service, sess)
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
    let (service, mut sess) = new_service().await;

    let journals = service
        .create(
            &mut sess,
            [JournalCommandCreate {
                name: "My Ledger".to_string(),
                description: "Personal finance".to_string(),
                tags: HashSet::from(["finance".to_string()]),
            }],
        )
        .await?;

    assert_eq!(journals.len(), 1);
    assert_eq!(journals[0].name.to_string(), "My Ledger");
    assert_eq!(journals[0].description, "Personal finance");

    Ok(())
}

#[tokio::test]
async fn test_create_batch_multiple_journals() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let journals = service
        .create(
            &mut sess,
            [create_cmd("Alpha"), create_cmd("Beta"), create_cmd("Gamma")],
        )
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
    let (service, mut sess) = new_service().await;
    let journals = service
        .create(&mut sess, Vec::<JournalCommandCreate>::new())
        .await?;
    assert!(journals.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_create_empty_name_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let err = service
        .create(&mut sess, [create_cmd("")])
        .await
        .unwrap_err();
    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::NonEmpty)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_duplicate_names_within_batch_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let err = service
        .create(&mut sess, [create_cmd("Ledger"), create_cmd("Ledger")])
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
    let (service, mut sess) = new_service().await;

    service.create(&mut sess, [create_cmd("Existing")]).await?;

    let err = service
        .create(&mut sess, [create_cmd("Existing")])
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

// ── Update tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_update_single_journal() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service.create(&mut sess, [create_cmd("Original")]).await?;
    let id = created[0].id.clone();

    let updated = service
        .update(
            &mut sess,
            [JournalCommandUpdate {
                id: id.clone(),
                name: "Renamed".to_string(),
                description: Some("New desc".to_string()),
                tags: Some(HashSet::from(["tag1".to_string()])),
            }],
        )
        .await?;

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].name.to_string(), "Renamed");
    assert_eq!(updated[0].description, "New desc");

    Ok(())
}

#[tokio::test]
async fn test_update_nonexistent_id_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let err = service
        .update(
            &mut sess,
            [JournalCommandUpdate {
                id: JournalId::from("nonexistent"),
                name: "Whatever".to_string(),
                description: None,
                tags: None,
            }],
        )
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::NotFound)
    );

    Ok(())
}

#[tokio::test]
async fn test_update_name_swap_is_allowed() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service
        .create(&mut sess, [create_cmd("Alpha"), create_cmd("Beta")])
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

    let updated = service
        .update(
            &mut sess,
            [
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
            ],
        )
        .await?;

    let alpha_journal = updated.iter().find(|j| j.id == alpha_id).unwrap();
    let beta_journal = updated.iter().find(|j| j.id == beta_id).unwrap();
    assert_eq!(alpha_journal.name.to_string(), "Beta");
    assert_eq!(beta_journal.name.to_string(), "Alpha");

    Ok(())
}

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_single_journal() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service.create(&mut sess, [create_cmd("ToDelete")]).await?;
    let id = created[0].id.clone();

    let deleted = service.delete(&mut sess, [id.clone()]).await?;
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].name.to_string(), "ToDelete");

    let recreated = service.create(&mut sess, [create_cmd("ToDelete")]).await?;
    assert_eq!(recreated.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_id_is_silently_ignored() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let deleted = service
        .delete(&mut sess, [JournalId::from("nonexistent")])
        .await?;
    assert!(deleted.is_empty());
    Ok(())
}

// ── Batch tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_delete_create_update_together() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service
        .create(&mut sess, [create_cmd("ToDelete"), create_cmd("ToUpdate")])
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
        .batch(
            &mut sess,
            JournalCommandBatch {
                delete: HashSet::from([to_delete_id]),
                create: vec![create_cmd("NewJournal")],
                update: vec![JournalCommandUpdate {
                    id: to_update_id.clone(),
                    name: "Updated".to_string(),
                    description: None,
                    tags: None,
                }],
            },
        )
        .await?;

    assert_eq!(result.len(), 2);
    let names: HashSet<_> = result.iter().map(|j| j.name.to_string()).collect();
    assert!(names.contains("NewJournal"));
    assert!(names.contains("Updated"));

    Ok(())
}

#[tokio::test]
async fn test_batch_delete_frees_name_for_create() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service.create(&mut sess, [create_cmd("Reusable")]).await?;
    let id = created[0].id.clone();

    let result = service
        .batch(
            &mut sess,
            JournalCommandBatch {
                delete: HashSet::from([id]),
                create: vec![create_cmd("Reusable")],
                update: vec![],
            },
        )
        .await?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.to_string(), "Reusable");

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_create_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service
        .create(&mut sess, [create_cmd("WillSurvive")])
        .await?;
    let id = created[0].id.clone();

    let result = service
        .batch(
            &mut sess,
            JournalCommandBatch {
                delete: HashSet::from([id.clone()]),
                create: vec![create_cmd("Dup"), create_cmd("Dup")],
                update: vec![],
            },
        )
        .await;

    assert!(result.is_err());

    let found = service.create(&mut sess, [create_cmd("WillSurvive")]).await;
    assert!(
        found.is_err(),
        "WillSurvive should still exist (rollback worked)"
    );

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_update_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    service
        .create(&mut sess, [create_cmd("A"), create_cmd("B")])
        .await?;

    let result = service
        .batch(
            &mut sess,
            JournalCommandBatch {
                delete: HashSet::new(),
                create: vec![create_cmd("New")],
                update: vec![JournalCommandUpdate {
                    id: JournalId::from("nonexistent"),
                    name: "Whatever".to_string(),
                    description: None,
                    tags: None,
                }],
            },
        )
        .await;

    assert!(result.is_err());

    let created = service.create(&mut sess, [create_cmd("New")]).await?;
    assert_eq!(created.len(), 1, "New should not exist (rollback worked)");

    let err = service.create(&mut sess, [create_cmd("A")]).await;
    assert!(err.is_err(), "A should still exist");
    let err = service.create(&mut sess, [create_cmd("B")]).await;
    assert!(err.is_err(), "B should still exist");

    Ok(())
}
