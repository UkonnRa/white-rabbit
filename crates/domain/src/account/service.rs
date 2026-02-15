use crate::account::command::{
    AccountCommandArchive, AccountCommandBatch, AccountCommandCreate, AccountCommandUpdate,
};
use crate::account::repository::AccountRepository;
use crate::account::specification::AccountSpecification;
use crate::account::{Account, AccountId, AccountInput};
use crate::error::{ErrorKind, Result};
use shared::RepositorySession;
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
    pub async fn create(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = AccountCommandCreate>,
    ) -> Result<Vec<Account>> {
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
    pub async fn update(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = AccountCommandUpdate>,
    ) -> Result<Vec<Account>> {
        Self::do_update(&self.repository, sess, commands.into_iter().collect()).await
    }

    /// Delete [`Account`]s by their IDs.
    ///
    /// Cascades: all descendants are deleted too. Duplicate and nonexistent IDs
    /// are silently ignored.
    pub async fn delete(
        &self,
        sess: &mut R::Session,
        ids: impl IntoIterator<Item = impl Into<AccountId>>,
    ) -> Result<Vec<Account>> {
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
    pub async fn archive(
        &self,
        sess: &mut R::Session,
        command: AccountCommandArchive,
    ) -> Result<Vec<Account>> {
        Self::do_archive(&self.repository, sess, command).await
    }

    /// Execute a batch of create, update, delete, and archive operations atomically.
    ///
    /// Order: **delete -> archive -> create -> update**.
    pub async fn batch(
        &self,
        sess: &mut R::Session,
        command: AccountCommandBatch,
    ) -> Result<Vec<Account>> {
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
    ) -> Result<Vec<Account>> {
        Self::do_delete(repo, sess, command.delete).await?;
        for archive_cmd in command.archive {
            Self::do_archive(repo, sess, archive_cmd).await?;
        }
        let created = Self::do_create(repo, sess, command.create).await?;
        let updated = Self::do_update(repo, sess, command.update).await?;

        let mut result: HashMap<_, _> = created.into_iter().map(|a| (a.id.clone(), a)).collect();
        for a in updated {
            result.insert(a.id.clone(), a);
        }
        Ok(result.into_values().collect())
    }

    // ── Internal implementations ─────────────────────────────────

    async fn do_create(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<AccountCommandCreate>,
    ) -> Result<Vec<Account>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        // 1. Validate reserved names
        for cmd in &commands {
            if Account::is_reserved_name(&cmd.name) {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::TYPE)
                    .with_field("name")
                    .with_detail(format!("'{}' is a reserved root account name", cmd.name)));
            }
        }

        // 2. Fetch parents and validate they exist and are not archived
        let parent_ids: Vec<_> = commands.iter().map(|c| c.parent_id.clone()).collect();
        let parents = repo
            .find_all_by_ids(sess, &parent_ids)
            .await
            .map_err(|e| e.convert())?;

        for cmd in &commands {
            let parent = parents.get(&cmd.parent_id).ok_or_else(|| {
                ErrorKind::not_found()
                    .with_resource_type(Account::TYPE)
                    .with_field("parent_id")
                    .with_detail(format!("parent {} not found", cmd.parent_id))
            })?;
            if parent.is_archived() {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::TYPE)
                    .with_field("parent_id")
                    .with_detail(format!("parent {} is archived", cmd.parent_id)));
            }
        }

        // 3. Check name uniqueness among siblings (per parent)
        // Group commands by parent_id, check for dups within each group
        let mut names_by_parent: HashMap<&AccountId, HashSet<&str>> = HashMap::new();
        for cmd in &commands {
            if !names_by_parent
                .entry(&cmd.parent_id)
                .or_default()
                .insert(&cmd.name)
            {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Account::TYPE)
                    .with_field("name"));
            }
        }

        // Check against existing siblings in the DB
        for (parent_id, new_names) in &names_by_parent {
            let spec = AccountSpecification::parent_id((*parent_id).clone())
                & AccountSpecification::names(new_names.iter().copied());
            if let Some(existing) = repo.find_one(sess, &spec).await.map_err(|e| e.convert())? {
                return Err(ErrorKind::duplicate_values(&existing.name)
                    .with_resource_type(Account::TYPE)
                    .with_field("name"));
            }
        }

        // 4. Build entities — type is inherited from parent
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
        Ok(saved.into_values().collect())
    }

    async fn do_update(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<AccountCommandUpdate>,
    ) -> Result<Vec<Account>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        // 1. Reject duplicate IDs
        let batch_ids: HashSet<_> = commands.iter().map(|c| &c.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Account::TYPE)
                .with_field("id"));
        }

        // 2. Fetch existing accounts
        let id_vec: Vec<_> = commands.iter().map(|c| c.id.clone()).collect();
        let existing = repo
            .find_all_by_ids(sess, &id_vec)
            .await
            .map_err(|e| e.convert())?;

        for id in &id_vec {
            if !existing.contains_key(id) {
                return Err(ErrorKind::not_found()
                    .with_resource_type(Account::TYPE)
                    .with_field("id")
                    .with_detail(format!("account {id} not found")));
            }
        }

        // 3. Validate reserved names
        for cmd in &commands {
            if !cmd.name.is_empty() && Account::is_reserved_name(&cmd.name) {
                return Err(ErrorKind::conflict()
                    .with_resource_type(Account::TYPE)
                    .with_field("name")
                    .with_detail(format!("'{}' is a reserved root account name", cmd.name)));
            }
        }

        // 4. Check name uniqueness among siblings (same parent), with swap support
        // Group by parent_id, then check each group
        let mut new_names: HashSet<&str> = HashSet::new();
        for cmd in &commands {
            if !cmd.name.is_empty() && !new_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Account::TYPE)
                    .with_field("name"));
            }
        }

        // For each command with a new name, check against existing siblings NOT in this batch
        for cmd in &commands {
            if cmd.name.is_empty() {
                continue;
            }
            let account = &existing[&cmd.id];
            if let Some(parent_id) = &account.parent_id {
                let spec = AccountSpecification::parent_id(parent_id.clone())
                    & AccountSpecification::name(&cmd.name);
                let conflicts = repo
                    .find_all(sess, &spec, None)
                    .await
                    .map_err(|e| e.convert())?;
                for (conflict_id, conflict) in &conflicts {
                    if !batch_ids.contains(conflict_id) {
                        return Err(ErrorKind::duplicate_values(&conflict.name)
                            .with_resource_type(Account::TYPE)
                            .with_field("name"));
                    }
                }
            }
        }

        // 5. Apply updates
        let commands_by_id: HashMap<_, _> =
            commands.into_iter().map(|c| (c.id.clone(), c)).collect();

        let accounts = id_vec
            .iter()
            .map(|id| {
                let account = &existing[id];
                let cmd = &commands_by_id[id];
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
            })
            .collect::<Result<Vec<_>>>()?;

        let saved = repo
            .save_all(sess, &accounts)
            .await
            .map_err(|e| e.convert())?;
        Ok(saved.into_values().collect())
    }

    async fn do_delete(
        repo: &R,
        sess: &mut R::Session,
        ids: HashSet<AccountId>,
    ) -> Result<Vec<Account>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Collect all descendants to delete (cascade)
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
        let deleted = repo
            .delete_all_by_ids(sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;
        Ok(deleted.into_values().collect())
    }

    async fn do_archive(
        repo: &R,
        sess: &mut R::Session,
        command: AccountCommandArchive,
    ) -> Result<Vec<Account>> {
        if command.ids.is_empty() {
            return Ok(vec![]);
        }

        // Collect all descendants (cascade archive)
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

        // Fetch all accounts to archive
        let ids_vec: Vec<_> = all_ids.into_iter().collect();
        let existing = repo
            .find_all_by_ids(sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;

        // Update archived_at on each (skip already-archived)
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
        Ok(saved.into_values().collect())
    }
}
