use std::collections::HashSet;
use std::sync::Arc;

use database_seaorm::repository::SeaOrmSession;
use sea_orm::Database;

use database_seaorm_migration::{Migrator, MigratorTrait};
use domain::account::command::AccountCommandCreate;
use domain::account::event::AccountEvent;
use domain::account::service::AccountService;
use domain::account::{AccountId, AccountType};
use domain::journal::JournalId;
use domain::record::command::{
    RecordCommandBatch, RecordCommandCreate, RecordCommandItem, RecordCommandUpdate,
};
use domain::record::event::RecordEvent;
use domain::record::service::RecordService;
use domain::record::{RecordId, RecordItemKind, RecordItems};

use crate::account::SeaOrmAccountRepository;
use crate::record::SeaOrmRecordRepository;

// ── Helpers ──────────────────────────────────────────────────────

type TestRecordService = RecordService<SeaOrmRecordRepository, SeaOrmAccountRepository>;

async fn new_service() -> (TestRecordService, SeaOrmSession) {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    let service = RecordService {
        repository: Arc::new(SeaOrmRecordRepository),
        account_repository: Arc::new(SeaOrmAccountRepository),
    };
    let sess = SeaOrmSession::new(db);
    (service, sess)
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

/// Insert a root account directly via the repo.
async fn insert_root(
    sess: &mut SeaOrmSession,
    journal_id: &JournalId,
    account_type: AccountType,
) -> AccountId {
    use domain::account::{Account, AccountInput};
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

/// Set up a journal with root accounts and child accounts for testing.
async fn setup_accounts(sess: &mut SeaOrmSession) -> (JournalId, AccountId, AccountId, AccountId) {
    let jid = JournalId::from("j1");
    insert_journal(sess, "j1").await;
    let asset_root = insert_root(sess, &jid, AccountType::Asset).await;
    let expense_root = insert_root(sess, &jid, AccountType::Expense).await;
    let equity_root = insert_root(sess, &jid, AccountType::Equity).await;

    let account_service = AccountService {
        repository: Arc::new(SeaOrmAccountRepository),
    };

    let events = account_service
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

    fn find_account_created_id(events: &[AccountEvent], name: &str) -> AccountId {
        events
            .iter()
            .find_map(|e| match e {
                AccountEvent::Created(c) if c.name == name => Some(c.id.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no Created event with name '{name}'"))
    }

    let asset_id = find_account_created_id(&events, "Cash");
    let expense_id = find_account_created_id(&events, "Food");
    let equity_id = find_account_created_id(&events, "Opening");

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

fn created_id(events: &[RecordEvent]) -> RecordId {
    events
        .iter()
        .find_map(|e| match e {
            RecordEvent::Created(c) => Some(c.id.clone()),
            _ => None,
        })
        .expect("no Created event found")
}

// ── Create tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_single_transaction_record() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let events = service
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

    assert_eq!(events.len(), 1);
    let c = match &events[0] {
        RecordEvent::Created(c) => c,
        other => panic!("expected Created event, got {other:?}"),
    };
    assert_eq!(c.description, "Lunch");
    assert_eq!(c.payee, "Restaurant");
    assert_eq!(c.journal_id, jid);

    Ok(())
}

#[tokio::test]
async fn test_create_validation_record() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, _, _) = setup_accounts(&mut sess).await;

    let events = service
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

    assert_eq!(events.len(), 1);
    let c = match &events[0] {
        RecordEvent::Created(c) => c,
        other => panic!("expected Created event, got {other:?}"),
    };
    assert!(matches!(c.items, RecordItems::Validations(_)));

    Ok(())
}

#[tokio::test]
async fn test_create_nonexistent_account_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
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
async fn test_create_empty_items_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
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
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let events = service
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
    let id = created_id(&events);

    let new_date = chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
    let events = service
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

    assert_eq!(events.len(), 1);
    let u = match &events[0] {
        RecordEvent::Updated(u) => u,
        other => panic!("expected Updated event, got {other:?}"),
    };
    assert_eq!(u.date, Some(new_date));
    assert_eq!(u.description.as_deref(), Some("Updated description"));
    assert_eq!(u.payee.as_deref(), Some("New Payee"));

    Ok(())
}

#[tokio::test]
async fn test_update_replace_items() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, expense_id, equity_id) = setup_accounts(&mut sess).await;

    let events = service
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
    let id = created_id(&events);

    let events = service
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

    assert_eq!(events.len(), 1);
    let u = match &events[0] {
        RecordEvent::Updated(u) => u,
        other => panic!("expected Updated event, got {other:?}"),
    };
    assert!(matches!(u.items, Some(RecordItems::Transactions(_))));

    Ok(())
}

