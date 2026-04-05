pub mod account;
pub mod error;
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
        account::create_account,
        account::get_account,
        account::list_accounts,
        account::update_account,
        account::delete_account,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    register_handlers(tauri::Builder::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

            let database_url = match std::env::var("DATABASE_URL") {
                Ok(url) => url,
                Err(_) => {
                    // For MacOS, it's `~/Library/Application Support`
                    // So, when tauri build, do not add DATABASE_URL
                    let data_dir = app
                        .path()
                        .app_data_dir()
                        .expect("failed to resolve app data dir");
                    std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");
                    let db_path = data_dir.join("data.db");
                    format!("sqlite:{}?mode=rwc", db_path.display())
                }
            };
            log::info!("database url: {database_url}");

            let state = rt.block_on(async {
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
