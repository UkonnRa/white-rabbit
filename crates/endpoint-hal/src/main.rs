use database_seaorm_migration::{Migrator, MigratorTrait};
use endpoint_hal::state::AppState;
use sea_orm::Database;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

    let db = Database::connect(&database_url)
        .await
        .expect("failed to connect to database");

    Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    let state = AppState::new(db);

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            let host = origin.as_bytes();
            host.starts_with(b"http://localhost")
                || host.starts_with(b"http://127.0.0.1")
                || host.starts_with(b"http://0.0.0.0")
        }))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let app = endpoint_hal::router(state)
        .nest_service("/swagger-ui", ServeDir::new(format!("{manifest_dir}/dist")))
        .route_service(
            "/openapi.yaml",
            tower_http::services::ServeFile::new(format!("{manifest_dir}/openapi/openapi.yaml")),
        )
        .layer(cors);

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on {addr}");
    println!("swagger UI: http://{addr}/swagger-ui/");
    axum::serve(listener, app).await.unwrap();
}
