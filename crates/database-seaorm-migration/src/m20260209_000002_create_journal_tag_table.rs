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
                    .table(JournalTag::Table)
                    .if_not_exists()
                    .col(string(JournalTag::JournalId).not_null())
                    .col(string(JournalTag::Tag).not_null())
                    .primary_key(
                        Index::create()
                            .col(JournalTag::JournalId)
                            .col(JournalTag::Tag),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(JournalTag::Table, JournalTag::JournalId)
                            .to(Journal::Table, Journal::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JournalTag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum JournalTag {
    Table,
    JournalId,
    Tag,
}
