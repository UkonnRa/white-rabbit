use std::collections::HashSet;
use std::sync::Arc;

use database_inmemory::repository::InMemorySession;
use domain::account::command::AccountCommandCreate;
use domain::account::service::AccountService;
use domain::account::{Account, AccountId, AccountType};
use domain::journal::JournalId;
use domain::record::command::{
    RecordCommandBatch, RecordCommandCreate, RecordCommandItem, RecordCommandUpdate,
};
use domain::record::service::RecordService;
use domain::record::{RecordId, RecordItemKind};
use shared::{Entity, EntityId};

use crate::account::{AccountPo, InMemoryAccountRepository};
use crate::record::InMemoryRecordRepository;

// ── Helpers ──────────────────────────────────────────────────────

type TestRecordService = RecordService<InMemoryRecordRepository, InMemoryAccountRepository>;

fn new_service() -> (TestRecordService, InMemorySession) {
    let service = RecordService {
        repository: Arc::new(InMemoryRecordRepository),
        account_repository: Arc::new(InMemoryAccountRepository),
    };
    let sess = InMemorySession::default();
    (service, sess)
}

/// Insert a root account directly in the session.
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

/// Set up a journal with root accounts and child accounts for testing.
/// Returns (journal_id, asset_account_id, expense_account_id, equity_account_id).
async fn setup_accounts(
    sess: &mut InMemorySession,
) -> (JournalId, AccountId, AccountId, AccountId) {
    let jid = JournalId::from("j1");
    let asset_root = insert_root(sess, &jid, AccountType::Asset);
    let expense_root = insert_root(sess, &jid, AccountType::Expense);
    let equity_root = insert_root(sess, &jid, AccountType::Equity);

    let account_service = AccountService {
        repository: Arc::new(InMemoryAccountRepository),
    };

    let accounts = account_service
        .create(
            sess,
            [
                AccountCommandCreate {
                    journal_id: jid.clone(),
                    parent_id: asset_root,
                    name: "Cash".to_string(),
                    description: String::new(),
                    tags: HashSet::new(),
                },
                AccountCommandCreate {
                    journal_id: jid.clone(),
                    parent_id: expense_root,
                    name: "Food".to_string(),
                    description: String::new(),
                    tags: HashSet::new(),
                },
                AccountCommandCreate {
                    journal_id: jid.clone(),
                    parent_id: equity_root,
                    name: "Opening".to_string(),
                    description: String::new(),
                    tags: HashSet::new(),
                },
            ],
        )
        .await
        .unwrap();

    let asset_id = accounts
        .iter()
        .find(|a| a.name.to_string() == "Cash")
        .unwrap()
        .id
        .clone();
    let expense_id = accounts
        .iter()
        .find(|a| a.name.to_string() == "Food")
        .unwrap()
        .id
        .clone();
    let equity_id = accounts
        .iter()
        .find(|a| a.name.to_string() == "Opening")
        .unwrap()
        .id
        .clone();

    (jid, asset_id, expense_id, equity_id)
}

fn transaction_item(account_id: &AccountId, amount: &str) -> RecordCommandItem {
    RecordCommandItem {
        account_id: account_id.clone(),
        amount: amount.to_string(),
        description: String::new(),
        price: None,
        cost: HashSet::new(),
    }
}

fn create_cmd(journal_id: &JournalId, items: Vec<RecordCommandItem>) -> RecordCommandCreate {
    RecordCommandCreate {
        journal_id: journal_id.clone(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        kind: RecordItemKind::Transaction,
        items,
        description: "Test record".to_string(),
        tags: HashSet::new(),
        payee: String::new(),
    }
}

// ── Create tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_single_transaction_record() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let records = service
        .create(
            &mut sess,
            [RecordCommandCreate {
                journal_id: jid.clone(),
                date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                kind: RecordItemKind::Transaction,
                items: vec![
                    transaction_item(&asset_id, "100 USD"),
                    transaction_item(&expense_id, "100 USD"),
                ],
                description: "Lunch".to_string(),
                tags: HashSet::from(["food".to_string()]),
                payee: "Restaurant".to_string(),
            }],
        )
        .await?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].description, "Lunch");
    assert_eq!(records[0].payee, "Restaurant");
    assert_eq!(records[0].journal_id, jid);

    Ok(())
}

