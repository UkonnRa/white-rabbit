use sea_orm_migration::{prelude::*, schema::*};

use super::m20260209_000001_create_journal_table::Journal;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(string(Account::Id).primary_key())
                    .col(integer(Account::Version).not_null().default(0))
                    .col(timestamp_with_time_zone_null(Account::CreatedAt))
                    .col(timestamp_with_time_zone_null(Account::LastModifiedAt))
                    .col(timestamp_with_time_zone_null(Account::ArchivedAt))
                    .col(string(Account::JournalId).not_null())
                    .col(string_null(Account::ParentId))
                    .col(string(Account::Type).not_null().default("Asset"))
                    .col(string(Account::Name).not_null())
                    .col(text(Account::Description).not_null().default(""))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Account::Table, Account::JournalId)
                            .to(Journal::Table, Journal::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Account::Table, Account::ParentId)
                            .to(Account::Table, Account::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: name is unique among siblings (same parent + journal)
        manager
            .create_index(
                Index::create()
                    .name("idx_account_unique_sibling_name")
                    .table(Account::Table)
                    .col(Account::JournalId)
                    .col(Account::ParentId)
                    .col(Account::Name)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Account {
    Table,
    Id,
    Version,
    CreatedAt,
    LastModifiedAt,
    ArchivedAt,
    JournalId,
    ParentId,
    Type,
    Name,
    Description,
}
