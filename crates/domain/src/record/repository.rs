use shared::WriteRepository;

use crate::record::Record;
use crate::record::specification::RecordSpec;

pub trait RecordRepository: WriteRepository<RecordSpec, Entity = Record> {}
