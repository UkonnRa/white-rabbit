pub mod journal;
pub mod state;

use database_seaorm_migration::MigratorTrait;
use tauri::Manager;

use state::AppState;

pub fn register_handlers<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        journal::create_journal,
        journal::get_journal,
        journal::list_journals,
        journal::update_journal,
        journal::delete_journal,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    register_handlers(tauri::Builder::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            let state = rt.block_on(async {
                let database_url =
                    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
                let db = sea_orm::Database::connect(&database_url)
                    .await
                    .expect("failed to connect to database");
                database_seaorm_migration::Migrator::up(&db, None)
                    .await
                    .expect("failed to run migrations");
                AppState::new(db)
            });
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
