use sea_orm_migration::{prelude::*, schema::*};

use super::m20260209_000003_create_account_table::Account;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccountTag::Table)
                    .if_not_exists()
                    .col(string(AccountTag::AccountId).not_null())
                    .col(string(AccountTag::Tag).not_null())
                    .primary_key(
                        Index::create()
                            .col(AccountTag::AccountId)
                            .col(AccountTag::Tag),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AccountTag::Table, AccountTag::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountTag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AccountTag {
    Table,
    AccountId,
    Tag,
}
