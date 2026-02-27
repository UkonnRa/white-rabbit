use axum::body::Body;
use axum::http::{Request, StatusCode};
use database_seaorm_migration::{Migrator, MigratorTrait};
use endpoint_hal::state::AppState;
use http_body_util::BodyExt;
use sea_orm::Database;
use tower::ServiceExt;

async fn app_state() -> AppState {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    AppState::new(db)
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// POST /journals
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_returns_201_with_hal_resource() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Personal","description":"My ledger","tags":["finance"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);

    let json = body_json(resp).await;
    assert_eq!(json["name"], "Personal");
    assert_eq!(json["description"], "My ledger");

    let id = json["id"].as_str().unwrap();
    assert!(!id.is_empty());

    let self_href = json["_links"]["self"]["href"].as_str().unwrap();
    assert_eq!(self_href, format!("/journals/{id}"));

    assert!(json.get("created_at").is_some());
    assert!(json.get("last_modified_at").is_some());
    assert!(json.get("archived_at").is_some());
}

#[tokio::test]
async fn create_with_only_name_uses_defaults() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Minimal"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Minimal");
    assert_eq!(json["description"], "");
    assert_eq!(json["tags"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_with_tags_round_trips() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Tagged","tags":["alpha","beta","gamma"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;

    let mut tags: Vec<String> = json["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    tags.sort();
    assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
}

#[tokio::test]
async fn create_empty_name_returns_422() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = body_json(resp).await;
    assert_eq!(json["type"], "about:blank");
    assert_eq!(json["status"], 422);
    assert!(json["title"].as_str().is_some());
    assert_eq!(json["instance"], "/journals");
}

#[tokio::test]
async fn create_duplicate_name_returns_422() {
    let state = app_state().await;

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Dup"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Dup"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = body_json(resp).await;
    assert_eq!(json["status"], 422);
    assert!(json["detail"].as_str().unwrap().contains("Dup"));
}

#[tokio::test]
async fn create_invalid_json_returns_400() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"not valid json"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_missing_name_field_returns_422() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"description":"no name"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ═══════════════════════════════════════════════════════════════════
// GET /journals/:id
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_returns_full_hal_resource() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Full","description":"desc","tags":["t1"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert_eq!(json["id"], id);
    assert_eq!(json["name"], "Full");
    assert_eq!(json["description"], "desc");
    assert_eq!(json["tags"].as_array().unwrap().len(), 1);
    assert_eq!(json["_links"]["self"]["href"], format!("/journals/{id}"));
    assert!(json.get("created_at").is_some());
    assert!(json.get("last_modified_at").is_some());
    assert!(json.get("archived_at").is_some());
}

#[tokio::test]
async fn get_nonexistent_returns_404_problem_detail() {
    let state = app_state().await;
    let fake_id = "00000000-0000-0000-0000-000000000000";

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/journals/{fake_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = body_json(resp).await;
    assert_eq!(json["type"], "about:blank");
    assert_eq!(json["title"], "Not found");
    assert_eq!(json["status"], 404);
    assert_eq!(json["instance"], format!("/journals/{fake_id}"));
}

#[tokio::test]
async fn get_returns_data_matching_create() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Mirror","description":"check","tags":["x","y"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let created = body_json(create_resp).await;
    let id = created["id"].as_str().unwrap();

    let get_resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let fetched = body_json(get_resp).await;

    assert_eq!(fetched["id"], created["id"]);
    assert_eq!(fetched["name"], created["name"]);
    assert_eq!(fetched["description"], created["description"]);
    assert_eq!(fetched["created_at"], created["created_at"]);
}

// ═══════════════════════════════════════════════════════════════════
// GET /journals (list)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_empty_returns_collection_with_zero_total() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["total"], 0);
    assert!(json["_embedded"]["journals"].as_array().unwrap().is_empty());
    assert_eq!(json["_links"]["self"]["href"], "/journals");
}

