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

// ═══════════════════════════════════════════════════════════════════
// create_journal
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_journal_returns_journal_with_id() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "create_journal",
        serde_json::json!({
            "request": {
                "name": "Personal",
                "description": "My ledger",
                "tags": ["finance"]
            }
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["name"], "Personal");
    assert_eq!(result["description"], "My ledger");
    assert!(result["id"].as_str().is_some());
    assert!(!result["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn create_journal_with_defaults() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "create_journal",
        serde_json::json!({
            "request": { "name": "Minimal" }
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["name"], "Minimal");
    assert_eq!(result["description"], "");
    assert_eq!(result["tags"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_journal_empty_name_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "create_journal",
        serde_json::json!({
            "request": { "name": "" }
        }),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn create_duplicate_name_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Dup" } }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "create_journal",
        serde_json::json!({ "request": { "name": "Dup" } }),
    )
    .await;

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// get_journal
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_journal_returns_created_journal() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let created = invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Fetch Me" } }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let fetched = invoke(webview, "get_journal", serde_json::json!({ "id": id }))
        .await
        .unwrap();

    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["name"], "Fetch Me");
}

#[tokio::test]
async fn get_journal_nonexistent_returns_error() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "get_journal",
        serde_json::json!({ "id": "00000000-0000-0000-0000-000000000000" }),
    )
    .await;

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// list_journals
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_journals_empty() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "list_journals",
        serde_json::json!({ "filter": {} }),
    )
    .await
    .unwrap();

    assert_eq!(result.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_journals_returns_all() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "A" } }),
    )
    .await
    .unwrap();
    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "B" } }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "list_journals",
        serde_json::json!({ "filter": {} }),
    )
    .await
    .unwrap();

    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_journals_filter_by_name() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "FindMe" } }),
    )
    .await
    .unwrap();
    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "NotMe" } }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "list_journals",
        serde_json::json!({ "filter": { "name": "FindMe" } }),
    )
    .await
    .unwrap();

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "FindMe");
}

#[tokio::test]
async fn list_journals_filter_by_tag() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Work", "tags": ["office"] } }),
    )
    .await
    .unwrap();
    invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Home", "tags": ["personal"] } }),
    )
    .await
    .unwrap();

    let result = invoke(
        webview,
        "list_journals",
        serde_json::json!({ "filter": { "tag": "office" } }),
    )
    .await
    .unwrap();

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "Work");
}

// ═══════════════════════════════════════════════════════════════════
// update_journal
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn update_journal_changes_name() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let created = invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Before" } }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let updated = invoke(
        webview,
        "update_journal",
        serde_json::json!({ "id": id, "request": { "name": "After" } }),
    )
    .await
    .unwrap();

    assert_eq!(updated["name"], "After");
}

#[tokio::test]
async fn update_journal_preserves_description() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let created = invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Keep", "description": "keep me" } }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let updated = invoke(
        webview,
        "update_journal",
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
        "update_journal",
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "request": { "name": "Nope" }
        }),
    )
    .await;

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// delete_journal
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn delete_journal_removes_it() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let created = invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Doomed" } }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    invoke(
        webview.clone(),
        "delete_journal",
        serde_json::json!({ "id": id }),
    )
    .await
    .unwrap();

    let get_result = invoke(webview, "get_journal", serde_json::json!({ "id": id })).await;

    assert!(get_result.is_err());
}

#[tokio::test]
async fn delete_nonexistent_succeeds() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let result = invoke(
        webview,
        "delete_journal",
        serde_json::json!({ "id": "00000000-0000-0000-0000-000000000000" }),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_is_idempotent() {
    let app = create_test_app().await;
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let created = invoke(
        webview.clone(),
        "create_journal",
        serde_json::json!({ "request": { "name": "Twice" } }),
    )
    .await
    .unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    invoke(
        webview.clone(),
        "delete_journal",
        serde_json::json!({ "id": id }),
    )
    .await
    .unwrap();

    let result = invoke(webview, "delete_journal", serde_json::json!({ "id": id })).await;

    assert!(result.is_ok());
}