#[tokio::test]
async fn test_create_validation_record() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, _, _) = setup_accounts(&mut sess).await;

    let records = service
        .create(
            &mut sess,
            [RecordCommandCreate {
                journal_id: jid.clone(),
                date: chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                kind: RecordItemKind::Validation,
                items: vec![RecordCommandItem {
                    account_id: asset_id.clone(),
                    amount: "500 USD".to_string(),
                    description: "Balance check".to_string(),
                    price: None,
                    cost: HashSet::new(),
                }],
                description: "Month-end validation".to_string(),
                tags: HashSet::new(),
                payee: String::new(),
            }],
        )
        .await?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].is_balanced(), None);

    Ok(())
}

#[tokio::test]
async fn test_create_with_price() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, _, equity_id) = setup_accounts(&mut sess).await;

    let records = service
        .create(
            &mut sess,
            [RecordCommandCreate {
                journal_id: jid.clone(),
                date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                kind: RecordItemKind::Transaction,
                items: vec![
                    RecordCommandItem {
                        account_id: asset_id.clone(),
                        amount: "400 USD".to_string(),
                        description: String::new(),
                        price: Some("1.09 CAD".to_string()),
                        cost: HashSet::new(),
                    },
                    transaction_item(&equity_id, "436 CAD"),
                ],
                description: "Currency exchange".to_string(),
                tags: HashSet::new(),
                payee: String::new(),
            }],
        )
        .await?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].is_balanced(), Some(true));

    Ok(())
}

#[tokio::test]
async fn test_create_empty_batch_succeeds() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();

    let records = service
        .create(&mut sess, Vec::<RecordCommandCreate>::new())
        .await?;
    assert!(records.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_create_nonexistent_account_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, _, _) = setup_accounts(&mut sess).await;

    let err = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "100 USD"),
                    transaction_item(&AccountId::from("nonexistent"), "100 USD"),
                ],
            )],
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
async fn test_create_archived_account_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    // Archive the expense account
    let account_service = AccountService {
        repository: Arc::new(InMemoryAccountRepository),
    };
    account_service
        .archive(
            &mut sess,
            domain::account::command::AccountCommandArchive {
                ids: HashSet::from([expense_id.clone()]),
                archived_at: chrono::Utc::now(),
            },
        )
        .await?;

    let err = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "100 USD"),
                    transaction_item(&expense_id, "100 USD"),
                ],
            )],
        )
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::Conflict)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_wrong_journal_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (_, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let wrong_jid = JournalId::from("other-journal");

    let err = service
        .create(
            &mut sess,
            [create_cmd(
                &wrong_jid,
                vec![
                    transaction_item(&asset_id, "100 USD"),
                    transaction_item(&expense_id, "100 USD"),
                ],
            )],
        )
        .await
        .unwrap_err();

    assert!(matches!(
        err.error,
        domain::error::ErrorKind::Mismatch { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn test_create_invalid_amount_format_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, _, _) = setup_accounts(&mut sess).await;

    let err = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![transaction_item(&asset_id, "not-a-number")],
            )],
        )
        .await
        .unwrap_err();

    assert!(matches!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::InvalidFormat { .. })
    ));

    Ok(())
}

#[tokio::test]
async fn test_create_empty_items_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, _, _, _) = setup_accounts(&mut sess).await;

    let err = service
        .create(&mut sess, [create_cmd(&jid, vec![])])
        .await
        .unwrap_err();

    assert_eq!(
        err.error,
        domain::error::ErrorKind::Shared(shared::ErrorKind::NonEmpty)
    );

    Ok(())
}

// ── Update tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_update_description_and_date() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "50 USD"),
                    transaction_item(&expense_id, "50 USD"),
                ],
            )],
        )
        .await?;
    let id = created[0].id.clone();

    let new_date = chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
    let updated = service
        .update(
            &mut sess,
            [RecordCommandUpdate {
                id: id.clone(),
                date: Some(new_date),
                items: None,
                description: Some("Updated description".to_string()),
                tags: Some(HashSet::from(["updated".to_string()])),
                payee: Some("New Payee".to_string()),
            }],
        )
        .await?;

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].date, new_date);
    assert_eq!(updated[0].description, "Updated description");
    assert_eq!(updated[0].payee, "New Payee");

    Ok(())
}

#[tokio::test]
async fn test_update_replace_items() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, equity_id) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "50 USD"),
                    transaction_item(&expense_id, "50 USD"),
                ],
            )],
        )
        .await?;
    let id = created[0].id.clone();

    let updated = service
        .update(
            &mut sess,
            [RecordCommandUpdate {
                id,
                date: None,
                items: Some(vec![
                    transaction_item(&asset_id, "200 USD"),
                    transaction_item(&equity_id, "200 USD"),
                ]),
                description: None,
                tags: None,
                payee: None,
            }],
        )
        .await?;

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].is_balanced(), Some(true));

    Ok(())
}

