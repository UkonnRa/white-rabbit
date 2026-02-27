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
    pub async fn create(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = RecordCommandCreate>,
    ) -> Result<Vec<Record>> {
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
    /// - If `items` is `Some`, re-validates like create (parse amounts, lookup accounts, fill types).
    /// - `journal_id` and `kind` are immutable after creation.
    pub async fn update(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = RecordCommandUpdate>,
    ) -> Result<Vec<Record>> {
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
    pub async fn delete(
        &self,
        sess: &mut R::Session,
        ids: impl IntoIterator<Item = impl Into<RecordId>>,
    ) -> Result<()> {
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
    pub async fn batch(
        &self,
        sess: &mut R::Session,
        command: RecordCommandBatch,
    ) -> Result<Vec<Record>> {
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
    ) -> Result<Vec<Record>> {
        Self::do_delete(repo, sess, command.delete).await?;
        let created = Self::do_create(repo, account_repo, sess, command.create).await?;
        let updated = Self::do_update(repo, account_repo, sess, command.update).await?;

        let mut result: HashMap<_, _> = created.into_iter().map(|r| (r.id.clone(), r)).collect();
        for r in updated {
            result.insert(r.id.clone(), r);
        }
        Ok(result.into_values().collect())
    }

    // ── Internal implementations ─────────────────────────────────

    async fn do_create(
        repo: &R,
        account_repo: &AR,
        sess: &mut R::Session,
        commands: Vec<RecordCommandCreate>,
    ) -> Result<Vec<Record>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

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
        Ok(saved.into_values().collect())
    }

    async fn do_update(
        repo: &R,
        account_repo: &AR,
        sess: &mut R::Session,
        commands: Vec<RecordCommandUpdate>,
    ) -> Result<Vec<Record>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        // 1. Reject duplicate IDs
        let batch_ids: HashSet<_> = commands.iter().map(|c| &c.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Record::ENTITY_TYPE)
                .with_field("id")
                .convert());
        }

        // 2. Fetch existing records
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

        // 3. Apply updates
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
        Ok(saved.into_values().collect())
    }

    async fn do_delete(repo: &R, sess: &mut R::Session, ids: HashSet<RecordId>) -> Result<()> {
        let ids: Vec<_> = ids.into_iter().collect();
        repo.delete_all_by_ids(sess, &ids)
            .await
            .map_err(|e| e.convert())?;
        Ok(())
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
        // Batch-fetch all referenced accounts
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

            // Account must belong to the record's journal
            if account.journal_id != *journal_id {
                return Err(
                    crate::error::ErrorKind::mismatch(journal_id, &account.journal_id)
                        .with_resource_type(Record::ENTITY_TYPE)
                        .with_field("journal_id"),
                );
            }

            // Account must not be archived
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
        let now = Utc::now();
        match command {
            RecordCommand::Create(cmd) => {
                let records = self.create(sess, [cmd]).await?;
                Ok(records
                    .into_iter()
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
            RecordCommand::Update(cmd) => {
                let id = cmd.id.clone();
                let date = cmd.date;
                let description = cmd.description.clone();
                let tags = cmd.tags.clone();
                let payee = cmd.payee.clone();
                let updated = self.update(sess, [cmd]).await?;
                let items = updated.first().map(|r| r.items.clone());
                Ok(vec![RecordEvent::Updated(RecordUpdated {
                    id,
                    date,
                    items,
                    description,
                    tags,
                    payee,
                    last_modified_at: now,
                })])
            }
            RecordCommand::Delete(ids) => {
                let events: Vec<_> = ids
                    .iter()
                    .map(|id| RecordEvent::Deleted(RecordDeleted { id: id.clone() }))
                    .collect();
                self.delete(sess, ids).await?;
                Ok(events)
            }
            RecordCommand::Batch(cmd) => {
                let delete_events: Vec<_> = cmd
                    .delete
                    .iter()
                    .map(|id| RecordEvent::Deleted(RecordDeleted { id: id.clone() }))
                    .collect();
                let create_count = cmd.create.len();
                let update_cmds: Vec<_> = cmd
                    .update
                    .iter()
                    .map(|u| {
                        (
                            u.id.clone(),
                            u.date,
                            u.description.clone(),
                            u.tags.clone(),
                            u.payee.clone(),
                        )
                    })
                    .collect();

                let results = self.batch(sess, cmd).await?;

                let mut events = delete_events;
                for r in results.iter().take(create_count) {
                    events.push(RecordEvent::Created(RecordCreated {
                        id: r.id.clone(),
                        journal_id: r.journal_id.clone(),
                        date: r.date,
                        items: r.items.clone(),
                        description: r.description.clone(),
                        tags: r.tags.iter().map(|t| t.to_string()).collect(),
                        payee: r.payee.clone(),
                        created_at: r.created_at.unwrap_or(now),
                    }));
                }
                for (id, date, description, tags, payee) in update_cmds {
                    let items = results.iter().find(|r| r.id == id).map(|r| r.items.clone());
                    events.push(RecordEvent::Updated(RecordUpdated {
                        id,
                        date,
                        items,
                        description,
                        tags,
                        payee,
                        last_modified_at: now,
                    }));
                }
                Ok(events)
            }
        }
    }
}
