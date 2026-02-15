use shared::WriteRepository;

use crate::account::Account;
use crate::account::specification::AccountSpec;

pub trait AccountRepository: WriteRepository<AccountSpec, Entity = Account> {}
