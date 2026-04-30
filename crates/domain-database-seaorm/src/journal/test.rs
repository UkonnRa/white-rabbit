use std::collections::HashSet;
use std::sync::Arc;

use database_seaorm::repository::SeaOrmSession;
use sea_orm::Database;

use database_seaorm_migration::{Migrator, MigratorTrait};
use domain::journal::JournalId;
use domain::journal::command::{
    JournalCommand, JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate,
};
use domain::journal::event::JournalEvent;
use domain::journal::service::JournalService;
use shared::WriteService;

use super::SeaOrmJournalRepository;
use crate::account::SeaOrmAccountRepository;

// ── Helpers ──────────────────────────────────────────────────────

async fn new_service() -> (
    JournalService<SeaOrmJournalRepository, SeaOrmAccountRepository>,
    SeaOrmSession,
) {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    let service = JournalService {
        journal_repo: Arc::new(SeaOrmJournalRepository),
        account_repo: Arc::new(SeaOrmAccountRepository),
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

fn created_id(events: &[JournalEvent]) -> JournalId {
    events
        .iter()
        .find_map(|e| match e {
            JournalEvent::Created(c) => Some(c.id.clone()),
            _ => None,
        })
        .expect("no Created event found")
}

fn created_id_by_name(events: &[JournalEvent], name: &str) -> JournalId {
    events
        .iter()
        .find_map(|e| match e {
            JournalEvent::Created(c) if c.name == name => Some(c.id.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no Created event with name '{name}'"))
}

// ── Create tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_single_journal() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Create(JournalCommandCreate {
                name: "My Ledger".to_string(),
                description: "Personal finance".to_string(),
                tags: HashSet::from(["finance".to_string()]),
            }),
        )
        .await?;

    assert_eq!(result.events.len(), 1);
    let JournalEvent::Created(e) = &result.events[0] else {
        panic!("expected Created")
    };
    assert_eq!(e.name, "My Ledger");
    assert_eq!(e.description, "Personal finance");

    Ok(())
}

#[tokio::test]
async fn test_create_batch_multiple_journals() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Alpha"), create_cmd("Beta"), create_cmd("Gamma")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    assert_eq!(result.events.len(), 3);
    let names: HashSet<_> = result
        .events
        .iter()
        .filter_map(|e| match e {
            JournalEvent::Created(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();
    assert!(names.contains("Alpha"));
    assert!(names.contains("Beta"));
    assert!(names.contains("Gamma"));

    Ok(())
}

#[tokio::test]
async fn test_create_empty_batch_succeeds() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch::default()),
        )
        .await?;
    assert!(result.events.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_create_empty_name_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let err = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("")))
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
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Ledger"), create_cmd("Ledger")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
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

    service
        .handle(&mut sess, JournalCommand::Create(create_cmd("Existing")))
        .await?;

    let err = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("Existing")))
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

    let created = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("Original")))
        .await?;
    let id = created_id(&created.events);

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Update(JournalCommandUpdate {
                id: id.clone(),
                name: "Renamed".to_string(),
                description: Some("New desc".to_string()),
                tags: Some(HashSet::from(["tag1".to_string()])),
            }),
        )
        .await?;

    assert_eq!(result.events.len(), 1);
    let JournalEvent::Updated(e) = &result.events[0] else {
        panic!("expected Updated")
    };
    assert_eq!(e.name.as_deref(), Some("Renamed"));
    assert_eq!(e.description.as_deref(), Some("New desc"));

    Ok(())
}

#[tokio::test]
async fn test_update_nonexistent_id_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let err = service
        .handle(
            &mut sess,
            JournalCommand::Update(JournalCommandUpdate {
                id: JournalId::from("nonexistent"),
                name: "Whatever".to_string(),
                description: None,
                tags: None,
            }),
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
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Alpha"), create_cmd("Beta")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;
    let alpha_id = created_id_by_name(&created.events, "Alpha");
    let beta_id = created_id_by_name(&created.events, "Beta");

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![],
                update: vec![
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
                delete: HashSet::new(),
            }),
        )
        .await?;

    let alpha_event = result
        .events
        .iter()
        .find_map(|e| match e {
            JournalEvent::Updated(u) if u.id == alpha_id => Some(u),
            _ => None,
        })
        .expect("no Updated event for alpha");
    let beta_event = result
        .events
        .iter()
        .find_map(|e| match e {
            JournalEvent::Updated(u) if u.id == beta_id => Some(u),
            _ => None,
        })
        .expect("no Updated event for beta");
    assert_eq!(alpha_event.name.as_deref(), Some("Beta"));
    assert_eq!(beta_event.name.as_deref(), Some("Alpha"));

    Ok(())
}

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_single_journal() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("ToDelete")))
        .await?;
    let id = created_id(&created.events);

    service
        .handle(&mut sess, JournalCommand::Delete(HashSet::from([id])))
        .await?;

    let recreated = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("ToDelete")))
        .await?;
    assert_eq!(recreated.events.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_id_is_silently_ignored() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    service
        .handle(
            &mut sess,
            JournalCommand::Delete(HashSet::from([JournalId::from("nonexistent")])),
        )
        .await?;
    Ok(())
}

