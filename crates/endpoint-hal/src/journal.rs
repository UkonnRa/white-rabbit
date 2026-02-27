pub mod dto;

use std::collections::HashSet;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use shared::{EntityId, ReadRepository};

use crate::error::ProblemDetail;
use crate::hal::HalCollection;
use crate::state::AppState;

use dto::{CreateJournalRequest, JournalProperties, UpdateJournalRequest};

// ── Query params ─────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
pub struct JournalFilterParams {
    #[serde(default, rename = "filter[id]")]
    pub id: Option<String>,
    #[serde(default, rename = "filter[name]")]
    pub name: Option<String>,
    #[serde(default, rename = "filter[tag]")]
    pub tag: Option<String>,
    #[serde(default, rename = "filter[fullText]")]
    pub full_text: Option<String>,
}

// ── POST /journals ───────────────────────────────────────────────

pub async fn create_journal(
    State(state): State<AppState>,
    Json(body): Json<CreateJournalRequest>,
) -> Result<impl IntoResponse, ProblemDetail> {
    let mut sess = state.new_session();

    let cmd = domain::journal::command::JournalCommandCreate {
        name: body.name,
        description: body.description,
        tags: body.tags,
    };

    let events = state
        .journal_service
        .create(&mut sess, [cmd])
        .await
        .map_err(|e| ProblemDetail::from_domain_error(e, "/journals"))?;

    let created_id = events
        .iter()
        .find_map(|e| match e {
            domain::journal::event::JournalEvent::Created(c) => Some(c.id.clone()),
            _ => None,
        })
        .expect("create must produce a Created event");

    let journal = state
        .journal_repo
        .find_one_by_id(&sess, &created_id)
        .await
        .map_err(|e: shared::Error| ProblemDetail::from_domain_error(e.convert(), "/journals"))?
        .ok_or_else(|| ProblemDetail::not_found("/journals"))?;

    let hal = JournalProperties::from(&journal).into_hal();
    Ok((StatusCode::CREATED, Json(hal)))
}

// ── GET /journals/:id ────────────────────────────────────────────

pub async fn get_journal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ProblemDetail> {
    let sess = state.new_session();
    let instance = format!("/journals/{id}");
    let journal_id = domain::journal::JournalId::from_value(&id);

    let journal = state
        .journal_repo
        .find_one_by_id(&sess, &journal_id)
        .await
        .map_err(|e: shared::Error| ProblemDetail::from_domain_error(e.convert(), &instance))?
        .ok_or_else(|| ProblemDetail::not_found(&instance))?;

    let hal = JournalProperties::from(&journal).into_hal();
    Ok(Json(hal))
}

// ── GET /journals ────────────────────────────────────────────────

pub async fn list_journals(
    State(state): State<AppState>,
    Query(params): Query<JournalFilterParams>,
) -> Result<impl IntoResponse, ProblemDetail> {
    use domain::journal::specification::JournalSpecification;
    use shared::SpecificationExpression;

    let sess = state.new_session();

    let mut filters: Vec<SpecificationExpression<JournalSpecification>> = Vec::new();

    if let Some(id) = &params.id {
        let ids: HashSet<_> = id.split(',').map(|s| s.trim().to_string()).collect();
        filters.push(JournalSpecification::ids(ids));
    }
    if let Some(name) = &params.name {
        filters.push(JournalSpecification::name(name));
    }
    if let Some(tag) = &params.tag {
        let tags: HashSet<_> = tag.split(',').map(|s| s.trim().to_string()).collect();
        filters.push(JournalSpecification::tags(tags));
    }
    if let Some(ft) = &params.full_text {
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
        .map_err(|e: shared::Error| ProblemDetail::from_domain_error(e.convert(), "/journals"))?;

    let self_href = build_self_href(&params);
    let items: Vec<_> = journals
        .values()
        .map(|j| JournalProperties::from(j).into_hal())
        .collect();

    let collection = HalCollection::new(self_href, "journals", items);
    Ok(Json(collection))
}

fn build_self_href(params: &JournalFilterParams) -> String {
    let mut parts = Vec::new();
    if let Some(id) = &params.id {
        parts.push(format!("filter[id]={id}"));
    }
    if let Some(name) = &params.name {
        parts.push(format!("filter[name]={name}"));
    }
    if let Some(tag) = &params.tag {
        parts.push(format!("filter[tag]={tag}"));
    }
    if let Some(ft) = &params.full_text {
        parts.push(format!("filter[fullText]={ft}"));
    }
    if parts.is_empty() {
        "/journals".to_string()
    } else {
        format!("/journals?{}", parts.join("&"))
    }
}

// ── PATCH /journals/:id ──────────────────────────────────────────

pub async fn update_journal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateJournalRequest>,
) -> Result<impl IntoResponse, ProblemDetail> {
    let mut sess = state.new_session();
    let instance = format!("/journals/{id}");
    let journal_id = domain::journal::JournalId::from_value(&id);

    let cmd = domain::journal::command::JournalCommandUpdate {
        id: journal_id.clone(),
        name: body.name,
        description: body.description,
        tags: body.tags,
    };

    state
        .journal_service
        .update(&mut sess, [cmd])
        .await
        .map_err(|e| ProblemDetail::from_domain_error(e, &instance))?;

    let journal = state
        .journal_repo
        .find_one_by_id(&sess, &journal_id)
        .await
        .map_err(|e: shared::Error| ProblemDetail::from_domain_error(e.convert(), &instance))?
        .ok_or_else(|| ProblemDetail::not_found(&instance))?;

    let hal = JournalProperties::from(&journal).into_hal();
    Ok(Json(hal))
}

// ── DELETE /journals/:id ─────────────────────────────────────────

pub async fn delete_journal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ProblemDetail> {
    let mut sess = state.new_session();
    let instance = format!("/journals/{id}");
    let journal_id = domain::journal::JournalId::from_value(&id);

    state
        .journal_service
        .delete(&mut sess, [journal_id])
        .await
        .map_err(|e| ProblemDetail::from_domain_error(e, &instance))?;

    Ok(StatusCode::NO_CONTENT)
}
