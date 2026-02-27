use chrono::Utc;

use crate::account::command::{
    AccountCommand, AccountCommandArchive, AccountCommandBatch, AccountCommandCreate,
    AccountCommandUpdate,
};
use crate::account::event::{
    AccountArchived, AccountCreated, AccountDeleted, AccountEvent, AccountUpdated,
};
use crate::account::repository::AccountRepository;
use crate::account::specification::AccountSpecification;
use crate::account::{Account, AccountId, AccountInput};
use crate::error::Result;
use shared::{Entity, ErrorKind, RepositorySession, WriteService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct AccountService<R: AccountRepository> {
    pub repository: Arc<R>,
}

impl<R: AccountRepository> AccountService<R> {
    /// Create new [`Account`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `parent_id` must refer to an existing, non-archived account.
    /// - Each `name` must be unique among siblings (same parent).
    /// - Names must not be one of the 5 reserved root names (case-insensitive).
    /// - The account type is inherited from the parent.
    ///
    /// # Returns
    ///
    /// One [`AccountEvent::Created`] per successfully created account.
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(NotFound)`](shared::ErrorKind::NotFound) — parent not found.
    /// - [`ErrorKind::Shared(Conflict)`](shared::ErrorKind::Conflict) — parent is archived
    ///   or name is reserved.
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues) — duplicate
    ///   names among siblings.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty) — empty name.
    pub async fn create(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = AccountCommandCreate>,
    ) -> Result<Vec<AccountEvent>> {
        Self::do_create(&self.repository, sess, commands.into_iter().collect()).await
    }

    /// Update existing [`Account`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `id` must refer to an existing account.
    /// - Each `id` must appear at most once in the batch.
    /// - New names must be unique among siblings (same parent), with swap support.
    /// - New names must not be reserved root names.
    ///
    /// # Returns
    ///
    /// One [`AccountEvent::Updated`] per successfully updated account.
    pub async fn update(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = AccountCommandUpdate>,
    ) -> Result<Vec<AccountEvent>> {
        Self::do_update(&self.repository, sess, commands.into_iter().collect()).await
    }

    /// Delete [`Account`]s by their IDs.
    ///
    /// Cascades: all descendants are deleted too. Duplicate and nonexistent IDs
    /// are silently ignored.
    ///
    /// # Returns
    ///
    /// One [`AccountEvent::Deleted`] per actually-deleted account (including
    /// cascade-deleted descendants).
    pub async fn delete(
        &self,
        sess: &mut R::Session,
        ids: impl IntoIterator<Item = impl Into<AccountId>>,
    ) -> Result<Vec<AccountEvent>> {
        Self::do_delete(
            &self.repository,
            sess,
            ids.into_iter().map(Into::into).collect(),
        )
        .await
    }

    /// Archive [`Account`]s and all their descendants.
    ///
    /// Sets `archived_at` on the specified accounts and all descendants.
    /// Already-archived accounts are silently skipped.
    ///
    /// # Returns
    ///
    /// One [`AccountEvent::Archived`] per newly-archived account (excludes
    /// already-archived accounts).
    pub async fn archive(
        &self,
        sess: &mut R::Session,
        command: AccountCommandArchive,
    ) -> Result<Vec<AccountEvent>> {
        Self::do_archive(&self.repository, sess, command).await
    }

    /// Execute a batch of create, update, delete, and archive operations atomically.
    ///
    /// Order: **delete -> archive -> create -> update**.
    ///
    /// # Returns
    ///
    /// All events produced by the sub-operations, in execution order.
    pub async fn batch(
        &self,
        sess: &mut R::Session,
        command: AccountCommandBatch,
    ) -> Result<Vec<AccountEvent>> {
        sess.begin().await.map_err(|e| e.convert())?;

        let result = Self::do_batch(&self.repository, sess, command).await;

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
        sess: &mut R::Session,
        command: AccountCommandBatch,
    ) -> Result<Vec<AccountEvent>> {
        let mut events = Vec::new();
        events.extend(Self::do_delete(repo, sess, command.delete).await?);
        for archive_cmd in command.archive {
            events.extend(Self::do_archive(repo, sess, archive_cmd).await?);
        }
        events.extend(Self::do_create(repo, sess, command.create).await?);
        events.extend(Self::do_update(repo, sess, command.update).await?);
        Ok(events)
    }

    // ── Internal implementations ─────────────────────────────────

    async fn do_create(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<AccountCommandCreate>,
    ) -> Result<Vec<AccountEvent>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();

        for cmd in &commands {
            if Account::is_reserved_name(&cmd.name) {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("name")
                    .with_detail(format!("'{}' is a reserved root account name", cmd.name))
                    .convert());
            }
        }

        let parent_ids: Vec<_> = commands.iter().map(|c| c.parent_id.clone()).collect();
        let parents = repo
            .find_all_by_ids(sess, &parent_ids)
            .await
            .map_err(|e| e.convert())?;

        for cmd in &commands {
            let parent = parents.get(&cmd.parent_id).ok_or_else(|| {
                ErrorKind::not_found()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("parent_id")
                    .with_detail(format!("parent {} not found", cmd.parent_id))
                    .convert()
            })?;
            if parent.is_archived() {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("parent_id")
                    .with_detail(format!("parent {} is archived", cmd.parent_id))
                    .convert());
            }
        }

        let mut names_by_parent: HashMap<&AccountId, HashSet<&str>> = HashMap::new();
        for cmd in &commands {
            if !names_by_parent
                .entry(&cmd.parent_id)
                .or_default()
                .insert(&cmd.name)
            {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        for (parent_id, new_names) in &names_by_parent {
            let spec = AccountSpecification::parent_id((*parent_id).clone())
                & AccountSpecification::names(new_names.iter().copied());
            if let Some(existing) = repo.find_one(sess, &spec).await.map_err(|e| e.convert())? {
                return Err(ErrorKind::duplicate_values(&existing.name)
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let accounts = commands
            .into_iter()
            .map(|cmd| {
                let parent = &parents[&cmd.parent_id];
                let input = AccountInput {
                    journal_id: cmd.journal_id,
                    parent_id: Some(cmd.parent_id),
                    r#type: parent.r#type,
                    name: cmd.name,
                    description: cmd.description,
                    tags: cmd.tags.into_iter().collect(),
                    ..Default::default()
                };
                input.try_into()
            })
            .collect::<Result<Vec<Account>>>()?;

        let saved = repo
            .save_all(sess, &accounts)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|a| {
                AccountEvent::Created(AccountCreated {
                    id: a.id,
                    journal_id: a.journal_id,
                    parent_id: a.parent_id,
                    r#type: a.r#type,
                    name: a.name.to_string(),
                    description: a.description,
                    tags: a.tags.iter().map(|t| t.to_string()).collect(),
                    created_at: a.created_at.unwrap_or(now),
                })
            })
            .collect())
    }

    fn validate_update_ids(commands: &[AccountCommandUpdate]) -> Result<HashSet<&AccountId>> {
        let batch_ids: HashSet<_> = commands.iter().map(|c| &c.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Account::ENTITY_TYPE)
                .with_field("id")
                .convert());
        }
        Ok(batch_ids)
    }

    fn validate_update_names(commands: &[AccountCommandUpdate]) -> Result<()> {
        let mut new_names: HashSet<&str> = HashSet::new();
        for cmd in commands {
            if cmd.name.is_empty() {
                continue;
            }
            if Account::is_reserved_name(&cmd.name) {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("name")
                    .with_detail(format!("'{}' is a reserved root account name", cmd.name))
                    .convert());
            }
            if !new_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }
        Ok(())
    }

    async fn validate_update_name_conflicts(
        repo: &R,
        sess: &mut R::Session,
        commands: &[AccountCommandUpdate],
        existing: &HashMap<AccountId, Account>,
        batch_ids: &HashSet<&AccountId>,
    ) -> Result<()> {
        for cmd in commands {
            if cmd.name.is_empty() {
                continue;
            }
            let account = &existing[&cmd.id];
            let Some(parent_id) = &account.parent_id else {
                continue;
            };
            let spec = AccountSpecification::parent_id(parent_id.clone())
                & AccountSpecification::name(&cmd.name);
            let conflicts = repo
                .find_all(sess, &spec, None)
                .await
                .map_err(|e| e.convert())?;
            for (conflict_id, conflict) in &conflicts {
                if !batch_ids.contains(conflict_id) {
                    return Err(ErrorKind::duplicate_values(&conflict.name)
                        .with_resource_type(Account::ENTITY_TYPE)
                        .with_field("name")
                        .convert());
                }
            }
        }
        Ok(())
    }

    fn build_updated_account(account: &Account, cmd: &AccountCommandUpdate) -> Result<Account> {
        let input = AccountInput {
            id: account.id.clone(),
            version: account.version,
            created_at: account.created_at,
            last_modified_at: account.last_modified_at,
            archived_at: account.archived_at,
            journal_id: account.journal_id.clone(),
            parent_id: account.parent_id.clone(),
            r#type: account.r#type,
            name: if cmd.name.is_empty() {
                account.name.to_string()
            } else {
                cmd.name.clone()
            },
            description: cmd
                .description
                .clone()
                .unwrap_or_else(|| account.description.clone()),
            tags: cmd
                .tags
                .clone()
                .map(|t| t.into_iter().collect())
                .unwrap_or_else(|| account.tags.iter().map(|t| t.to_string()).collect()),
        };
        input.try_into()
    }

    async fn do_update(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<AccountCommandUpdate>,
    ) -> Result<Vec<AccountEvent>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();
        let batch_ids = Self::validate_update_ids(&commands)?;
        Self::validate_update_names(&commands)?;

        let id_vec: Vec<_> = commands.iter().map(|c| c.id.clone()).collect();
        let existing = repo
            .find_all_by_ids(sess, &id_vec)
            .await
            .map_err(|e| e.convert())?;

        for id in &id_vec {
            if !existing.contains_key(id) {
                return Err(ErrorKind::not_found()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("id")
                    .with_detail(format!("account {id} not found"))
                    .convert());
            }
        }

        Self::validate_update_name_conflicts(repo, sess, &commands, &existing, &batch_ids).await?;

        let commands_by_id: HashMap<_, _> =
            commands.into_iter().map(|c| (c.id.clone(), c)).collect();

        let accounts = id_vec
            .iter()
            .map(|id| Self::build_updated_account(&existing[id], &commands_by_id[id]))
            .collect::<Result<Vec<_>>>()?;

        let saved = repo
            .save_all(sess, &accounts)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|a| {
                AccountEvent::Updated(AccountUpdated {
                    id: a.id,
                    name: Some(a.name.to_string()),
                    description: Some(a.description),
                    tags: Some(a.tags.iter().map(|t| t.to_string()).collect()),
                    last_modified_at: a.last_modified_at.unwrap_or(now),
                })
            })
            .collect())
    }

    async fn do_delete(
        repo: &R,
        sess: &mut R::Session,
        ids: HashSet<AccountId>,
    ) -> Result<Vec<AccountEvent>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut all_ids: HashSet<AccountId> = ids;
        let mut frontier: Vec<AccountId> = all_ids.iter().cloned().collect();

        while !frontier.is_empty() {
            let spec = AccountSpecification::parent_ids(frontier.iter().cloned());
            let children = repo
                .find_all(sess, &spec, None)
                .await
                .map_err(|e| e.convert())?;
            frontier = children
                .keys()
                .filter(|id| all_ids.insert((*id).clone()))
                .cloned()
                .collect();
        }

        let ids_vec: Vec<_> = all_ids.into_iter().collect();
        repo.delete_all_by_ids(sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;

        Ok(ids_vec
            .into_iter()
            .map(|id| AccountEvent::Deleted(AccountDeleted { id }))
            .collect())
    }

    async fn do_archive(
        repo: &R,
        sess: &mut R::Session,
        command: AccountCommandArchive,
    ) -> Result<Vec<AccountEvent>> {
        if command.ids.is_empty() {
            return Ok(vec![]);
        }

        let mut all_ids: HashSet<AccountId> = command.ids;
        let mut frontier: Vec<AccountId> = all_ids.iter().cloned().collect();

        while !frontier.is_empty() {
            let spec = AccountSpecification::parent_ids(frontier.iter().cloned());
            let children = repo
                .find_all(sess, &spec, None)
                .await
                .map_err(|e| e.convert())?;
            frontier = children
                .keys()
                .filter(|id| all_ids.insert((*id).clone()))
                .cloned()
                .collect();
        }

        let ids_vec: Vec<_> = all_ids.into_iter().collect();
        let existing = repo
            .find_all_by_ids(sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;

        let to_update: Vec<Account> = existing
            .into_values()
            .filter(|a| !a.is_archived())
            .map(|mut a| {
                a.archived_at = Some(command.archived_at);
                a
            })
            .collect();

        if to_update.is_empty() {
            return Ok(vec![]);
        }

        let saved = repo
            .save_all(sess, &to_update)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|a| {
                AccountEvent::Archived(AccountArchived {
                    id: a.id,
                    archived_at: a.archived_at.unwrap_or(command.archived_at),
                })
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl<R: AccountRepository> WriteService<AccountCommand> for AccountService<R> {
    type Event = AccountEvent;
    type Session = R::Session;
    type Error = crate::error::Error;

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: AccountCommand,
    ) -> Result<Vec<AccountEvent>> {
        match command {
            AccountCommand::Create(cmd) => self.create(sess, [cmd]).await,
            AccountCommand::Update(cmd) => self.update(sess, [cmd]).await,
            AccountCommand::Delete(ids) => self.delete(sess, ids).await,
            AccountCommand::Archive(cmd) => self.archive(sess, cmd).await,
            AccountCommand::Batch(cmd) => self.batch(sess, cmd).await,
        }
    }
}
