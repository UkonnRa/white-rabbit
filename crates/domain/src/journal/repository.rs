use shared::WriteRepository;

use crate::journal::Journal;
use crate::journal::specification::JournalSpec;

pub trait JournalRepository: WriteRepository<JournalSpec, Entity = Journal> {}
