use chrono::Utc;

use crate::error::Result;
use crate::journal::command::{
    JournalCommand, JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate,
};
use crate::journal::event::*;
use crate::journal::repository::JournalRepository;
use crate::journal::specification::JournalSpecification;
use crate::journal::{Journal, JournalId, JournalInput};
use shared::{Entity, ErrorKind, RepositorySession, WriteService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct JournalService<R: JournalRepository> {
    pub repository: Arc<R>,
}

impl<R: JournalRepository> JournalService<R> {
    /// Create new [`Journal`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `name` must be unique both within the batch and across
    ///   all existing journals.
    ///
    /// # Returns
    ///
    /// One [`JournalEvent::Created`] per successfully created journal
    /// (with DB-assigned IDs; any client-provided IDs in the commands
    /// are ignored).
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — duplicate names within the batch or against existing journals.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty)
    ///   — a name is empty or blank.
    pub async fn create(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = JournalCommandCreate>,
    ) -> Result<Vec<JournalEvent>> {
        Self::do_create(&self.repository, sess, commands.into_iter().collect()).await
    }

    /// Update existing [`Journal`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `id` must refer to an existing journal.
    /// - Each `id` must appear at most once in the batch.
    /// - The resulting `name`s must be unique:
    ///   - No two commands in the batch may produce the same name.
    ///   - No resulting name may collide with an existing journal that is
    ///     **not** part of this batch.
    ///
    /// This means **name swaps are allowed**: if Journal A has name "X" and
    /// Journal B has name "Y", a batch that renames A→"Y" and B→"X" succeeds
    /// because both A and B are in the batch — the conflict check excludes
    /// journals that are being updated.
    ///
    /// # Algorithm (name uniqueness)
    ///
    /// 1. Collect the set of **new names** from the batch.
    /// 2. Reject if any new name appears more than once in the batch.
    /// 3. Query existing journals whose names match any new name.
    /// 4. For each match, if its `id` is **not** in the batch → conflict.
    ///    (If it **is** in the batch, its name is about to change — no conflict.)
    ///
    /// # Returns
    ///
    /// One [`JournalEvent::Updated`] per successfully updated journal.
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(NotFound)`](shared::ErrorKind::NotFound)
    ///   — a journal ID does not exist.
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — duplicate names within the batch or against existing journals
    ///   not being updated.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty)
    ///   — a name is empty or blank.
    pub async fn update(
        &self,
        sess: &mut R::Session,
        commands: impl IntoIterator<Item = JournalCommandUpdate>,
    ) -> Result<Vec<JournalEvent>> {
        Self::do_update(&self.repository, sess, commands.into_iter().collect()).await
    }

    /// Delete [`Journal`]s by their IDs.
    ///
    /// Duplicate IDs are deduplicated automatically. IDs that do not match
    /// any existing journal are silently ignored — this is intentional to
    /// prevent ID-guessing attacks from inferring which IDs exist via error
    /// responses.
    ///
    /// # Returns
    ///
    /// One [`JournalEvent::Deleted`] per ID that was passed in (regardless
    /// of whether the journal existed).
    pub async fn delete(
        &self,
        sess: &mut R::Session,
        ids: impl IntoIterator<Item = impl Into<JournalId>>,
    ) -> Result<Vec<JournalEvent>> {
        Self::do_delete(
            &self.repository,
            sess,
            ids.into_iter().map(Into::into).collect(),
        )
        .await
    }

    /// Execute a batch of create, update, and delete operations atomically.
    ///
    /// Operations are applied in the order: **delete → create → update**.
    /// This ordering ensures:
    /// - Deleted journals free their names for reuse by create/update.
    /// - Newly created journals are visible to the update name-conflict check.
    ///
    /// The entire batch is wrapped in a transaction: if any phase fails,
    /// all mutations are rolled back.
    ///
    /// # Returns
    ///
    /// All events produced by the sub-operations, in execution order.
    pub async fn batch(
        &self,
        sess: &mut R::Session,
        command: JournalCommandBatch,
    ) -> Result<Vec<JournalEvent>> {
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
        command: JournalCommandBatch,
    ) -> Result<Vec<JournalEvent>> {
        let mut events = Vec::new();
        events.extend(Self::do_delete(repo, sess, command.delete).await?);
        events.extend(Self::do_create(repo, sess, command.create).await?);
        events.extend(Self::do_update(repo, sess, command.update).await?);
        Ok(events)
    }

    // ── Internal implementations ─────────────────────────────────

    async fn do_create(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<JournalCommandCreate>,
    ) -> Result<Vec<JournalEvent>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();

        let mut seen_names = HashSet::new();
        for cmd in &commands {
            if !seen_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let spec = JournalSpecification::names(seen_names.iter().copied());
        if let Some(existing) = repo.find_one(sess, &spec).await.map_err(|e| e.convert())? {
            return Err(ErrorKind::duplicate_values(&existing.name)
                .with_resource_type(Journal::ENTITY_TYPE)
                .with_field("name")
                .convert());
        }

        let journals = commands
            .into_iter()
            .map(|cmd| {
                let input = JournalInput {
                    name: cmd.name,
                    description: cmd.description,
                    tags: cmd.tags,
                    ..Default::default()
                };
                input.try_into()
            })
            .collect::<Result<Vec<Journal>>>()?;

        let saved = repo
            .save_all(sess, &journals)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|j| {
                JournalEvent::Created(JournalCreated {
                    id: j.id,
                    name: j.name.to_string(),
                    description: j.description,
                    tags: j.tags.iter().map(|t| t.to_string()).collect(),
                    created_at: j.created_at.unwrap_or(now),
                })
            })
            .collect())
    }

    async fn do_update(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<JournalCommandUpdate>,
    ) -> Result<Vec<JournalEvent>> {
        if commands.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();

        let batch_ids: HashSet<_> = commands.iter().map(|cmd| &cmd.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Journal::ENTITY_TYPE)
                .with_field("id")
                .convert());
        }

        let id_vec: Vec<_> = commands.iter().map(|cmd| cmd.id.clone()).collect();
        let existing = repo
            .find_all_by_ids(sess, &id_vec)
            .await
            .map_err(|e| e.convert())?;

        for id in &id_vec {
            if !existing.contains_key(id) {
                return Err(ErrorKind::not_found()
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("id")
                    .with_detail(format!("journal {id} not found"))
                    .convert());
            }
        }

        let mut new_names = HashSet::new();
        for cmd in &commands {
            if !new_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let spec = JournalSpecification::names(new_names.iter().copied());
        let conflicts = repo
            .find_all(sess, &spec, None)
            .await
            .map_err(|e| e.convert())?;

        for (conflict_id, conflict) in &conflicts {
            if !batch_ids.contains(conflict_id) {
                return Err(ErrorKind::duplicate_values(&conflict.name)
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let commands_by_id: HashMap<_, _> = commands
            .into_iter()
            .map(|cmd| (cmd.id.clone(), cmd))
            .collect();

        let journals = id_vec
            .iter()
            .map(|id| {
                let journal = &existing[id];
                let cmd = &commands_by_id[id];
                let input = JournalInput {
                    id: journal.id.clone(),
                    version: journal.version,
                    created_at: journal.created_at,
                    last_modified_at: journal.last_modified_at,
                    archived_at: journal.archived_at,
                    name: if cmd.name.is_empty() {
                        journal.name.to_string()
                    } else {
                        cmd.name.clone()
                    },
                    description: cmd
                        .description
                        .clone()
                        .unwrap_or_else(|| journal.description.clone()),
                    tags: cmd
                        .tags
                        .clone()
                        .unwrap_or_else(|| journal.tags.iter().map(|t| t.to_string()).collect()),
                };
                input.try_into()
            })
            .collect::<Result<Vec<_>>>()?;

        let saved = repo
            .save_all(sess, &journals)
            .await
            .map_err(|e| e.convert())?;

        Ok(saved
            .into_values()
            .map(|j| {
                JournalEvent::Updated(JournalUpdated {
                    id: j.id,
                    name: Some(j.name.to_string()),
                    description: Some(j.description),
                    tags: Some(j.tags.iter().map(|t| t.to_string()).collect()),
                    last_modified_at: j.last_modified_at.unwrap_or(now),
                })
            })
            .collect())
    }

    async fn do_delete(
        repo: &R,
        sess: &mut R::Session,
        ids: HashSet<JournalId>,
    ) -> Result<Vec<JournalEvent>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let ids_vec: Vec<_> = ids.into_iter().collect();
        repo.delete_all_by_ids(sess, &ids_vec)
            .await
            .map_err(|e| e.convert())?;
        Ok(ids_vec
            .into_iter()
            .map(|id| JournalEvent::Deleted(JournalDeleted { id }))
            .collect())
    }
}

#[async_trait::async_trait]
impl<R: JournalRepository> WriteService<JournalCommand> for JournalService<R> {
    type Event = JournalEvent;
    type Session = R::Session;
    type Error = crate::error::Error;

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: JournalCommand,
    ) -> Result<Vec<JournalEvent>> {
        match command {
            JournalCommand::Create(cmd) => self.create(sess, [cmd]).await,
            JournalCommand::Update(cmd) => self.update(sess, [cmd]).await,
            JournalCommand::Delete(ids) => self.delete(sess, ids).await,
            JournalCommand::Batch(cmd) => self.batch(sess, cmd).await,
        }
    }
}
