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
                    .table(RecordItem::Table)
                    .if_not_exists()
                    .col(string(RecordItem::Id).primary_key())
                    .col(string(RecordItem::RecordId).not_null())
                    .col(string(RecordItem::AccountId).not_null())
                    .col(string_null(RecordItem::AccountType))
                    .col(string(RecordItem::AmountNumber).not_null())
                    .col(string(RecordItem::AmountUnit).not_null())
                    .col(text(RecordItem::Description).not_null().default(""))
                    .col(string_null(RecordItem::PriceNumber))
                    .col(string_null(RecordItem::PriceUnit))
                    .col(text_null(RecordItem::Costs))
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecordItem::Table, RecordItem::RecordId)
                            .to(Record::Table, Record::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RecordItem::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum RecordItem {
    Table,
    Id,
    RecordId,
    AccountId,
    AccountType,
    AmountNumber,
    AmountUnit,
    Description,
    PriceNumber,
    PriceUnit,
    Costs,
}
