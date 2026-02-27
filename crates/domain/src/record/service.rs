use chrono::Utc;

use crate::account::repository::AccountRepository;
use crate::account::{Account, AccountId};
use crate::error::Result;
use crate::record::command::{
    RecordCommand, RecordCommandBatch, RecordCommandCreate, RecordCommandItem, RecordCommandUpdate,
};
use crate::record::event::*;
use crate::record::repository::RecordRepository;
use crate::record::{
    AmountInput, CostInput, Record, RecordId, RecordInput, RecordItemInput, RecordItemKind,
};
use shared::{Entity, ErrorKind, RepositorySession, WriteService};
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
    ///
    /// # Returns
    ///
    /// One [`RecordEvent::Created`] per successfully created record, carrying
    /// the fully resolved items (with looked-up account types and parsed amounts).
    pub async fn create(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = RecordCommandCreate>,
    ) -> Result<Vec<RecordEvent>> {
        Self::do_create(
            &self.repository,
            &self.account_repository,
            sess,
            commands.into_iter().collect(),
        )
        .await
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
    ///
    /// # Returns
    ///
    /// One [`RecordEvent::Updated`] per successfully updated record.
    pub async fn update(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = RecordCommandUpdate>,
    ) -> Result<Vec<RecordEvent>> {
        Self::do_update(
            &self.repository,
            &self.account_repository,
            sess,
            commands.into_iter().collect(),
        )
        .await
    }

    /// Delete [`Record`]s by their IDs.
    ///
    /// Nonexistent IDs are silently ignored.
    ///
    /// # Returns
    ///
    /// One [`RecordEvent::Deleted`] per ID that was passed in.
    pub async fn delete(
        &self,
        sess: &mut R::Session,
        ids: impl IntoIterator<Item = impl Into<RecordId>>,
    ) -> Result<Vec<RecordEvent>> {
        Self::do_delete(
            &self.repository,
            sess,
            ids.into_iter().map(Into::into).collect(),
        )
        .await
    }

    /// Execute a batch of create, update, and delete operations atomically.
    ///
    /// Order: **delete -> create -> update**.
    ///
    /// # Returns
    ///
    /// All events produced by the sub-operations, in execution order.
    pub async fn batch(
        &self,
        sess: &mut R::Session,
        command: RecordCommandBatch,
    ) -> Result<Vec<RecordEvent>> {
        sess.begin().await.map_err(|e| e.convert())?;

        let result =
            Self::do_batch(&self.repository, &self.account_repository, sess, command).await;

        match result {
            Ok(v) => {
                sess.commit().await.map_err(|e| e.convert())?;
                Ok(v)
            }
            Err(e) => {
                let _ = sess.rollback().await;
                Err(e)
            }
        }
    }

    async fn do_batch(
        repo: &R,
        account_repo: &AR,
        sess: &mut R::Session,
        command: RecordCommandBatch,
    ) -> Result<Vec<RecordEvent>> {
        let mut events = Vec::new();
        events.extend(Self::do_delete(repo, sess, command.delete).await?);
        events.extend(Self::do_create(repo, account_repo, sess, command.create).await?);
        events.extend(Self::do_update(repo, account_repo, sess, command.update).await?);
        Ok(events)
    }

    // ── Internal implementations ─────────────────────────────────

    async fn do_create(
        repo: &R,
        account_repo: &AR,
        sess: &mut R::Session,
        commands: Vec<RecordCommandCreate>,
    ) -> Result<Vec<RecordEvent>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();

        let mut records = Vec::with_capacity(commands.len());
        for cmd in commands {
            let item_inputs =
                Self::resolve_items(account_repo, sess, &cmd.journal_id, cmd.kind, &cmd.items)
                    .await?;

            let input = RecordInput {
                journal_id: cmd.journal_id,
                date: cmd.date,
                items: item_inputs,
                description: cmd.description,
                tags: cmd.tags.into_iter().collect(),
                payee: cmd.payee,
                ..Default::default()
            };
            records.push(Record::try_from(input)?);
        }

        let saved = repo
            .save_all(sess, &records)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|r| {
                RecordEvent::Created(RecordCreated {
                    id: r.id,
                    journal_id: r.journal_id,
                    date: r.date,
                    items: r.items,
                    description: r.description,
                    tags: r.tags.iter().map(|t| t.to_string()).collect(),
                    payee: r.payee,
                    created_at: r.created_at.unwrap_or(now),
                })
            })
            .collect())
    }

    async fn do_update(
        repo: &R,
        account_repo: &AR,
        sess: &mut R::Session,
        commands: Vec<RecordCommandUpdate>,
    ) -> Result<Vec<RecordEvent>> {
        if commands.is_empty() {
            return Ok(vec![]);
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
        let existing = repo
            .find_all_by_ids(sess, &id_vec)
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

        let mut records = Vec::with_capacity(id_vec.len());
        for id in &id_vec {
            let record = &existing[id];
            let cmd = &commands_by_id[id];

            let kind = match &record.items {
                crate::record::RecordItems::Transactions(_) => RecordItemKind::Transaction,
                crate::record::RecordItems::Validations(_) => RecordItemKind::Validation,
            };

            let item_inputs = if let Some(new_items) = &cmd.items {
                Self::resolve_items(account_repo, sess, &record.journal_id, kind, new_items).await?
            } else {
                Self::items_to_inputs(record)
            };

            let input = RecordInput {
                id: record.id.clone(),
                version: record.version,
                created_at: record.created_at,
                last_modified_at: record.last_modified_at,
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
            records.push(Record::try_from(input)?);
        }

        let saved = repo
            .save_all(sess, &records)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|r| {
                RecordEvent::Updated(RecordUpdated {
                    id: r.id,
                    date: Some(r.date),
                    items: Some(r.items),
                    description: Some(r.description),
                    tags: Some(r.tags.iter().map(|t| t.to_string()).collect()),
                    payee: Some(r.payee),
                    last_modified_at: r.last_modified_at.unwrap_or(now),
                })
            })
            .collect())
    }

    async fn do_delete(
        repo: &R,
        sess: &mut R::Session,
        ids: HashSet<RecordId>,
    ) -> Result<Vec<RecordEvent>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let ids_vec: Vec<_> = ids.into_iter().collect();
        repo.delete_all_by_ids(sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;
        Ok(ids_vec
            .into_iter()
            .map(|id| RecordEvent::Deleted(RecordDeleted { id }))
            .collect())
    }

    // ── Helpers ──────────────────────────────────────────────────

    /// Parse command items, look up accounts, validate, and build [`RecordItemInput`]s.
    async fn resolve_items(
        account_repo: &AR,
        sess: &mut R::Session,
        journal_id: &crate::journal::JournalId,
        kind: RecordItemKind,
        cmd_items: &[RecordCommandItem],
    ) -> Result<Vec<RecordItemInput>> {
        let account_ids: Vec<AccountId> = cmd_items.iter().map(|i| i.account_id.clone()).collect();
        let accounts: HashMap<AccountId, Account> = account_repo
            .find_all_by_ids(sess, &account_ids)
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
    type Event = RecordEvent;
    type Session = R::Session;
    type Error = crate::error::Error;

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: RecordCommand,
    ) -> Result<Vec<RecordEvent>> {
        match command {
            RecordCommand::Create(cmd) => self.create(sess, [cmd]).await,
            RecordCommand::Update(cmd) => self.update(sess, [cmd]).await,
            RecordCommand::Delete(ids) => self.delete(sess, ids).await,
            RecordCommand::Batch(cmd) => self.batch(sess, cmd).await,
        }
    }
}
