use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use database_inmemory::repository::InMemorySession;
use domain::account::command::{
    AccountCommandArchive, AccountCommandBatch, AccountCommandCreate, AccountCommandUpdate,
};
use domain::account::service::AccountService;
use domain::account::{AccountId, AccountType};
use domain::journal::JournalId;
use shared::EntityId;

use super::{AccountPo, InMemoryAccountRepository};

// ── Helpers ──────────────────────────────────────────────────────

fn new_service() -> (
    AccountService<InMemoryAccountRepository>,
    InMemorySession<AccountPo>,
) {
    let service = AccountService {
        repository: Arc::new(InMemoryAccountRepository),
    };
    let sess = InMemorySession::<AccountPo>::default();
    (service, sess)
}

/// Create a root account directly in the session (roots are system-created, not via service).
fn insert_root(
    sess: &mut InMemorySession<AccountPo>,
    journal_id: &JournalId,
    account_type: AccountType,
) -> AccountId {
    let id = AccountId::default();
    let po = AccountPo {
        id: id.clone(),
        version: 0,
        created_at: None,
        last_modified_at: None,
        archived_at: None,
        journal_id: journal_id.clone(),
        parent_id: None,
        r#type: account_type,
        name: account_type.to_string(),
        description: String::new(),
        tags: HashSet::new(),
    };
    sess.storage.insert(id.value().to_string(), po);
    id
}

fn create_cmd(journal_id: &JournalId, parent_id: &AccountId, name: &str) -> AccountCommandCreate {
    AccountCommandCreate {
        journal_id: journal_id.clone(),
        parent_id: parent_id.clone(),
        name: name.to_string(),
        description: String::new(),
        tags: HashSet::new(),
    }
}

// ── Create tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_single_account() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let accounts = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name.to_string(), "Cash");
    assert_eq!(accounts[0].r#type, AccountType::Asset);
    assert_eq!(accounts[0].parent_id.as_ref(), Some(&root_id));

    Ok(())
}

#[tokio::test]
async fn test_create_reserved_name_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let err = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Asset")])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::Conflict)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_reserved_name_case_insensitive() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Expense);

    let err = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "eXpEnSe")])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::Conflict)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_parent_not_found_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");

    let err = service
        .create(
            &mut sess,
            [create_cmd(&jid, &AccountId::from("nonexistent"), "Cash")],
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
async fn test_create_under_archived_parent_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    // Archive the root
    service
        .archive(
            &mut sess,
            AccountCommandArchive {
                ids: HashSet::from([root_id.clone()]),
                archived_at: Utc::now(),
            },
        )
        .await?;

    let err = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::Conflict)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_duplicate_sibling_name_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;

    let err = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::DuplicateValues {
            value: "Cash".to_string()
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_create_inherits_type_from_parent() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Liability);

    let accounts = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Mortgage")])
        .await?;

    assert_eq!(accounts[0].r#type, AccountType::Liability);

    Ok(())
}

// ── Update tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_update_account_name() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let created = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "OldName")])
        .await?;

    let updated = service
        .update(
            &mut sess,
            [AccountCommandUpdate {
                id: created[0].id.clone(),
                name: "NewName".to_string(),
                description: None,
                tags: None,
            }],
        )
        .await?;

    assert_eq!(updated[0].name.to_string(), "NewName");

    Ok(())
}

#[tokio::test]
async fn test_update_reserved_name_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let created = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;

    let err = service
        .update(
            &mut sess,
            [AccountCommandUpdate {
                id: created[0].id.clone(),
                name: "Equity".to_string(),
                description: None,
                tags: None,
            }],
        )
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::Conflict)
    );

    Ok(())
}

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_cascades_to_children() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let parent = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Bank")])
        .await?;
    let parent_id = parent[0].id.clone();

    service
        .create(&mut sess, [create_cmd(&jid, &parent_id, "Checking")])
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid, &parent_id, "Savings")])
        .await?;

    // Delete parent — children should be deleted too
    let deleted = service.delete(&mut sess, [parent_id]).await?;
    assert_eq!(deleted.len(), 3); // parent + 2 children

    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_is_silent() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();

    let deleted = service
        .delete(&mut sess, [AccountId::from("nonexistent")])
        .await?;
    assert!(deleted.is_empty());

    Ok(())
}

// ── Archive tests ────────────────────────────────────────────────

#[tokio::test]
async fn test_archive_cascades_to_children() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let parent = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Bank")])
        .await?;
    let parent_id = parent[0].id.clone();

    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &parent_id, "Checking"),
                create_cmd(&jid, &parent_id, "Savings"),
            ],
        )
        .await?;

    let archived = service
        .archive(
            &mut sess,
            AccountCommandArchive {
                ids: HashSet::from([parent_id.clone()]),
                archived_at: Utc::now(),
            },
        )
        .await?;

    // Parent + 2 children should all be archived
    assert_eq!(archived.len(), 3);
    for a in &archived {
        assert!(a.is_archived(), "{} should be archived", a.name);
    }

    Ok(())
}

#[tokio::test]
async fn test_archive_already_archived_is_noop() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let created = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;

    service
        .archive(
            &mut sess,
            AccountCommandArchive {
                ids: HashSet::from([created[0].id.clone()]),
                archived_at: Utc::now(),
            },
        )
        .await?;

    // Archive again — should return empty (already archived)
    let result = service
        .archive(
            &mut sess,
            AccountCommandArchive {
                ids: HashSet::from([created[0].id.clone()]),
                archived_at: Utc::now(),
            },
        )
        .await?;

    assert!(result.is_empty());

    Ok(())
}

// ── Batch tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_rollback_on_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let created = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "WillSurvive")])
        .await?;
    let id = created[0].id.clone();

    // Batch: delete WillSurvive + create with reserved name (fails)
    let result = service
        .batch(
            &mut sess,
            AccountCommandBatch {
                delete: HashSet::from([id]),
                create: vec![create_cmd(&jid, &root_id, "Asset")], // reserved name
                update: vec![],
                archive: vec![],
            },
        )
        .await;

    assert!(result.is_err());

    // WillSurvive should still exist (rollback worked)
    let err = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "WillSurvive")])
        .await;
    assert!(err.is_err(), "WillSurvive should still exist");

    Ok(())
}
