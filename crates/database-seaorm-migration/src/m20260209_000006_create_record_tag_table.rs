use sea_orm_migration::{prelude::*, schema::*};

use super::m20260209_000005_create_record_table::Record;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RecordTag::Table)
                    .if_not_exists()
                    .col(string(RecordTag::RecordId).not_null())
                    .col(string(RecordTag::Tag).not_null())
                    .primary_key(Index::create().col(RecordTag::RecordId).col(RecordTag::Tag))
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecordTag::Table, RecordTag::RecordId)
                            .to(Record::Table, Record::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RecordTag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum RecordTag {
    Table,
    RecordId,
    Tag,
}
