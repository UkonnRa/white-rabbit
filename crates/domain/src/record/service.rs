use chrono::Utc;

use crate::account::repository::AccountRepository;
use crate::account::specification::AccountSpecification;
use crate::account::{Account, AccountId};
use crate::error::Result;
use crate::record::command::{
    RecordCommand, RecordCommandBatch, RecordCommandCreate, RecordCommandItem, RecordCommandUpdate,
};
use crate::record::event::{RecordCreated, RecordDeleted, RecordEvent, RecordUpdated};
use crate::record::repository::RecordRepository;
use crate::record::specification::RecordSpecification;
use crate::record::{
    AmountInput, CostInput, Record, RecordId, RecordInput, RecordItemInput, RecordItemKind,
};
use shared::{Entity, ErrorKind, HandleResult, RepositorySession, UnitOfWork, WriteService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct RecordService<R: RecordRepository, AR: AccountRepository<Session = R::Session>> {
    pub repository: Arc<R>,
    pub account_repository: Arc<AR>,
}

impl<R, AR> RecordService<R, AR>
where
    R: RecordRepository,
    AR: AccountRepository<Session = R::Session>,
{
    /// Create new [`Record`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each item must reference an existing, non-archived account.
    /// - All referenced accounts must belong to the command's `journal_id`.
    /// - Items must be non-empty and amount strings must be parseable.
    ///
    /// Balance is NOT enforced — [`Record::is_balanced`] remains advisory.
    async fn do_create(
        account_repo: &AR,
        uow: &mut UnitOfWork,
        sess: &R::Session,
        commands: Vec<RecordCommandCreate>,
    ) -> Result<()> {
        if commands.is_empty() {
            return Ok(());
        }

        let now = Utc::now();

        for cmd in commands {
            let item_inputs = Self::resolve_items(
                account_repo,
                uow,
                sess,
                &cmd.journal_id,
                cmd.kind,
                &cmd.items,
            )
            .await?;

            let input = RecordInput {
                journal_id: cmd.journal_id,
                date: cmd.date,
                items: item_inputs,
                description: cmd.description,
                tags: cmd.tags.into_iter().collect(),
                payee: cmd.payee,
                created_at: Some(now),
                ..Default::default()
            };
            let record = Record::try_from(input)?;

            uow.add_event(RecordEvent::Created(RecordCreated {
                id: record.id.clone(),
                journal_id: record.journal_id.clone(),
                date: record.date,
                items: record.items.clone(),
                description: record.description.clone(),
                tags: record.tags.iter().map(|t| t.to_string()).collect(),
                payee: record.payee.clone(),
                created_at: now,
            }));
            uow.register_new::<Record>(record);
        }

        Ok(())
    }

    /// Update existing [`Record`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `id` must refer to an existing record.
    /// - Each `id` must appear at most once in the batch.
    /// - If `items` is `Some`, re-validates like create (parse amounts,
    ///   lookup accounts, fill types).
    /// - `journal_id` and `kind` are immutable after creation.
    async fn do_update(
        repo: &R,
        account_repo: &AR,
        uow: &mut UnitOfWork,
        sess: &R::Session,
        commands: Vec<RecordCommandUpdate>,
    ) -> Result<()> {
        if commands.is_empty() {
            return Ok(());
        }

        let now = Utc::now();

        let batch_ids: HashSet<_> = commands.iter().map(|c| &c.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Record::ENTITY_TYPE)
                .with_field("id")
                .convert());
        }

        let id_vec: Vec<_> = commands.iter().map(|c| c.id.clone()).collect();
        let existing = uow
            .find_all_by_ids::<Record, RecordSpecification, R>(repo, sess, &id_vec)
            .await
            .map_err(|e| e.convert())?;

        for id in &id_vec {
            if !existing.contains_key(id) {
                return Err(ErrorKind::not_found()
                    .with_resource_type(Record::ENTITY_TYPE)
                    .with_field("id")
                    .with_detail(format!("record {id} not found"))
                    .convert());
            }
        }

        let commands_by_id: HashMap<_, _> =
            commands.into_iter().map(|c| (c.id.clone(), c)).collect();

        for id in &id_vec {
            let record = &existing[id];
            let cmd = &commands_by_id[id];

            let kind = match &record.items {
                crate::record::RecordItems::Transactions(_) => RecordItemKind::Transaction,
                crate::record::RecordItems::Validations(_) => RecordItemKind::Validation,
            };

            let item_inputs = if let Some(new_items) = &cmd.items {
                Self::resolve_items(account_repo, uow, sess, &record.journal_id, kind, new_items)
                    .await?
            } else {
                Self::items_to_inputs(record)
            };

            let input = RecordInput {
                id: record.id.clone(),
                version: record.version,
                created_at: record.created_at,
                last_modified_at: Some(now),
                journal_id: record.journal_id.clone(),
                date: cmd.date.unwrap_or(record.date),
                items: item_inputs,
                description: cmd
                    .description
                    .clone()
                    .unwrap_or_else(|| record.description.clone()),
                tags: cmd
                    .tags
                    .clone()
                    .map(|t| t.into_iter().collect())
                    .unwrap_or_else(|| record.tags.iter().map(|t| t.to_string()).collect()),
                payee: cmd.payee.clone().unwrap_or_else(|| record.payee.clone()),
            };
            let record = Record::try_from(input)?;

            uow.add_event(RecordEvent::Updated(RecordUpdated {
                id: record.id.clone(),
                date: Some(record.date),
                items: Some(record.items.clone()),
                description: Some(record.description.clone()),
                tags: Some(record.tags.iter().map(|t| t.to_string()).collect()),
                payee: Some(record.payee.clone()),
                last_modified_at: now,
            }));
            uow.register_dirty::<Record>(record);
        }

        Ok(())
    }

    /// Delete [`Record`]s by their IDs.
    ///
    /// Nonexistent IDs are silently ignored.
    async fn do_delete(uow: &mut UnitOfWork, ids: HashSet<RecordId>) -> Result<()> {
        for id in ids {
            uow.add_event(RecordEvent::Deleted(RecordDeleted { id: id.clone() }));
            uow.register_deleted::<Record>(id);
        }
        Ok(())
    }

    /// Execute a batch of create, update, and delete operations atomically.
    ///
    /// Order: **delete → create → update**.
    async fn do_batch(
        repo: &R,
        account_repo: &AR,
        uow: &mut UnitOfWork,
        sess: &R::Session,
        command: RecordCommandBatch,
    ) -> Result<()> {
        Self::do_delete(uow, command.delete).await?;
        Self::do_create(account_repo, uow, sess, command.create).await?;
        Self::do_update(repo, account_repo, uow, sess, command.update).await?;
        Ok(())
    }

    // ── Helpers ──────────────────────────────────────────────────

    /// Parse command items, look up accounts, validate, and build [`RecordItemInput`]s.
    async fn resolve_items(
        account_repo: &AR,
        uow: &UnitOfWork,
        sess: &R::Session,
        journal_id: &crate::journal::JournalId,
        kind: RecordItemKind,
        cmd_items: &[RecordCommandItem],
    ) -> Result<Vec<RecordItemInput>> {
        let account_ids: Vec<AccountId> = cmd_items.iter().map(|i| i.account_id.clone()).collect();
        let accounts: HashMap<AccountId, Account> = uow
            .find_all_by_ids::<Account, AccountSpecification, AR>(account_repo, sess, &account_ids)
            .await
            .map_err(|e| e.convert())?;

        let mut inputs = Vec::with_capacity(cmd_items.len());
        for item in cmd_items {
            let account = accounts.get(&item.account_id).ok_or_else(|| {
                ErrorKind::not_found()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("account_id")
                    .with_detail(format!("account {} not found", item.account_id))
                    .convert()
            })?;

            if account.journal_id != *journal_id {
                return Err(
                    crate::error::ErrorKind::mismatch(journal_id, &account.journal_id)
                        .with_resource_type(Record::ENTITY_TYPE)
                        .with_field("journal_id"),
                );
            }

            if account.is_archived() {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("account_id")
                    .with_detail(format!("account {} is archived", item.account_id))
                    .convert());
            }

            inputs.push(RecordItemInput {
                account_id: item.account_id.clone(),
                account_type: account.r#type,
                kind,
                amount: AmountInput::from(item.amount.clone()),
                description: item.description.clone(),
                price: item.price.as_ref().map(|p| AmountInput::from(p.clone())),
                cost: item
                    .cost
                    .iter()
                    .map(|c| CostInput::from(c.clone()))
                    .collect(),
            });
        }

        Ok(inputs)
    }

    /// Convert an existing record's items back into [`RecordItemInput`]s
    /// (used when update keeps existing items unchanged).
    fn items_to_inputs(record: &Record) -> Vec<RecordItemInput> {
        match &record.items {
            crate::record::RecordItems::Transactions(txns) => txns
                .iter()
                .map(|t| RecordItemInput {
                    account_id: t.account_id.clone(),
                    account_type: t.account_type,
                    kind: RecordItemKind::Transaction,
                    amount: AmountInput::from(format!("{} {}", *t.amount.amount, t.amount.unit)),
                    description: t.description.clone(),
                    price: t
                        .price
                        .as_ref()
                        .map(|p| AmountInput::from(format!("{} {}", *p.amount, p.unit))),
                    cost: t
                        .cost
                        .iter()
                        .map(|c| match c {
                            crate::record::Cost::Price(a) => {
                                CostInput::from(format!("{} {}", *a.amount, a.unit))
                            }
                            crate::record::Cost::Date(d) => CostInput::from(d.to_string()),
                            crate::record::Cost::Reference(r) => CostInput::from(r.to_string()),
                        })
                        .collect(),
                })
                .collect(),
            crate::record::RecordItems::Validations(vals) => vals
                .iter()
                .map(|v| RecordItemInput {
                    account_id: v.account_id.clone(),
                    account_type: crate::account::AccountType::Asset,
                    kind: RecordItemKind::Validation,
                    amount: AmountInput::from(format!("{} {}", *v.amount.amount, v.amount.unit)),
                    description: v.description.clone(),
                    price: None,
                    cost: HashSet::new(),
                })
                .collect(),
        }
    }
}

