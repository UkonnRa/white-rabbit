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
use shared::{Entity, ErrorKind, HandleResult, RepositorySession, UnitOfWork, WriteService};
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
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(NotFound)`](shared::ErrorKind::NotFound) — parent not found.
    /// - [`ErrorKind::Shared(Conflict)`](shared::ErrorKind::Conflict) — parent is archived
    ///   or name is reserved.
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues) — duplicate
    ///   names among siblings.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty) — empty name.
    async fn do_create(
        repo: &R,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        commands: Vec<AccountCommandCreate>,
    ) -> Result<()> {
        if commands.is_empty() {
            return Ok(());
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
        let parents = uow
            .find_all_by_ids::<Account, AccountSpecification, R>(repo, sess, &parent_ids)
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
            if let Some(existing) = uow
                .find_one::<Account, AccountSpecification, R>(repo, sess, &spec)
                .await
                .map_err(|e| e.convert())?
            {
                return Err(ErrorKind::duplicate_values(&existing.name)
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        for cmd in commands {
            let parent = &parents[&cmd.parent_id];
            let input = AccountInput {
                journal_id: cmd.journal_id,
                parent_id: Some(cmd.parent_id),
                r#type: parent.r#type,
                name: cmd.name,
                description: cmd.description,
                tags: cmd.tags.into_iter().collect(),
                created_at: Some(now),
                ..Default::default()
            };
            let account: Account = input.try_into()?;
            uow.add_event(AccountEvent::Created(AccountCreated {
                id: account.id.clone(),
                journal_id: account.journal_id.clone(),
                parent_id: account.parent_id.clone(),
                r#type: account.r#type,
                name: account.name.to_string(),
                description: account.description.clone(),
                tags: account.tags.iter().map(|t| t.to_string()).collect(),
                created_at: now,
            }));
            uow.register_new::<Account>(account);
        }

        Ok(())
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
        uow: &UnitOfWork,
        repo: &R,
        sess: &R::Session,
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
            let conflicts = uow
                .find_all::<Account, AccountSpecification, R>(repo, sess, &spec, None)
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

    fn build_updated_account(
        account: &Account,
        cmd: &AccountCommandUpdate,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Account> {
        let input = AccountInput {
            id: account.id.clone(),
            version: account.version,
            created_at: account.created_at,
            last_modified_at: Some(now),
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

    /// Update existing [`Account`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `id` must refer to an existing account.
    /// - Each `id` must appear at most once in the batch.
    /// - New names must be unique among siblings (same parent), with swap support.
    /// - New names must not be reserved root names.
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(NotFound)`](shared::ErrorKind::NotFound)
    ///   — an account ID does not exist.
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — duplicate names among siblings.
    /// - [`ErrorKind::Shared(Conflict)`](shared::ErrorKind::Conflict)
    ///   — name is reserved.
    async fn do_update(
        repo: &R,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        commands: Vec<AccountCommandUpdate>,
    ) -> Result<()> {
        if commands.is_empty() {
            return Ok(());
        }

        let now = Utc::now();
        let batch_ids = Self::validate_update_ids(&commands)?;
        Self::validate_update_names(&commands)?;

        let id_vec: Vec<_> = commands.iter().map(|c| c.id.clone()).collect();
        let existing = uow
            .find_all_by_ids::<Account, AccountSpecification, R>(repo, sess, &id_vec)
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

        Self::validate_update_name_conflicts(uow, repo, sess, &commands, &existing, &batch_ids)
            .await?;

        let commands_by_id: HashMap<_, _> =
            commands.into_iter().map(|c| (c.id.clone(), c)).collect();

        for id in &id_vec {
            let account = Self::build_updated_account(&existing[id], &commands_by_id[id], now)?;
            uow.add_event(AccountEvent::Updated(AccountUpdated {
                id: account.id.clone(),
                name: Some(account.name.to_string()),
                description: Some(account.description.clone()),
                tags: Some(account.tags.iter().map(|t| t.to_string()).collect()),
                last_modified_at: now,
            }));
            uow.register_dirty::<Account>(account);
        }

        Ok(())
    }

    /// Delete [`Account`]s by their IDs.
    ///
    /// Cascades: all descendants are deleted too. Duplicate and nonexistent
    /// IDs are silently ignored.
    async fn do_delete(
        repo: &R,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        ids: HashSet<AccountId>,
    ) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut all_ids: HashSet<AccountId> = ids;
        let mut frontier: Vec<AccountId> = all_ids.iter().cloned().collect();

        while !frontier.is_empty() {
            let spec = AccountSpecification::parent_ids(frontier.iter().cloned());
            let children = uow
                .find_all::<Account, AccountSpecification, R>(repo, sess, &spec, None)
                .await
                .map_err(|e| e.convert())?;
            frontier = children
                .keys()
                .filter(|id| all_ids.insert((*id).clone()))
                .cloned()
                .collect();
        }

        for id in all_ids {
            uow.add_event(AccountEvent::Deleted(AccountDeleted { id: id.clone() }));
            uow.register_deleted::<Account>(id);
        }

        Ok(())
    }

    /// Archive [`Account`]s and all their descendants.
    ///
    /// Sets `archived_at` on the specified accounts and all descendants.
    /// Already-archived accounts are silently skipped.
    async fn do_archive(
        repo: &R,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        command: AccountCommandArchive,
    ) -> Result<()> {
        if command.ids.is_empty() {
            return Ok(());
        }

        let mut all_ids: HashSet<AccountId> = command.ids;
        let mut frontier: Vec<AccountId> = all_ids.iter().cloned().collect();

        while !frontier.is_empty() {
            let spec = AccountSpecification::parent_ids(frontier.iter().cloned());
            let children = uow
                .find_all::<Account, AccountSpecification, R>(repo, sess, &spec, None)
                .await
                .map_err(|e| e.convert())?;
            frontier = children
                .keys()
                .filter(|id| all_ids.insert((*id).clone()))
                .cloned()
                .collect();
        }

        let ids_vec: Vec<_> = all_ids.into_iter().collect();
        let existing = uow
            .find_all_by_ids::<Account, AccountSpecification, R>(repo, sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;

        for account in existing.into_values() {
            if account.is_archived() {
                continue;
            }
            let mut archived = account;
            archived.archived_at = Some(command.archived_at);
            uow.add_event(AccountEvent::Archived(AccountArchived {
                id: archived.id.clone(),
                archived_at: command.archived_at,
            }));
            uow.register_dirty::<Account>(archived);
        }

        Ok(())
    }

    /// Execute a batch of create, update, delete, and archive operations
    /// atomically.
    ///
    /// Order: **delete → archive → create → update**.
    async fn do_batch(
        repo: &R,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        command: AccountCommandBatch,
    ) -> Result<()> {
        Self::do_delete(repo, sess, uow, command.delete).await?;
        for archive_cmd in command.archive {
            Self::do_archive(repo, sess, uow, archive_cmd).await?;
        }
        Self::do_create(repo, sess, uow, command.create).await?;
        Self::do_update(repo, sess, uow, command.update).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<R: AccountRepository> WriteService<AccountCommand> for AccountService<R> {
    type Entity = Account;
    type Event = AccountEvent;
    type Session = R::Session;
    type Error = crate::error::Error;

    async fn do_handle(
        &self,
        sess: &Self::Session,
        uow: &mut UnitOfWork,
        command: AccountCommand,
    ) -> Result<()> {
        match command {
            AccountCommand::Create(cmd) => {
                Self::do_create(&self.repository, sess, uow, vec![cmd]).await
            }
            AccountCommand::Update(cmd) => {
                Self::do_update(&self.repository, sess, uow, vec![cmd]).await
            }
            AccountCommand::Delete(ids) => Self::do_delete(&self.repository, sess, uow, ids).await,
            AccountCommand::Archive(cmd) => {
                Self::do_archive(&self.repository, sess, uow, cmd).await
            }
            AccountCommand::Batch(cmd) => Self::do_batch(&self.repository, sess, uow, cmd).await,
        }
    }

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: AccountCommand,
    ) -> Result<HandleResult<Account, AccountEvent>> {
        let mut uow = UnitOfWork::new();

        sess.begin().await.map_err(|e| e.convert())?;

        let result = self.do_handle(sess, &mut uow, command).await;

        match result {
            Ok(()) => {
                let events = uow.events::<AccountEvent>();
                let mut saved_entities = Vec::new();

                if let Some(cs) = uow.take_change_set::<Account>() {
                    if !cs.deleted.is_empty() {
                        let ids: Vec<_> = cs.deleted.into_iter().collect();
                        if let Err(e) = self.repository.delete_all_by_ids(sess, &ids).await {
                            let _ = sess.rollback().await;
                            return Err(e.convert());
                        }
                    }

                    let to_save: Vec<_> =
                        cs.new.into_values().chain(cs.dirty.into_values()).collect();
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
            Err(e) => {
                let _ = sess.rollback().await;
                Err(e)
            }
        }
    }
}
