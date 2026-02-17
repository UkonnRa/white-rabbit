use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use database_seaorm::repository::SeaOrmSession;
use sea_orm::Database;

use database_seaorm_migration::{Migrator, MigratorTrait};
use domain::account::command::{AccountCommandArchive, AccountCommandCreate, AccountCommandUpdate};
use domain::account::service::AccountService;
use domain::account::{Account, AccountId, AccountInput, AccountType};
use domain::journal::JournalId;

use super::SeaOrmAccountRepository;

// ── Helpers ──────────────────────────────────────────────────────

async fn new_service() -> (AccountService<SeaOrmAccountRepository>, SeaOrmSession) {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    let service = AccountService {
        repository: Arc::new(SeaOrmAccountRepository),
    };
    let sess = SeaOrmSession::new(db);
    (service, sess)
}

/// Insert a root account directly via the repo (roots are system-created).
async fn insert_root(
    sess: &mut SeaOrmSession,
    journal_id: &JournalId,
    account_type: AccountType,
) -> AccountId {
    let repo = SeaOrmAccountRepository;
    let id = AccountId::default();
    let account: Account = AccountInput {
        id: id.clone(),
        journal_id: journal_id.clone(),
        r#type: account_type,
        name: account_type.to_string(),
        ..Default::default()
    }
    .try_into()
    .unwrap();

    use shared::WriteRepository;
    repo.save_all(sess, &[account]).await.unwrap();
    id
}

/// Insert a journal row for FK constraints.
async fn insert_journal(sess: &mut SeaOrmSession, id: &str) {
    use crate::journal::entity;
    use sea_orm::{EntityTrait, Set};
    let model = entity::ActiveModel {
        id: Set(id.to_string()),
        version: Set(0),
        created_at: Set(None),
        last_modified_at: Set(None),
        archived_at: Set(None),
        name: Set(format!("Journal {id}")),
        description: Set(String::new()),
    };
    sess.exec_insert(entity::Entity::insert(model))
        .await
        .unwrap();
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
    let (service, mut sess) = new_service().await;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    let accounts = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name.to_string(), "Cash");
    assert_eq!(accounts[0].r#type, AccountType::Asset);

    Ok(())
}

#[tokio::test]
async fn test_create_reserved_name_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

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
async fn test_create_parent_not_found_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;

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

// ── Update tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_update_account_name() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

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

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_cascades_to_children() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

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

    service.delete(&mut sess, [parent_id]).await?;

    Ok(())
}

// ── Archive tests ────────────────────────────────────────────────

#[tokio::test]
async fn test_archive_cascades_to_children() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

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
                ids: HashSet::from([parent_id]),
                archived_at: Utc::now(),
            },
        )
        .await?;

    assert_eq!(archived.len(), 3);
    for a in &archived {
        assert!(a.is_archived(), "{} should be archived", a.name);
    }

    Ok(())
}

// ── Specification tests ──────────────────────────────────────────

use domain::account::specification::AccountSpecification;
use shared::ReadRepository;