#[async_trait::async_trait]
impl<R, AR> WriteService<RecordCommand> for RecordService<R, AR>
where
    R: RecordRepository,
    AR: AccountRepository<Session = R::Session>,
{
    type Entity = Record;
    type Event = RecordEvent;
    type Session = R::Session;
    type Error = crate::error::Error;

    async fn do_handle(
        &self,
        sess: &Self::Session,
        uow: &mut UnitOfWork,
        command: RecordCommand,
    ) -> Result<()> {
        match command {
            RecordCommand::Create(cmd) => {
                Self::do_create(&self.account_repository, uow, sess, vec![cmd]).await
            }
            RecordCommand::Update(cmd) => {
                Self::do_update(
                    &self.repository,
                    &self.account_repository,
                    uow,
                    sess,
                    vec![cmd],
                )
                .await
            }
            RecordCommand::Delete(ids) => Self::do_delete(uow, ids).await,
            RecordCommand::Batch(cmd) => {
                Self::do_batch(&self.repository, &self.account_repository, uow, sess, cmd).await
            }
        }
    }

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: RecordCommand,
    ) -> Result<HandleResult<Record, RecordEvent>> {
        let mut uow = UnitOfWork::new();
        sess.begin().await.map_err(|e| e.convert())?;

        if let Err(e) = self.do_handle(sess, &mut uow, command).await {
            let _ = sess.rollback().await;
            return Err(e);
        }

        let events = uow.events::<RecordEvent>();
        let mut saved_entities = Vec::new();

        if let Some(cs) = uow.take_change_set::<Record>() {
            if !cs.deleted.is_empty() {
                let ids: Vec<_> = cs.deleted.into_iter().collect();
                if let Err(e) = self.repository.delete_all_by_ids(sess, &ids).await {
                    let _ = sess.rollback().await;
                    return Err(e.convert());
                }
            }

            let to_save: Vec<_> = cs.new.into_values().chain(cs.dirty.into_values()).collect();
            if !to_save.is_empty() {
                match self.repository.save_all(sess, &to_save).await {
                    Ok(saved) => saved_entities = saved.into_values().collect(),
                    Err(e) => {
                        let _ = sess.rollback().await;
                        return Err(e.convert());
                    }
                }
            }
        }

        sess.commit().await.map_err(|e| e.convert())?;
        Ok(HandleResult {
            entities: saved_entities,
            events,
        })
    }
}