// ── Batch tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_delete_create_update_together() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let setup = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("ToDelete"), create_cmd("ToUpdate")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;
    let to_delete_id = created_id_by_name(&setup.events, "ToDelete");
    let to_update_id = created_id_by_name(&setup.events, "ToUpdate");

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                delete: HashSet::from([to_delete_id]),
                create: vec![create_cmd("NewJournal")],
                update: vec![JournalCommandUpdate {
                    id: to_update_id.clone(),
                    name: "Updated".to_string(),
                    description: None,
                    tags: None,
                }],
            }),
        )
        .await?;

    assert_eq!(result.events.len(), 3);
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e, JournalEvent::Created(c) if c.name == "NewJournal"))
    );
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e, JournalEvent::Updated(u) if u.name.as_deref() == Some("Updated")))
    );

    Ok(())
}

#[tokio::test]
async fn test_batch_delete_frees_name_for_create() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("Reusable")))
        .await?;
    let id = created_id(&created.events);

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                delete: HashSet::from([id]),
                create: vec![create_cmd("Reusable")],
                update: vec![],
            }),
        )
        .await?;

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e, JournalEvent::Created(c) if c.name == "Reusable"))
    );

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_create_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

    let created = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("WillSurvive")))
        .await?;
    let id = created_id(&created.events);

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                delete: HashSet::from([id.clone()]),
                create: vec![create_cmd("Dup"), create_cmd("Dup")],
                update: vec![],
            }),
        )
        .await;

    assert!(result.is_err());

    let found = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("WillSurvive")))
        .await;
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
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("A"), create_cmd("B")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                delete: HashSet::new(),
                create: vec![create_cmd("New")],
                update: vec![JournalCommandUpdate {
                    id: JournalId::from("nonexistent"),
                    name: "Whatever".to_string(),
                    description: None,
                    tags: None,
                }],
            }),
        )
        .await;

    assert!(result.is_err());

    let created = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("New")))
        .await?;
    assert_eq!(
        created.events.len(),
        1,
        "New should not exist (rollback worked)"
    );

    let err = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("A")))
        .await;
    assert!(err.is_err(), "A should still exist");
    let err = service
        .handle(&mut sess, JournalCommand::Create(create_cmd("B")))
        .await;
    assert!(err.is_err(), "B should still exist");

    Ok(())
}

// ── Specification tests ──────────────────────────────────────────

use domain::journal::specification::JournalSpecification;
use shared::ReadRepository;

#[tokio::test]
async fn test_spec_find_by_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Alpha"), create_cmd("Beta")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;
    let alpha_id = created_id_by_name(&result.events, "Alpha");

    let found = repo
        .find_all(&sess, &JournalSpecification::id(alpha_id.clone()), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert!(found.contains_key(&alpha_id));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_name() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Alpha"), create_cmd("Beta"), create_cmd("Gamma")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let found = repo
        .find_all(
            &sess,
            &JournalSpecification::names(["Alpha", "Gamma"]),
            None,
        )
        .await?;
    assert_eq!(found.len(), 2);
    let names: HashSet<_> = found.values().map(|j| j.name.to_string()).collect();
    assert!(names.contains("Alpha"));
    assert!(names.contains("Gamma"));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_tag() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![
                    JournalCommandCreate {
                        name: "Personal".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["finance".to_string(), "personal".to_string()]),
                    },
                    JournalCommandCreate {
                        name: "Business".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["finance".to_string(), "business".to_string()]),
                    },
                    create_cmd("NoTags"),
                ],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let found = repo
        .find_all(&sess, &JournalSpecification::tag("personal"), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().name.to_string(), "Personal");

    let found = repo
        .find_all(&sess, &JournalSpecification::tag("finance"), None)
        .await?;
    assert_eq!(found.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_full_text() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![
                    JournalCommandCreate {
                        name: "My Ledger".to_string(),
                        description: "Tracks personal spending".to_string(),
                        tags: HashSet::new(),
                    },
                    JournalCommandCreate {
                        name: "Work".to_string(),
                        description: "Business expenses".to_string(),
                        tags: HashSet::new(),
                    },
                    create_cmd("Empty"),
                ],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let found = repo
        .find_all(&sess, &JournalSpecification::full_text("Ledger"), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(&sess, &JournalSpecification::full_text("personal"), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_no_match_returns_empty() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(&mut sess, JournalCommand::Create(create_cmd("Alpha")))
        .await?;

    let found = repo
        .find_all(&sess, &JournalSpecification::name("Nonexistent"), None)
        .await?;
    assert!(found.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_spec_all_and() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    let result = service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![
                    JournalCommandCreate {
                        name: "Alpha".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["tag1".to_string()]),
                    },
                    JournalCommandCreate {
                        name: "Beta".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["tag1".to_string()]),
                    },
                    create_cmd("Gamma"),
                ],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;
    let alpha_id = created_id_by_name(&result.events, "Alpha");

    let spec = JournalSpecification::tag("tag1") & JournalSpecification::id(alpha_id.clone());
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);
    assert!(found.contains_key(&alpha_id));

    Ok(())
}

