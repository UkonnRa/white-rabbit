pub mod dto;

use std::collections::HashSet;

use shared::{EntityId, ReadRepository, WriteService};

use crate::error::CommandError;
use crate::state::AppState;
use dto::{AccountFilter, AccountResponse, CreateAccountRequest, UpdateAccountRequest};

// ── POST /accounts ──────────────────────────────────────────────

#[tauri::command]
pub async fn create_account(
    state: tauri::State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<AccountResponse, String> {
    let mut sess = state.new_session();

    let cmd = domain::account::command::AccountCommandCreate {
        journal_id: domain::journal::JournalId::from_value(&request.journal_id),
        parent_id: domain::account::AccountId::from_value(&request.parent_id),
        name: request.name,
        description: request.description,
        tags: request.tags,
    };

    let result = state
        .account_service
        .handle(
            &mut sess,
            domain::account::command::AccountCommand::Create(cmd),
        )
        .await
        .map_err(CommandError::from_domain)?;

    let account = result
        .entities
        .into_iter()
        .next()
        .ok_or_else(|| String::from("account not found after creation"))?;

    Ok(AccountResponse::from_account(&account))
}

// ── GET /accounts/:id ───────────────────────────────────────────

#[tauri::command]
pub async fn get_account(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<AccountResponse, String> {
    let sess = state.new_session();
    let account_id = domain::account::AccountId::from_value(&id);

    let account = state
        .account_repo
        .find_one_by_id(&sess, &account_id)
        .await
        .map_err(CommandError::from_shared)?
        .ok_or_else(|| format!("account {id} not found"))?;

    Ok(AccountResponse::from_account(&account))
}

// ── GET /accounts ───────────────────────────────────────────────

#[tauri::command]
pub async fn list_accounts(
    state: tauri::State<'_, AppState>,
    filter: AccountFilter,
) -> Result<Vec<AccountResponse>, String> {
    use domain::account::specification::AccountSpecification;
    use shared::SpecificationExpression;

    let sess = state.new_session();

    let mut filters: Vec<SpecificationExpression<AccountSpecification>> = Vec::new();

    if let Some(id) = &filter.id {
        let ids: HashSet<_> = id.split(',').map(|s| s.trim().to_string()).collect();
        filters.push(AccountSpecification::ids(ids));
    }
    if let Some(journal_id) = &filter.journal_id {
        let ids: HashSet<_> = journal_id
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        filters.push(SpecificationExpression::Leaf(
            AccountSpecification::JournalId(
                ids.into_iter()
                    .map(|s| domain::journal::JournalId::from_value(&s))
                    .collect(),
            ),
        ));
    }
    if let Some(parent_id) = &filter.parent_id {
        filters.push(AccountSpecification::parent_id(
            domain::account::AccountId::from_value(parent_id),
        ));
    }
    if let Some(name) = &filter.name {
        filters.push(AccountSpecification::name(name));
    }
    if let Some(account_type) = &filter.r#type
        && let Some(t) = parse_account_type(account_type) {
            filters.push(AccountSpecification::account_type(t));
        }
    if let Some(tag) = &filter.tag {
        let tags: HashSet<_> = tag.split(',').map(|s| s.trim().to_string()).collect();
        filters.push(AccountSpecification::tags(tags));
    }
    if let Some(ft) = &filter.full_text {
        filters.push(AccountSpecification::full_text(ft));
    }

    let spec = if filters.is_empty() {
        SpecificationExpression::All(vec![])
    } else if filters.len() == 1 {
        filters.remove(0)
    } else {
        SpecificationExpression::All(filters)
    };

    let accounts = state
        .account_repo
        .find_all(&sess, &spec, None)
        .await
        .map_err(CommandError::from_shared)?;

    Ok(accounts
        .values()
        .map(AccountResponse::from_account)
        .collect())
}

// ── PATCH /accounts/:id ─────────────────────────────────────────

#[tauri::command]
pub async fn update_account(
    state: tauri::State<'_, AppState>,
    id: String,
    request: UpdateAccountRequest,
) -> Result<AccountResponse, String> {
    let mut sess = state.new_session();
    let account_id = domain::account::AccountId::from_value(&id);

    let cmd = domain::account::command::AccountCommandUpdate {
        id: account_id,
        name: request.name,
        description: request.description,
        tags: request.tags,
    };

    let result = state
        .account_service
        .handle(
            &mut sess,
            domain::account::command::AccountCommand::Update(cmd),
        )
        .await
        .map_err(CommandError::from_domain)?;

    let account = result
        .entities
        .into_iter()
        .next()
        .ok_or_else(|| format!("account {id} not found after update"))?;

    Ok(AccountResponse::from_account(&account))
}

// ── DELETE /accounts/:id ────────────────────────────────────────

#[tauri::command]
pub async fn delete_account(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut sess = state.new_session();
    let account_id = domain::account::AccountId::from_value(&id);

    state
        .account_service
        .handle(
            &mut sess,
            domain::account::command::AccountCommand::Delete(HashSet::from([account_id])),
        )
        .await
        .map_err(CommandError::from_domain)?;

    Ok(())
}

fn parse_account_type(s: &str) -> Option<domain::account::AccountType> {
    match s.to_lowercase().as_str() {
        "asset" => Some(domain::account::AccountType::Asset),
        "liability" => Some(domain::account::AccountType::Liability),
        "equity" => Some(domain::account::AccountType::Equity),
        "income" => Some(domain::account::AccountType::Income),
        "expense" => Some(domain::account::AccountType::Expense),
        _ => None,
    }
}