#[tokio::test]
async fn test_update_nonexistent_record_fails() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;

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

// ── Delete tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_existing_record() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let events = service
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
    let id = created_id(&events);

    service.delete(&mut sess, [id]).await?;

    Ok(())
}

#[tokio::test]
async fn test_delete_nonexistent_is_silent() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    service
        .delete(&mut sess, [RecordId::from("nonexistent")])
        .await?;
    Ok(())
}

// ── Batch tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_delete_create_update_together() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, expense_id, equity_id) = setup_accounts(&mut sess).await;

    let events = service
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

    let ids: Vec<RecordId> = events
        .iter()
        .filter_map(|e| match e {
            RecordEvent::Created(c) => Some(c.id.clone()),
            _ => None,
        })
        .collect();
    let to_delete_id = ids[0].clone();
    let to_update_id = ids[1].clone();

    let events = service
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

    let updated_event = events
        .iter()
        .find_map(|e| match e {
            RecordEvent::Updated(u) if u.id == to_update_id => Some(u),
            _ => None,
        })
        .expect("no Updated event for to_update_id");
    assert_eq!(updated_event.description.as_deref(), Some("Batch updated"));

    Ok(())
}

#[tokio::test]
async fn test_batch_rollback_on_create_failure() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let events = service
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
    let id = created_id(&events);

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

// ── Specification tests ──────────────────────────────────────────

use domain::record::specification::RecordSpecification;
use shared::ReadRepository;

#[tokio::test]
async fn test_spec_find_by_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
    let (jid, asset_id, expense_id, _) = setup_accounts(&mut sess).await;

    let events = service
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
    let id0 = created_id(&events);

    let found = repo
        .find_all(&sess, &RecordSpecification::id(id0.clone()), None)
        .await?;
    assert_eq!(found.len(), 1);
    assert!(found.contains_key(&id0));

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_journal_id() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
    let (jid, asset_id, expense_id, equity_id) = setup_accounts(&mut sess).await;

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
            [create_cmd(
                &jid,
                vec![
                    transaction_item(&asset_id, "20 USD"),
                    transaction_item(&equity_id, "20 USD"),
                ],
            )],
        )
        .await?;

    let found = repo
        .find_all(
            &sess,
            &RecordSpecification::account_id(asset_id.clone()),
            None,
        )
        .await?;
    assert_eq!(found.len(), 2);

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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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

    let found = repo
        .find_all(&sess, &RecordSpecification::date_from(feb1), None)
        .await?;
    assert_eq!(found.len(), 2);

    let found = repo
        .find_all(&sess, &RecordSpecification::date_to(feb1), None)
        .await?;
    assert_eq!(found.len(), 2);

    let spec = RecordSpecification::date_from(feb1) & RecordSpecification::date_to(feb1);
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_find_by_payee() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
                    tags: HashSet::new(),
                    payee: "Supermarket".to_string(),
                },
            ],
        )
        .await?;

    let found = repo
        .find_all(&sess, &RecordSpecification::full_text("Lunch"), None)
        .await?;
    assert_eq!(found.len(), 1);

    let found = repo
        .find_all(&sess, &RecordSpecification::full_text("Supermarket"), None)
        .await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_all_and() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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

    let spec = RecordSpecification::date(jan15) & RecordSpecification::payee("Cafe");
    let found = repo.find_all(&sess, &spec, None).await?;
    assert_eq!(found.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_spec_any_or() -> anyhow::Result<()> {
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
    let (service, mut sess) = new_service().await;
    let repo = SeaOrmRecordRepository;
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
