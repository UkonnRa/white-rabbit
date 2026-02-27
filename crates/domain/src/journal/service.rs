use chrono::Utc;

use crate::journal::command::{
    JournalCommand, JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate,
};
use crate::journal::event::*;
use crate::journal::repository::JournalRepository;
use crate::journal::specification::JournalSpecification;
use crate::journal::{JournalId, JournalInput};
use crate::{error::Result, journal::Journal};
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
    /// The persisted [`Journal`] list as returned by the repository (with
    /// DB-assigned IDs; any client-provided IDs in the commands are ignored).
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
    ) -> Result<Vec<Journal>> {
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
    /// The updated [`Journal`] list as returned by the repository.
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
    ) -> Result<Vec<Journal>> {
        Self::do_update(&self.repository, sess, commands.into_iter().collect()).await
    }

    /// Delete [`Journal`]s by their IDs.
    ///
    /// Duplicate IDs are deduplicated automatically. IDs that do not match
    /// any existing journal are silently ignored — this is intentional to
    /// prevent ID-guessing attacks from inferring which IDs exist via error
    /// responses.
    pub async fn delete(
        &self,
        sess: &mut R::Session,
        ids: impl IntoIterator<Item = impl Into<JournalId>>,
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
    /// The journals that remain in the database after the batch — i.e. created
    /// and updated journals, deduplicated by ID (if a journal is created then
    /// updated in the same batch, only the final state is returned).
    /// Deleted journals are not included.
    pub async fn batch(
        &self,
        sess: &mut R::Session,
        command: JournalCommandBatch,
    ) -> Result<Vec<Journal>> {
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
    ) -> Result<Vec<Journal>> {
        Self::do_delete(repo, sess, command.delete).await?;
        let created = Self::do_create(repo, sess, command.create).await?;
        let updated = Self::do_update(repo, sess, command.update).await?;

        let mut result: HashMap<_, _> = created.into_iter().map(|j| (j.id.clone(), j)).collect();
        for j in updated {
            result.insert(j.id.clone(), j);
        }

        Ok(result.into_values().collect())
    }

    // ── Internal implementations ─────────────────────────────────

    async fn do_create(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<JournalCommandCreate>,
    ) -> Result<Vec<Journal>> {
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
        Ok(saved.into_values().collect())
    }

    async fn do_update(
        repo: &R,
        sess: &mut R::Session,
        commands: Vec<JournalCommandUpdate>,
    ) -> Result<Vec<Journal>> {
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
        Ok(saved.into_values().collect())
    }

    async fn do_delete(repo: &R, sess: &mut R::Session, ids: HashSet<JournalId>) -> Result<()> {
        let ids: Vec<_> = ids.into_iter().collect();
        repo.delete_all_by_ids(sess, &ids)
            .await
            .map_err(|e| e.convert())?;
        Ok(())
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
        let now = Utc::now();
        match command {
            JournalCommand::Create(cmd) => {
                let journals = self.create(sess, [cmd]).await?;
                Ok(journals
                    .into_iter()
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
            JournalCommand::Update(cmd) => {
                let id = cmd.id.clone();
                let name = if cmd.name.is_empty() {
                    None
                } else {
                    Some(cmd.name.clone())
                };
                let description = cmd.description.clone();
                let tags = cmd.tags.clone();
                self.update(sess, [cmd]).await?;
                Ok(vec![JournalEvent::Updated(JournalUpdated {
                    id,
                    name,
                    description,
                    tags,
                    last_modified_at: now,
                })])
            }
            JournalCommand::Delete(ids) => {
                let events: Vec<_> = ids
                    .iter()
                    .map(|id| JournalEvent::Deleted(JournalDeleted { id: id.clone() }))
                    .collect();
                self.delete(sess, ids).await?;
                Ok(events)
            }
            JournalCommand::Batch(cmd) => {
                let delete_events: Vec<_> = cmd
                    .delete
                    .iter()
                    .map(|id| JournalEvent::Deleted(JournalDeleted { id: id.clone() }))
                    .collect();
                let create_cmds = cmd.create.clone();
                let update_cmds = cmd.update.clone();

                self.batch(sess, cmd).await?;

                let mut events = delete_events;
                for create_cmd in create_cmds {
                    events.push(JournalEvent::Created(JournalCreated {
                        id: JournalId::default(),
                        name: create_cmd.name,
                        description: create_cmd.description,
                        tags: create_cmd.tags,
                        created_at: now,
                    }));
                }
                for update_cmd in update_cmds {
                    let name = if update_cmd.name.is_empty() {
                        None
                    } else {
                        Some(update_cmd.name)
                    };
                    events.push(JournalEvent::Updated(JournalUpdated {
                        id: update_cmd.id,
                        name,
                        description: update_cmd.description,
                        tags: update_cmd.tags,
                        last_modified_at: now,
                    }));
                }
                Ok(events)
            }
        }
    }
}
