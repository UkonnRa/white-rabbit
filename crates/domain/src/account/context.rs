use std::collections::HashMap;

use crate::account::{Account, AccountId};
use crate::error::{ErrorKind, Result};
use crate::journal::Journal;

pub struct AccountContext {
    pub entity: Account,

    pub journal: Journal,
    pub accounts: HashMap<AccountId, Account>,
}

impl AccountContext {
    pub fn validate(&self) -> Result<()> {
        self.validate_type_matches_ancestors()?;
        Ok(())
    }

    /// Walk the parent chain: every ancestor must have the same account type.
    fn validate_type_matches_ancestors(&self) -> Result<()> {
        let mut current = &self.entity;
        while let Some(parent_id) = &current.parent_id {
            let parent = self.accounts.get(parent_id).ok_or_else(|| {
                ErrorKind::not_found()
                    .with_resource_type(Account::TYPE)
                    .with_field("parent_id")
                    .with_detail(format!("parent {} not found", parent_id.as_ref()))
            })?;

            if current.r#type != parent.r#type {
                return Err(ErrorKind::mismatch(parent.r#type, current.r#type)
                    .with_resource_type(Account::TYPE)
                    .with_field("type")
                    .with_detail(format!("must match parent {} type", parent_id.as_ref(),)));
            }
            current = parent;
        }
        Ok(())
    }
}