#[tokio::test]
async fn list_returns_all_created_journals() {
    let state = app_state().await;

    for name in ["A", "B", "C"] {
        endpoint_hal::router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/journals")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"name":"{name}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 3);

    let items = json["_embedded"]["journals"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    for item in items {
        assert!(item["_links"]["self"]["href"].as_str().is_some());
        assert!(item["id"].as_str().is_some());
    }
}

#[tokio::test]
async fn list_filter_by_name() {
    let state = app_state().await;

    for name in ["FindMe", "NotMe"] {
        endpoint_hal::router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/journals")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"name":"{name}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals?filter%5Bname%5D=FindMe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 1);
    assert_eq!(json["_embedded"]["journals"][0]["name"], "FindMe");
}

#[tokio::test]
async fn list_filter_by_tag() {
    let state = app_state().await;

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Work","tags":["office"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Home","tags":["personal"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals?filter%5Btag%5D=office")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 1);
    assert_eq!(json["_embedded"]["journals"][0]["name"], "Work");
}

#[tokio::test]
async fn list_filter_by_full_text() {
    let state = app_state().await;

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Investments","description":"Stock portfolio tracking"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Daily","description":"Groceries"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals?filter%5BfullText%5D=portfolio")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 1);
    assert_eq!(json["_embedded"]["journals"][0]["name"], "Investments");
}

#[tokio::test]
async fn list_filter_by_id() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Target"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Other"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/journals?filter%5Bid%5D={id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 1);
    assert_eq!(json["_embedded"]["journals"][0]["id"], id);
}

#[tokio::test]
async fn list_filter_no_match_returns_empty_collection() {
    let state = app_state().await;

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Exists"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals?filter%5Bname%5D=Nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 0);
    assert!(json["_embedded"]["journals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_each_item_has_hal_self_link() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Linked"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    let item = &json["_embedded"]["journals"][0];
    assert_eq!(item["_links"]["self"]["href"], format!("/journals/{id}"));
}

// ═══════════════════════════════════════════════════════════════════
// PATCH /journals/:id
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn update_name_only_preserves_description() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Before","description":"keep me"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/journals/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"After"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "After");
    assert_eq!(json["description"], "keep me");
}

#[tokio::test]
async fn update_description_only_preserves_name() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Stable"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/journals/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"description":"new desc"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Stable");
    assert_eq!(json["description"], "new desc");
}

#[tokio::test]
async fn update_tags() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Taggable","tags":["old"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/journals/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"tags":["new1","new2"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let mut tags: Vec<String> = json["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    tags.sort();
    assert_eq!(tags, vec!["new1", "new2"]);
}

#[tokio::test]
async fn update_changes_name_on_subsequent_get() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"BeforeUpdate"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/journals/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"AfterUpdate"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["name"], "AfterUpdate");
}

#[tokio::test]
async fn update_returns_hal_self_link() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Links"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/journals/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Links2"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["_links"]["self"]["href"], format!("/journals/{id}"));
}

#[tokio::test]
async fn update_nonexistent_returns_404() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/journals/00000000-0000-0000-0000-000000000000")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Nope"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = body_json(resp).await;
    assert_eq!(json["status"], 404);
    assert_eq!(
        json["instance"],
        "/journals/00000000-0000-0000-0000-000000000000"
    );
}

#[tokio::test]
async fn update_to_duplicate_name_returns_422() {
    let state = app_state().await;

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Taken"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Free"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/journals/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Taken"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = body_json(resp).await;
    assert_eq!(json["status"], 422);
}

// ═══════════════════════════════════════════════════════════════════
// DELETE /journals/:id
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn delete_existing_returns_204() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Doomed"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_makes_get_return_404() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Gone"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_removes_from_list() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Vanish"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Stay"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(resp).await;
    assert_eq!(json["total"], 1);
    assert_eq!(json["_embedded"]["journals"][0]["name"], "Stay");
}

#[tokio::test]
async fn delete_nonexistent_returns_204() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/journals/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_is_idempotent() {
    let state = app_state().await;

    let create_resp = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Twice"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let id = body_json(create_resp).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp1 = endpoint_hal::router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp1.status(), StatusCode::NO_CONTENT);

    let resp2 = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/journals/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp2.status(), StatusCode::NO_CONTENT);
}

// ═══════════════════════════════════════════════════════════════════
// RFC 9457 error response structure
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn error_response_has_problem_json_content_type() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/journals/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(ct, "application/problem+json");
}

#[tokio::test]
async fn error_response_has_all_rfc9457_fields() {
    let state = app_state().await;

    let resp = endpoint_hal::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/journals")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(resp).await;

    assert_eq!(json["type"], "about:blank");
    assert!(json["title"].as_str().is_some());
    assert!(json["status"].as_u64().is_some());
    assert!(json["instance"].as_str().is_some());
}
