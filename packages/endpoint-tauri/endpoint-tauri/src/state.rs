use std::sync::Arc;

use database_seaorm::repository::SeaOrmSession;
use domain::account::service::AccountService;
use domain::journal::service::JournalService;
use domain_database_seaorm::account::SeaOrmAccountRepository;
use domain_database_seaorm::journal::SeaOrmJournalRepository;
use sea_orm::DatabaseConnection;

pub type JournalRepo = SeaOrmJournalRepository;
pub type JournalSvc = JournalService<JournalRepo>;

pub type AccountRepo = SeaOrmAccountRepository;
pub type AccountSvc = AccountService<AccountRepo>;

pub struct AppState {
    pub journal_service: Arc<JournalSvc>,
    pub journal_repo: Arc<JournalRepo>,
    pub account_service: Arc<AccountSvc>,
    pub account_repo: Arc<AccountRepo>,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        let journal_repo = Arc::new(SeaOrmJournalRepository);
        let journal_service = Arc::new(JournalService {
            repository: Arc::clone(&journal_repo),
        });
        let account_repo = Arc::new(SeaOrmAccountRepository);
        let account_service = Arc::new(AccountService {
            repository: Arc::clone(&account_repo),
        });
        Self {
            journal_service,
            journal_repo,
            account_service,
            account_repo,
            db,
        }
    }

    pub fn new_session(&self) -> SeaOrmSession {
        SeaOrmSession::new(self.db.clone())
    }
}
