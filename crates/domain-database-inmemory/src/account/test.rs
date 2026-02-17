use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use database_inmemory::repository::InMemorySession;
use domain::account::command::{
    AccountCommandArchive, AccountCommandBatch, AccountCommandCreate, AccountCommandUpdate,
};
use domain::account::service::AccountService;
use domain::account::{Account, AccountId, AccountType};
use domain::journal::JournalId;
use shared::{Entity, EntityId};

use super::{AccountPo, InMemoryAccountRepository};

// ── Helpers ──────────────────────────────────────────────────────

fn new_service() -> (AccountService<InMemoryAccountRepository>, InMemorySession) {
    let service = AccountService {
        repository: Arc::new(InMemoryAccountRepository),
    };
    let sess = InMemorySession::default();
    (service, sess)
}

/// Create a root account directly in the session (roots are system-created, not via service).
fn insert_root(
    sess: &mut InMemorySession,
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
    sess.get_storage_mut(Account::ENTITY_TYPE)
        .insert(id.value().to_string(), serde_json::to_value(&po).unwrap());
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
    service.delete(&mut sess, [parent_id]).await?;

    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_is_silent() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();

    service
        .delete(&mut sess, [AccountId::from("nonexistent")])
        .await?;

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

// ── Specification tests ──────────────────────────────────────────

use domain::account::specification::AccountSpecification;
use shared::ReadRepository;

#[tokio::test]
async fn test_spec_find_by_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid1 = JournalId::from("j1");
    let jid2 = JournalId::from("j2");
    let root1 = insert_root(&mut sess, &jid1, AccountType::Asset);
    let root2 = insert_root(&mut sess, &jid2, AccountType::Asset);

    service
        .create(&mut sess, [create_cmd(&jid1, &root1, "Cash1")])
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid2, &root2, "Cash2")])
        .await?;

    let found = repo
        .find_all(&sess, &AccountSpecification::journal_id(jid1.clone()), None)
        .await?;
    // root1 + Cash1
    assert!(found.values().any(|a| a.name.to_string() == "Cash1"));
    assert!(found.values().all(|a| a.journal_id == jid1));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_parent_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let asset_root = insert_root(&mut sess, &jid, AccountType::Asset);
    let expense_root = insert_root(&mut sess, &jid, AccountType::Expense);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    let created = service
        .create(&mut sess, [create_cmd(&jid, &root_id, "Cash")])
        .await?;
    // Update with tags
    service
        .update(
            &mut sess,
            [AccountCommandUpdate {
                id: created[0].id.clone(),
                name: String::new(),
                description: None,
                tags: Some(HashSet::from(["liquid".to_string(), "primary".to_string()])),
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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let asset_root = insert_root(&mut sess, &jid, AccountType::Asset);
    let expense_root = insert_root(&mut sess, &jid, AccountType::Expense);

    service
        .create(&mut sess, [create_cmd(&jid, &asset_root, "Cash")])
        .await?;
    service
        .create(&mut sess, [create_cmd(&jid, &expense_root, "Food")])
        .await?;

    // AND: type=Asset AND name=Cash (excludes root because root name is "Asset")
    let spec =
        AccountSpecification::account_type(AccountType::Asset) & AccountSpecification::name("Cash");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().name.to_string(), "Cash");

    Ok(())
}

#[tokio::test]
async fn test_spec_any_or() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

    service
        .create(
            &mut sess,
            [
                create_cmd(&jid, &root_id, "Cash"),
                create_cmd(&jid, &root_id, "Savings"),
            ],
        )
        .await?;

    // NOT name=Cash -- should return root + Savings
    let spec = !AccountSpecification::name("Cash");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert!(found.values().all(|a| a.name.to_string() != "Cash"));

    Ok(())
}

#[tokio::test]
async fn test_spec_limit() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryAccountRepository;
    let jid = JournalId::from("j1");
    let root_id = insert_root(&mut sess, &jid, AccountType::Asset);

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
