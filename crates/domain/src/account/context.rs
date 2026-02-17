use std::collections::HashMap;

use shared::Entity;

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
        self.validate_parent_not_archived()?;
        self.validate_name_not_reserved()?;
        Ok(())
    }

    /// Walk the parent chain: every ancestor must have the same account type.
    fn validate_type_matches_ancestors(&self) -> Result<()> {
        let mut current = &self.entity;
        while let Some(parent_id) = &current.parent_id {
            let parent = self.accounts.get(parent_id).ok_or_else(|| {
                shared::ErrorKind::not_found()
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("parent_id")
                    .with_detail(format!("parent {} not found", parent_id.as_ref()))
                    .convert()
            })?;

            if current.r#type != parent.r#type {
                return Err(ErrorKind::mismatch(parent.r#type, current.r#type)
                    .with_resource_type(Account::ENTITY_TYPE)
                    .with_field("type")
                    .with_detail(format!("must match parent {} type", parent_id.as_ref(),)));
            }
            current = parent;
        }
        Ok(())
    }

    /// The parent account must not be archived.
    fn validate_parent_not_archived(&self) -> Result<()> {
        if let Some(parent_id) = &self.entity.parent_id
            && let Some(parent) = self.accounts.get(parent_id)
            && parent.is_archived()
        {
            return Err(shared::ErrorKind::conflict()
                .with_resource_type(Account::ENTITY_TYPE)
                .with_field("parent_id")
                .with_detail(format!(
                    "parent {} is archived; cannot create child under archived account",
                    parent_id.as_ref()
                ))
                .convert());
        }
        Ok(())
    }

    /// User accounts cannot use the 5 reserved root names (case-insensitive).
    /// Root accounts (no parent) are exempt — they ARE the root names.
    fn validate_name_not_reserved(&self) -> Result<()> {
        if self.entity.parent_id.is_some()
            && Account::is_reserved_name(&self.entity.name.to_string())
        {
            return Err(shared::ErrorKind::conflict()
                .with_resource_type(Account::ENTITY_TYPE)
                .with_field("name")
                .with_detail(format!(
                    "'{}' is a reserved root account name",
                    self.entity.name
                ))
                .convert());
        }
        Ok(())
    }
}
