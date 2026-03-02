pub mod dto;

use std::collections::HashSet;

use shared::{EntityId, ReadRepository, WriteService};

use crate::error::CommandError;
use crate::state::AppState;
use dto::{CreateJournalRequest, JournalFilter, JournalResponse, UpdateJournalRequest};

// ── POST /journals ───────────────────────────────────────────────

#[tauri::command]
pub async fn create_journal(
    state: tauri::State<'_, AppState>,
    request: CreateJournalRequest,
) -> Result<JournalResponse, String> {
    let mut sess = state.new_session();

    let cmd = domain::journal::command::JournalCommandCreate {
        name: request.name,
        description: request.description,
        tags: request.tags,
    };

    let result = state
        .journal_service
        .handle(
            &mut sess,
            domain::journal::command::JournalCommand::Create(cmd),
        )
        .await
        .map_err(CommandError::from_domain)?;

    let journal = result
        .entities
        .into_iter()
        .next()
        .ok_or_else(|| String::from("journal not found after creation"))?;

    Ok(JournalResponse::from_journal(&journal))
}

// ── GET /journals/:id ────────────────────────────────────────────

#[tauri::command]
pub async fn get_journal(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<JournalResponse, String> {
    let sess = state.new_session();
    let journal_id = domain::journal::JournalId::from_value(&id);

    let journal = state
        .journal_repo
        .find_one_by_id(&sess, &journal_id)
        .await
        .map_err(CommandError::from_shared)?
        .ok_or_else(|| format!("journal {id} not found"))?;

    Ok(JournalResponse::from_journal(&journal))
}

// ── GET /journals ────────────────────────────────────────────────

#[tauri::command]
pub async fn list_journals(
    state: tauri::State<'_, AppState>,
    filter: JournalFilter,
) -> Result<Vec<JournalResponse>, String> {
    use domain::journal::specification::JournalSpecification;
    use shared::SpecificationExpression;

    let sess = state.new_session();

    let mut filters: Vec<SpecificationExpression<JournalSpecification>> = Vec::new();

    if let Some(id) = &filter.id {
        let ids: HashSet<_> = id.split(',').map(|s| s.trim().to_string()).collect();
        filters.push(JournalSpecification::ids(ids));
    }
    if let Some(name) = &filter.name {
        filters.push(JournalSpecification::name(name));
    }
    if let Some(tag) = &filter.tag {
        let tags: HashSet<_> = tag.split(',').map(|s| s.trim().to_string()).collect();
        filters.push(JournalSpecification::tags(tags));
    }
    if let Some(ft) = &filter.full_text {
        filters.push(JournalSpecification::full_text(ft));
    }

    let spec = if filters.is_empty() {
        SpecificationExpression::All(vec![])
    } else if filters.len() == 1 {
        filters.remove(0)
    } else {
        SpecificationExpression::All(filters)
    };

    let journals = state
        .journal_repo
        .find_all(&sess, &spec, None)
        .await
        .map_err(CommandError::from_shared)?;

    Ok(journals
        .values()
        .map(JournalResponse::from_journal)
        .collect())
}

// ── PATCH /journals/:id ──────────────────────────────────────────

#[tauri::command]
pub async fn update_journal(
    state: tauri::State<'_, AppState>,
    id: String,
    request: UpdateJournalRequest,
) -> Result<JournalResponse, String> {
    let mut sess = state.new_session();
    let journal_id = domain::journal::JournalId::from_value(&id);

    let cmd = domain::journal::command::JournalCommandUpdate {
        id: journal_id.clone(),
        name: request.name,
        description: request.description,
        tags: request.tags,
    };

    let result = state
        .journal_service
        .handle(
            &mut sess,
            domain::journal::command::JournalCommand::Update(cmd),
        )
        .await
        .map_err(CommandError::from_domain)?;

    let journal = result
        .entities
        .into_iter()
        .next()
        .ok_or_else(|| format!("journal {id} not found after update"))?;

    Ok(JournalResponse::from_journal(&journal))
}

// ── DELETE /journals/:id ─────────────────────────────────────────

#[tauri::command]
pub async fn delete_journal(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut sess = state.new_session();
    let journal_id = domain::journal::JournalId::from_value(&id);

    state
        .journal_service
        .handle(
            &mut sess,
            domain::journal::command::JournalCommand::Delete(HashSet::from([journal_id])),
        )
        .await
        .map_err(CommandError::from_domain)?;

    Ok(())
}
