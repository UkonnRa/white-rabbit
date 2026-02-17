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
                    .table(Record::Table)
                    .if_not_exists()
                    .col(string(Record::Id).primary_key())
                    .col(integer(Record::Version).not_null().default(0))
                    .col(timestamp_with_time_zone_null(Record::CreatedAt))
                    .col(timestamp_with_time_zone_null(Record::LastModifiedAt))
                    .col(string(Record::JournalId).not_null())
                    .col(date(Record::Date).not_null())
                    .col(string(Record::Kind).not_null().default("Transaction"))
                    .col(text(Record::Description).not_null().default(""))
                    .col(text(Record::Payee).not_null().default(""))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Record::Table, Record::JournalId)
                            .to(Journal::Table, Journal::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Record::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Record {
    Table,
    Id,
    Version,
    CreatedAt,
    LastModifiedAt,
    JournalId,
    Date,
    Kind,
    Description,
    Payee,
}