#[tokio::test]
async fn test_spec_any_or() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Alpha"), create_cmd("Beta"), create_cmd("Gamma")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let spec = JournalSpecification::name("Alpha") | JournalSpecification::name("Gamma");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 2);
    let names: HashSet<_> = found.values().map(|j| j.name.to_string()).collect();
    assert!(names.contains("Alpha"));
    assert!(names.contains("Gamma"));

    Ok(())
}

#[tokio::test]
async fn test_spec_not() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![create_cmd("Alpha"), create_cmd("Beta"), create_cmd("Gamma")],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let spec = !JournalSpecification::name("Alpha");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 2);
    let names: HashSet<_> = found.values().map(|j| j.name.to_string()).collect();
    assert!(names.contains("Beta"));
    assert!(names.contains("Gamma"));

    Ok(())
}

#[tokio::test]
async fn test_spec_nested_combination() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![
                    JournalCommandCreate {
                        name: "Alpha".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["a".to_string()]),
                    },
                    JournalCommandCreate {
                        name: "Beta".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["b".to_string()]),
                    },
                    JournalCommandCreate {
                        name: "Gamma".to_string(),
                        description: String::new(),
                        tags: HashSet::from(["a".to_string()]),
                    },
                ],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    // (tag=a AND name=Alpha) OR name=Beta -> Alpha, Beta
    let spec = (JournalSpecification::tag("a") & JournalSpecification::name("Alpha"))
        | JournalSpecification::name("Beta");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 2);
    let names: HashSet<_> = found.values().map(|j| j.name.to_string()).collect();
    assert!(names.contains("Alpha"));
    assert!(names.contains("Beta"));

    Ok(())
}

#[tokio::test]
async fn test_spec_limit() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Batch(JournalCommandBatch {
                create: vec![
                    create_cmd("A"),
                    create_cmd("B"),
                    create_cmd("C"),
                    create_cmd("D"),
                ],
                update: vec![],
                delete: HashSet::new(),
            }),
        )
        .await?;

    let spec = JournalSpecification::names(["A", "B", "C", "D"]);
    let found = repo.find_all(&sess, &spec, Some(2)).await?;
    assert_eq!(found.len(), 2);

    Ok(())
}

// ── Case-insensitive query tests ─────────────────────────────────

#[tokio::test]
async fn test_spec_find_by_name_case_insensitive() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Create(create_cmd("Personal Ledger")),
        )
        .await?;

    let found = repo
        .find_all(&sess, &JournalSpecification::name("personal ledger"), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(&sess, &JournalSpecification::name("PERSONAL LEDGER"), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_tag_case_insensitive() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Create(JournalCommandCreate {
                name: "Tagged".to_string(),
                description: String::new(),
                tags: HashSet::from(["Finance".to_string()]),
            }),
        )
        .await?;

    let found = repo
        .find_all(&sess, &JournalSpecification::tag("finance"), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(&sess, &JournalSpecification::tag("FINANCE"), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_full_text_case_insensitive() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmJournalRepository;

    service
        .handle(
            &mut sess,
            JournalCommand::Create(JournalCommandCreate {
                name: "Investment Portfolio".to_string(),
                description: "Tracks Stock Holdings".to_string(),
                tags: HashSet::new(),
            }),
        )
        .await?;

    let found = repo
        .find_all(&sess, &JournalSpecification::full_text("investment"), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(
            &sess,
            &JournalSpecification::full_text("STOCK HOLDINGS"),
            None,
        )
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}
