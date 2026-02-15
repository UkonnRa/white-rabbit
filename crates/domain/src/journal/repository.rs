use shared::WriteRepository;

use crate::journal::Journal;
use crate::journal::specification::JournalSpecification;

pub trait JournalRepository: WriteRepository<JournalSpecification, Entity = Journal> {}
