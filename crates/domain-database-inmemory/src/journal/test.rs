use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::Mutex;

use domain::journal::command::JournalCommandCreate;
use domain::journal::service::JournalService;

use super::InMemoryJournalRepository;

#[tokio::test]
async fn test_create_journal_via_service() -> anyhow::Result<()> {
    let repo = InMemoryJournalRepository::default();
    let service = JournalService {
        repository: Arc::new(Mutex::new(repo)),
    };

    let command = JournalCommandCreate {
        name: "My Ledger".to_string(),
        description: "Personal finance".to_string(),
        tags: HashSet::from(["finance".to_string()]),
    };

    let journal = service.create(command).await?;
    assert!(journal.name.to_string().contains("My Ledger"));
    assert_eq!(journal.description, "Personal finance");

    Ok(())
}

#[tokio::test]
async fn test_create_journal_with_empty_name_fails() -> anyhow::Result<()> {
    let repo = InMemoryJournalRepository::default();
    let service = JournalService {
        repository: Arc::new(Mutex::new(repo)),
    };

    let command = JournalCommandCreate {
        name: "".to_string(),
        description: "Should fail".to_string(),
        tags: HashSet::new(),
    };

    if let Result::Err(err) = service.create(command).await {
        assert_eq!(
            err.error,
            domain::error::ErrorKind::Shared(shared::ErrorKind::NonEmpty)
        );
    } else {
        anyhow::bail!("Expected error, got success");
    }

    Ok(())
}