#[tokio::test]
async fn test_update_keep_existing_items() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, _, equity_id) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "50 USD"),
                    transaction_item(&equity_id, "50 USD"),
                ],
            )],
        )
        .await?;
    let id = created[0].id.clone();

    let updated = service
        .update(
            &mut sess,
            [RecordCommandUpdate {
                id,
                date: None,
                items: None,
                description: Some("Only description changed".to_string()),
                tags: None,
                payee: None,
            }],
        )
        .await?;

    assert_eq!(updated[0].description, "Only description changed");
    assert_eq!(updated[0].is_balanced(), Some(true));

    Ok(())
}

#[tokio::test]
async fn test_update_nonexistent_record_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();

    let err = service
        .update(
            &mut sess,
            [RecordCommandUpdate {
                id: RecordId::from("nonexistent"),
                date: None,
                items: None,
                description: Some("Whatever".to_string()),
                tags: None,
                payee: None,
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
async fn test_update_duplicate_ids_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "50 USD"),
                    transaction_item(&expense_id, "50 USD"),
                ],
            )],
        )
        .await?;
    let id = created[0].id.clone();

    let err = service
        .update(
            &mut sess,
            [
                RecordCommandUpdate {
                    id: id.clone(),
                    date: None,
                    items: None,
                    description: Some("First".to_string()),
                    tags: None,
                    payee: None,
                },
                RecordCommandUpdate {
                    id,
                    date: None,
                    items: None,
                    description: Some("Second".to_string()),
                    tags: None,
                    payee: None,
                },
            ],
        )
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
async fn test_update_empty_batch_succeeds() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let updated = service
        .update(&mut sess, Vec::<RecordCommandUpdate>::new())
        .await?;
    assert!(updated.is_empty());
    Ok(())
}

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_existing_record() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "100 USD"),
                    transaction_item(&expense_id, "100 USD"),
                ],
            )],
        )
        .await?;
    let id = created[0].id.clone();

    service.delete(&mut sess, [id]).await?;

    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_is_silent() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    service
        .delete(&mut sess, [RecordId::from("nonexistent")])
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_empty_batch_succeeds() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    service.delete(&mut sess, Vec::<RecordId>::new()).await?;
    Ok(())
}

// ── Batch tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_empty_succeeds() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let result = service
        .batch(&mut sess, RecordCommandBatch::default())
        .await?;
    assert!(result.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_batch_delete_create_update_together() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, equity_id) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [
                create_cmd(
                    &jid,
                    vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                ),
                create_cmd(
                    &jid,
                    vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                ),
            ],
        )
        .await?;

    let to_delete_id = created[0].id.clone();
    let to_update_id = created[1].id.clone();

    let result = service
        .batch(
            &mut sess,
            RecordCommandBatch {
                delete: HashSet::from([to_delete_id]),
                create: vec![create_cmd(
                    &jid,
                    vec![
                        transaction_item(&asset_id, "300 USD"),
                        transaction_item(&equity_id, "300 USD"),
                    ],
                )],
                update: vec![RecordCommandUpdate {
                    id: to_update_id.clone(),
                    date: None,
                    items: None,
                    description: Some("Batch updated".to_string()),
                    tags: None,
                    payee: None,
                }],
            },
        )
        .await?;

    assert_eq!(result.len(), 2);
    let updated_record = result.iter().find(|r| r.id == to_update_id).unwrap();
    assert_eq!(updated_record.description, "Batch updated");

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_create_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "50 USD"),
                    transaction_item(&expense_id, "50 USD"),
                ],
            )],
        )
        .await?;
    let id = created[0].id.clone();

    // Batch: delete existing + create with empty items (fails)
    let result = service
        .batch(
            &mut sess,
            RecordCommandBatch {
                delete: HashSet::from([id.clone()]),
                create: vec![create_cmd(&jid, vec![])],
                update: vec![],
            },
        )
        .await;

    assert!(result.is_err());

    // The deleted record should still exist (rollback worked)
    let found = service
        .update(
            &mut sess,
            [RecordCommandUpdate {
                id,
                date: None,
                items: None,
                description: Some("Still here".to_string()),
                tags: None,
                payee: None,
            }],
        )
        .await;
    assert!(found.is_ok(), "Record should still exist after rollback");

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_update_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    // Create a record first
    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "50 USD"),
                    transaction_item(&expense_id, "50 USD"),
                ],
            )],
        )
        .await?;

    // Batch: create new + update nonexistent (fails)
    let result = service
        .batch(
            &mut sess,
            RecordCommandBatch {
                delete: HashSet::new(),
                create: vec![create_cmd(
                    &jid,
                    vec![
                        transaction_item(&asset_id, "100 USD"),
                        transaction_item(&expense_id, "100 USD"),
                    ],
                )],
                update: vec![RecordCommandUpdate {
                    id: RecordId::from("nonexistent"),
                    date: None,
                    items: None,
                    description: Some("whatever".to_string()),
                    tags: None,
                    payee: None,
                }],
            },
        )
        .await;

    assert!(result.is_err());

    Ok(())
}