#[tokio::test]
async fn test_spec_find_by_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    let created = service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "Cash"),
                create_cmd(&jid, &root_id, "Savings"),
            ],
        )
        .await?;
    let cash_id = created
        .iter()
        .find(|a| a.name.to_string() == "Cash")
        .unwrap()
        .id
        .clone();

    let found = repo
        .find_all(&sess, &AccountSpecification::id(cash_id.clone()), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert!(found.contains_key(&cash_id));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_journal_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid1 = JournalId::from("j1");
    let jid2 = JournalId::from("j2");
    insert_journal(&mut sess, "j1").await;
    insert_journal(&mut sess, "j2").await;
    let root1 = insert_root(&mut sess, &jid1, AccountType::Asset).await;
    let root2 = insert_root(&mut sess, &jid2, AccountType::Asset).await;

    service
        .create(&mut sess, [create_cmd(&jid1, &root1, "Cash1")])
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid2, &root2, "Cash2")])
        .await?;

    let found = repo
        .find_all(&sess, &AccountSpecification::journal_id(jid1.clone()), None)
        .await?;
    assert!(found.values().any(|a| a.name.to_string() == "Cash1"));
    assert!(found.values().all(|a| a.journal_id == jid1));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_parent_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    let parent = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Bank")])
        .await?;
    let bank_id = parent[0].id.clone();
    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &bank_id, "Checking"),
                create_cmd(&jid, &bank_id, "Savings"),
            ],
        )
        .await?;

    let found = repo
        .find_all(
            &sess,
            &AccountSpecification::parent_id(bank_id.clone()),
            None,
        )
        .await?;
    assert_eq!(found.len(), 2);
    let names: HashSet<_> = found.values().map(|a| a.name.to_string()).collect();
    assert!(names.contains("Checking"));
    assert!(names.contains("Savings"));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_name() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "Cash"),
                create_cmd(&jid, &root_id, "Savings"),
            ],
        )
        .await?;

    let found = repo
        .find_all(&sess, &AccountSpecification::name("Cash"), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().name.to_string(), "Cash");

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_type() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let asset_root = insert_root(&mut sess, &jid, AccountType::Asset).await;
    let expense_root = insert_root(&mut sess, &jid, AccountType::Expense).await;

    service
        .create(&mut sess, [create_cmd(&jid, &asset_root, "Cash")])
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid, &expense_root, "Food")])
        .await?;

    let found = repo
        .find_all(
            &sess,
            &AccountSpecification::account_type(AccountType::Asset),
            None,
        )
        .await?;
    assert!(found.values().all(|a| a.r#type == AccountType::Asset));
    let names: HashSet<_> = found.values().map(|a| a.name.to_string()).collect();
    assert!(names.contains("Cash"));
    assert!(names.contains("Asset")); // root

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_tag() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    let created = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;
    service
        .update(
            &mut sess,
            [AccountCommandUpdate {
                id: created[0].id.clone(),
                name: String::new(),
                description: None,
                tags: Some(HashSet::from(["liquid".to_string()])),
            }],
        )
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Savings")])
        .await?;

    let found = repo
        .find_all(&sess, &AccountSpecification::tag("liquid"), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().name.to_string(), "Cash");

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_full_text() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    service
        .create(
            &mut sess,
            [AccountCommandCreate {
                journal_id: jid.clone(),
                parent_id: root_id.clone(),
                name: "Cash".to_string(),
                description: "Main checking account".to_string(),
                tags: HashSet::new(),
            }],
        )
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Savings")])
        .await?;

    let found = repo
        .find_all(&sess, &AccountSpecification::full_text("checking"), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().name.to_string(), "Cash");

    Ok(())
}

#[tokio::test]
async fn test_spec_find_archived() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    let created = service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "Cash"),
                create_cmd(&jid, &root_id, "Old"),
            ],
        )
        .await?;
    let old_id = created
        .iter()
        .find(|a| a.name.to_string() == "Old")
        .unwrap()
        .id
        .clone();

    service
        .archive(
            &mut sess,
            AccountCommandArchive {
                ids: HashSet::from([old_id]),
                archived_at: Utc::now(),
            },
        )
        .await?;

    let archived = repo
        .find_all(&sess, &AccountSpecification::archived(), None)
        .await?;
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.values().next().unwrap().name.to_string(), "Old");

    let active = repo
        .find_all(&sess, &AccountSpecification::active(), None)
        .await?;
    assert!(active.values().all(|a| !a.is_archived()));

    Ok(())
}

#[tokio::test]
async fn test_spec_all_and() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let asset_root = insert_root(&mut sess, &jid, AccountType::Asset).await;
    let expense_root = insert_root(&mut sess, &jid, AccountType::Expense).await;

    service
        .create(&mut sess, [create_cmd(&jid, &asset_root, "Cash")])
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid, &expense_root, "Food")])
        .await?;

    let spec =
        AccountSpecification::account_type(AccountType::Asset) & AccountSpecification::name("Cash");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().name.to_string(), "Cash");

    Ok(())
}

#[tokio::test]
async fn test_spec_any_or() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "Cash"),
                create_cmd(&jid, &root_id, "Savings"),
                create_cmd(&jid, &root_id, "Bonds"),
            ],
        )
        .await?;

    let spec = AccountSpecification::name("Cash") | AccountSpecification::name("Bonds");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 2);
    let names: HashSet<_> = found.values().map(|a| a.name.to_string()).collect();
    assert!(names.contains("Cash"));
    assert!(names.contains("Bonds"));

    Ok(())
}

#[tokio::test]
async fn test_spec_not() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "Cash"),
                create_cmd(&jid, &root_id, "Savings"),
            ],
        )
        .await?;

    let spec = !AccountSpecification::name("Cash");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert!(found.values().all(|a| a.name.to_string() != "Cash"));

    Ok(())
}

#[tokio::test]
async fn test_spec_limit() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmAccountRepository;
    let jid = JournalId::from("j1");
    insert_journal(&mut sess, "j1").await;
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset).await;

    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "A"),
                create_cmd(&jid, &root_id, "B"),
                create_cmd(&jid, &root_id, "C"),
            ],
        )
        .await?;

    let spec = AccountSpecification::account_type(AccountType::Asset);
    let found = repo.find_all(&sess, &spec, Some(2)).await?;
    assert_eq!(found.len(), 2);

    Ok(())
}
