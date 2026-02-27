use endpoint_hal::state::AppState;
use sea_orm::Database;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

    let db = Database::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let state = AppState::new(db);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let app = endpoint_hal::router(state)
        .nest_service("/swagger-ui", ServeDir::new(format!("{manifest_dir}/dist")))
        .route_service(
            "/openapi.yaml",
            tower_http::services::ServeFile::new(format!("{manifest_dir}/openapi/openapi.yaml")),
        );

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on {addr}");
    println!("swagger UI: http://{addr}/swagger-ui/");
    axum::serve(listener, app).await.unwrap();
}