// ── Specification tests ──────────────────────────────────────────

use domain::record::specification::RecordSpecification;
use shared::ReadRepository;

#[tokio::test]
async fn test_spec_find_by_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let created = service
        .create(
            &mut sess,
            [
                create_cmd(
                    &jid,
                    vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                ),
                create_cmd(
                    &jid,
                    vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                ),
            ],
        )
        .await?;
    let id0 = created[0].id.clone();

    let found = repo
        .find_all(&sess, &RecordSpecification::id(id0.clone()), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert!(found.contains_key(&id0));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_journal_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "10 USD"),
                    transaction_item(&expense_id, "10 USD"),
                ],
            )],
        )
        .await?;

    let found = repo
        .find_all(&sess, &RecordSpecification::journal_id(jid.clone()), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found_other = repo
        .find_all(&sess, &RecordSpecification::journal_id("other"), None)
        .await?;
    assert!(found_other.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_account_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, equity_id) = setup_accounts(&mut sess).await;

    // Record 1 uses asset + expense
    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "10 USD"),
                    transaction_item(&expense_id, "10 USD"),
                ],
            )],
        )
        .await?;
    // Record 2 uses asset + equity
    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "20 USD"),
                    transaction_item(&equity_id, "20 USD"),
                ],
            )],
        )
        .await?;

    // Both records reference asset
    let found = repo
        .find_all(
            &sess,
            &RecordSpecification::account_id(asset_id.clone()),
            None,
        )
        .await?;
    assert_eq!(found.len(), 2);

    // Only record 1 references expense
    let found = repo
        .find_all(
            &sess,
            &RecordSpecification::account_id(expense_id.clone()),
            None,
        )
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_date() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let jan15 = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let feb1 = chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();

    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "10 USD"),
                    transaction_item(&expense_id, "10 USD"),
                ],
            )],
        )
        .await?;
    service
        .create(
            &mut sess,
            [RecordCommandCreate {
                journal_id: jid.clone(),
                date: feb1,
                kind: RecordItemKind::Transaction,
                items: vec![
                    transaction_item(&asset_id, "20 USD"),
                    transaction_item(&expense_id, "20 USD"),
                ],
                description: String::new(),
                tags: HashSet::new(),
                payee: String::new(),
            }],
        )
        .await?;

    let found = repo
        .find_all(&sess, &RecordSpecification::date(jan15), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(&sess, &RecordSpecification::date(feb1), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_date_range() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let jan15 = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let feb1 = chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
    let mar1 = chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();

    for date in [jan15, feb1, mar1] {
        service
            .create(
                &mut sess,
                [RecordCommandCreate {
                    journal_id: jid.clone(),
                    date,
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: String::new(),
                }],
            )
            .await?;
    }

    // DateFrom feb1 -> feb1, mar1
    let found = repo
        .find_all(&sess, &RecordSpecification::date_from(feb1), None)
        .await?;
    assert_eq!(found.len(), 2);

    // DateTo feb1 -> jan15, feb1
    let found = repo
        .find_all(&sess, &RecordSpecification::date_to(feb1), None)
        .await?;
    assert_eq!(found.len(), 2);

    // DateFrom AND DateTo -> just feb1
    let spec = RecordSpecification::date_from(feb1) & RecordSpecification::date_to(feb1);
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_payee() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    service
        .create(
            &mut sess,
            [
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Cafe".to_string(),
                },
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Grocery".to_string(),
                },
            ],
        )
        .await?;

    let found = repo
        .find_all(&sess, &RecordSpecification::payee("Cafe"), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_tag() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    service
        .create(
            &mut sess,
            [RecordCommandCreate {
                journal_id: jid.clone(),
                date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                kind: RecordItemKind::Transaction,
                items: vec![
                    transaction_item(&asset_id, "10 USD"),
                    transaction_item(&expense_id, "10 USD"),
                ],
                description: String::new(),
                tags: HashSet::from(["food".to_string(), "lunch".to_string()]),
                payee: String::new(),
            }],
        )
        .await?;
    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "20 USD"),
                    transaction_item(&expense_id, "20 USD"),
                ],
            )],
        )
        .await?;

    let found = repo
        .find_all(&sess, &RecordSpecification::tag("food"), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(&sess, &RecordSpecification::tag("nonexistent"), None)
        .await?;
    assert!(found.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_item_kind() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    // Transaction record
    service
        .create(
            &mut sess,
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "10 USD"),
                    transaction_item(&expense_id, "10 USD"),
                ],
            )],
        )
        .await?;
    // Validation record
    service
        .create(
            &mut sess,
            [RecordCommandCreate {
                journal_id: jid.clone(),
                date: chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                kind: RecordItemKind::Validation,
                items: vec![RecordCommandItem {
                    account_id: asset_id.clone(),
                    amount: "500 USD".to_string(),
                    description: String::new(),
                    price: None,
                    cost: HashSet::new(),
                }],
                description: String::new(),
                tags: HashSet::new(),
                payee: String::new(),
            }],
        )
        .await?;

    let found = repo
        .find_all(
            &sess,
            &RecordSpecification::item_kind(RecordItemKind::Transaction),
            None,
        )
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(
            &sess,
            &RecordSpecification::item_kind(RecordItemKind::Validation),
            None,
        )
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_full_text() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    service
        .create(
            &mut sess,
            [
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: "Lunch at the cafe".to_string(),
                    tags: HashSet::new(),
                    payee: "SomeCafe".to_string(),
                },
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                    description: "Grocery shopping".to_string(),
                    tags: HashSet::from(["weekly".to_string()]),
                    payee: "Supermarket".to_string(),
                },
            ],
        )
        .await?;

    // Match description
    let found = repo
        .find_all(&sess, &RecordSpecification::full_text("lunch"), None)
        .await?;
    assert_eq!(found.len(), 1);

    // Match payee
    let found = repo
        .find_all(&sess, &RecordSpecification::full_text("supermarket"), None)
        .await?;
    assert_eq!(found.len(), 1);

    // Match tag
    let found = repo
        .find_all(&sess, &RecordSpecification::full_text("weekly"), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_all_and() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let jan15 = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

    service
        .create(
            &mut sess,
            [
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: jan15,
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Cafe".to_string(),
                },
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: jan15,
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Grocery".to_string(),
                },
            ],
        )
        .await?;

    // AND: date=jan15 AND payee=Cafe -> 1
    let spec = RecordSpecification::date(jan15) & RecordSpecification::payee("Cafe");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_any_or() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    service
        .create(
            &mut sess,
            [
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Cafe".to_string(),
                },
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Grocery".to_string(),
                },
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "30 USD"),
                        transaction_item(&expense_id, "30 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Restaurant".to_string(),
                },
            ],
        )
        .await?;

    let spec = RecordSpecification::payee("Cafe") | RecordSpecification::payee("Restaurant");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_spec_not() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    service
        .create(
            &mut sess,
            [
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Cafe".to_string(),
                },
                RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "20 USD"),
                        transaction_item(&expense_id, "20 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: "Grocery".to_string(),
                },
            ],
        )
        .await?;

    let spec = !RecordSpecification::payee("Cafe");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.values().next().unwrap().payee, "Grocery");

    Ok(())
}

#[tokio::test]
async fn test_spec_limit() -> anyhow::Result<()> {
    let (service, mut sess) = new_service();
    let repo = InMemoryRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    for i in 0..5 {
        service
            .create(
                &mut sess,
                [RecordCommandCreate {
                    journal_id: jid.clone(),
                    date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15 + i).unwrap(),
                    kind: RecordItemKind::Transaction,
                    items: vec![
                        transaction_item(&asset_id, "10 USD"),
                        transaction_item(&expense_id, "10 USD"),
                    ],
                    description: String::new(),
                    tags: HashSet::new(),
                    payee: String::new(),
                }],
            )
            .await?;
    }

    let spec = RecordSpecification::journal_id(jid);
    let found = repo.find_all(&sess, &spec, Some(3)).await?;
    assert_eq!(found.len(), 3);

    Ok(())
}
