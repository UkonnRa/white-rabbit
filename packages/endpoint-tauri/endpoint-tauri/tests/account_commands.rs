use database_seaorm_migration::MigratorTrait;
use endpoint_tauri_lib::state::AppState;
use tauri::Manager;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
use tauri::webview::InvokeRequest;

async fn create_test_app() -> tauri::App<tauri::test::MockRuntime> {
    let app = endpoint_tauri_lib::register_handlers(mock_builder())
        .build(mock_context(noop_assets()))
        .expect("failed to build test app");

    let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    database_seaorm_migration::Migrator::up(&db, None)
        .await
        .unwrap();
    app.manage(AppState::new(db));
    app
}

async fn invoke(
    webview: tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let cmd = cmd.to_string();
    tokio::task::spawn_blocking(move || {
        get_ipc_response(
            &webview,
            InvokeRequest {
                cmd,
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<serde_json::Value>().unwrap())
        .map_err(|e| format!("{e:?}"))
    })
    .await
    .unwrap()
}

/// Create a journal via IPC and return its id.
async fn seed_journal(
    webview: tauri::WebviewWindow<tauri::test::MockRuntime>,
    name: &str,
) -> String {
    let result = invoke(
        webview,
        "create_journal",
        serde_json::json!({ "request": { "name": name } }),
    )
    .await
    .unwrap();
    result["id"].as_str().unwrap().to_string()
}

/// Shared setup: create a journal (which auto-seeds 5 root accounts),
/// then find the Asset root. Returns (journal_id, root_id).
async fn setup(
    _app: &tauri::App<tauri::test::MockRuntime>,
    webview: tauri::WebviewWindow<tauri::test::MockRuntime>,
) -> (String, String) {
    let journal_id = seed_journal(webview.clone(), "Test Journal").await;

    let result = invoke(
        webview,
        "list_accounts",
        serde_json::json!({ "filter": { "journalId": &journal_id, "type": "Asset" } }),
    )
    .await
    .unwrap();

    let arr = result.as_array().unwrap();
    let root_id = arr[0]["id"].as_str().unwrap().to_string();

    (journal_id, root_id)
}

// ═══════════════════════════════════════════════════════════════════
// create_account
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_account_returns_account_with_id() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let result = invoke(
        webview,
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Cash",
                "description": "Petty cash",
                "tags": ["liquid"]
            }
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["name"], "Cash");
    assert_eq!(result["description"], "Petty cash");
    assert_eq!(result["type"], "Asset");
    assert_eq!(result["journalId"], journal_id);
    assert!(result["id"].as_str().is_some());
    assert!(!result["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn create_account_with_defaults() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let result = invoke(
        webview,
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Minimal"
            }
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["name"], "Minimal");
    assert_eq!(result["description"], "");
    assert_eq!(result["tags"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_account_empty_name_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let result = invoke(
        webview,
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": ""
            }
        }),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn create_account_reserved_name_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let result = invoke(
        webview,
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Asset"
            }
        }),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn create_duplicate_name_under_same_parent_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Dup"
            }
        }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Dup"
            }
        }),
    )
    .await;

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// get_account
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_account_returns_created_account() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let created = invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Checking"
            }
        }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let fetched = invoke(webview, "get_account", serde_json::json!({ "id": id }))
        .await
        .unwrap();

    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["name"], "Checking");
}

#[tokio::test]
async fn get_account_nonexistent_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "get_account",
        serde_json::json!({ "id": "00000000-0000-0000-0000-000000000000" }),
    )
    .await;

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// list_accounts
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_accounts_by_journal() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "A"
            }
        }),
    )
    .await
    .unwrap();
    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "B"
            }
        }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "list_accounts",
        serde_json::json!({ "filter": { "journalId": journal_id } }),
    )
    .await
    .unwrap();

    // 5 roots + 2 children
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 7);
}

#[tokio::test]
async fn list_accounts_filter_by_name() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "FindMe"
            }
        }),
    )
    .await
    .unwrap();
    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "NotMe"
            }
        }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "list_accounts",
        serde_json::json!({ "filter": { "name": "FindMe" } }),
    )
    .await
    .unwrap();

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "FindMe");
}

#[tokio::test]
async fn list_accounts_filter_by_tag() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Cash",
                "tags": ["liquid"]
            }
        }),
    )
    .await
    .unwrap();
    invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Building",
                "tags": ["fixed"]
            }
        }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "list_accounts",
        serde_json::json!({ "filter": { "tag": "liquid" } }),
    )
    .await
    .unwrap();

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "Cash");
}

// ═══════════════════════════════════════════════════════════════════
// update_account
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn update_account_changes_name() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let created = invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Before"
            }
        }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let updated = invoke(
        webview,
        "update_account",
        serde_json::json!({ "id": id, "request": { "name": "After" } }),
    )
    .await
    .unwrap();

    assert_eq!(updated["name"], "After");
}

#[tokio::test]
async fn update_account_preserves_description() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let created = invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Keep",
                "description": "keep me"
            }
        }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let updated = invoke(
        webview,
        "update_account",
        serde_json::json!({ "id": id, "request": { "name": "Renamed" } }),
    )
    .await
    .unwrap();

    assert_eq!(updated["name"], "Renamed");
    assert_eq!(updated["description"], "keep me");
}

#[tokio::test]
async fn update_nonexistent_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "update_account",
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "request": { "name": "Nope" }
        }),
    )
    .await;

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// delete_account
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn delete_account_removes_it() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let created = invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Doomed"
            }
        }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    invoke(
        webview.clone(),
        "delete_account",
        serde_json::json!({ "id": id }),
    )
    .await
    .unwrap();

    let result = invoke(webview, "get_account", serde_json::json!({ "id": id })).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_nonexistent_succeeds() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "delete_account",
        serde_json::json!({ "id": "00000000-0000-0000-0000-000000000000" }),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_cascades_to_children() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let (journal_id, root_id) = setup(&app, webview.clone()).await;

    let parent = invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": root_id,
                "name": "Parent"
            }
        }),
    )
    .await
    .unwrap();
    let parent_id = parent["id"].as_str().unwrap().to_string();

    let child = invoke(
        webview.clone(),
        "create_account",
        serde_json::json!({
            "request": {
                "journalId": journal_id,
                "parentId": parent_id,
                "name": "Child"
            }
        }),
    )
    .await
    .unwrap();
    let child_id = child["id"].as_str().unwrap().to_string();

    // Delete parent — child should cascade
    invoke(
        webview.clone(),
        "delete_account",
        serde_json::json!({ "id": parent_id }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "get_account",
        serde_json::json!({ "id": child_id }),
    )
    .await;
    assert!(result.is_err());
}
