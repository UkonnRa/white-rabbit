use std::sync::Arc;

use database_seaorm::repository::SeaOrmSession;
use domain::journal::service::JournalService;
use domain_database_seaorm::journal::SeaOrmJournalRepository;
use sea_orm::DatabaseConnection;

pub type JournalRepo = SeaOrmJournalRepository;
pub type JournalSvc = JournalService<JournalRepo>;

pub struct AppState {
    pub journal_service: Arc<JournalSvc>,
    pub journal_repo: Arc<JournalRepo>,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        let journal_repo = Arc::new(SeaOrmJournalRepository);
        let journal_service = Arc::new(JournalService {
            repository: Arc::clone(&journal_repo),
        });
        Self {
            journal_service,
            journal_repo,
            db,
        }
    }

    pub fn new_session(&self) -> SeaOrmSession {
        SeaOrmSession::new(self.db.clone())
    }
}
